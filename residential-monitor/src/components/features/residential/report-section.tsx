import { useEffect, useId, useState } from "react";
import type { ReportQuery, ReportResult } from "../../../dto";
import { RESIDENTIAL_ACCOUNTING_FILTER } from "../../../format/rank";
import { formatUtc } from "../../../format/units";
import { buildReportQuery, granularityForTimeRange, snapTimeRangeToMinute } from "../../../hooks/use-report";
import { renderReportHtml, useReportArchive } from "../../../hooks/use-report-archive";
import { t, type UiLocale } from "../../../i18n";
import { isTauriRuntime } from "../../../ipc/live-session";
import type { TimeRange } from "../../../lib/time-range";
import { invokeErrorZh } from "../../../lib/utils";
import { Button } from "../../ui/button";
import { Card } from "../../ui/card";
import { Switch } from "../../ui/switch";
import { CapabilityNote } from "../dimension/capability-note";
import { CapabilityPanel } from "../reports/capability-panel";
import { CoveragePanel } from "../reports/coverage-panel";
import { ExportPanel } from "../reports/export-panel";
import { residentialReportFilters } from "./aggregate-model";

export function buildResidentialManualQuery(
  timeRange: TimeRange,
  targetPolicy: ReportQuery["targetPolicy"]
): ReportQuery {
  return {
    ...buildReportQuery({
      grouping: "host",
      timeRange,
      granularity: granularityForTimeRange(timeRange.preset),
      topN: 20,
      filters: residentialReportFilters()
    }),
    targetPolicy
  };
}

export function isResidentialManualReport(result: Pick<ReportResult, "queryEcho">): boolean {
  return (
    result.queryEcho.grouping === "host" &&
    result.queryEcho.filters.category === RESIDENTIAL_ACCOUNTING_FILTER
  );
}

function presetLabel(locale: UiLocale, timeRange: TimeRange): string {
  if (timeRange.preset === "today") {
    return t(locale, "time.preset.today");
  }
  return `${t(locale, "time.recent_prefix")} ${t(locale, `time.preset.${timeRange.preset}`)}`;
}

function statusLine(
  locale: UiLocale,
  loading: boolean,
  report: ReportResult | null,
  statusZh: string
): string {
  if (loading) {
    return statusZh;
  }
  if (!report) {
    return t(locale, "residential.report.none");
  }
  return t(locale, "residential.report.ready");
}

