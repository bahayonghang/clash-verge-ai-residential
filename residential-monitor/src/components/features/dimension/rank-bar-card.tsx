import type { ReportResult } from "../../../dto";
import {
  formatRankLabel,
  missingDimensionLabel,
  rankingTraffic,
  TOP_N_OPTIONS,
  type DimensionKind,
  type TopNOption
} from "../../../format/rank";
import { formatBytes } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { cn } from "../../../lib/utils";
import { RankBar } from "../../charts/rank-bar";
import { OverviewCard } from "../../common/overview-card";
import { Button } from "../../ui/button";
import { CapabilityNote, resolvedCapabilityNote } from "./capability-note";
import { AttributionQualityNote } from "./attribution-quality-note";

export function RankBarCard({
  locale,
  title,
  kind,
  result,
  loading,
  errorZh,
  topN,
  onTopNChange
}: {
  locale: UiLocale;
  title: string;
  kind: DimensionKind;
  result: ReportResult | null;
  loading: boolean;
  errorZh: string | null;
  topN: TopNOption;
  onTopNChange: (next: TopNOption) => void;
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
          label: formatRankLabel(
            row.identity,
            row.label,
            unknown,
            missingDimensionLabel(locale, kind)
          ),
          value: rankingTraffic(row)
        }))
      : [];
  return (
    <OverviewCard
      title={title}
      icon={null}
      action={
        <div className="flex items-center gap-0.5 rounded-lg bg-muted/50 p-0.5">
          {TOP_N_OPTIONS.map((option) => (
            <Button
              key={option}
              type="button"
              variant="ghost"
              size="sm"
              className={cn(
                "h-7 rounded-md px-2.5 text-xs",
                topN === option
                  ? "bg-background font-medium text-primary shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              )}
              onClick={() => onTopNChange(option)}
            >
              {option}
            </Button>
          ))}
        </div>
      }
    >
      {!exactTopN || (errorZh && !result) ? <CapabilityNote locale={locale} noteZh={noteZh} /> : null}
      <AttributionQualityNote locale={locale} result={result} />
      {exactTopN ? (
        <RankBar
          locale={locale}
          data={data}
          loading={loading && data.length === 0}
          emptyHint={errorZh ?? undefined}
          valueFormatter={(value) => formatBytes(value, unknown)}
        />
      ) : null}
    </OverviewCard>
  );
}
