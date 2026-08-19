import {
  decodeAbout,
  decodeAlertCenter,
  decodeDeletePreview,
  decodeDeleteReport,
  decodeDiagnostics,
  decodeReportArchivePage,
  decodeReportResult,
  decodeShellStatus,
  type AboutDto,
  type AlertCenterPage,
  type AlertRule,
  type BootstrapDto,
  type CloseState,
  type DeletePreview,
  type DeleteReport,
  type DiagnosticsSnapshot,
  type LiveConnectionView,
  type LiveOverview,
  type NotifyCapability,
  type ReportArchivePage,
  type ReportArchiveSummary,
  type ReportQuery,
  type ReportResult,
  type RetentionPreview,
  type RouteId
} from "./dto";
import { decodeMonitorMessage } from "./ipc/decoder";
import { liveEmptyCopy, liveEmptyKind } from "./ipc/live-empty";
import { displayLiveRow } from "./format/live-row";
import {
  NUMERIC_FILTER_FIELDS,
  NUMERIC_MODES,
  clauseForField,
  defaultFilterUnit,
  isNumericFilterField,
  toQueryClause,
  unitLabelKey,
  unitsForField
} from "./format/live-filter-units";
import {
  defaultLiveQuery,
  fetchTraySummary,
  isTauriRuntime,
  queryLiveConnections,
  resyncMonitor,
  subscribeMonitor,
  type LiveConnectionQuery,
  type LiveFilterClause
} from "./ipc/live-session";
import { applySecretField, secretFieldMarkup } from "./ipc/secret-field";
import {
  emptyMonitorState,
  markCloseAccepted,
  reduceMonitor,
  type MonitorState
} from "./ipc/reducer";
import { categoryRows } from "./format/overview";
import {
  applyPresetRange,
  defaultReportForm,
  formFromQueryEcho,
  formatSharePct,
  isArchiveKindFilter,
  isReportPreset,
  reportShareModel,
  reportTrendModel,
  type ArchiveKindFilter,
  type ReportForm,
  type ShareRow
} from "./format/report-view";
import { reportPieSvg, reportTrendSvg } from "./format/report-svg";
import { formatBytes, formatUtc, unknownOr } from "./format/units";
import { healthAction, healthTitle, parseUiLocale, t, type UiLocale } from "./i18n";
import { BRAND_MARK, ROUTE_ICONS } from "./nav-icons";
import { applyTheme, parseUiTheme, type UiTheme } from "./theme";
import {
  ACTION_WIDTH,
  DATA_COLUMNS,
  columnLabelKey,
  columnWidth,
  defaultLiveTableLayout,
  isDataColumn,
  isNumericColumn,
  parseLiveTableLayout,
  setColumnHidden,
  setColumnWidth,
  tablePixelWidth,
  visibleDataColumns,
  type DataColumnId,
  type LiveTableLayout
} from "./live-table-layout";
import { nextLiveSort, sortAria, sortMarker } from "./live-table-sort";

let uiLocale: UiLocale = "zh";
let uiTheme: UiTheme = "mocha";
let liveQuery: LiveConnectionQuery = defaultLiveQuery();
let liveTableLayout = defaultLiveTableLayout();
let liveTableDragging = false;
let liveColumnPanelOpen = false;
let liveResize: { col: DataColumnId; startX: number; startW: number } | null = null;
let reportForm: ReportForm = defaultReportForm();
let archiveKindFilter: ArchiveKindFilter = "all";

const FILTER_FIELDS = [
  "host",
  "chain",
  "rule",
  "process",
  "source",
  "destination",
  "type",
  ...NUMERIC_FILTER_FIELDS
] as const;

function tx(key: string): string {
  return t(uiLocale, key);
}

function fmt(key: string, vars: Record<string, string | number>): string {
  let text = tx(key);
  for (const [name, value] of Object.entries(vars)) {
    text = text.replaceAll(`{${name}}`, String(value));
  }
  return text;
}

function unknownLabel(): string {
  return tx("common.unknown");
}

function liveRowCell(view: ReturnType<typeof displayLiveRow>, column: DataColumnId): string {
  switch (column) {
    case "host":
      return view.host;
    case "download":
      return view.download;
    case "upload":
      return view.upload;
    case "rateDownload":
      return view.dlSpeed;
    case "rateUpload":
      return view.ulSpeed;
    case "chain":
      return view.chains;
    case "rule":
      return view.rule;
    case "process":
      return view.process;
    case "duration":
      return view.time;
    case "source":
      return view.source;
    case "destination":
      return view.destination;
    case "type":
      return view.type;
  }
}

function applyLocale(locale: UiLocale): void {
  uiLocale = locale;
}

function adoptTheme(theme: UiTheme): void {
  uiTheme = theme;
  applyTheme(theme);
}

function localizeRoutes(routes: BootstrapDto["routes"]): BootstrapDto["routes"] {
  return routes.map((route) => ({
    ...route,
    titleZh: t(uiLocale, `route.${route.id}`)
  }));
}

function healthOf(session: string): { title: string; action: string } {
  const title = healthTitle(uiLocale, session);
  if (title === `health.${session}`) {
    return { title: session, action: tx("health.view_diag") };
  }
  return { title, action: healthAction(uiLocale, session) };
}

function caliberPair(
  title: string,
  upload: number | null,
  download: number | null
): string {
  const unknown = unknownLabel();
  return `<article class="caliber">
      <h3>${title}</h3>
      <dl>
        <div>
          <dt>${tx("overview.dir.up")}</dt>
          <dd>${formatBytes(upload, unknown)}</dd>
        </div>
        <div>
          <dt>${tx("overview.dir.down")}</dt>
          <dd>${formatBytes(download, unknown)}</dd>
        </div>
      </dl>
    </article>`;
}

function renderOverview(overview: LiveOverview): string {
  const health = healthOf(overview.health.session);
  const unknown = unknownLabel();
  const rows = categoryRows(overview.categoryUpload, overview.categoryDownload);
  const categoryBody = rows.length
    ? rows
        .map(
          (row) =>
            `<tr><td>${row.name}</td><td>${formatBytes(row.upload, unknown)}</td><td>${formatBytes(row.download, unknown)}</td></tr>`
        )
        .join("")
    : `<tr><td colspan="3">${tx("common.none")}</td></tr>`;
  const coverage = overview.coverageKind
    ? `<p class="gap">${fmt("overview.coverage_gap", { kind: overview.coverageKind, reason: unknownOr(overview.coverageReason, unknownLabel()) })}</p>`
    : `<p>${fmt("overview.coverage_ok", { time: formatUtc(overview.lastSampleUtc, tx("common.no_sample")) })}</p>`;
  return `
    <section class="overview">
      <section class="caliber-grid" aria-label="${tx("overview.aria")}">
        ${caliberPair(tx("overview.meter"), overview.meterUpload, overview.meterDownload)}
        ${caliberPair(tx("overview.attr"), overview.attributedUpload, overview.attributedDownload)}
        ${caliberPair(tx("overview.other"), overview.otherUpload, overview.otherDownload)}
        ${caliberPair(tx("overview.gap"), overview.gapUpload, overview.gapDownload)}
        ${caliberPair(tx("overview.over"), overview.overUpload, overview.overDownload)}
        <article class="caliber session">
          <h3>${tx("overview.active")}</h3>
          <p class="caliber-active">${overview.activeCount}</p>
          ${coverage}
          <p class="status" data-state="${overview.health.session}">${health.title}。${tx("common.next")}：${health.action}</p>
        </article>
      </section>
      <section class="panel categories">
        <h2>${tx("overview.categories")}</h2>
        <table class="data">
          <thead><tr><th>${tx("overview.col.name")}</th><th>${tx("overview.col.upload")}</th><th>${tx("overview.col.download")}</th></tr></thead>
          <tbody>${categoryBody}</tbody>
        </table>
      </section>
    </section>
  `;
}

