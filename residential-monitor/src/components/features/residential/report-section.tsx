import { useState } from "react";
import type { ReportQuery } from "../../../dto";
import { buildReportQuery, granularityForTimeRange } from "../../../hooks/use-report";
import { useReportArchive } from "../../../hooks/use-report-archive";
import { t, type UiLocale } from "../../../i18n";
import type { TimeRange } from "../../../lib/time-range";
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

export function ReportSection({ locale, timeRange }: { locale: UiLocale; timeRange: TimeRange }) {
  const archive = useReportArchive(locale);
  const [wantCurrent, setWantCurrent] = useState(false);
  const report = archive.report;
  const currentAllowed = report ? report.drilldownCapability.currentPolicy : true;
  const currentOn = wantCurrent && currentAllowed;
  const showCurrentNote = Boolean(report && !report.drilldownCapability.currentPolicy);

  function run(): void {
    const query = buildResidentialManualQuery(
      timeRange,
      currentOn ? "current" : "historical"
    );
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
            {t(locale, "report.run")}
          </Button>
        </div>
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
          {archive.statusZh}
        </p>
        {archive.errorZh ? (
          <p className="text-sm text-destructive" role="alert">
            {archive.errorZh}
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
    </section>
  );
}
