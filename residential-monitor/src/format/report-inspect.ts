import type { ShareModel, ShareRow } from "./report-view";

const SKIP_REPORT_PAINT_KINDS = new Set([
  "connectionDelta",
  "healthChanged",
  "summaryChanged",
  "alertChanged"
]);

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

export function shouldSkipReportPaint(
  route: string,
  messageKind: string,
  errorZh: string | null,
  paintedErrorZh: string | null
): boolean {
  if (route !== "reports") {
    return false;
  }
  if (errorZh !== paintedErrorZh) {
    return false;
  }
  return SKIP_REPORT_PAINT_KINDS.has(messageKind);
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

export function readReportScroll(root: ParentNode): ReportScrollState {
  const workspace = root.querySelector(".workspace");
  const wraps: Record<string, number> = {};
  root.querySelectorAll("[data-report-scroll]").forEach((node) => {
    if (!(node instanceof HTMLElement)) {
      return;
    }
    const id = node.dataset.reportScroll;
    if (id) {
      wraps[id] = node.scrollTop;
    }
  });
  return {
    workspace: workspace instanceof HTMLElement ? workspace.scrollTop : 0,
    wraps
  };
}

export function writeReportScroll(root: ParentNode, state: ReportScrollState): void {
  const workspace = root.querySelector(".workspace");
  if (workspace instanceof HTMLElement) {
    workspace.scrollTop = state.workspace;
  }
  for (const [id, top] of Object.entries(state.wraps)) {
    const node = root.querySelector(`[data-report-scroll="${id}"]`);
    if (node instanceof HTMLElement) {
      node.scrollTop = top;
    }
  }
}

export function inspectKeyExists(root: ParentNode, key: string | null): boolean {
  if (!key) {
    return false;
  }
  const nodes = root.querySelectorAll("[data-inspect]");
  for (const node of nodes) {
    if (inspectKeysMatch(node.getAttribute("data-inspect") ?? "", key)) {
      return true;
    }
  }
  return false;
}
