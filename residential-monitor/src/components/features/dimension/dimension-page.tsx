import { useEffect, useMemo, useState } from "react";
import { Cpu, Globe, Link2, Route as RouteIcon, type LucideIcon } from "lucide-react";
import {
  drilldownTargets,
  emptyReportFilters,
  filtersForDrilldown,
  isUnknownIdentity,
  RESIDENTIAL_ACCOUNTING_FILTER,
  type DimensionKind,
  type TopNOption
} from "../../../format/rank";
import type { LiveOverview } from "../../../dto";
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
  timeRange,
  overview
}: {
  locale: UiLocale;
  kind: DimensionKind;
  timeRange: TimeRange;
  overview: LiveOverview;
}) {
  const [topN, setTopN] = useState<TopNOption>(20);
  const [selected, setSelected] = useState<{ identity: string; label: string } | null>(null);
  const [residentialOnly, setResidentialOnly] = useState(false);
  const targets = drilldownTargets(kind);
  const [targetKind, setTargetKind] = useState<DimensionKind>(targets[0]);
  const granularity = granularityForTimeRange(timeRange.preset);

  useEffect(() => {
    setSelected(null);
    setTargetKind(drilldownTargets(kind)[0]);
    setResidentialOnly(false);
  }, [kind]); // kind 变化时清掉上一维选中行，避免 filters 串维。
  const parentFilters = useMemo(() => {
    if (kind !== "process" || !residentialOnly) {
      return undefined;
    }
    return { ...emptyReportFilters(), category: RESIDENTIAL_ACCOUNTING_FILTER };
  }, [kind, residentialOnly]);
  const parent = useReport({
    grouping: kind,
    timeRange,
    granularity,
    topN,
    filters: parentFilters
  });
  const drillFilters = useMemo(() => {
    if (!selected) {
      return undefined;
    }
    const base = parentFilters ?? emptyReportFilters();
    return filtersForDrilldown(kind, selected.identity, base);
  }, [kind, parentFilters, selected]);
  const canDrill =
    parent.result?.drilldownCapability.crossDimension === true &&
    selected !== null &&
    (!isUnknownIdentity(selected.identity) || kind === "host" || kind === "process");
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
      <div className="flex flex-wrap items-center gap-3">
        <div className="flex items-center gap-2">
          <Icon className="h-5 w-5 text-muted-foreground" />
          <h1 className="text-lg font-semibold">{title}</h1>
        </div>
        {kind === "process" ? (
          <label className="flex items-center gap-2 text-sm text-muted-foreground">
            <input
              type="checkbox"
              data-residential-only="1"
              checked={residentialOnly}
              onChange={(event) => setResidentialOnly(event.target.checked)}
            />
            {t(locale, "dimension.residential_only")}
          </label>
        ) : null}
      </div>
      <RankBarCard
        locale={locale}
        title={title}
        kind={kind}
        result={parent.result}
        loading={parent.loading}
        errorZh={parent.errorZh}
        topN={topN}
        onTopNChange={setTopN}
        coverage={overview.metadataCoverage}
      />
      <RankTable
        locale={locale}
        kind={kind}
        result={parent.result}
        loading={parent.loading}
        errorZh={parent.errorZh}
        selectedIdentity={selected?.identity ?? null}
        onSelect={(identity, label) => {
          if (isUnknownIdentity(identity) && kind !== "host" && kind !== "process") {
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
