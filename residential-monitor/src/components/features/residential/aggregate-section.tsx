import { useState } from "react";
import type { ReportResult, ResidentialShare } from "../../../dto";
import { formatRankLabel, rankingShare, TOP_N_OPTIONS, type TopNOption } from "../../../format/rank";
import { formatBytes, formatUtc } from "../../../format/units";
import { granularityForTimeRange, useReport } from "../../../hooks/use-report";
import { t, type UiLocale } from "../../../i18n";
import type { TimeRange } from "../../../lib/time-range";
import { RankBar } from "../../charts/rank-bar";
import { TrendArea } from "../../charts/trend-area";
import { OverviewCard } from "../../common/overview-card";
import { SortableTh } from "../../common/sortable-th";
import {
  directionTraffic,
  matchesResidentialRankQuery,
  residentialAggregateState,
  residentialReportFilters,
  shouldShowResidentialRankLoading,
  type ResidentialDirection
} from "./aggregate-model";
import { CaliberNote } from "./caliber-note";
import { ShareReadout } from "./share-readout";
import { TrendTable } from "./trend-table";

export function AggregateSection({
  locale,
  timeRange,
  autoRefresh,
  share,
  shareLoading,
  shareError
}: {
  locale: UiLocale;
  timeRange: TimeRange;
  autoRefresh: boolean;
  share: ResidentialShare | null;
  shareLoading: boolean;
  shareError: string | null;
}) {
  const [topN, setTopN] = useState<TopNOption>(20);
  const [direction, setDirection] = useState<ResidentialDirection>("download");
  const granularity = granularityForTimeRange(timeRange.preset);
  const report = useReport({
    grouping: "host",
    timeRange,
    granularity,
    topN,
    filters: residentialReportFilters(),
    sort: { field: direction, descending: true }
  });
  const rankResult = matchesResidentialRankQuery(report.result, direction, topN)
    ? report.result
    : null;
  const rankLoading = shouldShowResidentialRankLoading(
    report.loading,
    report.errorZh,
    report.result !== null,
    rankResult !== null
  );
  const aggregateState = residentialAggregateState(
    report.result,
    report.loading,
    report.errorZh,
    autoRefresh
  );
  const emptyHint =
    aggregateState === "ready" || aggregateState === "paused"
      ? t(locale, "dimension.empty")
      : t(locale, `residential.aggregate.state.${aggregateState}`);
  return (
    <section className="space-y-4" aria-labelledby="residential-aggregate-title">
      <div>
        <h2 id="residential-aggregate-title" className="text-sm font-semibold">
          {t(locale, "residential.aggregate")}
        </h2>
        <CaliberNote locale={locale} kind="accounting" />
        <AggregateStatus
          locale={locale}
          result={report.result}
          state={aggregateState}
          errorZh={report.errorZh}
          autoRefresh={autoRefresh}
        />
      </div>
      <ShareReadout locale={locale} share={share} loading={shareLoading} errorZh={shareError} />
      <RankBlock
        locale={locale}
        result={rankResult}
        loading={rankLoading}
        emptyHint={emptyHint}
        topN={topN}
        onTopN={setTopN}
        direction={direction}
        onDirection={setDirection}
      />
      <TrendBlock
        locale={locale}
        result={report.result}
        loading={report.loading}
        emptyHint={emptyHint}
      />
    </section>
  );
}

export function AggregateStatus({
  locale,
  result,
  state,
  errorZh,
  autoRefresh
}: {
  locale: UiLocale;
  result: ReportResult | null;
  state: ReturnType<typeof residentialAggregateState>;
  errorZh: string | null;
  autoRefresh: boolean;
}) {
  return (
    <div className="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
      {result ? (
        <p>
          {t(locale, "residential.aggregate.window")}
          {locale === "zh" ? "：" : ": "}
          <time>{formatUtc(result.queryEcho.rangeStartUtc)}</time>
          {" — "}
          <time>{formatUtc(result.queryEcho.rangeEndUtc)}</time>
        </p>
      ) : null}
      {result ? (
        <p>
          {t(locale, "residential.aggregate.updated")}
          {locale === "zh" ? "：" : ": "}
          <time>{formatUtc(result.generatedUtc)}</time>
        </p>
      ) : null}
      <p
        data-state={state}
        className={state === "error" ? "text-destructive" : undefined}
        role={state === "error" ? "alert" : "status"}
        aria-live="polite"
      >
        {t(locale, `residential.aggregate.state.${state}`)}
        {state === "error" && errorZh ? ` ${errorZh}` : ""}
        {state === "unsupported" && result?.drilldownCapability.noteZh
          ? ` ${result.drilldownCapability.noteZh}`
          : ""}
        {!autoRefresh && state !== "paused"
          ? ` ${t(locale, "residential.aggregate.state.paused")}`
          : ""}
      </p>
    </div>
  );
}

