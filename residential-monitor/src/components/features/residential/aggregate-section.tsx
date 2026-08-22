import { useState } from "react";
import type { ReportResult, ResidentialShare } from "../../../dto";
import { rankDisplayLabel, rankingShare, rankingTraffic, TOP_N_OPTIONS, type TopNOption } from "../../../format/rank";
import { formatBytes, formatUtc } from "../../../format/units";
import { granularityForTimeRange, useReport } from "../../../hooks/use-report";
import { t, type UiLocale } from "../../../i18n";
import type { TimeRange } from "../../../lib/time-range";
import { RankBar } from "../../charts/rank-bar";
import { TrendArea } from "../../charts/trend-area";
import { OverviewCard } from "../../common/overview-card";
import { CapabilityNote, resolvedCapabilityNote } from "../dimension/capability-note";
import { CaliberNote } from "./caliber-note";
import { ShareReadout } from "./share-readout";

export function AggregateSection({
  locale,
  timeRange,
  share,
  shareLoading,
  shareError
}: {
  locale: UiLocale;
  timeRange: TimeRange;
  share: ResidentialShare | null;
  shareLoading: boolean;
  shareError: string | null;
}) {
  const [topN, setTopN] = useState<TopNOption>(20);
  const granularity = granularityForTimeRange(timeRange.preset);
  const report = useReport({
    grouping: "category",
    timeRange,
    granularity,
    topN
  });
  return (
    <section className="space-y-4" aria-labelledby="residential-aggregate-title">
      <div>
        <h2 id="residential-aggregate-title" className="text-sm font-semibold">
          {t(locale, "residential.aggregate")}
        </h2>
        <CaliberNote locale={locale} kind="accounting" />
      </div>
      <ShareReadout locale={locale} share={share} loading={shareLoading} errorZh={shareError} />
      <RankBlock locale={locale} result={report.result} loading={report.loading} errorZh={report.errorZh} topN={topN} onTopN={setTopN} />
      <TrendBlock locale={locale} result={report.result} loading={report.loading} errorZh={report.errorZh} />
    </section>
  );
}

function RankBlock({
  locale,
  result,
  loading,
  errorZh,
  topN,
  onTopN
}: {
  locale: UiLocale;
  result: ReportResult | null;
  loading: boolean;
  errorZh: string | null;
  topN: TopNOption;
  onTopN: (next: TopNOption) => void;
}) {
  const unknown = t(locale, "common.unknown");
  const exactTopN = result?.drilldownCapability.exactTopN !== false;
  const noteZh = resolvedCapabilityNote(
    locale,
    errorZh ?? result?.drilldownCapability.noteZh,
    "dimension.exact_top_n_off"
  );
  const data =
    result && exactTopN
      ? result.rankings.map((row) => ({
          label: rankDisplayLabel(row.identity, row.label, unknown),
          value: rankingTraffic(row)
        }))
      : [];
  return (
    <OverviewCard
      title={t(locale, "residential.rank")}
      icon={null}
      action={
        <div className="flex items-center gap-0.5 rounded-lg bg-muted/50 p-0.5">
          {TOP_N_OPTIONS.map((option) => (
            <button
              key={option}
              type="button"
              className={
                topN === option
                  ? "h-7 rounded-md bg-background px-2.5 text-xs font-medium text-primary shadow-sm"
                  : "h-7 rounded-md px-2.5 text-xs text-muted-foreground"
              }
              onClick={() => onTopN(option)}
            >
              {option}
            </button>
          ))}
        </div>
      }
    >
      {!exactTopN || (errorZh && !result) ? <CapabilityNote locale={locale} noteZh={noteZh} /> : null}
      {exactTopN ? (
        <RankBar
          locale={locale}
          data={data}
          loading={loading && data.length === 0}
          emptyHint={errorZh ?? undefined}
          valueFormatter={(value) => formatBytes(value, unknown)}
        />
      ) : null}
      {exactTopN ? (
        <div className="mt-3 overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border/60 text-left text-muted-foreground">
                <th className="py-2 font-medium">{t(locale, "overview.col.name")}</th>
                <th className="py-2 font-medium">{t(locale, "overview.col.upload")}</th>
                <th className="py-2 font-medium">{t(locale, "overview.col.download")}</th>
                <th className="py-2 font-medium">{t(locale, "report.col.share")}</th>
              </tr>
            </thead>
            <tbody>
              {(result?.rankings ?? []).length === 0 ? (
                <tr>
                  <td className="py-3 text-muted-foreground" colSpan={4}>
                    {loading ? t(locale, "report.running") : t(locale, "dimension.empty")}
                  </td>
                </tr>
              ) : (
                (result?.rankings ?? []).map((row) => {
                  const label = rankDisplayLabel(row.identity, row.label, unknown);
                  const share = rankingShare(row.download, result?.totals.download ?? 0);
                  return (
                    <tr key={row.identity} data-identity={row.identity} className="border-b border-border/40 last:border-0">
                      <td className="py-2">{label}</td>
                      <td className="py-2 tabular-nums">{formatBytes(row.upload, unknown)}</td>
                      <td className="py-2 tabular-nums">{formatBytes(row.download, unknown)}</td>
                      <td className="py-2 tabular-nums">
                        {result && result.totals.download > 0 ? `${(share * 100).toFixed(1)}%` : unknown}
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>
      ) : null}
    </OverviewCard>
  );
}

function TrendBlock({
  locale,
  result,
  loading,
  errorZh
}: {
  locale: UiLocale;
  result: ReportResult | null;
  loading: boolean;
  errorZh: string | null;
}) {
  const unknown = t(locale, "common.unknown");
  const series = result?.series ?? [];
  return (
    <OverviewCard title={t(locale, "overview.trend")} icon={null}>
      {errorZh && !result ? <CapabilityNote locale={locale} noteZh={errorZh} /> : null}
      <TrendArea
        locale={locale}
        data={series}
        loading={loading && series.length === 0}
        emptyHint={errorZh ?? undefined}
      />
      <div className="mt-3 max-h-56 overflow-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-border/60 text-left text-muted-foreground">
              <th className="py-2 font-medium">{t(locale, "report.col.time")}</th>
              <th className="py-2 font-medium">{t(locale, "report.col.upload")}</th>
              <th className="py-2 font-medium">{t(locale, "report.col.download")}</th>
            </tr>
          </thead>
          <tbody>
            {series.length === 0 ? (
              <tr>
                <td className="py-3 text-muted-foreground" colSpan={3}>
                  {loading ? t(locale, "report.running") : t(locale, "chart.empty")}
                </td>
              </tr>
            ) : (
              series.map((point) => (
                <tr key={point.bucketUtc} className="border-b border-border/40 last:border-0">
                  <td className="py-2">{formatUtc(point.bucketUtc)}</td>
                  <td className="py-2 tabular-nums">{formatBytes(point.upload, unknown)}</td>
                  <td className="py-2 tabular-nums">{formatBytes(point.download, unknown)}</td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </OverviewCard>
  );
}
