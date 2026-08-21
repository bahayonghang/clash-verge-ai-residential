import type { ReactNode } from "react";
import { ArrowRight, BarChart3, Cpu, Globe, Link2 } from "lucide-react";
import type { ReportResult, RouteId } from "../../../dto";
import { rankDisplayLabel, rankingSortValue, rankingTraffic, type TopSort } from "../../../format/rank";
import { formatBytes } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { cn } from "../../../lib/utils";
import { OverviewCard } from "../../common/overview-card";
import { TopListItem } from "../../common/top-list-item";
import { Button } from "../../ui/button";
import { Skeleton } from "../../ui/skeleton";
import { CapabilityNote, resolvedCapabilityNote } from "../dimension/capability-note";

function SortToggle({
  locale,
  value,
  onChange
}: {
  locale: UiLocale;
  value: TopSort;
  onChange: (next: TopSort) => void;
}) {
  return (
    <div className="flex items-center gap-0.5 rounded-lg bg-muted/50 p-0.5">
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className={cn(
          "h-7 w-7 rounded-md",
          value === "traffic" ? "bg-background text-primary shadow-sm" : "text-muted-foreground"
        )}
        aria-label={t(locale, "overview.sort.traffic")}
        title={t(locale, "overview.sort.traffic")}
        onClick={() => onChange("traffic")}
      >
        <BarChart3 className="h-4 w-4" />
      </Button>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        className={cn(
          "h-7 w-7 rounded-md",
          value === "connections" ? "bg-background text-primary shadow-sm" : "text-muted-foreground"
        )}
        aria-label={t(locale, "overview.sort.connections")}
        title={t(locale, "overview.sort.connections")}
        onClick={() => onChange("connections")}
      >
        <Link2 className="h-4 w-4" />
      </Button>
    </div>
  );
}

function TopColumn({
  locale,
  title,
  icon,
  color,
  result,
  loading,
  errorZh,
  sort,
  onSortChange,
  onViewAll
}: {
  locale: UiLocale;
  title: string;
  icon: ReactNode;
  color: string;
  result: ReportResult | null;
  loading: boolean;
  errorZh: string | null;
  sort: TopSort;
  onSortChange: (next: TopSort) => void;
  onViewAll: () => void;
}) {
  const unknown = t(locale, "common.unknown");
  const rankings = result?.rankings ?? [];
  const sorted = [...rankings].sort(
    (left, right) => rankingSortValue(right, sort) - rankingSortValue(left, sort)
  );
  const total =
    sort === "traffic"
      ? (result?.totals.upload ?? 0) + (result?.totals.download ?? 0)
      : (result?.totals.connectionCount ?? 0);
  const exactTopN = result?.drilldownCapability.exactTopN !== false;
  return (
    <OverviewCard
      title={title}
      icon={icon}
      action={<SortToggle locale={locale} value={sort} onChange={onSortChange} />}
      footer={
        <Button type="button" variant="ghost" size="sm" className="h-8 px-0" onClick={onViewAll}>
          {t(locale, "overview.view_all")}
          <ArrowRight className="h-4 w-4" />
        </Button>
      }
    >
      {errorZh && !result ? <CapabilityNote locale={locale} noteZh={errorZh} /> : null}
      {result && !exactTopN ? (
        <CapabilityNote
          locale={locale}
          noteZh={resolvedCapabilityNote(locale, result.drilldownCapability.noteZh, "dimension.exact_top_n_off")}
        />
      ) : null}
      {loading && rankings.length === 0 ? (
        <div className="space-y-3">
          {[1, 2, 3, 4, 5].map((row) => (
            <Skeleton key={row} className="h-10 w-full" />
          ))}
        </div>
      ) : null}
      {exactTopN && !loading && rankings.length === 0 && !errorZh ? (
        <p className="py-6 text-center text-sm text-muted-foreground">{t(locale, "report.empty")}</p>
      ) : null}
      {exactTopN
        ? sorted.map((row, index) => {
            const value = sort === "traffic" ? rankingTraffic(row) : row.connectionCount;
            return (
              <TopListItem
                key={row.identity}
                rank={index + 1}
                icon={icon}
                title={rankDisplayLabel(row.identity, row.label, unknown)}
                value={value}
                total={total}
                color={color}
                valueFormatter={(amount) =>
                  sort === "traffic" ? formatBytes(amount, unknown) : String(amount)
                }
              />
            );
          })
        : null}
    </OverviewCard>
  );
}

export function TopColumns({
  locale,
  host,
  chain,
  process,
  hostSort,
  chainSort,
  processSort,
  onHostSort,
  onChainSort,
  onProcessSort,
  onNavigate
}: {
  locale: UiLocale;
  host: { result: ReportResult | null; loading: boolean; errorZh: string | null };
  chain: { result: ReportResult | null; loading: boolean; errorZh: string | null };
  process: { result: ReportResult | null; loading: boolean; errorZh: string | null };
  hostSort: TopSort;
  chainSort: TopSort;
  processSort: TopSort;
  onHostSort: (next: TopSort) => void;
  onChainSort: (next: TopSort) => void;
  onProcessSort: (next: TopSort) => void;
  onNavigate: (route: RouteId) => void;
}) {
  return (
    <section className="grid grid-cols-1 gap-6 lg:grid-cols-3">
      <TopColumn
        locale={locale}
        title={t(locale, "overview.top.host")}
        icon={<Globe className="h-4 w-4" />}
        color="#3B82F6"
        result={host.result}
        loading={host.loading}
        errorZh={host.errorZh}
        sort={hostSort}
        onSortChange={onHostSort}
        onViewAll={() => onNavigate("host")}
      />
      <TopColumn
        locale={locale}
        title={t(locale, "overview.top.chain")}
        icon={<Link2 className="h-4 w-4" />}
        color="#8B5CF6"
        result={chain.result}
        loading={chain.loading}
        errorZh={chain.errorZh}
        sort={chainSort}
        onSortChange={onChainSort}
        onViewAll={() => onNavigate("chain")}
      />
      <TopColumn
        locale={locale}
        title={t(locale, "overview.top.process")}
        icon={<Cpu className="h-4 w-4" />}
        color="#10B981"
        result={process.result}
        loading={process.loading}
        errorZh={process.errorZh}
        sort={processSort}
        onSortChange={onProcessSort}
        onViewAll={() => onNavigate("process")}
      />
    </section>
  );
}
