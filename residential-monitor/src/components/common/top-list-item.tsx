import type { ReactNode } from "react";
import { rankingShare } from "../../format/rank";
import { cn } from "../../lib/utils";

const RANK_COLORS: Record<number, string> = {
  1: "bg-amber-500/10 text-amber-500",
  2: "bg-slate-400/10 text-slate-400",
  3: "bg-orange-600/10 text-orange-600"
};

export function TopListItem({
  rank,
  icon,
  title,
  subtitle,
  value,
  total,
  color = "var(--primary)",
  valueFormatter,
  className
}: {
  rank: number;
  icon: ReactNode;
  title: string;
  subtitle?: string;
  value: number;
  total: number;
  color?: string;
  valueFormatter?: (value: number) => string;
  className?: string;
}) {
  const percentage = rankingShare(value, total) * 100;
  const display = valueFormatter ? valueFormatter(value) : String(value);
  return (
    <div className={cn("group relative -mx-2 rounded-lg px-2 py-2.5 hover:bg-muted/50", className)}>
      <div className="flex items-center gap-3">
        <div
          className={cn(
            "flex h-6 w-6 shrink-0 items-center justify-center rounded-md text-xs font-bold",
            rank <= 3 ? RANK_COLORS[rank] : "bg-muted text-muted-foreground"
          )}
        >
          {rank}
        </div>
        <div className="flex h-8 w-8 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-muted/50">
          {icon}
        </div>
        <div className="min-w-0 flex-1">
          <div className="mb-1 flex items-center justify-between gap-2">
            <div className="flex min-w-0 items-center gap-2">
              <span className="truncate text-sm font-medium" title={title}>
                {title}
              </span>
              {subtitle ? (
                <span className="hidden text-xs text-muted-foreground sm:inline">{subtitle}</span>
              ) : null}
            </div>
            <span className="ml-2 shrink-0 text-sm font-semibold tabular-nums">{display}</span>
          </div>
          <div className="h-1.5 overflow-hidden rounded-full bg-muted">
            <div
              className="h-full rounded-full"
              style={{
                width: `${percentage}%`,
                backgroundColor: color,
                opacity: 0.7 + (percentage / 100) * 0.3
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
