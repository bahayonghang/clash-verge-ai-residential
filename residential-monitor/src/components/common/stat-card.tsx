import type { ReactNode } from "react";
import { cn } from "../../lib/utils";
import { Skeleton } from "../ui/skeleton";

export function StatCard({
  icon,
  label,
  value,
  subvalue,
  color,
  colorToken,
  loading = false,
  unavailable = false,
  className
}: {
  icon: ReactNode;
  label: string;
  value: string;
  subvalue?: string;
  /** 兼容既有 hex 调用方；新代码优先用 colorToken 走主题令牌。 */
  color?: string;
  /** 映射 var(--chart-N)，深浅四主题下均正确。 */
  colorToken?: 1 | 2 | 3 | 4 | 5;
  loading?: boolean;
  unavailable?: boolean;
  className?: string;
}) {
  const tokenColor = colorToken ? `var(--chart-${colorToken})` : undefined;
  const iconColor = tokenColor ?? color ?? "var(--chart-1)";
  const iconBackground = tokenColor
    ? `color-mix(in srgb, ${tokenColor} 8%, transparent)`
    : `${iconColor}15`;
  return (
    <div className={cn("flex flex-col rounded-xl border bg-card p-3.5 shadow-xs", className)}>
      <div
        className="mb-2.5 flex h-8 w-8 items-center justify-center rounded-md"
        style={{ backgroundColor: iconBackground }}
      >
        <span
          className="flex h-4 w-4 items-center justify-center [&_svg]:h-4 [&_svg]:w-4"
          style={{ color: iconColor }}
        >
          {icon}
        </span>
      </div>
      <p className="truncate text-[11px] font-medium uppercase tracking-[0.14em] text-muted-foreground">
        {label}
      </p>
      {loading ? (
        <Skeleton className="mt-2.5 h-5 w-24" />
      ) : unavailable ? (
        <p className="mt-2.5 truncate text-lg font-semibold leading-none text-muted-foreground tabular-nums">
          --
        </p>
      ) : (
        <p className="mt-2.5 truncate text-lg font-semibold leading-none tabular-nums" title={value}>
          {value}
        </p>
      )}
      {loading ? (
        <Skeleton className="mt-1.5 h-3 w-12" />
      ) : subvalue ? (
        <p className="mt-1.5 truncate text-sm text-muted-foreground">{subvalue}</p>
      ) : null}
    </div>
  );
}
