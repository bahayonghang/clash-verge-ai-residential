import { Activity } from "lucide-react";
import type { ReportResult } from "../../../dto";
import { formatBytes } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { cn, formatTemplate } from "../../../lib/utils";
import { TREND_PRESETS, type TrendPreset } from "../../../hooks/use-report";
import { TrendArea } from "../../charts/trend-area";
import { OverviewCard } from "../../common/overview-card";
import { Button } from "../../ui/button";
import { CapabilityNote } from "../dimension/capability-note";

export function TrendCard({
  locale,
  result,
  loading,
  errorZh,
  activePreset,
  onPresetChange
}: {
  locale: UiLocale;
  result: ReportResult | null;
  loading: boolean;
  errorZh: string | null;
  activePreset: TrendPreset | null;
  onPresetChange: (preset: TrendPreset) => void;
}) {
  const unknown = t(locale, "common.unknown");
  const series = result?.series ?? [];
  const totals = result?.totals;
  const coverage = result
    ? formatTemplate(t(locale, "report.coverage"), {
        status: result.coverage.status,
        gap: result.coverage.gapSec,
        unit: result.unit
      })
    : null;
  return (
    <OverviewCard
      title={t(locale, "overview.trend")}
      icon={<Activity className="h-4 w-4" />}
      action={
        <div className="flex items-center gap-0.5 rounded-lg bg-muted/50 p-0.5">
          {TREND_PRESETS.map((preset) => (
            <Button
              key={preset}
              type="button"
              variant="ghost"
              size="sm"
              className={cn(
                "h-7 rounded-md px-3 text-xs",
                activePreset === preset
                  ? "bg-background font-medium text-primary shadow-sm"
                  : "text-muted-foreground hover:text-foreground"
              )}
              onClick={() => onPresetChange(preset)}
            >
              {t(locale, `time.preset.${preset}`)}
            </Button>
          ))}
        </div>
      }
    >
      {errorZh && !result ? <CapabilityNote locale={locale} noteZh={errorZh} /> : null}
      {errorZh && result ? (
        <p className="mb-2 text-xs text-destructive" role="alert">
          {errorZh}
        </p>
      ) : null}
      {totals ? (
        <div className="mb-3 flex flex-wrap items-center gap-4 text-xs">
          <span>
            <span className="text-muted-foreground">{t(locale, "overview.dir.down")}：</span>
            <span className="font-semibold tabular-nums" style={{ color: "#3b82f6" }}>
              {formatBytes(totals.download, unknown)}
            </span>
          </span>
          <span>
            <span className="text-muted-foreground">{t(locale, "overview.dir.up")}：</span>
            <span className="font-semibold tabular-nums" style={{ color: "#a855f7" }}>
              {formatBytes(totals.upload, unknown)}
            </span>
          </span>
          {coverage ? <span className="ml-auto text-muted-foreground">{coverage}</span> : null}
        </div>
      ) : null}
      <TrendArea
        locale={locale}
        data={series}
        loading={loading && series.length === 0}
        emptyHint={errorZh ?? undefined}
      />
    </OverviewCard>
  );
}