function renderLive(
  state: MonitorState,
  rows: LiveConnectionView[],
  address: string,
  collectorRunning: boolean | null
): string {
  const snapshot = state.snapshot;
  const session = snapshot?.health.session ?? "no_data";
  const health = healthOf(session);
  const kind = liveEmptyKind({
    address,
    session,
    collectorRunning,
    coverageKind: snapshot?.coverageKind ?? null,
    coverageReason: snapshot?.coverageReason ?? null,
    rowCount: rows.length,
    needResync: state.needResync,
    frozen: state.frozen,
    errorZh: state.errorZh
  });
  const emptyText =
    kind === "disconnected"
      ? `${health.title}。${tx("common.next")}：${health.action}`
      : (liveEmptyCopy(kind, uiLocale) ?? health.title);
  const unknown = unknownLabel();
  const visible = visibleDataColumns(liveTableLayout);
  const rowHtml = rows
    .map((row) => {
      const mark = state.closeMarks.get(row.identity);
      const closeLabel =
        mark === "accepted"
          ? tx("live.close_accepted")
          : mark === "closed"
            ? tx("live.close_done")
            : mark === "unconfirmed"
              ? tx("live.close_unconfirmed")
              : tx("live.close");
      const view = displayLiveRow(row, uiLocale, unknown);
      const cells = visible
        .map((column) => {
          const num = isNumericColumn(column) ? " class=\"num\"" : "";
          return `<td${num}>${liveRowCell(view, column)}</td>`;
        })
        .join("");
      return `<tr>
        ${cells}
        <td><button type="button" data-close="${row.identity}" ${mark ? "disabled" : ""}>${closeLabel}</button></td>
      </tr>`;
    })
    .join("");
  const action =
    kind === "unconfigured"
      ? `<button type="button" data-route="settings-data">${tx("live.go_settings")}</button>`
      : kind === "needResync"
        ? `<button type="button" id="resync-monitor">${tx("live.resync")}</button>`
        : "";
  const pauseNote =
    collectorRunning === false ? `<p>${tx("live.paused")}</p>` : "";
  const clauseHtml = liveQuery.filter.clauses
    .map((clause, index) => {
      const fields = FILTER_FIELDS.map(
        (field) =>
          `<option value="${field}" ${clause.field === field ? "selected" : ""}>${tx(`live.filter.field.${field}`)}</option>`
      ).join("");
      const numeric = isNumericFilterField(clause.field);
      const modes = numeric
        ? NUMERIC_MODES.map(
            (mode) =>
              `<option value="${mode}" ${clause.mode === mode ? "selected" : ""}>${tx(`live.filter.${mode}`)}</option>`
          ).join("")
        : `<option value="contains" ${clause.mode === "contains" ? "selected" : ""}>${tx("live.filter.contains")}</option>
          <option value="exact" ${clause.mode === "exact" ? "selected" : ""}>${tx("live.filter.exact")}</option>`;
      const unit = clause.unit ?? defaultFilterUnit(clause.field);
      const valueControl = numeric
        ? `<input type="number" min="0" step="any" data-filter-value="${index}" value="${clause.value.replaceAll('"', "&quot;")}" />
          <select data-filter-unit="${index}">
            ${unitsForField(clause.field)
              .map(
                (item) =>
                  `<option value="${item}" ${unit === item ? "selected" : ""}>${tx(unitLabelKey(clause.field, item))}</option>`
              )
              .join("")}
          </select>`
        : `<input data-filter-value="${index}" value="${clause.value.replaceAll('"', "&quot;")}" />`;
      return `<div class="filter-row">
        <select data-filter-field="${index}">${fields}</select>
        <select data-filter-mode="${index}">${modes}</select>
        ${valueControl}
        <button type="button" class="btn-secondary" data-filter-remove="${index}">${tx("live.filter.remove")}</button>
      </div>`;
    })
    .join("");
  return `
    <section class="live-page">
      <header class="live-toolbar">
        <p class="status" data-state="${session}">${health.title}。${tx("common.next")}：${health.action}</p>
        <p class="live-sample">${fmt("live.last_sample", { time: formatUtc(snapshot?.lastSampleUtc ?? null, tx("common.no_sample")) })}</p>
        ${pauseNote}
        ${action}
        <div class="live-filter-bar">
          <label class="inline"><input type="checkbox" id="live-residential" ${liveQuery.filter.residentialOnly ? "checked" : ""} /> ${tx("live.filter.residential")}</label>
          <button type="button" class="btn-secondary" id="live-add-clause" ${liveQuery.filter.clauses.length >= 8 ? "disabled" : ""}>${tx("live.filter.add")}</button>
          <div class="live-columns">
            <button type="button" class="btn-secondary" id="live-columns">${tx("live.columns")}</button>
            ${
              liveColumnPanelOpen
                ? `<div class="live-columns-panel" id="live-columns-panel">
              <p>${tx("live.columns.panel")}</p>
              ${DATA_COLUMNS.map((column) => {
                const checked = !liveTableLayout.hidden.includes(column) ? "checked" : "";
                return `<label class="inline"><input type="checkbox" id="live-col-vis-${column}" data-col-visible="${column}" ${checked} /> ${tx(columnLabelKey(column))}</label>`;
              }).join("")}
              <button type="button" class="btn-secondary" id="live-columns-reset">${tx("live.columns.reset")}</button>
            </div>`
                : ""
            }
          </div>
        </div>
        ${clauseHtml ? `<div class="filter-clauses">${clauseHtml}</div>` : ""}
      </header>
      <div class="live-table-wrap">
      <table class="data live-table" style="width:${tablePixelWidth(liveTableLayout)}px">
        <colgroup>
          ${visible
            .map(
              (column) =>
                `<col data-col="${column}" style="width:${columnWidth(liveTableLayout, column)}px" />`
            )
            .join("")}
          <col data-col="action" style="width:${ACTION_WIDTH}px" />
        </colgroup>
        <thead><tr>
          ${visible
            .map((column) => {
              const num = isNumericColumn(column) ? " class=\"num\"" : "";
              const current = { sortField: liveQuery.sortField, descending: liveQuery.descending };
              const aria = sortAria(column, current);
              return `<th data-col="${column}"${num}><button type="button" class="live-sort" data-sort="${column}" aria-sort="${aria}">${tx(columnLabelKey(column))}${sortMarker(column, current)}</button><span class="live-col-resize" data-col-resize="${column}"></span></th>`;
            })
            .join("")}
          <th data-col="action">${tx("live.col.action")}</th>
        </tr></thead>
        <tbody>${rowHtml || `<tr><td colspan="${visible.length + 1}">${emptyText}</td></tr>`}</tbody>
      </table>
      </div>
    </section>
  `;
}

function defaultReportQuery(): ReportQuery {
  const end = Math.floor(Date.now() / 1000);
  return {
    rangeStartUtc: end - 3600,
    rangeEndUtc: end,
    displayTimezone: "local",
    granularity: "hour",
    filters: { category: null, host: null, process: null, rule: null, chain: null, network: null },
    grouping: "host",
    targetPolicy: "historical",
    comparison: { previousEqualWindow: true },
    sort: { field: "download", descending: true },
    page: { limit: 200, after: null },
    topN: 20,
    includeSessions: false
  };
}

type ReportSource = "auto-hour" | "auto-day" | "manual" | null;

function archiveKindLabel(kind: string): string {
  if (kind === "day") {
    return tx("report.archive.kind.day");
  }
  return tx("report.archive.kind.hour");
}

function archiveStatusLabel(status: string): string {
  if (status === "ok") {
    return tx("report.archive.status.ok");
  }
  return tx("report.archive.status.failed");
}

