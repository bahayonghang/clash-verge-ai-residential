import type { ReactNode } from "react";

export function ChartHover({
  title,
  titleMuted = false,
  children
}: {
  title?: string;
  titleMuted?: boolean;
  children: ReactNode;
}) {
  return (
    <div className="rounded-lg border border-border bg-popover p-3 text-popover-foreground shadow-lg">
      {title ? (
        <p className={titleMuted ? "mb-2 text-xs text-muted-foreground" : "mb-1 text-sm font-medium"}>
          {title}
        </p>
      ) : null}
      {children}
    </div>
  );
}

export function RankBarHover({
  active,
  payload,
  label,
  formatValue
}: {
  active?: boolean;
  payload?: ReadonlyArray<{ value?: number | string; payload?: { label?: string } }>;
  label?: string | number;
  formatValue: (value: number) => string;
}) {
  if (!active || !payload || payload.length === 0) {
    return null;
  }
  const item = payload[0];
  const raw = item?.value;
  const value = typeof raw === "number" ? raw : Number(raw ?? 0);
  const title = String(item?.payload?.label ?? label ?? "");
  return (
    <ChartHover title={title || undefined}>
      <p className="text-sm font-medium tabular-nums">{formatValue(Number.isFinite(value) ? value : 0)}</p>
    </ChartHover>
  );
}

export function ShareSliceHover({
  active,
  payload,
  formatValue
}: {
  active?: boolean;
  payload?: ReadonlyArray<{
    name?: string;
    value?: number | string;
    payload?: { label?: string };
  }>;
  formatValue?: (value: number, name: string) => string;
}) {
  if (!active || !payload || payload.length === 0) {
    return null;
  }
  const item = payload[0];
  const name = String(item?.payload?.label ?? item?.name ?? "");
  const raw = item?.value;
  const value = typeof raw === "number" ? raw : Number(raw ?? 0);
  const display = formatValue
    ? formatValue(Number.isFinite(value) ? value : 0, name)
    : String(Number.isFinite(value) ? value : raw ?? "");
  return (
    <ChartHover title={name || undefined}>
      <p className="text-sm font-medium tabular-nums">{display}</p>
    </ChartHover>
  );
}