export function ReportSection({ locale, timeRange }: { locale: UiLocale; timeRange: TimeRange }) {
  const archive = useReportArchive(locale);
  const titleId = useId();
  const [wantCurrent, setWantCurrent] = useState(false);
  const [html, setHtml] = useState<string | null>(null);
  const [htmlError, setHtmlError] = useState<string | null>(null);
  const [viewerOpen, setViewerOpen] = useState(false);
  const report = archive.report;
  const currentAllowed = report ? report.drilldownCapability.currentPolicy : true;
  const currentOn = wantCurrent && currentAllowed;
  const showCurrentNote = Boolean(report && !report.drilldownCapability.currentPolicy);
  const snapped = snapTimeRangeToMinute(timeRange);
  const windowStart = Math.floor(snapped.startUtc / 1000);
  const windowEnd = Math.floor(snapped.endUtc / 1000);

  useEffect(() => {
    void archive.restoreResidentialManual();
    // 进页只回看一次家宽手动档案。
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!report || !isTauriRuntime()) {
      setHtml(null);
      return;
    }
    let cancelled = false;
    setHtml(null);
    setHtmlError(null);
    void renderReportHtml(report.reportSnapshotToken)
      .then((next) => {
        if (cancelled) {
          return;
        }
        setHtml(next);
      })
      .catch((caught: unknown) => {
        if (cancelled) {
          return;
        }
        setHtml(null);
        setHtmlError(invokeErrorZh(caught, t(locale, "report.export_fail")));
      });
    return () => {
      cancelled = true;
    };
  }, [locale, report]);

  useEffect(() => {
    if (!viewerOpen) {
      return;
    }
    function onKey(event: KeyboardEvent): void {
      if (event.key === "Escape") {
        event.preventDefault();
        setViewerOpen(false);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [viewerOpen]);

  function run(): void {
    const query = buildResidentialManualQuery(
      timeRange,
      currentOn ? "current" : "historical"
    );
    setViewerOpen(false);
    void archive.runQuery(query);
  }

  return (
    <section className="space-y-4" aria-labelledby="residential-report-title">
      <div>
        <h2 id="residential-report-title" className="text-base font-semibold">
          {t(locale, "residential.report")}
        </h2>
        <p className="mt-1 text-xs text-muted-foreground/80">
          {t(locale, "residential.report.policy.note")}
        </p>
      </div>
      <Card className="gap-3 p-4">
        <div className="flex flex-wrap items-center gap-3">
          <label className="flex items-center gap-2 text-sm">
            <Switch
              checked={currentOn}
              disabled={!currentAllowed}
              onCheckedChange={(checked) => setWantCurrent(checked)}
            />
            {t(locale, currentOn ? "residential.report.policy.current" : "residential.report.policy.historical")}
          </label>
          <Button type="button" disabled={archive.loading} onClick={run}>
            {t(locale, "residential.report.create")}
          </Button>
          <Button
            type="button"
            variant="outline"
            className="ml-auto"
            disabled={!html}
            onClick={() => setViewerOpen(true)}
          >
            {t(locale, "residential.report.view")}
          </Button>
        </div>
        <p className="text-xs text-muted-foreground/80">
          {t(locale, "residential.report.window")} {presetLabel(locale, timeRange)} ·{" "}
          <time>{formatUtc(windowStart)}</time>
          {" → "}
          <time>{formatUtc(windowEnd)}</time>
        </p>
        {report ? (
          <>
            <p className="text-xs text-muted-foreground/80">
              {t(locale, "residential.report.created")} <time>{formatUtc(report.generatedUtc)}</time>
            </p>
            <p className="text-xs text-muted-foreground/80">
              {t(locale, "residential.report.frozen")}{" "}
              <time>{formatUtc(report.queryEcho.rangeStartUtc)}</time>
              {" → "}
              <time>{formatUtc(report.queryEcho.rangeEndUtc)}</time>
            </p>
          </>
        ) : null}
        {showCurrentNote ? (
          <CapabilityNote
            locale={locale}
            noteZh={report?.drilldownCapability.noteZh || t(locale, "residential.report.current_off")}
          />
        ) : null}
        <p
          className="text-sm text-muted-foreground"
          data-state={report ? "connected" : "no_data"}
          role="status"
        >
          {statusLine(locale, archive.loading, report, archive.statusZh)}
        </p>
        {archive.errorZh || htmlError ? (
          <p className="text-sm text-destructive" role="alert">
            {archive.errorZh ?? htmlError}
          </p>
        ) : null}
        {report ? (
          <p className="text-xs text-muted-foreground/80">
            {t(locale, "residential.report.policy")} {report.policyMetadata.targetPolicy}
            {report.policyMetadata.policyVersion != null ? ` · v${report.policyMetadata.policyVersion}` : ""}
          </p>
        ) : null}
      </Card>
      {report ? (
        <div className="rounded-xl border bg-card p-4 shadow-xs">
          <CoveragePanel locale={locale} report={report} />
        </div>
      ) : null}
      <CapabilityPanel locale={locale} report={report} />
      <div className="rounded-xl border bg-card p-4 shadow-xs">
        <ExportPanel
          locale={locale}
          preview={archive.exportPreview}
          disabled={!archive.report}
          onPreview={(spec) => void archive.previewExport(spec)}
          onExport={(spec) => void archive.exportReport(spec)}
        />
      </div>
      {viewerOpen && html ? (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
          role="dialog"
          aria-modal="true"
          aria-labelledby={titleId}
          onClick={() => setViewerOpen(false)}
        >
          <div
            className="flex h-[90vh] w-[min(96vw,1100px)] flex-col overflow-hidden rounded-xl border bg-card shadow-lg"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="flex items-center justify-between gap-3 border-b px-4 py-3">
              <h3 id={titleId} className="text-sm font-semibold">
                {t(locale, "residential.report.viewer_title")}
              </h3>
              <Button type="button" variant="outline" size="sm" onClick={() => setViewerOpen(false)}>
                {t(locale, "a11y.close")}
              </Button>
            </div>
            <iframe
              title={t(locale, "residential.report.viewer_title")}
              sandbox=""
              srcDoc={html}
              className="min-h-0 flex-1 bg-background"
            />
          </div>
        </div>
      ) : null}
    </section>
  );
}