function escapeHtml(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

function optionHtml(value: string, current: string, label: string): string {
  return `<option value="${value}"${value === current ? " selected" : ""}>${label}</option>`;
}

function shareRowHtml(row: ShareRow): string {
  const up = row.upload === null ? tx("report.dash") : formatBytes(row.upload, unknownLabel());
  const width = row.share === null ? 0 : Math.max(0, Math.round(row.share * 100));
  const bar = row.share === null ? "" : `<span class="bar" style="width:${width}%"></span>`;
  const klass = row.kind === "remainder" ? " class=\"report-remainder\"" : "";
  return `<tr${klass}><td>${escapeHtml(row.label)}</td><td class="num">${up}</td><td class="num">${formatBytes(row.download, unknownLabel())}</td><td><div class="share"><span class="share-pct">${formatSharePct(row.share, unknownLabel())}</span><span class="share-track">${bar}</span></div></td></tr>`;
}

function renderReports(
  report: ReportResult | null,
  statusZh: string,
  archives: ReportArchivePage | null,
  selectedArchiveId: string | null,
  reportSource: ReportSource
): string {
  const unknown = unknownLabel();
  const share = report
    ? reportShareModel(report, { unknown, remainder: tx("report.remainder") })
    : null;
  const trend = reportTrendModel(report?.series ?? []);
  const seriesRows =
    report?.series
      .map(
        (point) =>
          `<tr><td>${formatUtc(point.bucketUtc)}</td><td class="num">${formatBytes(point.upload)}</td><td class="num">${formatBytes(point.download)}</td></tr>`
      )
      .join("") ?? "";
  const rankRows = share?.rows.map(shareRowHtml).join("") ?? "";
  const coverage = report
    ? `<p>${fmt("report.coverage", { status: report.coverage.status, gap: report.coverage.gapSec, unit: report.unit })}</p>`
    : "";
  const notes = report
    ? `<details class="report-notes"><summary>${tx("report.notes")}</summary><p>${escapeHtml(report.drilldownCapability.noteZh)} ${fmt("report.tier", { tier: report.dataTier })} ${escapeHtml(report.policyMetadata.noteZh)}</p></details>`
    : "";
  const sourceLine =
    reportSource === "manual"
      ? tx("report.archive.kind.manual")
      : reportSource === "auto-day"
        ? tx("report.archive.kind.day")
        : reportSource === "auto-hour"
          ? tx("report.archive.kind.hour")
          : "";
  const archiveRows =
    archives?.items
      .map((item) => {
        const current = item.archiveId === selectedArchiveId ? " aria-current=\"true\"" : "";
        const down =
          item.totalsDownload === null || item.totalsDownload === undefined
            ? unknown
            : formatBytes(item.totalsDownload, unknown);
        return `<tr data-archive-id="${escapeHtml(item.archiveId)}"${current}><td>${formatUtc(item.rangeStartUtc)}</td><td>${archiveKindLabel(item.kind)}</td><td class="num">${down}</td><td>${archiveStatusLabel(item.status)}</td></tr>`;
      })
      .join("") ?? "";
  const pieSlices =
    share?.rows.map((row) => ({ kind: row.kind, value: row.download })) ?? [];
  const trendSvg = reportTrendSvg(trend, tx("report.trend"));
  const pieSvg = share?.drawPie ? reportPieSvg(pieSlices, tx("report.pie")) : "";
  const pieNote =
    report && share && !share.drawPie && !share.capabilityUnsupported ? `<p>${tx("report.pie.unavailable")}</p>` : "";
  const rankBody = share?.capabilityUnsupported
    ? `<tr><td colspan="4">${tx("report.empty_cap")}</td></tr>`
    : rankRows || `<tr><td colspan="4">${tx("report.empty_cap")}</td></tr>`;
  const totals = report
    ? `<dl class="report-metrics">
        <div class="report-metric"><dt>${tx("report.metric.upload")}</dt><dd>${formatBytes(report.totals.upload, unknown)}</dd></div>
        <div class="report-metric"><dt>${tx("report.metric.download")}</dt><dd>${formatBytes(report.totals.download, unknown)}</dd></div>
        <div class="report-metric"><dt>${tx("report.metric.connections")}</dt><dd>${report.totals.connectionCount}</dd></div>
      </dl>`
    : `<p>${tx("report.none")}</p>`;
  return `
    <div class="reports">
    <section class="panel">
      <p>${tx("report.same_token")}</p>
      <div class="report-toolbar">
      <label class="stack">${tx("report.preset")}
        <select id="report-preset">
          ${optionHtml("hour", reportForm.preset, tx("report.preset.hour"))}
          ${optionHtml("day", reportForm.preset, tx("report.preset.day"))}
          ${optionHtml("7", reportForm.preset, tx("report.preset.7"))}
          ${optionHtml("30", reportForm.preset, tx("report.preset.30"))}
          ${optionHtml("month", reportForm.preset, tx("report.preset.month"))}
        </select>
      </label>
      <label class="stack">${tx("report.granularity")}
        <select id="report-granularity">
          ${optionHtml("hour", reportForm.granularity, tx("report.granularity.hour"))}
          ${optionHtml("day", reportForm.granularity, tx("report.granularity.day"))}
          ${optionHtml("month", reportForm.granularity, tx("report.granularity.month"))}
        </select>
      </label>
      <label class="stack">${tx("report.grouping")}
        <select id="report-grouping">
          ${optionHtml("host", reportForm.grouping, tx("report.grouping.host"))}
          ${optionHtml("process", reportForm.grouping, tx("report.grouping.process"))}
          ${optionHtml("rule", reportForm.grouping, tx("report.grouping.rule"))}
          ${optionHtml("chain", reportForm.grouping, tx("report.grouping.chain"))}
          ${optionHtml("network", reportForm.grouping, tx("report.grouping.network"))}
          ${optionHtml("category", reportForm.grouping, tx("report.grouping.category"))}
        </select>
      </label>
      <div class="report-actions">
      <button type="button" id="run-report">${tx("report.run")}</button>
      <button type="button" class="btn-secondary" id="export-csv">${tx("report.export_csv")}</button>
      <button type="button" class="btn-secondary" id="export-json">${tx("report.export_json")}</button>
      <button type="button" class="btn-secondary" id="export-html">${tx("report.export_html")}</button>
      </div>
      ${reportForm.windowSource === "archive" ? `<p class="report-window">${tx("report.preset.archive")}</p>` : ""}
      </div>
      <p class="status" data-state="${report ? "connected" : "no_data"}">${statusZh}</p>
      ${sourceLine ? `<p>${sourceLine}</p>` : ""}
      ${coverage}
      ${notes}
    </section>
    <section class="panel" aria-label="${tx("report.numbers")}">
      <h2>${tx("report.totals")}</h2>
      ${totals}
    </section>
    <div class="report-visuals">
    <section class="panel">
      <h2>${tx("report.trend")}</h2>
      ${trendSvg}
      <div class="report-table-wrap" aria-label="${tx("report.chart_table")}">
      <table class="data"><thead><tr><th>${tx("report.col.time")}</th><th class="num">${tx("report.col.upload")}</th><th class="num">${tx("report.col.download")}</th></tr></thead>
      <tbody>${seriesRows || `<tr><td colspan="3">${tx("report.empty")}</td></tr>`}</tbody></table>
      </div>
    </section>
    <section class="panel">
      <h2>${tx("report.topn")}</h2>
      <div class="report-topn-body${pieSvg ? " has-pie" : ""}">
      ${pieSvg}
      <div>
      ${pieNote}
      <div class="report-table-wrap">
      <table class="data"><thead><tr><th>${tx("report.col.name")}</th><th class="num">${tx("report.col.upload")}</th><th class="num">${tx("report.col.download")}</th><th>${tx("report.col.share")}</th></tr></thead>
      <tbody>${rankBody}</tbody></table>
      </div>
      </div>
      </div>
    </section>
    </div>
    <section class="panel report-archives">
      <h2>${tx("report.archive.list")}</h2>
      <p>${tx("report.archive.failed_retry")}</p>
      <label class="stack">${tx("report.archive.filter")}
        <select id="archive-kind">
          ${optionHtml("all", archiveKindFilter, tx("report.archive.filter.all"))}
          ${optionHtml("day", archiveKindFilter, tx("report.archive.kind.day"))}
          ${optionHtml("hour", archiveKindFilter, tx("report.archive.kind.hour"))}
        </select>
      </label>
      <div class="report-archive-wrap">
      <table class="data"><thead><tr><th>${tx("report.archive.col.time")}</th><th>${tx("report.archive.col.kind")}</th><th class="num">${tx("report.archive.col.download")}</th><th>${tx("report.archive.col.status")}</th></tr></thead>
      <tbody>${archiveRows || `<tr><td colspan="4">${tx("report.archive.empty")}</td></tr>`}</tbody></table>
      </div>
    </section>
    </div>
  `;
}

function renderSettings(
  boot: BootstrapDto,
  about: AboutDto | null,
  deletePreview: DeletePreview | null,
  deleteReport: DeleteReport | null,
  probeStatus: string,
  probeState: string
): string {
  const aboutBlock = about
    ? `<p>${fmt("settings.about_meta", { version: about.version, identifier: about.identifier, aumid: about.aumid })}</p>
       <p>${fmt("settings.about_sign", { state: about.signed ? tx("settings.signed") : tx("settings.unsigned"), note: about.signatureNoteZh })}</p>
       <p>${fmt("settings.about_release", { url: about.releasesUrl })}</p>`
    : `<p>${tx("settings.about_idle")}</p>`;
  const deleteItems =
    deletePreview?.items
      .map(
        (item) =>
          `<li><strong>${item.id}</strong>：${item.noteZh} ${item.exists ? tx("settings.exists") : tx("settings.missing")}</li>`
      )
      .join("") ?? `<li>${tx("settings.preview_idle")}</li>`;
  const deleteResult = deleteReport
    ? `<p class="status" data-state="${deleteReport.allDeclaredOk ? "connected" : "storage_failure"}">${deleteReport.summaryZh}</p>`
    : "";
  return `
    <section class="panel">
      <h2>${tx("settings.wizard")}</h2>
      <ol>
        <li>${tx("settings.wizard.1")}</li>
        <li>${tx("settings.wizard.2")}</li>
        <li>${tx("settings.wizard.3")}</li>
        <li>${tx("settings.wizard.4")}</li>
        <li>${tx("settings.wizard.5")}</li>
      </ol>
      <label class="stack">${tx("settings.locale")}
        <select id="ui-locale">
          <option value="zh" ${uiLocale === "zh" ? "selected" : ""}>${tx("settings.locale.zh")}</option>
          <option value="en" ${uiLocale === "en" ? "selected" : ""}>${tx("settings.locale.en")}</option>
        </select>
      </label>
      <label class="stack">${tx("settings.theme")}
        <select id="ui-theme">
          <option value="latte" ${uiTheme === "latte" ? "selected" : ""}>${tx("settings.theme.latte")}</option>
          <option value="frappe" ${uiTheme === "frappe" ? "selected" : ""}>${tx("settings.theme.frappe")}</option>
          <option value="macchiato" ${uiTheme === "macchiato" ? "selected" : ""}>${tx("settings.theme.macchiato")}</option>
          <option value="mocha" ${uiTheme === "mocha" ? "selected" : ""}>${tx("settings.theme.mocha")}</option>
        </select>
      </label>
      <label class="stack">${tx("settings.address")}
        <input id="controller-address" value="${boot.settings.address || "127.0.0.1:9097"}" />
      </label>
      ${secretFieldMarkup(uiLocale)}
      <label class="stack">${tx("settings.targets")}
        <input id="targets" value="家宽" />
      </label>
      <p>${fmt("settings.cred", { status: boot.settings.hasSecret ? tx("settings.cred_yes") : tx("settings.cred_no"), mode: boot.settings.secretMode })}</p>
      <p>${tx("settings.port_note")}</p>
      <div class="actions">
        <button type="button" id="save-settings">${tx("settings.save")}</button>
        <button type="button" id="test-controller">${tx("settings.test")}</button>
        <button type="button" id="disconnect-controller">${tx("settings.disconnect")}</button>
      </div>
      <p id="controller-probe" class="status" data-state="${probeState}">${probeStatus}</p>
    </section>
    <section class="panel">
      <h2>${tx("settings.data")}</h2>
      <p>${tx("settings.data_help")}</p>
      <p>${tx("settings.log_dir")}</p>
      <p id="log-dir-path"></p>
      <button type="button" id="open-log-dir" ${boot.logDir ? "" : "disabled"}>${tx("settings.open_log_dir")}</button>
      <button type="button" id="create-backup">${tx("settings.backup")}</button>
      <button type="button" id="restore-backup">${tx("settings.restore")}</button>
      <button type="button" id="retention-preview">${tx("settings.retention_preview")}</button>
      <button type="button" id="run-retention">${tx("settings.retention_run")}</button>
      <p id="data-note">${tx("settings.retention_note")}</p>
      <button type="button" id="run-vacuum">${tx("settings.vacuum")}</button>
    </section>
    <section class="panel" id="about">
      <h2>${tx("settings.about")}</h2>
      ${aboutBlock}
      <button type="button" id="load-about">${tx("settings.refresh_about")}</button>
      <button type="button" id="open-releases">${tx("settings.open_releases")}</button>
    </section>
    <section class="panel">
      <h2>${tx("settings.delete_title")}</h2>
      <p>${deletePreview?.noteZh ?? tx("settings.delete_help")}</p>
      ${uiLocale === "en" ? `<p>${tx("settings.delete_phrase_en")}</p>` : ""}
      <ul>${deleteItems}</ul>
      <label class="stack">${tx("settings.delete_phrase")}
        <input id="delete-phrase" autocomplete="off" />
      </label>
      <button type="button" id="preview-delete">${tx("settings.preview_delete")}</button>
      <button type="button" id="confirm-delete">${tx("settings.confirm_delete")}</button>
      ${deleteResult}
    </section>
  `;
}

function renderRecovery(boot: BootstrapDto): string {
  const recovery = boot.recovery;
  if (!recovery) {
    return `<section class="panel"><p>${tx("recovery.missing")}</p></section>`;
  }
  const backups = recovery.backups.map((item) => `<li>${item}</li>`).join("");
  return `
    <section class="panel recovery">
      <h2>${tx("recovery.title")}</h2>
      <p>${fmt("recovery.meta", { app: recovery.appVersion, db: recovery.userVersion, max: recovery.supportedMax })}</p>
      <p>${recovery.future ? tx("recovery.future") : tx("recovery.unreadable")}</p>
      <p>${recovery.restoreNoteZh}</p>
      <p>${tx("recovery.log_dir")}</p>
      <p id="log-dir-path"></p>
      <button type="button" id="open-log-dir" ${boot.logDir ? "" : "disabled"}>${tx("recovery.open_log_dir")}</button>
      <button type="button" id="restore-backup" ${recovery.restoreAvailable ? "" : "disabled"}>${tx("recovery.run")}</button>
      <h3>${tx("recovery.backups")}</h3>
      <ul>${backups || `<li>${tx("common.none")}</li>`}</ul>
    </section>
  `;
}

function renderUnavailable(name: string, until: string): string {
  return `<section class="panel"><h2>${name}</h2><p>${fmt("unavailable.body", { until })}</p></section>`;
}

function renderAlerts(
  page: AlertCenterPage | null,
  statusZh: string,
  diagnostics: DiagnosticsSnapshot | null,
  notify: NotifyCapability | null
): string {
  const rows =
    page?.items
      .map((item) => {
        const observed =
          item.evidence.observedValue === null ? unknownLabel() : String(item.evidence.observedValue);
        const recovered = item.resolvedUtc === null ? "—" : formatUtc(item.resolvedUtc);
        return `<tr>
          <td>${item.ruleId} v${item.ruleVersion}</td>
          <td>${item.status}</td>
          <td>${item.selectorIdentity}</td>
          <td>${observed}</td>
          <td>${item.evidence.coverageSummary}</td>
          <td>${recovered}</td>
          <td>${item.evidence.notEvaluableReason ?? "—"}</td>
        </tr>`;
      })
      .join("") ?? "";
  const notifyZh = notify
    ? `${notify.available ? tx("alerts.notify_on") : tx("alerts.notify_off")}。${notify.reasonZh}`
    : tx("alerts.notify_idle");
  const diag = diagnostics
    ? `<p>${fmt("alerts.diag_app", { app: diagnostics.appVersion, schema: diagnostics.sqliteUserVersion, supported: diagnostics.supportedSchema, journal: diagnostics.journalMode, sync: diagnostics.synchronous })}</p>
       <p>${fmt("alerts.diag_ctrl", { transport: diagnostics.controllerTransportStatus, coverage: diagnostics.coverageSummary, wm: diagnostics.writerWatermark, active: diagnostics.alertActive, outbox: diagnostics.outboxBacklog })}</p>
       <p>${diagnostics.backupRetentionNoteZh} ${diagnostics.reconnectHintZh}</p>`
    : `<p>${tx("alerts.diag_idle")}</p>`;
  return `
    <section class="panel">
      <p>${tx("alerts.intro")}</p>
      <p class="status">${statusZh}</p>
      <p>${notifyZh}</p>
      <button type="button" id="refresh-alerts">${tx("alerts.refresh")}</button>
      <button type="button" id="test-notification">${tx("alerts.test")}</button>
      <button type="button" id="export-diagnostics">${tx("alerts.export_diag")}</button>
    </section>
    <section class="panel">
      <h2>${tx("alerts.rules")}</h2>
      <p>${tx("alerts.rules_help")}</p>
      <label class="stack">${tx("alerts.rule_id")} <input id="alert-rule-id" value="rate-home" /></label>
      <label class="stack">${tx("alerts.kind")}
        <select id="alert-kind">
          <option value="rate">${tx("alerts.kind.rate")}</option>
          <option value="period-usage">${tx("alerts.kind.period")}</option>
          <option value="health">${tx("alerts.kind.health")}</option>
        </select>
      </label>
      <label class="stack">${tx("alerts.selector")}
        <select id="alert-selector-kind">
          <option value="primary-category">${tx("alerts.selector.category")}</option>
          <option value="domain">${tx("alerts.selector.domain")}</option>
          <option value="process">${tx("alerts.selector.process")}</option>
          <option value="health-kind">${tx("alerts.selector.health")}</option>
        </select>
      </label>
      <label class="stack">${tx("alerts.selector_value")} <input id="alert-selector-value" value="家宽" /></label>
      <label class="stack">${tx("alerts.direction")}
        <select id="alert-direction">
          <option value="download">${tx("alerts.dir.down")}</option>
          <option value="upload">${tx("alerts.dir.up")}</option>
          <option value="combined">${tx("alerts.dir.combined")}</option>
        </select>
      </label>
      <label class="stack">${tx("alerts.threshold")} <input id="alert-threshold" type="number" value="1000000" /></label>
      <label class="stack">${tx("alerts.recovery")} <input id="alert-recovery" type="number" value="400000" /></label>
      <label class="stack">${tx("alerts.period")}
        <select id="alert-period">
          <option value="">${tx("alerts.period.none")}</option>
          <option value="rolling-1h">${tx("alerts.period.1h")}</option>
          <option value="local-day">${tx("alerts.period.day")}</option>
          <option value="local-month">${tx("alerts.period.month")}</option>
        </select>
      </label>
      <label class="stack">${tx("alerts.timezone")} <input id="alert-timezone" value="Asia/Shanghai" /></label>
      <button type="button" id="save-alert-rule">${tx("alerts.save")}</button>
    </section>
    <section class="panel">
      <h2>${tx("alerts.history")}</h2>
      <table class="data">
        <thead><tr><th>${tx("alerts.col.rule")}</th><th>${tx("alerts.col.status")}</th><th>${tx("alerts.col.target")}</th><th>${tx("alerts.col.observed")}</th><th>${tx("alerts.col.coverage")}</th><th>${tx("alerts.col.resolved")}</th><th>${tx("alerts.col.noteval")}</th></tr></thead>
        <tbody>${rows || `<tr><td colspan="7">${tx("alerts.empty")}</td></tr>`}</tbody>
      </table>
    </section>
    <section class="panel">
      <h2>脱敏诊断</h2>
      ${diag}
    </section>
  `;
}



function navHtml(active: RouteId, routes: BootstrapDto["routes"]): string {
  return routes
    .map((route) => {
      const current = route.id === active ? "aria-current=\"page\"" : "";
      const disabled = route.available ? "" : "data-disabled=\"true\"";
      const icon = ROUTE_ICONS[route.id];
      return `<button type="button" class="nav-item" data-route="${route.id}" ${current} ${disabled}><img src="${icon}" alt="" width="22" height="22" />${route.titleZh}</button>`;
    })
    .join("");
}

function previewBootstrap(): BootstrapDto {
  return {
    schemaVersion: 1,
    branch: "normal-ready",
    routes: [
      { id: "overview", titleZh: "概览", available: true, unavailableUntil: null },
      { id: "live", titleZh: "实时连接", available: true, unavailableUntil: null },
      { id: "reports", titleZh: "分析报告", available: true, unavailableUntil: null },
      { id: "alerts", titleZh: "告警", available: true, unavailableUntil: null },
      { id: "settings-data", titleZh: "设置 / 数据管理", available: true, unavailableUntil: null }
    ],
    overview: {
      schemaVersion: 1,
      meterUpload: null,
      meterDownload: null,
      attributedUpload: null,
      attributedDownload: null,
      categoryUpload: {},
      categoryDownload: {},
      otherUpload: null,
      otherDownload: null,
      gapUpload: null,
      gapDownload: null,
      overUpload: null,
      overDownload: null,
      activeCount: 0,
      lastSampleUtc: null,
      coverageKind: null,
      coverageReason: null,
      health: { session: "no_data", storageOk: true, storageReason: null }
    },
    settings: {
      transport: "tcp",
      address: "",
      credentialTarget: "io.github.bahayonghang.residential-monitor/controller",
      hasSecret: false,
      secretMode: "none"
    },
    wizardComplete: false,
    recovery: null,
    launchMode: "interactive",
    uiLocale: "zh",
    uiTheme: "mocha",
    liveTableLayout: defaultLiveTableLayout(),
    logDir: ""
  };
}

async function invokeCommand<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  const api = (
    globalThis as {
      __TAURI_INTERNALS__?: { invoke: (cmd: string, args?: Record<string, unknown>) => Promise<T> };
    }
  ).__TAURI_INTERNALS__;
  if (!api) {
    throw new Error("not-tauri");
  }
  return api.invoke(name, args);
}