function RankBlock({
  locale,
  result,
  loading,
  emptyHint,
  topN,
  onTopN,
  direction,
  onDirection
}: {
  locale: UiLocale;
  result: ReportResult | null;
  loading: boolean;
  emptyHint: string;
  topN: TopNOption;
  onTopN: (next: TopNOption) => void;
  direction: ResidentialDirection;
  onDirection: (next: ResidentialDirection) => void;
}) {
  const unknown = t(locale, "common.unknown");
  const missingHost = t(locale, "dimension.missing.host");
  const exactTopN = result?.drilldownCapability.exactTopN !== false;
  const data =
    result && exactTopN
      ? result.rankings.map((row) => ({
          label: formatRankLabel(row.identity, row.label, unknown, missingHost),
          value: directionTraffic(row, direction)
        }))
      : [];
  const total = result?.totals[direction] ?? 0;
  return (
    <OverviewCard
      title={t(locale, "residential.rank")}
      icon={null}
      action={
        <div className="flex flex-wrap items-center justify-end gap-2">
          <div
            className="flex items-center gap-0.5 rounded-lg bg-muted/50 p-0.5"
            role="group"
            aria-label={t(locale, "residential.rank.direction")}
          >
            {(["download", "upload"] as const).map((option) => (
              <button
                key={option}
                type="button"
                aria-pressed={direction === option}
                className={
                  direction === option
                    ? "h-7 rounded-md bg-background px-2.5 text-xs font-medium text-primary shadow-sm"
                    : "h-7 rounded-md px-2.5 text-xs text-muted-foreground"
                }
                onClick={() => onDirection(option)}
              >
                {t(locale, `residential.rank.direction.${option}`)}
              </button>
            ))}
          </div>
          <div className="flex items-center gap-0.5 rounded-lg bg-muted/50 p-0.5">
            {TOP_N_OPTIONS.map((option) => (
              <button
                key={option}
                type="button"
                aria-pressed={topN === option}
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
        </div>
      }
    >
      {exactTopN ? (
        <RankBar
          locale={locale}
          data={data}
          loading={loading && data.length === 0}
          emptyHint={emptyHint}
          valueFormatter={(value) => formatBytes(value, unknown)}
        />
      ) : null}
      {exactTopN ? (
        <div className="mt-3 overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border/60 text-left text-muted-foreground">
                <th className="py-2 font-medium">{t(locale, "overview.col.name")}</th>
                <SortableTh
                  label={t(locale, "overview.col.upload")}
                  ariaSort={direction === "upload" ? "descending" : "none"}
                  numeric
                  onClick={() => onDirection("upload")}
                />
                <SortableTh
                  label={t(locale, "overview.col.download")}
                  ariaSort={direction === "download" ? "descending" : "none"}
                  numeric
                  onClick={() => onDirection("download")}
                />
                <th className="px-2 py-2 text-right font-medium">
                  {t(locale, `residential.rank.share.${direction}`)}
                </th>
              </tr>
            </thead>
            <tbody>
              {(result?.rankings ?? []).length === 0 ? (
                <tr>
                  <td className="py-3 text-muted-foreground" colSpan={4}>
                    {loading ? t(locale, "report.running") : emptyHint}
                  </td>
                </tr>
              ) : (
                (result?.rankings ?? []).map((row) => {
                  const label = formatRankLabel(row.identity, row.label, unknown, missingHost);
                  const share = rankingShare(directionTraffic(row, direction), total);
                  return (
                    <tr key={row.identity} data-identity={row.identity} className="border-b border-border/40 last:border-0">
                      <td className="py-2">{label}</td>
                      <td className="px-2 py-2 text-right tabular-nums">{formatBytes(row.upload, unknown)}</td>
                      <td className="px-2 py-2 text-right tabular-nums">{formatBytes(row.download, unknown)}</td>
                      <td className="px-2 py-2 text-right tabular-nums">
                        {result && total > 0 ? `${(share * 100).toFixed(1)}%` : unknown}
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
  emptyHint
}: {
  locale: UiLocale;
  result: ReportResult | null;
  loading: boolean;
  emptyHint: string;
}) {
  const series = result?.series ?? [];
  return (
    <OverviewCard title={t(locale, "overview.trend")} icon={null}>
      <TrendArea
        locale={locale}
        data={series}
        loading={loading && series.length === 0}
        emptyHint={emptyHint}
      />
      <TrendTable locale={locale} series={series} loading={loading} />
    </OverviewCard>
  );
}
