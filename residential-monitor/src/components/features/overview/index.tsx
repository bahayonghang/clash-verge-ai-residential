import { useEffect, useState } from "react";
import type { LiveOverview, RouteId } from "../../../dto";
import type { TopSort } from "../../../format/rank";
import type { UiLocale } from "../../../i18n";
import { timeRangeFromPreset, type TimeRange } from "../../../lib/time-range";
import {
  granularityForTimeRange,
  granularityForTrendPreset,
  isTrendPreset,
  useReport,
  type TrendPreset
} from "../../../hooks/use-report";
import { CaliberGrid } from "./caliber-grid";
import { CategoryTable } from "./category-table";
import { TopColumns } from "./top-columns";
import { TrendCard } from "./trend-card";

export function OverviewPage({
  locale,
  timeRange,
  overview,
  onNavigate
}: {
  locale: UiLocale;
  timeRange: TimeRange;
  overview: LiveOverview;
  onNavigate: (route: RouteId) => void;
}) {
  const [trendOverride, setTrendOverride] = useState<TrendPreset | null>(null);
  const [hostSort, setHostSort] = useState<TopSort>("traffic");
  const [chainSort, setChainSort] = useState<TopSort>("traffic");
  const [processSort, setProcessSort] = useState<TopSort>("traffic");

  useEffect(() => {
    setTrendOverride(null);
  }, [timeRange.preset, timeRange.startUtc, timeRange.endUtc]);

  const queryRange = trendOverride ? timeRangeFromPreset(trendOverride) : timeRange;
  const granularity = trendOverride
    ? granularityForTrendPreset(trendOverride)
    : granularityForTimeRange(timeRange.preset);
  const activePreset: TrendPreset | null = trendOverride
    ? trendOverride
    : isTrendPreset(timeRange.preset)
      ? timeRange.preset
      : null;

  const host = useReport({ grouping: "host", timeRange: queryRange, granularity, topN: 10 });
  const chain = useReport({ grouping: "chain", timeRange: queryRange, granularity, topN: 10 });
  const process = useReport({ grouping: "process", timeRange: queryRange, granularity, topN: 10 });
  // 趋势图复用 host 的 series / totals，不发第四次查询。

  return (
    <div className="space-y-6">
      <CaliberGrid locale={locale} overview={overview} />
      <TrendCard
        locale={locale}
        result={host.result}
        loading={host.loading}
        errorZh={host.errorZh}
        activePreset={activePreset}
        onPresetChange={setTrendOverride}
      />
      <TopColumns
        locale={locale}
        host={host}
        chain={chain}
        process={process}
        hostSort={hostSort}
        chainSort={chainSort}
        processSort={processSort}
        onHostSort={setHostSort}
        onChainSort={setChainSort}
        onProcessSort={setProcessSort}
        onNavigate={onNavigate}
      />
      <CategoryTable locale={locale} overview={overview} />
    </div>
  );
}
