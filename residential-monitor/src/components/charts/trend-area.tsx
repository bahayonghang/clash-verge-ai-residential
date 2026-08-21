import type { ReactNode } from "react";
import { Activity, Loader2 } from "lucide-react";
import {
  Area,
  AreaChart,
  CartesianGrid,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis
} from "recharts";
import { inspectKeysMatch } from "../../format/report-inspect";
import { formatBytes } from "../../format/units";
import { t, type UiLocale } from "../../i18n";
import { cn } from "../../lib/utils";
import { Skeleton } from "../ui/skeleton";
import { ChartHover } from "./chart-hover";

export const TREND_DOWNLOAD_COLOR = "#3b82f6";
export const TREND_UPLOAD_COLOR = "#a855f7";

export interface TrendSeriesPoint {
  bucketUtc: number;
  upload: number;
  download: number;
  inspectKey?: string;
}

function inspectPointFromChart(state: unknown): TrendSeriesPoint | null {
  if (!state || typeof state !== "object") {
    return null;
  }
  const rec = state as { activePayload?: Array<{ payload?: TrendSeriesPoint }> };
  return rec.activePayload?.[0]?.payload ?? null;
}

function formatBucketLabel(bucketUtc: number): string {
  return new Date(bucketUtc * 1000).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false
  });
}

function TrendTooltip({
  active,
  payload,
  locale
}: {
  active?: boolean;
  payload?: ReadonlyArray<{ payload?: TrendSeriesPoint }>;
  locale: UiLocale;
}) {
  if (!active || !payload || payload.length === 0) {
    return null;
  }
  const point = payload[0]?.payload;
  if (!point) {
    return null;
  }
  const unknown = t(locale, "common.unknown");
  return (
    <ChartHover title={new Date(point.bucketUtc * 1000).toLocaleString()} titleMuted>
      <p className="flex items-center gap-2 text-sm">
        <span className="h-2 w-2 rounded-full" style={{ backgroundColor: TREND_DOWNLOAD_COLOR }} />
        <span className="text-muted-foreground">{t(locale, "overview.dir.down")}</span>
        <span className="font-medium tabular-nums">{formatBytes(point.download, unknown)}</span>
      </p>
      <p className="mt-1 flex items-center gap-2 text-sm">
        <span className="h-2 w-2 rounded-full" style={{ backgroundColor: TREND_UPLOAD_COLOR }} />
        <span className="text-muted-foreground">{t(locale, "overview.dir.up")}</span>
        <span className="font-medium tabular-nums">{formatBytes(point.upload, unknown)}</span>
      </p>
    </ChartHover>
  );
}

function ChartFrame({
  children,
  dashed = false,
  className
}: {
  children: ReactNode;
  dashed?: boolean;
  className?: string;
}) {
  return (
    <div
      className={cn(
        "flex h-[200px] w-full flex-col items-center justify-center rounded-xl",
        dashed ? "border border-dashed border-border/60 bg-card/20 px-4 text-center" : "relative",
        className
      )}
    >
      {children}
    </div>
  );
}

export function TrendArea({
  data,
  loading = false,
  emptyHint,
  locale = "zh",
  activeKey = null,
  onHover,
  onSelect
}: {
  data: TrendSeriesPoint[];
  loading?: boolean;
  emptyHint?: string;
  locale?: UiLocale;
  activeKey?: string | null;
  onHover?: (key: string | null) => void;
  onSelect?: (key: string) => void;
}) {
  if (loading && data.length === 0) {
    return (
      <ChartFrame>
        <div className="flex h-full w-full items-end justify-between gap-2 px-4 pb-4">
          {[35, 62, 28, 75, 45, 58, 32, 68, 40, 55, 30, 65].map((height, index) => (
            <Skeleton key={index} className="w-full rounded-t" style={{ height: `${height}%` }} />
          ))}
        </div>
      </ChartFrame>
    );
  }

  if (!loading && data.length === 0) {
    return (
      <ChartFrame dashed>
        <Activity className="mb-2 h-5 w-5 text-muted-foreground" />
        <p className="text-sm font-medium text-muted-foreground">{t(locale, "chart.empty")}</p>
        <p className="mt-1 text-xs text-muted-foreground/80">
          {emptyHint ?? t(locale, "chart.empty_hint")}
        </p>
      </ChartFrame>
    );
  }

  const chartData = data.map((point) => ({
    ...point,
    timeLabel: formatBucketLabel(point.bucketUtc)
  }));
  const unknown = t(locale, "common.unknown");
  const inspectEnabled = Boolean(onHover || onSelect);
  const activePoint =
    activeKey == null
      ? undefined
      : chartData.find(
          (point) => point.inspectKey != null && inspectKeysMatch(point.inspectKey, activeKey)
        );

  return (
    <ChartFrame>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart
          data={chartData}
          margin={{ top: 10, right: 10, left: 0, bottom: 0 }}
          onMouseMove={
            inspectEnabled
              ? (state) => {
                  const key = inspectPointFromChart(state)?.inspectKey;
                  onHover?.(key ?? null);
                }
              : undefined
          }
          onMouseLeave={inspectEnabled ? () => onHover?.(null) : undefined}
          onClick={
            inspectEnabled
              ? (state) => {
                  const key = inspectPointFromChart(state)?.inspectKey;
                  if (key) {
                    onSelect?.(key);
                  }
                }
              : undefined
          }
        >
          <defs>
            <linearGradient id="colorDownload" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor={TREND_DOWNLOAD_COLOR} stopOpacity={0.3} />
              <stop offset="95%" stopColor={TREND_DOWNLOAD_COLOR} stopOpacity={0} />
            </linearGradient>
            <linearGradient id="colorUpload" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor={TREND_UPLOAD_COLOR} stopOpacity={0.3} />
              <stop offset="95%" stopColor={TREND_UPLOAD_COLOR} stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="#888888" strokeOpacity={0.2} />
          <XAxis
            dataKey="timeLabel"
            axisLine={false}
            tickLine={false}
            tick={{ fontSize: 10, fill: "#888888" }}
            interval="preserveStartEnd"
            minTickGap={30}
          />
          <YAxis
            axisLine={false}
            tickLine={false}
            tick={{ fontSize: 10, fill: "#888888" }}
            width={50}
            tickFormatter={(value: number) => formatBytes(value, unknown).replace(" ", "")}
          />
          {onHover ? null : <Tooltip content={<TrendTooltip locale={locale} />} />}
          {activePoint ? <ReferenceLine x={activePoint.timeLabel} stroke="var(--ring)" /> : null}
          <Area
            type="monotone"
            dataKey="download"
            stroke={TREND_DOWNLOAD_COLOR}
            strokeWidth={2}
            fillOpacity={1}
            fill="url(#colorDownload)"
            isAnimationActive={false}
          />
          <Area
            type="monotone"
            dataKey="upload"
            stroke={TREND_UPLOAD_COLOR}
            strokeWidth={2}
            fillOpacity={1}
            fill="url(#colorUpload)"
            isAnimationActive={false}
          />
        </AreaChart>
      </ResponsiveContainer>
      {loading ? (
        <div className="pointer-events-none absolute inset-x-0 top-0 bottom-6 flex items-center justify-center bg-background/35">
          <div className="inline-flex items-center gap-2 rounded-full border border-border/60 bg-background/90 px-3 py-1.5 shadow-sm">
            <Loader2 className="h-4 w-4 animate-spin text-primary motion-reduce:animate-none" />
            <span className="text-xs font-medium">{t(locale, "chart.loading")}</span>
          </div>
        </div>
      ) : null}
    </ChartFrame>
  );
}
