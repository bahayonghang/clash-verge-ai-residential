import { Channel, invoke } from "@tauri-apps/api/core";
import type { LiveConnectionView } from "../dto";

/** 与 C2 `LIST_PAGE_DEFAULT` 对齐。 */
export const LIST_PAGE_DEFAULT = 200;

export interface LiveFilterClause {
  field: string;
  mode: "exact" | "contains";
  value: string;
}

export interface LiveConnectionQuery {
  filter: {
    category: string | null;
    host: string | null;
    process: string | null;
    rule: string | null;
    chain: string | null;
    network: string | null;
    residentialOnly: boolean;
    clauses: LiveFilterClause[];
  };
  sortField: string;
  descending: boolean;
  cursor: { sortKey: string; identity: string } | null;
  limit: number;
}

export interface LiveConnectionPage {
  rows: LiveConnectionView[];
  nextCursor: { sortKey: string; identity: string } | null;
}

export interface TraySummaryDto {
  collectorRunning: boolean;
  health: string;
  windowVisible: boolean;
}

let retainedChannel: Channel<unknown> | null = null;

export function isTauriRuntime(): boolean {
  return typeof (globalThis as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== "undefined";
}

export function defaultLiveQuery(): LiveConnectionQuery {
  return {
    filter: {
      category: null,
      host: null,
      process: null,
      rule: null,
      chain: null,
      network: null,
      residentialOnly: true,
      clauses: []
    },
    sortField: "identity",
    descending: false,
    cursor: null,
    limit: LIST_PAGE_DEFAULT
  };
}

function retainChannel(channel: Channel<unknown>): void {
  retainedChannel = channel;
}

export function retainedMonitorChannel(): Channel<unknown> | null {
  return retainedChannel;
}

export async function subscribeMonitor(onRaw: (raw: unknown) => void): Promise<number> {
  const channel = new Channel<unknown>(onRaw);
  retainChannel(channel);
  return invoke<number>("subscribe_monitor", { onEvent: channel });
}

export async function resyncMonitor(
  subscriptionId: number,
  onRaw: (raw: unknown) => void
): Promise<number> {
  const channel = new Channel<unknown>(onRaw);
  retainChannel(channel);
  return invoke<number>("resync_monitor", { subscriptionId, onEvent: channel });
}

export async function queryLiveConnections(
  query: LiveConnectionQuery = defaultLiveQuery()
): Promise<LiveConnectionPage> {
  const raw = await invoke<unknown>("query_live_connections", { query });
  if (!raw || typeof raw !== "object" || !Array.isArray((raw as { rows?: unknown }).rows)) {
    throw new Error("连接页无效");
  }
  const page = raw as LiveConnectionPage;
  return {
    rows: page.rows,
    nextCursor: page.nextCursor ?? null
  };
}

export async function fetchTraySummary(): Promise<TraySummaryDto> {
  return invoke<TraySummaryDto>("tray_summary");
}
