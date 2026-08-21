import { useEffect, useMemo, useState } from "react";
import { Cpu, Globe, Link2, Route as RouteIcon, type LucideIcon } from "lucide-react";
import {
  drilldownTargets,
  filtersForDrilldown,
  isUnknownIdentity,
  type DimensionKind,
  type TopNOption
} from "../../../format/rank";
import { t, type UiLocale } from "../../../i18n";
import type { TimeRange } from "../../../lib/time-range";
import { granularityForTimeRange, useReport } from "../../../hooks/use-report";
import { DrilldownPanel } from "./drilldown-panel";
import { RankBarCard } from "./rank-bar-card";
import { RankTable } from "./rank-table";

const KIND_ICON: Record<DimensionKind, LucideIcon> = {
  host: Globe,
  rule: RouteIcon,
  chain: Link2,
  process: Cpu
};

export function DimensionPage({
  locale,
  kind,
  timeRange
}: {
  locale: UiLocale;
  kind: DimensionKind;
  timeRange: TimeRange;
}) {
  const [topN, setTopN] = useState<TopNOption>(20);
  const [selected, setSelected] = useState<{ identity: string; label: string } | null>(null);
  const targets = drilldownTargets(kind);
  const [targetKind, setTargetKind] = useState<DimensionKind>(targets[0]);
  const granularity = granularityForTimeRange(timeRange.preset);

  useEffect(() => {
    setSelected(null);
    setTargetKind(drilldownTargets(kind)[0]);
  }, [kind]); // kind 变化时清掉上一维选中行，避免 filters 串维。
  const parent = useReport({
    grouping: kind,
    timeRange,
    granularity,
    topN
  });
  const drillFilters = useMemo(
    () => (selected ? filtersForDrilldown(kind, selected.identity) : undefined),
    [kind, selected]
  );
  const canDrill = parent.result?.drilldownCapability.crossDimension === true && selected !== null;
  const drill = useReport({
    grouping: targetKind,
    timeRange,
    granularity,
    topN,
    filters: drillFilters,
    enabled: canDrill
  });
  const Icon = KIND_ICON[kind];
  const title = t(locale, `route.${kind}`);

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-2">
        <Icon className="h-5 w-5 text-muted-foreground" />
        <h1 className="text-lg font-semibold">{title}</h1>
      </div>
      <RankBarCard
        locale={locale}
        title={title}
        result={parent.result}
        loading={parent.loading}
        errorZh={parent.errorZh}
        topN={topN}
        onTopNChange={setTopN}
      />
      <RankTable
        locale={locale}
        kind={kind}
        result={parent.result}
        loading={parent.loading}
        errorZh={parent.errorZh}
        selectedIdentity={selected?.identity ?? null}
        onSelect={(identity, label) => {
          if (isUnknownIdentity(identity)) {
            return;
          }
          setSelected({ identity, label });
          setTargetKind(targets[0]);
        }}
      />
      <DrilldownPanel
        locale={locale}
        kind={kind}
        selected={selected}
        targetKind={targetKind}
        onTargetKindChange={setTargetKind}
        onClear={() => setSelected(null)}
        parentResult={parent.result}
        drillResult={drill.result}
        drillLoading={drill.loading}
        drillErrorZh={drill.errorZh}
      />
    </div>
  );
}
