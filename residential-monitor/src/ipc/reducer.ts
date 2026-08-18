import type { LiveConnectionView, LiveOverview, MonitorStreamMessage } from "../dto";

export interface MonitorState {
  subscriptionId: number | null;
  schemaVersion: number | null;
  lastSeq: number | null;
  snapshot: LiveOverview | null;
  frozen: boolean;
  needResync: boolean;
  errorZh: string | null;
  connections: Map<string, LiveConnectionView>;
  closeMarks: Map<string, "accepted" | "closed" | "unconfirmed">;
}

export function emptyMonitorState(): MonitorState {
  return {
    subscriptionId: null,
    schemaVersion: null,
    lastSeq: null,
    snapshot: null,
    frozen: false,
    needResync: false,
    errorZh: null,
    connections: new Map(),
    closeMarks: new Map()
  };
}

export function reduceMonitor(state: MonitorState, message: MonitorStreamMessage): MonitorState {
  if (message.kind === "bootstrap") {
    return {
      subscriptionId: message.subscriptionId,
      schemaVersion: message.schemaVersion,
      lastSeq: message.baseSeq,
      snapshot: message.snapshot,
      frozen: false,
      needResync: false,
      errorZh: null,
      connections: new Map(),
      closeMarks: new Map(state.closeMarks)
    };
  }
  if (state.subscriptionId === null || message.subscriptionId !== state.subscriptionId) {
    return state;
  }
  if (state.frozen) {
    return state;
  }
  if (message.schemaVersion !== state.schemaVersion) {
    return {
      ...state,
      frozen: true,
      needResync: false,
      errorZh: "实时协议版本不兼容，请升级或重载窗口。"
    };
  }
  const seq = message.seq;
  const last = state.lastSeq ?? 0;
  if (seq <= last) {
    return state;
  }
  if (seq > last + 1) {
    return {
      ...state,
      frozen: true,
      needResync: true,
      errorZh: "实时序号出现缺口，已停止应用增量。"
    };
  }
  if (message.kind === "healthChanged") {
    if (!state.snapshot) {
      return { ...state, lastSeq: seq };
    }
    return {
      ...state,
      lastSeq: seq,
      snapshot: { ...state.snapshot, health: message.health }
    };
  }
  if (message.kind === "summaryChanged") {
    return { ...state, lastSeq: seq, snapshot: message.snapshot };
  }
  const connections = new Map(state.connections);
  const closeMarks = new Map(state.closeMarks);
  for (const row of message.upserts) {
    connections.set(row.identity, row);
  }
  for (const id of message.removes) {
    connections.delete(id);
    if (closeMarks.get(id) === "accepted") {
      closeMarks.set(id, "closed");
    }
  }
  return { ...state, lastSeq: seq, connections, closeMarks };
}

export function markCloseAccepted(state: MonitorState, identity: string): MonitorState {
  const closeMarks = new Map(state.closeMarks);
  closeMarks.set(identity, "accepted");
  return { ...state, closeMarks };
}

export function visibleRows(
  connections: Map<string, LiveConnectionView>,
  start: number,
  count: number,
  overscan: number
): LiveConnectionView[] {
  const rows = [...connections.values()];
  const from = Math.max(0, start - overscan);
  return rows.slice(from, start + count + overscan);
}
