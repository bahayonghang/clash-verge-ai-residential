import type { ReportQuery } from "../dto";
import { unknownOr } from "./units";

export type ReportPreset = "hour" | "day" | "7" | "30" | "month";
export type ArchiveKindFilter = "all" | "hour" | "day";
export type WindowSource = "preset" | "archive";

export interface ReportForm {
  preset: ReportPreset;
  granularity: ReportQuery["granularity"];
  grouping: ReportQuery["grouping"];
  windowSource: WindowSource;
}

export interface ShareRow {
  kind: "rank" | "remainder";
  identity: string | null;
  label: string;
  upload: number | null;
  download: number;
  share: number | null;
}

export interface ShareModel {
  drawPie: boolean;
  remainder: number;
  denominator: number;
  capabilityUnsupported: boolean;
  rows: ShareRow[];
}

export interface TrendPoint {
  bucketUtc: number;
  upload: number;
  download: number;
  x: number;
  yUp: number;
  yDown: number;
}

export interface TrendModel {
  kind: "empty" | "single" | "multi";
  max: number;
  points: TrendPoint[];
}

export interface ShareCopy {
  unknown: string;
  remainder: string;
}

const PRESET_SPANS: Record<ReportPreset, number> = {
  hour: 3600,
  day: 86400,
  "7": 7 * 86400,
  "30": 30 * 86400,
  month: 30 * 86400
};

export function defaultReportForm(): ReportForm {
  return { preset: "hour", granularity: "hour", grouping: "host", windowSource: "preset" };
}

export function isReportPreset(value: string): value is ReportPreset {
  return value === "hour" || value === "day" || value === "7" || value === "30" || value === "month";
}

export function isArchiveKindFilter(value: string): value is ArchiveKindFilter {
  return value === "all" || value === "hour" || value === "day";
}

export function presetFromSpan(span: number): ReportPreset | null {
  if (span === PRESET_SPANS.hour) {
    return "hour";
  }
  if (span === PRESET_SPANS.day) {
    return "day";
  }
  if (span === PRESET_SPANS["7"]) {
    return "7";
  }
  if (span === PRESET_SPANS["30"]) {
    return "30";
  }
  return null;
}

export function formFromQueryEcho(query: ReportQuery, previous: ReportForm = defaultReportForm()): ReportForm {
  const mapped = presetFromSpan(query.rangeEndUtc - query.rangeStartUtc);
  return {
    preset: mapped ?? previous.preset,
    granularity: query.granularity,
    grouping: query.grouping,
    windowSource: mapped ? "preset" : "archive"
  };
}

export function applyPresetRange(
  query: ReportQuery,
  form: ReportForm,
  nowUtc: number,
  archiveRange?: { start: number; end: number; timezone: string }
): ReportQuery {
  if (form.windowSource === "archive" && archiveRange) {
    return {
      ...query,
      rangeStartUtc: archiveRange.start,
      rangeEndUtc: archiveRange.end,
      displayTimezone: archiveRange.timezone,
      granularity: form.granularity,
      grouping: form.grouping
    };
  }
  return {
    ...query,
    rangeStartUtc: nowUtc - PRESET_SPANS[form.preset],
    rangeEndUtc: nowUtc,
    granularity: form.granularity,
    grouping: form.grouping
  };
}

export function reportShareModel(
  report: {
    totals: { download: number };
    rankings: Array<{ identity: string; label: string; upload: number; download: number }>;
    drilldownCapability: { exactTopN: boolean };
  },
  copy: ShareCopy
): ShareModel {
  const denominator = report.totals.download;
  const remainder = denominator - report.rankings.reduce((sum, row) => sum + row.download, 0);
  if (!report.drilldownCapability.exactTopN) {
    return { drawPie: false, remainder, denominator, capabilityUnsupported: true, rows: [] };
  }
  const shareOf = (value: number): number | null => (denominator > 0 ? value / denominator : null);
  const rows: ShareRow[] = report.rankings.map((row) => ({
    kind: "rank",
    identity: row.identity,
    label: unknownOr(row.label, copy.unknown),
    upload: row.upload,
    download: row.download,
    share: shareOf(row.download)
  }));
  if (remainder > 0) {
    rows.push({
      kind: "remainder",
      identity: null,
      label: copy.remainder,
      upload: null,
      download: remainder,
      share: shareOf(remainder)
    });
  }
  return {
    drawPie: denominator > 0 && remainder >= 0,
    remainder,
    denominator,
    capabilityUnsupported: false,
    rows
  };
}

export function formatSharePct(share: number | null, unknown: string): string {
  if (share === null) {
    return unknown;
  }
  const pct = share * 100;
  if (pct > 0 && pct < 0.1) {
    return "<0.1%";
  }
  return `${pct.toFixed(1)}%`;
}

export function reportTrendModel(
  series: Array<{ bucketUtc: number; upload: number; download: number }>
): TrendModel {
  if (series.length === 0) {
    return { kind: "empty", max: 1, points: [] };
  }
  const max = Math.max(1, ...series.map((point) => Math.max(point.upload, point.download)));
  const last = series.length - 1;
  const points = series.map((point, index) => ({
    bucketUtc: point.bucketUtc,
    upload: point.upload,
    download: point.download,
    x: last === 0 ? 0.5 : index / last,
    yUp: point.upload / max,
    yDown: point.download / max
  }));
  return { kind: series.length === 1 ? "single" : "multi", max, points };
}
