import type { ReactNode } from "react";
import { BarChart3, Loader2 } from "lucide-react";
import {
  Bar,
  BarChart,
  Cell,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis
} from "recharts";
import { t, type UiLocale } from "../../i18n";
import { cn } from "../../lib/utils";
import { Skeleton } from "../ui/skeleton";

export interface RankBarDatum {
  label: string;
  value: number;
}

const CHART_COLORS = [
  "var(--chart-1)",
  "var(--chart-2)",
  "var(--chart-3)",
  "var(--chart-4)",
  "var(--chart-5)"
];

function ChartFrame({
  children,
  dashed = false,
  height,
  className
}: {
  children: ReactNode;
  dashed?: boolean;
  height: number;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex w-full flex-col items-center justify-center rounded-xl",
        dashed ? "border border-dashed border-border/60 bg-card/20 px-4 text-center" : "relative",
        className
      )}
      style={{ height }}
    >
      {children}
    </div>
  );
}

export function RankBar({
  data,
  loading = false,
  emptyHint,
  locale,
  valueFormatter
}: {
  data: RankBarDatum[];
  loading?: boolean;
  emptyHint?: string;
  locale: UiLocale;
  valueFormatter?: (value: number) => string;
}) {
  const height = Math.max(200, Math.min(480, data.length * 28 + 48));
  const formatValue = valueFormatter ?? String;

  if (loading && data.length === 0) {
    return (
      <ChartFrame height={200}>
        <div className="flex h-full w-full flex-col justify-center gap-3 px-4">
          {[80, 64, 52, 40, 28].map((width, index) => (
            <Skeleton key={index} className="h-4" style={{ width: `${width}%` }} />
          ))}
        </div>
      </ChartFrame>
    );
  }

  if (!loading && data.length === 0) {
    return (
      <ChartFrame dashed height={200}>
        <BarChart3 className="mb-2 h-5 w-5 text-muted-foreground" />
        <p className="text-sm font-medium text-muted-foreground">{t(locale, "chart.rank_empty")}</p>
        <p className="mt-1 text-xs text-muted-foreground/80">
          {emptyHint ?? t(locale, "chart.empty_hint")}
        </p>
      </ChartFrame>
    );
  }

  return (
    <ChartFrame height={height}>
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={data} layout="vertical" margin={{ top: 8, right: 24, left: 8, bottom: 8 }}>
          <XAxis type="number" hide />
          <YAxis
            type="category"
            dataKey="label"
            width={120}
            axisLine={false}
            tickLine={false}
            tick={{ fontSize: 11, fill: "#888888" }}
          />
          <Tooltip
            formatter={(value) => formatValue(typeof value === "number" ? value : Number(value ?? 0))}
            contentStyle={{
              background: "var(--popover)",
              border: "1px solid var(--border)",
              borderRadius: "0.5rem"
            }}
          />
          <Bar dataKey="value" radius={[0, 4, 4, 0]} isAnimationActive={false}>
            {data.map((item, index) => (
              <Cell key={`${item.label}-${index}`} fill={CHART_COLORS[index % CHART_COLORS.length]} />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
      {loading ? (
        <div className="pointer-events-none absolute inset-0 flex items-center justify-center bg-background/35">
          <div className="inline-flex items-center gap-2 rounded-full border border-border/60 bg-background/90 px-3 py-1.5 shadow-sm">
            <Loader2 className="h-4 w-4 animate-spin text-primary motion-reduce:animate-none" />
            <span className="text-xs font-medium">{t(locale, "chart.loading")}</span>
          </div>
        </div>
      ) : null}
    </ChartFrame>
  );
}
