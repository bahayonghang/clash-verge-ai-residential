import type { ShareModel, ShareRow } from "./report-view";

export type ReportInspectModel =
  | {
      surface: "pie";
      key: string;
      label: string;
      upload: number | null;
      download: number;
      share: number | null;
    }
  | {
      surface: "trend";
      key: string;
      bucketUtc: number;
      direction: "up" | "down" | "both";
      upload: number;
      download: number;
    };

export interface ReportScrollState {
  workspace: number;
  wraps: Record<string, number>;
}

export function rankingInspectKey(row: Pick<ShareRow, "kind" | "identity">): string {
  return row.kind === "remainder" ? "remainder" : `rank:${row.identity ?? ""}`;
}

export function trendInspectKey(bucketUtc: number, direction?: "up" | "down"): string {
  return direction ? `trend:${bucketUtc}:${direction}` : `trend:${bucketUtc}`;
}

export function inspectGroup(key: string): string {
  if (key.startsWith("trend:")) {
    const parts = key.split(":");
    const bucket = parts[1];
    return bucket ? `trend:${bucket}` : key;
  }
  return key;
}

export function inspectKeysMatch(left: string, right: string): boolean {
  return left === right || inspectGroup(left) === inspectGroup(right);
}

export function reportInspectModel(
  key: string,
  share: ShareModel | null,
  series: Array<{ bucketUtc: number; upload: number; download: number }>
): ReportInspectModel | null {
  if (key === "remainder" || key.startsWith("rank:")) {
    const row = share?.rows.find((item) => rankingInspectKey(item) === key);
    if (!row) {
      return null;
    }
    return {
      surface: "pie",
      key,
      label: row.label,
      upload: row.upload,
      download: row.download,
      share: row.share
    };
  }
  if (!key.startsWith("trend:")) {
    return null;
  }
  const parts = key.split(":");
  const bucketUtc = Number(parts[1]);
  if (!Number.isFinite(bucketUtc)) {
    return null;
  }
  const point = series.find((item) => item.bucketUtc === bucketUtc);
  if (!point) {
    return null;
  }
  const dir = parts[2];
  return {
    surface: "trend",
    key,
    bucketUtc,
    direction: dir === "up" || dir === "down" ? dir : "both",
    upload: point.upload,
    download: point.download
  };
}

export function emptyReportScroll(): ReportScrollState {
  return { workspace: 0, wraps: {} };
}

export function applyReportScrollReset(captured: ReportScrollState, reset: boolean): ReportScrollState {
  return reset ? emptyReportScroll() : captured;
}
