import {
  decodeAbout,
  decodeAlertCenter,
  decodeDeletePreview,
  decodeDeleteReport,
  decodeDiagnostics,
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
  type ReportQuery,
  type ReportResult,
  type RetentionPreview,
  type RouteId
} from "./dto";
import { decodeMonitorMessage } from "./ipc/decoder";
import { liveEmptyCopy, liveEmptyKind } from "./ipc/live-empty";
import { displayLiveRow } from "./format/live-row";
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
import { formatBytes, formatUtc, unknownOr } from "./format/units";
import { healthAction, healthTitle, parseUiLocale, t, type UiLocale } from "./i18n";
import { BRAND_MARK, ROUTE_ICONS } from "./nav-icons";
import { applyTheme, parseUiTheme, type UiTheme } from "./theme";

let uiLocale: UiLocale = "zh";
let uiTheme: UiTheme = "mocha";
let liveQuery: LiveConnectionQuery = defaultLiveQuery();

const FILTER_FIELDS = ["host", "chain", "rule", "process", "source", "destination", "type"] as const;

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
      return `<tr>
        <td>${view.host}</td>
        <td>${view.download}</td>
        <td>${view.upload}</td>
        <td>${view.dlSpeed}</td>
        <td>${view.ulSpeed}</td>
        <td>${view.chains}</td>
        <td>${view.rule}</td>
        <td>${view.process}</td>
        <td>${view.time}</td>
        <td>${view.source}</td>
        <td>${view.destination}</td>
        <td>${view.type}</td>
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
      return `<div class="filter-row">
        <select data-filter-field="${index}">${fields}</select>
        <select data-filter-mode="${index}">
          <option value="contains" ${clause.mode === "contains" ? "selected" : ""}>${tx("live.filter.contains")}</option>
          <option value="exact" ${clause.mode === "exact" ? "selected" : ""}>${tx("live.filter.exact")}</option>
        </select>
        <input data-filter-value="${index}" value="${clause.value.replaceAll('"', "&quot;")}" />
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
        </div>
        ${clauseHtml ? `<div class="filter-clauses">${clauseHtml}</div>` : ""}
      </header>
      <div class="live-table-wrap">
      <table class="data live-table">
        <thead><tr>
          <th>${tx("live.col.host")}</th>
          <th>${tx("live.col.download")}</th>
          <th>${tx("live.col.upload")}</th>
          <th>${tx("live.col.dl_speed")}</th>
          <th>${tx("live.col.ul_speed")}</th>
          <th>${tx("live.col.chains")}</th>
          <th>${tx("live.col.rule")}</th>
          <th>${tx("live.col.process")}</th>
          <th>${tx("live.col.time")}</th>
          <th>${tx("live.col.source")}</th>
          <th>${tx("live.col.destination")}</th>
          <th>${tx("live.col.type")}</th>
          <th>${tx("live.col.action")}</th>
        </tr></thead>
        <tbody>${rowHtml || `<tr><td colspan="13">${emptyText}</td></tr>`}</tbody>
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