function probeErrorText(error: unknown): { messageZh: string; action: string; code: string } {
  if (!error || typeof error !== "object") {
    return { messageZh: tx("settings.connect_fail"), action: "", code: "" };
  }
  const rec = error as Record<string, unknown>;
  if (typeof rec.messageZh === "string") {
    return {
      messageZh: rec.messageZh,
      action: typeof rec.action === "string" ? rec.action : "",
      code: typeof rec.code === "string" ? rec.code : ""
    };
  }
  if (typeof rec.message === "string") {
    try {
      const parsed = JSON.parse(rec.message) as Record<string, unknown>;
      if (typeof parsed.messageZh === "string") {
        return {
          messageZh: parsed.messageZh,
          action: typeof parsed.action === "string" ? parsed.action : "",
          code: typeof parsed.code === "string" ? parsed.code : ""
        };
      }
    } catch {
      /* 非 JSON */
    }
  }
  return { messageZh: tx("settings.connect_fail"), action: "", code: "" };
}

function renderApp(
  root: HTMLElement,
  boot: BootstrapDto,
  state: MonitorState,
  route: RouteId,
  report: ReportResult | null,
  reportStatus: string,
  archives: ReportArchivePage | null,
  selectedArchiveId: string | null,
  reportSource: ReportSource,
  alerts: AlertCenterPage | null,
  alertStatus: string,
  diagnostics: DiagnosticsSnapshot | null,
  notify: NotifyCapability | null,
  about: AboutDto | null,
  deletePreview: DeletePreview | null,
  deleteReport: DeleteReport | null,
  probeStatus: string,
  probeState: string,
  liveRows: LiveConnectionView[],
  collectorRunning: boolean | null
): void {
  const focusedId = document.activeElement instanceof HTMLElement ? document.activeElement.id : "";
  const wrap = root.querySelector(".live-table-wrap");
  const scrollTop = wrap instanceof HTMLElement ? wrap.scrollTop : 0;
  const scrollLeft = wrap instanceof HTMLElement ? wrap.scrollLeft : 0;
  const body =
    boot.branch === "recovery-only"
      ? renderRecovery(boot)
      : route === "overview"
        ? renderOverview(state.snapshot ?? boot.overview)
        : route === "live"
          ? renderLive(state, liveRows, boot.settings.address, collectorRunning)
          : route === "settings-data"
            ? renderSettings(boot, about, deletePreview, deleteReport, probeStatus, probeState)
            : route === "reports"
              ? renderReports(report, reportStatus, archives, selectedArchiveId, reportSource)
              : route === "alerts"
                ? renderAlerts(alerts, alertStatus, diagnostics, notify)
                : renderUnavailable(tx("route.alerts"), "C4");
  const recovery = boot.branch === "recovery-only";
  root.innerHTML = `
    <aside class="shell">
      <div class="brand">
        <img class="brand-mark" src="${BRAND_MARK}" alt="" width="56" height="56" />
        <h1 class="brand-name">${tx("product.display_name")}</h1>
        <p class="brand-slogan">${tx("product.slogan")}</p>
      </div>
      ${
        recovery
          ? `<p class="shell-recovery">${tx("shell.recovery")}</p>`
          : `<nav class="nav" aria-label="${tx("nav.aria")}">${navHtml(route, boot.routes)}</nav>`
      }
    </aside>
    <main class="workspace" id="workspace" tabindex="-1">
      <div id="view">${body}</div>
      ${state.errorZh ? `<p class="gap" role="alert">${state.errorZh}</p>` : ""}
    </main>
  `;
  if (focusedId) {
    document.getElementById(focusedId)?.focus();
  }
  const logPath = root.querySelector("#log-dir-path");
  if (logPath instanceof HTMLElement) {
    logPath.textContent = boot.logDir ? boot.logDir : tx("settings.log_dir_unknown");
  }
  const nextWrap = root.querySelector(".live-table-wrap");
  if (nextWrap instanceof HTMLElement) {
    nextWrap.scrollTop = scrollTop;
    nextWrap.scrollLeft = scrollLeft;
  }
}

