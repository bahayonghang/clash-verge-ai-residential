import { Cell, Pie, PieChart, ResponsiveContainer, Tooltip } from "recharts";
import { inspectKeysMatch } from "../../format/report-inspect";
import { cn } from "../../lib/utils";

const FILLS = [
  "var(--chart-1)",
  "var(--chart-2)",
  "var(--chart-3)",
  "var(--chart-4)",
  "var(--chart-5)"
];

export interface ShareDonutRow {
  label: string;
  value: number;
  inspectKey?: string;
  remainder?: boolean;
}

export function ShareDonut({
  data,
  loading = false,
  emptyHint,
  activeKey = null,
  onHover,
  onSelect
}: {
  data: ShareDonutRow[];
  loading?: boolean;
  emptyHint: string;
  activeKey?: string | null;
  onHover?: (key: string | null) => void;
  onSelect?: (key: string) => void;
}) {
  const drawable = data.filter((row) => row.value > 0);
  if (loading) {
    return (
      <div
        className="flex h-[200px] items-end justify-center rounded-xl border border-dashed border-border/60 bg-muted/20 p-4"
        aria-busy="true"
      >
        <div className="h-24 w-24 animate-pulse rounded-full bg-muted/40" />
      </div>
    );
  }
  if (drawable.length === 0) {
    return (
      <div className="flex h-[200px] flex-col items-center justify-center rounded-xl border border-dashed border-border/60 bg-card/20 px-4 text-center">
        <p className="text-sm text-muted-foreground">{emptyHint}</p>
      </div>
    );
  }
  return (
    <div className="h-[200px] w-full">
      <ResponsiveContainer width="100%" height={200}>
        <PieChart>
          <Pie
            data={drawable}
            dataKey="value"
            nameKey="label"
            innerRadius={52}
            outerRadius={80}
            isAnimationActive={false}
            onMouseEnter={(_, index) => {
              const key = drawable[index]?.inspectKey;
              onHover?.(key ?? null);
            }}
            onMouseLeave={() => onHover?.(null)}
            onClick={(_, index) => {
              const key = drawable[index]?.inspectKey;
              if (key) {
                onSelect?.(key);
              }
            }}
          >
            {drawable.map((row, index) => {
              const key = row.inspectKey ?? `${row.label}-${index}`;
              const active = Boolean(row.inspectKey && activeKey && inspectKeysMatch(row.inspectKey, activeKey));
              const dimmed = Boolean(activeKey) && !active;
              return (
                <Cell
                  key={key}
                  fill={row.remainder ? "var(--muted-foreground)" : FILLS[index % FILLS.length]}
                  className={cn("cursor-pointer outline-none", dimmed && "opacity-35")}
                  stroke={active ? "var(--ring)" : "transparent"}
                  strokeWidth={active ? 2 : 0}
                />
              );
            })}
          </Pie>
          {onHover ? null : (
            <Tooltip
              isAnimationActive={false}
              formatter={(value, name) => [String(value ?? ""), String(name ?? "")]}
            />
          )}
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
}
