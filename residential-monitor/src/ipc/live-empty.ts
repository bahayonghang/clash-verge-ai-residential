export type LiveEmptyKind =
  | "unconfigured"
  | "disconnected"
  | "paused"
  | "connectedEmpty"
  | "needResync"
  | "hasRows";

export interface LiveEmptyInput {
  address: string;
  session: string | null;
  collectorRunning: boolean | null;
  coverageKind: string | null;
  coverageReason: string | null;
  rowCount: number;
  needResync: boolean;
  frozen: boolean;
  errorZh: string | null;
}

const DISCONNECTED_SESSIONS = new Set([
  "disconnected",
  "tcp_unauthorized",
  "pipe_access_denied",
  "pipe_busy_timeout",
  "endpoint_missing",
  "protocol_incompatible",
  "pid_mismatch",
  "core_restarted",
  "cancelled",
  "non_loopback",
  "connecting"
]);

export function isCollectorPaused(input: Pick<LiveEmptyInput, "collectorRunning" | "coverageKind" | "coverageReason" | "session">): boolean {
  if (input.collectorRunning === false) {
    return true;
  }
  if (input.coverageKind === "closed" && input.coverageReason === "pause_or_shutdown") {
    return true;
  }
  return input.session === "paused";
}

export function liveEmptyKind(input: LiveEmptyInput): LiveEmptyKind {
  if (input.needResync || input.frozen) {
    return "needResync";
  }
  if (input.rowCount > 0) {
    return "hasRows";
  }
  if (input.address.trim().length === 0) {
    return "unconfigured";
  }
  if (isCollectorPaused(input)) {
    return "paused";
  }
  if (input.session && DISCONNECTED_SESSIONS.has(input.session)) {
    return "disconnected";
  }
  return "connectedEmpty";
}

export function liveEmptyCopy(kind: LiveEmptyKind): string | null {
  switch (kind) {
    case "unconfigured":
      return "尚未配置控制器。请到设置页填写回环地址并测试连接。";
    case "paused":
      return "采集已暂停。可在托盘选择继续采集。";
    case "connectedEmpty":
      return "当前没有活跃连接";
    case "needResync":
      return "实时序号出现缺口或协议不兼容，已停止应用增量。请重新订阅或重载窗口。";
    case "disconnected":
    case "hasRows":
      return null;
  }
}