function renderReports(report: ReportResult | null, statusZh: string): string {
  const seriesRows =
    report?.series
      .map(
        (point) =>
          `<tr><td>${formatUtc(point.bucketUtc)}</td><td>${formatBytes(point.upload)}</td><td>${formatBytes(point.download)}</td></tr>`
      )
      .join("") ?? "";
  const rankRows =
    report?.rankings
      .map((row) => {
        const max = report.rankings[0]?.download || 1;
        const width = Math.max(4, Math.round((row.download / max) * 100));
        return `<tr><td>${row.label}</td><td>${formatBytes(row.upload)}</td><td>${formatBytes(row.download)}</td><td><span class="bar" style="width:${width}%"></span></td></tr>`;
      })
      .join("") ?? "";
  const capability = report
    ? `<p>${report.drilldownCapability.noteZh} ${fmt("report.tier", { tier: report.dataTier })} ${report.policyMetadata.noteZh}</p>
       <p>${fmt("report.coverage", { status: report.coverage.status, gap: report.coverage.gapSec, unit: report.unit })}</p>`
    : "";
  return `
    <section class="panel">
      <p>${tx("report.same_token")}</p>
      <label class="stack">${tx("report.preset")}
        <select id="report-preset">
          <option value="hour">${tx("report.preset.hour")}</option>
          <option value="day">${tx("report.preset.day")}</option>
          <option value="7">${tx("report.preset.7")}</option>
          <option value="30">${tx("report.preset.30")}</option>
          <option value="month">${tx("report.preset.month")}</option>
        </select>
      </label>
      <label class="stack">${tx("report.granularity")}
        <select id="report-granularity">
          <option value="hour">${tx("report.granularity.hour")}</option>
          <option value="day">${tx("report.granularity.day")}</option>
          <option value="month">${tx("report.granularity.month")}</option>
        </select>
      </label>
      <label class="stack">${tx("report.grouping")}
        <select id="report-grouping">
          <option value="host">${tx("report.grouping.host")}</option>
          <option value="process">${tx("report.grouping.process")}</option>
          <option value="rule">${tx("report.grouping.rule")}</option>
          <option value="chain">${tx("report.grouping.chain")}</option>
          <option value="network">${tx("report.grouping.network")}</option>
          <option value="category">${tx("report.grouping.category")}</option>
        </select>
      </label>
      <button type="button" id="run-report">${tx("report.run")}</button>
      <button type="button" id="export-csv">${tx("report.export_csv")}</button>
      <button type="button" id="export-json">${tx("report.export_json")}</button>
      <button type="button" id="export-html">${tx("report.export_html")}</button>
      <p class="status" data-state="${report ? "connected" : "no_data"}">${statusZh}</p>
      ${capability}
    </section>
    <section class="panel" aria-label="${tx("report.numbers")}">
      <h2>${tx("report.totals")}</h2>
      ${report ? `<p>${fmt("report.totals_line", { up: formatBytes(report.totals.upload, unknownLabel()), down: formatBytes(report.totals.download, unknownLabel()), count: report.totals.connectionCount })}</p>` : `<p>${tx("report.none")}</p>`}
    </section>
    <section class="panel">
      <h2>${tx("report.chart_table")}</h2>
      <table class="data"><thead><tr><th>${tx("report.col.time")}</th><th>${tx("report.col.upload")}</th><th>${tx("report.col.download")}</th></tr></thead>
      <tbody>${seriesRows || `<tr><td colspan="3">${tx("report.empty")}</td></tr>`}</tbody></table>
    </section>
    <section class="panel">
      <h2>${tx("report.topn")}</h2>
      <table class="data"><thead><tr><th>${tx("report.col.name")}</th><th>${tx("report.col.upload")}</th><th>${tx("report.col.download")}</th><th>${tx("report.col.share")}</th></tr></thead>
      <tbody>${rankRows || `<tr><td colspan="4">${tx("report.empty_cap")}</td></tr>`}</tbody></table>
    </section>
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
    uiTheme: "mocha"
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
              ? renderReports(report, reportStatus)
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

  let route: RouteId = boot.branch === "recovery-only" ? "settings-data" : "overview";
  let state = emptyMonitorState();
  let report: ReportResult | null = null;
  let reportStatus = tx("report.idle");
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
  const paint = (): void => {
    renderApp(
      app,
      boot,
      state,
      route,
      report,
      reportStatus,
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
      const page = await queryLiveConnections(liveQuery);
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

  const applyReport = (next: ReportResult | null, status: string): void => {
    report = next;
    reportStatus = status;
    paint();
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
    const query = defaultReportQuery();
    const preset = (document.querySelector("#report-preset") as HTMLSelectElement | null)?.value ?? "hour";
    const now = Math.floor(Date.now() / 1000);
    const spans: Record<string, number> = { hour: 3600, day: 86400, "7": 7 * 86400, "30": 30 * 86400, month: 30 * 86400 };
    query.rangeStartUtc = now - (spans[preset] ?? 3600);
    query.rangeEndUtc = now;
    query.granularity = ((document.querySelector("#report-granularity") as HTMLSelectElement | null)?.value ??
      "hour") as ReportQuery["granularity"];
    query.grouping = ((document.querySelector("#report-grouping") as HTMLSelectElement | null)?.value ??
      "host") as ReportQuery["grouping"];
    return query;
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
        item === index ? { ...clause, field: target.value } : clause
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
      applyReport(report, tx("report.running"));
      try {
        const raw = await invokeCommand<unknown>("run_report", { query: buildQuery() });
        const decoded = decodeReportResult(raw);
        applyReport(decoded, `已生成快照 ${decoded.reportSnapshotToken.slice(0, 8)}。`);
      } catch (error) {
        applyReport(null, error instanceof Error ? error.message : tx("report.fail"));
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