async function main(): Promise<void> {
  const app = document.querySelector("#app");
  if (!(app instanceof HTMLElement)) {
    return;
  }
  decodeShellStatus({
    schemaVersion: 1,
    kind: "shellStatus",
    identifier: "io.github.bahayonghang.residential-monitor",
    phase: "c2-shell",
    messageZh: "桌面外壳与实时监控"
  });

  let boot = previewBootstrap();
  try {
    boot = await invokeCommand<BootstrapDto>("get_bootstrap");
  } catch {
    boot = previewBootstrap();
  }
  applyLocale(parseUiLocale(boot.uiLocale));
  adoptTheme(parseUiTheme(boot.uiTheme));
  liveTableLayout = parseLiveTableLayout(boot.liveTableLayout);

  let route: RouteId = boot.branch === "recovery-only" ? "settings-data" : "overview";
  let state = emptyMonitorState();
  let report: ReportResult | null = null;
  let reportStatus = tx("report.idle");
  let archives: ReportArchivePage | null = null;
  let selectedArchiveId: string | null = null;
  let reportSource: ReportSource = null;
  let alerts: AlertCenterPage | null = null;
  let alertStatus = tx("alerts.idle");
  let diagnostics: DiagnosticsSnapshot | null = null;
  let notify: NotifyCapability | null = null;
  let about: AboutDto | null = null;
  let deletePreview: DeletePreview | null = null;
  let deleteReport: DeleteReport | null = null;
  let probeStatus = "";
  let probeState = "";
  let liveRows: LiveConnectionView[] = [];
  let collectorRunning: boolean | null = null;
  let resyncInFlight = false;
  let settingsSecret = "";
  let settingsSecretVisible = false;
  let settingsSecretLoaded = false;
  state.snapshot = boot.overview;
  const persistLayout = async (next: LiveTableLayout): Promise<void> => {
    liveTableLayout = next;
    if (!isTauriRuntime()) {
      return;
    }
    try {
      liveTableLayout = parseLiveTableLayout(
        await invokeCommand("save_live_table_layout", { layout: liveTableLayout })
      );
    } catch {
      /* 保持内存中的布局 */
    }
  };

  const paint = (): void => {
    if (liveTableDragging) {
      return;
    }
    renderApp(
      app,
      boot,
      state,
      route,
      report,
      reportStatus,
      archives,
      selectedArchiveId,
      reportSource,
      alerts,
      alertStatus,
      diagnostics,
      notify,
      about,
      deletePreview,
      deleteReport,
      probeStatus,
      probeState,
      liveRows,
      collectorRunning
    );
    applySecretField(app, settingsSecret, settingsSecretVisible, uiLocale);
  };

  const loadSettingsSecret = async (): Promise<void> => {
    if (settingsSecretLoaded || !isTauriRuntime()) {
      settingsSecretLoaded = true;
      return;
    }
    if (!boot.settings.hasSecret) {
      settingsSecretLoaded = true;
      return;
    }
    try {
      const value = await invokeCommand<string | null>("get_controller_secret");
      settingsSecret = value ?? "";
    } catch {
      settingsSecret = "";
    }
    settingsSecretLoaded = true;
  };

  const refreshLivePage = async (): Promise<void> => {
    if (!isTauriRuntime()) {
      liveRows = [];
      return;
    }
    try {
      const page = await queryLiveConnections({
        ...liveQuery,
        filter: {
          ...liveQuery.filter,
          clauses: liveQuery.filter.clauses.map(toQueryClause)
        }
      });
      liveRows = page.rows;
    } catch {
      liveRows = [];
    }
    try {
      const tray = await fetchTraySummary();
      collectorRunning = tray.collectorRunning;
    } catch {
      collectorRunning = null;
    }
  };

  const onMonitorRaw = (raw: unknown): void => {
    void handleMonitorRaw(raw);
  };

  const handleMonitorRaw = async (raw: unknown): Promise<void> => {
    try {
      const message = decodeMonitorMessage(raw);
      state = reduceMonitor(state, message);
      if (state.needResync && !resyncInFlight && state.subscriptionId !== null) {
        resyncInFlight = true;
        try {
          await resyncMonitor(state.subscriptionId, onMonitorRaw);
        } catch {
          /* 保持冻结，页面提供重新订阅 */
        } finally {
          resyncInFlight = false;
        }
      }
      if (message.kind === "bootstrap" || message.kind === "connectionDelta" || route === "live") {
        await refreshLivePage();
      }
      paint();
    } catch {
      /* 非 Channel 或解码失败 */
    }
  };
  paint();

  const apply = (next: MonitorState, nextRoute = route): void => {
    state = next;
    route = nextRoute;
    paint();
  };

  const applyReport = (
    next: ReportResult | null,
    status: string,
    source: ReportSource = reportSource
  ): void => {
    report = next;
    reportStatus = status;
    reportSource = source;
    if (next) {
      reportForm = formFromQueryEcho(next.queryEcho, reportForm);
    }
    paint();
  };

  const pickLatestArchive = (page: ReportArchivePage): ReportArchiveSummary | null => {
    const latestDay = page.items.find((item) => item.kind === "day" && item.status === "ok");
    if (latestDay) {
      return latestDay;
    }
    return page.items.find((item) => item.kind === "hour" && item.status === "ok") ?? null;
  };

  const loadArchiveList = async (): Promise<void> => {
    const kind = archiveKindFilter === "all" ? null : archiveKindFilter;
    archives = decodeReportArchivePage(
      await invokeCommand("list_report_archives", { kind, after: null, limit: 50 })
    );
  };

  const loadArchives = async (): Promise<void> => {
    applyReport(report, tx("report.archive.catchup"), reportSource);
    try {
      await loadArchiveList();
      const page = archives ?? { schemaVersion: 1, items: [], next: null };
      const latest = pickLatestArchive(page);
      if (!latest) {
        selectedArchiveId = null;
        const hasFailed = page.items.some((item) => item.status === "failed");
        if (hasFailed) {
          applyReport(null, tx("report.archive.failed"), null);
        } else if (page.items.length === 0) {
          applyReport(null, `${tx("report.archive.empty")} ${tx("report.archive.catchup")}`, null);
        } else {
          applyReport(null, tx("report.archive.none_closed"), null);
        }
        return;
      }
      selectedArchiveId = latest.archiveId;
      const decoded = decodeReportResult(
        await invokeCommand("get_report_archive", { archiveId: latest.archiveId })
      );
      const source: ReportSource = latest.kind === "day" ? "auto-day" : "auto-hour";
      applyReport(
        decoded,
        latest.kind === "day" ? tx("report.archive.loaded_day") : tx("report.archive.loaded_hour"),
        source
      );
    } catch {
      applyReport(null, tx("report.archive.unavailable"), null);
    }
  };

  const applyAlerts = (
    next: AlertCenterPage | null,
    status: string,
    nextDiag = diagnostics,
    nextNotify = notify
  ): void => {
    alerts = next;
    alertStatus = status;
    diagnostics = nextDiag;
    notify = nextNotify;
    paint();
  };

  const buildQuery = (): ReportQuery => {
    const archiveRange =
      reportForm.windowSource === "archive" && report
        ? {
            start: report.queryEcho.rangeStartUtc,
            end: report.queryEcho.rangeEndUtc,
            timezone: report.queryEcho.displayTimezone
          }
        : undefined;
    return applyPresetRange(defaultReportQuery(), reportForm, Math.floor(Date.now() / 1000), archiveRange);
  };

  app.addEventListener("input", (event) => {
    const target = event.target;
    if (target instanceof HTMLInputElement && target.id === "controller-secret") {
      settingsSecret = target.value;
    }
    if (target instanceof HTMLInputElement && target.dataset.filterValue != null) {
      const index = Number(target.dataset.filterValue);
      const clauses = liveQuery.filter.clauses.map((clause, item) =>
        item === index ? { ...clause, value: target.value } : clause
      );
      liveQuery = { ...liveQuery, filter: { ...liveQuery.filter, clauses } };
    }
  });

  app.addEventListener("change", async (event) => {
    const target = event.target;
    if (target instanceof HTMLInputElement && target.dataset.colVisible != null) {
      const column = target.dataset.colVisible;
      if (isDataColumn(column)) {
        await persistLayout(setColumnHidden(liveTableLayout, column, !target.checked));
        paint();
      }
      return;
    }
    if (target instanceof HTMLInputElement && target.id === "live-residential") {
      liveQuery = {
        ...liveQuery,
        filter: { ...liveQuery.filter, residentialOnly: target.checked }
      };
      await refreshLivePage();
      paint();
      return;
    }
    if (target instanceof HTMLSelectElement && target.dataset.filterField != null) {
      const index = Number(target.dataset.filterField);
      const clauses = liveQuery.filter.clauses.map((clause, item) =>
        item === index ? clauseForField(target.value) : clause
      );
      liveQuery = { ...liveQuery, filter: { ...liveQuery.filter, clauses } };
      await refreshLivePage();
      paint();
      return;
    }
    if (target instanceof HTMLSelectElement && target.dataset.filterUnit != null) {
      const index = Number(target.dataset.filterUnit);
      const clauses = liveQuery.filter.clauses.map((clause, item) =>
        item === index ? { ...clause, unit: target.value } : clause
      );
      liveQuery = { ...liveQuery, filter: { ...liveQuery.filter, clauses } };
      await refreshLivePage();
      paint();
      return;
    }
    if (target instanceof HTMLSelectElement && target.dataset.filterMode != null) {
      const index = Number(target.dataset.filterMode);
      const clauses = liveQuery.filter.clauses.map((clause, item) =>
        item === index ? { ...clause, mode: target.value as LiveFilterClause["mode"] } : clause
      );
      liveQuery = { ...liveQuery, filter: { ...liveQuery.filter, clauses } };
      await refreshLivePage();
      paint();
      return;
    }
    if (target instanceof HTMLInputElement && target.dataset.filterValue != null) {
      const index = Number(target.dataset.filterValue);
      const clauses = liveQuery.filter.clauses.map((clause, item) =>
        item === index ? { ...clause, value: target.value } : clause
      );
      liveQuery = { ...liveQuery, filter: { ...liveQuery.filter, clauses } };
      await refreshLivePage();
      paint();
      return;
    }
    if (target instanceof HTMLSelectElement && target.id === "ui-theme") {
      const nextTheme = parseUiTheme(target.value);
      try {
        const saved = await invokeCommand<string>("save_ui_theme", { theme: nextTheme });
        adoptTheme(parseUiTheme(saved));
        boot.uiTheme = uiTheme;
      } catch {
        adoptTheme(nextTheme);
        boot.uiTheme = nextTheme;
      }
      paint();
      return;
    }
    if (target instanceof HTMLSelectElement && target.id === "report-preset" && isReportPreset(target.value)) {
      reportForm = { ...reportForm, preset: target.value, windowSource: "preset" };
      paint();
      return;
    }
    if (target instanceof HTMLSelectElement && target.id === "report-granularity") {
      if (target.value === "hour" || target.value === "day" || target.value === "month") {
        reportForm = { ...reportForm, granularity: target.value };
      }
      return;
    }
    if (target instanceof HTMLSelectElement && target.id === "report-grouping") {
      const grouping = target.value;
      if (
        grouping === "host" ||
        grouping === "process" ||
        grouping === "rule" ||
        grouping === "chain" ||
        grouping === "network" ||
        grouping === "category"
      ) {
        reportForm = { ...reportForm, grouping };
      }
      return;
    }
    if (target instanceof HTMLSelectElement && target.id === "archive-kind" && isArchiveKindFilter(target.value)) {
      archiveKindFilter = target.value;
      try {
        await loadArchiveList();
      } catch {
        archives = null;
        reportStatus = tx("report.archive.unavailable");
      }
      paint();
      return;
    }
    if (!(target instanceof HTMLSelectElement) || target.id !== "ui-locale") {
      return;
    }
    const nextLocale = parseUiLocale(target.value);
    try {
      const saved = await invokeCommand<string>("save_ui_locale", { locale: nextLocale });
      applyLocale(parseUiLocale(saved));
      const next = await invokeCommand<BootstrapDto>("get_bootstrap");
      boot.routes = next.routes;
      boot.uiLocale = parseUiLocale(next.uiLocale);
      applyLocale(boot.uiLocale);
    } catch {
      applyLocale(nextLocale);
      boot.routes = localizeRoutes(boot.routes);
    }
    reportStatus = tx("report.idle");
    alertStatus = tx("alerts.idle");
    paint();
  });

  app.addEventListener("pointerdown", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLElement) || target.dataset.colResize == null) {
      return;
    }
    const column = target.dataset.colResize;
    if (!isDataColumn(column)) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    liveTableDragging = true;
    liveResize = {
      col: column,
      startX: event.clientX,
      startW: columnWidth(liveTableLayout, column)
    };
  });

  window.addEventListener("pointermove", (event) => {
    if (!liveResize) {
      return;
    }
    liveTableLayout = setColumnWidth(
      liveTableLayout,
      liveResize.col,
      liveResize.startW + (event.clientX - liveResize.startX)
    );
    const colEl = document.querySelector(`col[data-col="${liveResize.col}"]`);
    if (colEl instanceof HTMLElement) {
      colEl.style.width = `${columnWidth(liveTableLayout, liveResize.col)}px`;
    }
    const table = document.querySelector("table.live-table");
    if (table instanceof HTMLElement) {
      table.style.width = `${tablePixelWidth(liveTableLayout)}px`;
    }
  });

  window.addEventListener("pointerup", () => {
    if (!liveResize) {
      return;
    }
    liveResize = null;
    liveTableDragging = false;
    void persistLayout(liveTableLayout).then(() => paint());
  });

  app.addEventListener("click", async (event) => {
    const raw = event.target;
    if (!(raw instanceof Element)) {
      return;
    }
    const routeEl = raw.closest("[data-route]");
    const target = raw instanceof HTMLElement ? raw : raw.parentElement;
    if (!target) {
      return;
    }
    const sortEl = raw.closest("[data-sort]");
    if (sortEl instanceof HTMLElement && sortEl.dataset.sort && isDataColumn(sortEl.dataset.sort)) {
      const next = nextLiveSort(sortEl.dataset.sort, {
        sortField: liveQuery.sortField,
        descending: liveQuery.descending
      });
      liveQuery = { ...liveQuery, sortField: next.sortField, descending: next.descending, cursor: null };
      await refreshLivePage();
      paint();
      return;
    }
    if (raw.closest("#live-columns") && !raw.closest("#live-columns-panel")) {
      liveColumnPanelOpen = !liveColumnPanelOpen;
      paint();
      return;
    }
    if (raw.closest("#live-columns-reset")) {
      await persistLayout(defaultLiveTableLayout());
      paint();
      return;
    }
    if (liveColumnPanelOpen && !raw.closest(".live-columns")) {
      liveColumnPanelOpen = false;
      paint();
      return;
    }
    const nextRoute = (routeEl instanceof HTMLElement ? routeEl.dataset.route : undefined) as
      | RouteId
      | undefined;
    if (nextRoute) {
      apply(state, nextRoute);
      if (nextRoute === "settings-data") {
        await loadSettingsSecret();
        paint();
      }
      if (nextRoute === "live") {
        await refreshLivePage();
        paint();
      }
      if (nextRoute === "reports") {
        await loadArchives();
        return;
      }
      if (nextRoute === "alerts") {
        try {
          const page = decodeAlertCenter(
            await invokeCommand("list_alert_center", { status: null, after: null })
          );
          const diag = decodeDiagnostics(await invokeCommand("get_diagnostics"));
          applyAlerts(page, fmt("alerts.loaded", { count: page.items.length }), diag, notify);
        } catch {
          applyAlerts(alerts, tx("alerts.unavailable"));
        }
      }
      return;
    }
    const closeId = target.dataset.close;
    if (closeId) {
      try {
        const result = await invokeCommand<CloseState>("close_connection", {
          identity: closeId,
          requestId: `ui-${Date.now()}`
        });
        if (result.mark === "accepted") {
          apply(markCloseAccepted(state, closeId));
        }
      } catch {
        apply({ ...state, errorZh: tx("alerts.close_fail") });
      }
    }
    if (raw.closest("#toggle-secret")) {
      settingsSecretVisible = !settingsSecretVisible;
      applySecretField(app, settingsSecret, settingsSecretVisible, uiLocale);
      return;
    }
    if (target.id === "live-add-clause") {
      if (liveQuery.filter.clauses.length < 8) {
        const next: LiveFilterClause = { field: "host", mode: "contains", value: "" };
        liveQuery = {
          ...liveQuery,
          filter: { ...liveQuery.filter, clauses: [...liveQuery.filter.clauses, next] }
        };
        paint();
      }
      return;
    }
    const remove = target.dataset.filterRemove;
    if (remove != null) {
      const index = Number(remove);
      liveQuery = {
        ...liveQuery,
        filter: {
          ...liveQuery.filter,
          clauses: liveQuery.filter.clauses.filter((_, item) => item !== index)
        }
      };
      await refreshLivePage();
      paint();
      return;
    }
    if (target.id === "save-settings") {
      const address = (document.querySelector("#controller-address") as HTMLInputElement | null)?.value ?? "";
      const secret = (document.querySelector("#controller-secret") as HTMLInputElement | null)?.value;
      const targets = (document.querySelector("#targets") as HTMLInputElement | null)?.value ?? "";
      try {
        boot.settings = await invokeCommand("save_settings", {
          address,
          secret: secret && secret.length > 0 ? secret : null,
          sessionOnly: false
        });
        if (secret && secret.length > 0) {
          settingsSecret = secret;
          settingsSecretLoaded = true;
        }
        await invokeCommand("save_targets", {
          targets: targets
            .split(",")
            .map((item) => item.trim())
            .filter(Boolean)
        });
        const selected = (document.querySelector("#ui-locale") as HTMLSelectElement | null)?.value;
        const nextLocale = parseUiLocale(selected);
        const saved = await invokeCommand<string>("save_ui_locale", { locale: nextLocale });
        applyLocale(parseUiLocale(saved));
        boot.uiLocale = uiLocale;
        const selectedTheme = (document.querySelector("#ui-theme") as HTMLSelectElement | null)?.value;
        const nextTheme = parseUiTheme(selectedTheme);
        const savedTheme = await invokeCommand<string>("save_ui_theme", { theme: nextTheme });
        adoptTheme(parseUiTheme(savedTheme));
        boot.uiTheme = uiTheme;
        try {
          const next = await invokeCommand<BootstrapDto>("get_bootstrap");
          boot.routes = next.routes;
          boot.uiLocale = parseUiLocale(next.uiLocale);
          applyLocale(boot.uiLocale);
        } catch {
          boot.routes = localizeRoutes(boot.routes);
        }
        probeStatus = tx("settings.saved");
        probeState = "connected";
        apply(state, "settings-data");
      } catch {
        probeStatus = tx("settings.save_fail");
        probeState = "storage_failure";
        apply({ ...state, errorZh: tx("settings.save_fail") }, "settings-data");
      }
    }
    if (target.id === "test-controller") {
      const address = (document.querySelector("#controller-address") as HTMLInputElement | null)?.value ?? "";
      const secret = (document.querySelector("#controller-secret") as HTMLInputElement | null)?.value;
      probeStatus = "正在探测控制器。";
      probeState = "connecting";
      apply(state, "settings-data");
      try {
        const result = await invokeCommand<{ messageZh: string; status: string; action: string }>(
          "test_controller",
          {
            address,
            secret: secret && secret.length > 0 ? secret : null
          }
        );
        const next = await invokeCommand<BootstrapDto>("get_bootstrap");
        boot.settings = next.settings;
        boot.overview = next.overview;
        state.snapshot = next.overview;
        probeStatus = `${result.messageZh}下一步：${result.action}`;
        probeState = result.status;
        apply(state, "settings-data");
      } catch (error) {
        const dto = probeErrorText(error);
        const extra = dto.code === "endpoint_missing" ? " 本机 Verge 常用 127.0.0.1:9097。" : "";
        probeStatus = `${dto.messageZh}${dto.action ? `下一步：${dto.action}。` : ""}${extra}`;
        probeState = dto.code || "storage_failure";
        apply(state, "settings-data");
      }
    }
    if (target.id === "disconnect-controller") {
      try {
        const result = await invokeCommand<{ messageZh: string; status: string; action: string }>(
          "disconnect_controller"
        );
        const next = await invokeCommand<BootstrapDto>("get_bootstrap");
        boot.overview = next.overview;
        state.snapshot = next.overview;
        probeStatus = `${result.messageZh}下一步：${result.action}`;
        probeState = result.status;
        apply(state, "settings-data");
      } catch {
        probeStatus = "断开失败。";
        probeState = "storage_failure";
        apply(state, "settings-data");
      }
    }
    if (target.id === "run-report") {
      applyReport(report, tx("report.running"), "manual");
      try {
        const raw = await invokeCommand<unknown>("run_report", { query: buildQuery() });
        const decoded = decodeReportResult(raw);
        selectedArchiveId = null;
        applyReport(decoded, fmt("report.done", { token: decoded.reportSnapshotToken.slice(0, 8) }), "manual");
      } catch (error) {
        applyReport(null, error instanceof Error ? error.message : tx("report.fail"), "manual");
      }
    }
    const archiveEl = raw.closest("[data-archive-id]");
    if (archiveEl instanceof HTMLElement && archiveEl.dataset.archiveId) {
      const archiveId = archiveEl.dataset.archiveId;
      const item = archives?.items.find((row) => row.archiveId === archiveId);
      selectedArchiveId = archiveId;
      if (!item || item.status !== "ok") {
        applyReport(
          report,
          item?.noteZh ? item.noteZh : tx("report.archive.failed"),
          reportSource
        );
        return;
      }
      try {
        const decoded = decodeReportResult(
          await invokeCommand("get_report_archive", { archiveId })
        );
        const source: ReportSource = item.kind === "day" ? "auto-day" : "auto-hour";
        applyReport(
          decoded,
          item.kind === "day" ? tx("report.archive.loaded_day") : tx("report.archive.loaded_hour"),
          source
        );
      } catch {
        applyReport(report, tx("report.archive.unavailable"), reportSource);
      }
    }
    if (target.id === "export-csv" || target.id === "export-json" || target.id === "export-html") {
      if (!report) {
        applyReport(null, tx("report.need_run"));
        return;
      }
      const format = target.id === "export-csv" ? "csv" : target.id === "export-json" ? "json" : "html";
      try {
        const picked = await invokeCommand<string | null>("pick_file", {
          purpose: "report-export",
          mode: "save"
        });
        if (!picked) {
          applyReport(report, "已取消导出。");
          return;
        }
        await invokeCommand("export_report", {
          token: report.reportSnapshotToken,
          spec: {
            format,
            includeSeries: true,
            includeRankings: true,
            includeSessions: false,
            redactHost: "none",
            redactProcess: "none"
          },
          path: picked
        });
        applyReport(report, fmt("report.exported", { format: format.toUpperCase() }));
      } catch {
        applyReport(report, "导出失败。未覆盖已有文件。");
      }
    }
    if (target.id === "create-backup" || target.id === "restore-backup") {
      try {
        const picked = await invokeCommand<string | null>("pick_file", {
          purpose: target.id === "create-backup" ? "backup-create" : "backup-restore",
          mode: target.id === "create-backup" ? "save" : "open"
        });
        if (!picked) {
          return;
        }
        if (target.id === "create-backup") {
          await invokeCommand("create_backup", { path: picked });
          apply({ ...state, errorZh: null });
        } else {
          await invokeCommand("restore_backup", { path: picked });
        }
      } catch {
        apply({ ...state, errorZh: "备份或恢复失败。当前可用库未覆盖。" });
      }
    }
    if (target.id === "retention-preview" || target.id === "run-retention") {
      try {
        const preview = await invokeCommand<RetentionPreview>(
          target.id === "retention-preview" ? "retention_preview" : "run_retention",
          target.id === "run-retention" ? { delete: false } : undefined
        );
        const note = document.querySelector("#data-note");
        if (note) {
          note.textContent = `${preview.noteZh} raw ${preview.rawRows} 行，hourly ${preview.hourlyRows} 行。自动删除=${preview.autoDeleteEnabled}`;
        }
      } catch {
        apply({ ...state, errorZh: "保留预览失败。" });
      }
    }
    if (target.id === "refresh-alerts") {
      try {
        const page = decodeAlertCenter(
          await invokeCommand("list_alert_center", { status: null, after: null })
        );
        const diag = decodeDiagnostics(await invokeCommand("get_diagnostics"));
        applyAlerts(page, `已刷新 ${page.items.length} 条。`, diag, notify);
      } catch {
        applyAlerts(alerts, "刷新失败。");
      }
    }
    if (target.id === "test-notification") {
      try {
        const cap = await invokeCommand<NotifyCapability>("test_notification");
        applyAlerts(alerts, cap.available ? "已提交测试通知。" : cap.reasonZh, diagnostics, cap);
      } catch {
        applyAlerts(alerts, "测试通知失败。应用内记录仍可用。");
      }
    }
    if (target.id === "export-diagnostics") {
      try {
        const path = await invokeCommand<string | null>("pick_file", {
          purpose: "diagnostics-export",
          mode: "save"
        });
        if (path) {
          await invokeCommand("export_diagnostics", { path });
          applyAlerts(alerts, "诊断已导出。");
        }
      } catch {
        applyAlerts(alerts, "诊断导出失败。采集与告警未中断。");
      }
    }
    if (target.id === "load-about") {
      try {
        about = decodeAbout(await invokeCommand("get_about"));
        apply(state, "settings-data");
      } catch {
        apply({ ...state, errorZh: "无法读取关于信息。" });
      }
    }
    if (target.id === "open-log-dir") {
      try {
        await invokeCommand("open_log_dir");
      } catch {
        apply({ ...state, errorZh: tx("settings.open_log_dir_fail") });
      }
    }
    if (target.id === "open-releases") {
      try {
        const url = await invokeCommand<string>("open_releases");
        apply({ ...state, errorZh: `发布地址：${url}` });
      } catch {
        apply({ ...state, errorZh: "无法读取发布地址。" });
      }
    }
    if (target.id === "preview-delete") {
      try {
        deletePreview = decodeDeletePreview(await invokeCommand("preview_delete_local_data"));
        apply(state, "settings-data");
      } catch {
        apply({ ...state, errorZh: "无法预览删除范围。" });
      }
    }
    if (target.id === "confirm-delete") {
      const phrase = (document.querySelector("#delete-phrase") as HTMLInputElement | null)?.value ?? "";
      try {
        deleteReport = decodeDeleteReport(
          await invokeCommand("confirm_delete_local_data", { phrase })
        );
        apply(state, "settings-data");
      } catch {
        apply({ ...state, errorZh: "删除未执行。确认短语必须完全匹配。" });
      }
    }
    if (target.id === "resync-monitor") {
      if (state.subscriptionId !== null && isTauriRuntime()) {
        try {
          await resyncMonitor(state.subscriptionId, onMonitorRaw);
        } catch {
          apply({ ...state, errorZh: "重新订阅失败。请重载窗口。" });
        }
      }
    }
    if (target.id === "run-vacuum") {
      try {
        await invokeCommand("run_user_vacuum");
        apply({ ...state, errorZh: "VACUUM 已完成。freelist 不是已释放文件空间。" });
      } catch {
        apply({ ...state, errorZh: "VACUUM 未执行或空间不足。当前库未删除。" });
      }
    }
    if (target.id === "save-alert-rule") {
      const period = (document.querySelector("#alert-period") as HTMLSelectElement | null)?.value ?? "";
      const rule: AlertRule = {
        ruleId: (document.querySelector("#alert-rule-id") as HTMLInputElement | null)?.value ?? "rate-home",
        version: 1,
        enabled: true,
        kind: ((document.querySelector("#alert-kind") as HTMLSelectElement | null)?.value ?? "rate") as AlertRule["kind"],
        selectorKind: ((document.querySelector("#alert-selector-kind") as HTMLSelectElement | null)?.value ??
          "primary-category") as AlertRule["selectorKind"],
        selectorValue: (document.querySelector("#alert-selector-value") as HTMLInputElement | null)?.value || null,
        direction: ((document.querySelector("#alert-direction") as HTMLSelectElement | null)?.value ??
          "download") as AlertRule["direction"],
        thresholdValue: Number((document.querySelector("#alert-threshold") as HTMLInputElement | null)?.value ?? "1"),
        recoveryThreshold: Number((document.querySelector("#alert-recovery") as HTMLInputElement | null)?.value ?? "0"),
        period: period === "" ? null : (period as AlertRule["period"]),
        timezone: (document.querySelector("#alert-timezone") as HTMLInputElement | null)?.value ?? "UTC",
        cooldownSec: 300,
        quietStartMin: null,
        quietEndMin: null,
        createdUtc: 0,
        updatedUtc: 0
      };
      try {
        await invokeCommand("upsert_alert_rule", { rule });
        applyAlerts(alerts, "规则已保存。新版本不会继承旧连续命中。");
      } catch {
        applyAlerts(alerts, "规则无效。请检查滞回、时区和周期。");
      }
    }
  });

  try {
    const first = await invokeCommand<unknown>("get_bootstrap");
    if (first && typeof first === "object") {
      const maybe = first as BootstrapDto;
      if (maybe.overview) {
        boot = maybe;
        state.snapshot = maybe.overview;
        apply(state, route);
      }
    }
  } catch {
    /* 预览态没有 Tauri */
  }

  if (isTauriRuntime()) {
    try {
      await subscribeMonitor(onMonitorRaw);
    } catch {
      /* 订阅失败时保持可诊断空表 */
    }
  }
}

void main();
