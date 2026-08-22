import type { ConnectionHotspot, LiveConnectionPage } from "../ipc/live-session";
import { isCollectorPaused } from "../ipc/live-empty";

export type LiveHotspotStatus =
  | "ready"
  | "noMatch"
  | "paused"
  | "gap"
  | "unconfigured"
  | "disconnected"
  | "unknown";

export interface LiveHotspotStatusInput {
  page: LiveConnectionPage | null;
  address: string;
  session: string | null;
  collectorRunning: boolean | null;
  coverageKind: string | null;
  coverageReason: string | null;
  needResync: boolean;
  frozen: boolean;
}

export function isCoverageGap(coverageKind: string | null): boolean {
  return coverageKind === "gap" || coverageKind === "epoch";
}

function isCoverageUnknown(coverageKind: string | null, coverageReason: string | null): boolean {
  if (coverageKind === null) {
    return coverageReason !== null;
  }
  if (coverageKind === "gap" || coverageKind === "epoch") {
    return coverageReason !== "disconnect_or_sleep" && coverageReason !== "core_restart";
  }
  if (coverageKind === "closed") {
    return coverageReason !== "pause_or_shutdown";
  }
  return true;
}

/** 由摘要状态决定是否展示当前快照。不从分页 rows 计算 Top 1。 */
export function liveHotspotStatus(input: LiveHotspotStatusInput): LiveHotspotStatus {
  if (input.needResync || input.frozen) {
    return "gap";
  }
  if (
    isCollectorPaused({
      collectorRunning: input.collectorRunning,
      coverageKind: input.coverageKind,
      coverageReason: input.coverageReason,
      session: input.session
    })
  ) {
    return "paused";
  }
  if (input.address.trim().length === 0) {
    return "unconfigured";
  }
  if (input.session !== "connected") {
    return "disconnected";
  }
  if (input.collectorRunning !== true || isCoverageUnknown(input.coverageKind, input.coverageReason)) {
    return "unknown";
  }
  if (isCoverageGap(input.coverageKind)) {
    return "gap";
  }
  if (input.page === null || input.page.sampleUtc === null) return "unknown";
  return input.page.matchedCount === 0 ? "noMatch" : "ready";
}

/** 仅当前可用快照可展示方向数值；无匹配时保持 null，不写成 0。 */
export function canShowHotspotValue(status: LiveHotspotStatus): boolean {
  return status === "ready";
}

/** 暂停、缺口、未连接或能力未知时隐藏命中数和采样时间。 */
export function canShowHotspotSnapshotFacts(status: LiveHotspotStatus): boolean {
  return status === "ready" || status === "noMatch";
}

/** 优先使用后端 label，其余仅用可脱敏展示字段回退。 */
export function hotspotDisplayLabel(hotspot: ConnectionHotspot, fallback: string): string {
  return [hotspot.label, hotspot.host, hotspot.process, hotspot.destination].find(
    (value) => value !== null && value.trim().length > 0
  ) ?? fallback;
}

/** 次要标识只用安全展示字段，不暴露原始连接载荷。 */
export function hotspotDisplayDetail(hotspot: ConnectionHotspot, fallback: string): string {
  return [hotspot.host, hotspot.process, hotspot.destination].find(
    (value) => value !== null && value.trim().length > 0 && value !== hotspot.label
  ) ?? fallback;
}
