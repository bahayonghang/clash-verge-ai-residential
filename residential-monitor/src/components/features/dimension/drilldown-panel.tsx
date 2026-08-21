import type { ReportResult } from "../../../dto";
import { drilldownTargets, formatRankLabel, rankingTraffic, type DimensionKind } from "../../../format/rank";
import { formatBytes } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { cn, formatTemplate } from "../../../lib/utils";
import { RankBar } from "../../charts/rank-bar";
import { TrendArea } from "../../charts/trend-area";
import { OverviewCard } from "../../common/overview-card";
import { Button } from "../../ui/button";
import { CapabilityNote, resolvedCapabilityNote } from "./capability-note";

export function DrilldownPanel({
  locale,
  kind,
  selected,
  targetKind,
  onTargetKindChange,
  onClear,
  parentResult,
  drillResult,
  drillLoading,
  drillErrorZh
}: {
  locale: UiLocale;
  kind: DimensionKind;
  selected: { identity: string; label: string } | null;
  targetKind: DimensionKind;
  onTargetKindChange: (next: DimensionKind) => void;
  onClear: () => void;
  parentResult: ReportResult | null;
  drillResult: ReportResult | null;
  drillLoading: boolean;
  drillErrorZh: string | null;
}) {
  const unknown = t(locale, "common.unknown");
  const capability = parentResult?.drilldownCapability;
  if (capability && !capability.crossDimension) {
    return (
      <CapabilityNote
        locale={locale}
        noteZh={resolvedCapabilityNote(locale, capability.noteZh, "dimension.no_drill")}
      />
    );
  }

  const targets = drilldownTargets(kind);
  const exactTopN = drillResult?.drilldownCapability.exactTopN !== false;
  const barData =
    drillResult && exactTopN
      ? drillResult.rankings.map((row) => ({
          label: formatRankLabel(row.identity, row.label, unknown),
          value: rankingTraffic(row)
        }))
      : [];

  return (
    <OverviewCard
      title={t(locale, "dimension.drilldown")}
      icon={null}
      action={
        selected ? (
          <Button type="button" variant="ghost" size="sm" onClick={onClear}>
            {t(locale, "dimension.clear")}
          </Button>
        ) : null
      }
    >
      {!selected ? (
        <p className="py-4 text-sm text-muted-foreground">{t(locale, "dimension.pick_row")}</p>
      ) : (
        <div className="space-y-4">
          <p className="text-sm">{formatTemplate(t(locale, "dimension.selected"), { label: selected.label })}</p>
          <div className="flex flex-wrap gap-0.5 rounded-lg bg-muted/50 p-0.5">
            {targets.map((target) => (
              <Button
                key={target}
                type="button"
                variant="ghost"
                size="sm"
                data-drill-target={target}
                className={cn(
                  "h-7 rounded-md px-3 text-xs",
                  targetKind === target
                    ? "bg-background font-medium text-primary shadow-sm"
                    : "text-muted-foreground hover:text-foreground"
                )}
                onClick={() => onTargetKindChange(target)}
              >
                {t(locale, `route.${target}`)}
              </Button>
            ))}
          </div>
          {drillErrorZh && !drillResult ? <CapabilityNote locale={locale} noteZh={drillErrorZh} /> : null}
          {drillResult && !exactTopN ? (
            <CapabilityNote
              locale={locale}
              noteZh={resolvedCapabilityNote(
                locale,
                drillResult.drilldownCapability.noteZh,
                "dimension.exact_top_n_off"
              )}
            />
          ) : null}
          {exactTopN ? (
            <>
              <TrendArea
                locale={locale}
                data={drillResult?.series ?? []}
                loading={drillLoading && (drillResult?.series.length ?? 0) === 0}
                emptyHint={drillErrorZh ?? undefined}
              />
              <RankBar
                locale={locale}
                data={barData}
                loading={drillLoading && barData.length === 0}
                emptyHint={drillErrorZh ?? undefined}
                valueFormatter={(value) => formatBytes(value, unknown)}
              />
            </>
          ) : null}
        </div>
      )}
    </OverviewCard>
  );
}
