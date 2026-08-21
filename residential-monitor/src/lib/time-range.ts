export const TIME_RANGE_PRESETS = ["5m", "30m", "1h", "24h", "7d", "30d", "today"] as const;

export type TimeRangePreset = (typeof TIME_RANGE_PRESETS)[number];

export interface TimeRange {
  preset: TimeRangePreset;
  startUtc: number;
  endUtc: number;
}

const MINUTE = 60 * 1000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

export function timeRangeFromPreset(preset: TimeRangePreset, now = Date.now()): TimeRange {
  const endUtc = now;
  if (preset === "today") {
    const start = new Date(now);
    start.setHours(0, 0, 0, 0);
    return { preset, startUtc: start.getTime(), endUtc };
  }
  const span =
    preset === "5m"
      ? 5 * MINUTE
      : preset === "30m"
        ? 30 * MINUTE
        : preset === "1h"
          ? HOUR
          : preset === "24h"
            ? DAY
            : preset === "7d"
              ? 7 * DAY
              : 30 * DAY;
  return { preset, startUtc: endUtc - span, endUtc };
}

export function defaultTimeRange(now = Date.now()): TimeRange {
  return timeRangeFromPreset("24h", now);
}
