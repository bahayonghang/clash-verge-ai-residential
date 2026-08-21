import { useEffect, useMemo } from "react";
import type { ReportQuery } from "../../../dto";
import { reportShareModel } from "../../../format/report-view";
import { t, type UiLocale } from "../../../i18n";
import { useReport } from "../../../hooks/use-report";
import { useReportArchive, type ReportSource } from "../../../hooks/use-report-archive";
import { timeRangeFromPreset, type TimeRangePreset } from "../../../lib/time-range";
import { ArchiveList } from "./archive-list";
import { CapabilityPanel } from "./capability-panel";
import { CoveragePanel } from "./coverage-panel";
import { ExportPanel } from "./export-panel";
import { ReportInspectProvider } from "./inspect-context";
import { QueryForm } from "./query-form";
import { RankingTable } from "./ranking-table";
import { ShareDonutCard } from "./share-donut-card";
import { TotalsRow } from "./totals-row";
import { TrendCard } from "./trend-card";

function sourceLine(locale: UiLocale, source: ReportSource): string {
  if (source === "manual") {
    return t(locale, "report.archive.kind.manual");
  }
  if (source === "auto-day") {
    return t(locale, "report.archive.kind.day");
  }
  if (source === "auto-hour") {
    return t(locale, "report.archive.kind.hour");
  }
  return "";
}

function presetToTimePreset(preset: string): TimeRangePreset {
  if (preset === "hour") {
    return "1h";
  }
  if (preset === "day") {
    return "24h";
  }
  if (preset === "7") {
    return "7d";
  }
  return "30d";
}

function scrollWorkspaceTop(): void {
  document.getElementById("workspace")?.scrollTo(0, 0);
}

export function ReportsPage({
  locale,
  jumpQuery
}: {
  locale: UiLocale;
  jumpQuery?: ReportQuery | null;
}) {
  const archive = useReportArchive(locale);
  const timeRange = useMemo(
    () => timeRangeFromPreset(presetToTimePreset(archive.form.preset)),
    [archive.form.preset]
  );
  const live = useReport({
    grouping: archive.form.grouping,
    timeRange,
    granularity: archive.form.granularity,
    topN: archive.topN,
    enabled: false
  });
  const report = archive.report ?? live.result;
  const loading = archive.loading || live.loading;
  const errorZh = archive.errorZh ?? live.errorZh;
  const share = useMemo(
    () =>
      report
        ? reportShareModel(report, {
            unknown: t(locale, "common.unknown"),
            remainder: t(locale, "report.remainder")
          })
        : null,
    [locale, report]
  );

  useEffect(() => {
    void archive.loadArchives(true);
    // 进页只拉一次最新档案。
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    void archive.loadArchives(false);
    // 筛选变化后重拉列表，不自动改选中项。
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [archive.archiveKindFilter]);

  useEffect(() => {
    scrollWorkspaceTop();
  }, [archive.form.preset, archive.selectedArchiveId]);

  useEffect(() => {
    if (jumpQuery) {
      void archive.runQuery(jumpQuery);
    }
    // jumpQuery 由告警证据触发。
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [jumpQuery]);

  const source = sourceLine(locale, archive.reportSource);

  return (
    <div className="space-y-6">
      <QueryForm
        locale={locale}
        form={archive.form}
        topN={archive.topN}
        compare={archive.compare}
        loading={loading}
        onForm={archive.setForm}
        onTopN={archive.setTopN}
        onCompare={archive.setCompare}
        onRun={() => void archive.runManual()}
      />
      <p className="text-sm" data-state={report ? "connected" : "no_data"}>
        {archive.statusZh}
      </p>
      {source ? <p className="text-sm text-muted-foreground">{source}</p> : null}
      {errorZh ? (
        <p className="text-sm text-destructive" role="alert">
          {errorZh}
        </p>
      ) : null}
      <ReportInspectProvider locale={locale} share={share} series={report?.series ?? []}>
        <TotalsRow locale={locale} report={report} />
        <div className="grid gap-4 lg:grid-cols-2">
          <TrendCard locale={locale} series={report?.series ?? []} loading={loading} />
          <ShareDonutCard locale={locale} share={share} />
        </div>
        <RankingTable locale={locale} share={share} />
        <CoveragePanel locale={locale} report={report} />
        <CapabilityPanel locale={locale} report={report} />
        <ExportPanel
          locale={locale}
          preview={archive.exportPreview}
          disabled={!report}
          onPreview={(spec) => void archive.previewExport(spec)}
          onExport={(spec) => void archive.exportReport(spec)}
        />
      </ReportInspectProvider>
      <ArchiveList
        locale={locale}
        archives={archive.archives}
        filter={archive.archiveKindFilter}
        selectedId={archive.selectedArchiveId}
        onFilter={archive.setArchiveKindFilter}
        onSelect={(id) => void archive.selectArchive(id)}
      />
    </div>
  );
}
