import type { ReactNode } from "react";
import { cn } from "../../lib/utils";
import { Skeleton } from "../ui/skeleton";

export function StatCard({
  icon,
  label,
  value,
  subvalue,
  color,
  loading = false,
  unavailable = false,
  className
}: {
  icon: ReactNode;
  label: string;
  value: string;
  subvalue?: string;
  color: string;
  loading?: boolean;
  unavailable?: boolean;
  className?: string;
}) {
  return (
    <div className={cn("flex flex-col rounded-xl border bg-card p-3.5 shadow-xs", className)}>
      <div
        className="mb-2.5 flex h-8 w-8 items-center justify-center rounded-md"
        style={{ backgroundColor: `${color}15` }}
      >
        <span
          className="flex h-4 w-4 items-center justify-center [&_svg]:h-4 [&_svg]:w-4"
          style={{ color }}
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
