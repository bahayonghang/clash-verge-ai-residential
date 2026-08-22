import { Channel, invoke } from "@tauri-apps/api/core";
import type { LiveConnectionView } from "../dto";

/** 与 C2 `LIST_PAGE_DEFAULT` 对齐。 */
export const LIST_PAGE_DEFAULT = 200;

export interface LiveFilterClause {
  field: string;
  mode: "exact" | "contains" | "gt" | "gte" | "lt" | "lte" | "eq";
  value: string;
  unit?: string;
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
  matchedCount: number;
  sampleUtc: number | null;
  summary: ConnectionSummary;
}

/** 后端从完整匹配集合选出的方向热点。 */
export interface ConnectionHotspot {
  identity: string;
  label: string;
  host: string | null;
  process: string | null;
  destination: string | null;
  value: number;
}

export interface ConnectionSummary {
  topDownload: ConnectionHotspot | null;
  topUpload: ConnectionHotspot | null;
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
  return decodeLiveConnectionPage(raw);
}

/** 校验查询信封；缺失 summary 不得用分页 rows 伪造。 */
export function decodeLiveConnectionPage(value: unknown): LiveConnectionPage {
  if (!isRecord(value) || !Array.isArray(value.rows)) {
    throw new Error("连接页无效");
  }
  if (!hasOwn(value, "matchedCount") || !isUnsignedInteger(value.matchedCount)) {
    throw new Error("连接页 matchedCount 无效");
  }
  if (!hasOwn(value, "sampleUtc") || !isNullableSafeInteger(value.sampleUtc)) {
    throw new Error("连接页 sampleUtc 无效");
  }
  if (!hasOwn(value, "summary") || !isRecord(value.summary)) {
    throw new Error("连接页 summary 无效");
  }
  if (!hasOwn(value.summary, "topDownload") || !hasOwn(value.summary, "topUpload")) {
    throw new Error("连接页 summary 字段缺失");
  }
  return {
    rows: value.rows as LiveConnectionView[],
    nextCursor: decodeCursor(value.nextCursor),
    matchedCount: value.matchedCount,
    sampleUtc: value.sampleUtc,
    summary: {
      topDownload: decodeHotspot(value.summary.topDownload, "topDownload"),
      topUpload: decodeHotspot(value.summary.topUpload, "topUpload")
    }
  };
}

function decodeCursor(value: unknown): { sortKey: string; identity: string } | null {
  if (value === null || value === undefined) {
    return null;
  }
  if (!isRecord(value) || typeof value.sortKey !== "string" || typeof value.identity !== "string") {
    throw new Error("连接页 nextCursor 无效");
  }
  return { sortKey: value.sortKey, identity: value.identity };
}

function decodeHotspot(value: unknown, name: string): ConnectionHotspot | null {
  if (value === null) {
    return null;
  }
  if (
    !isRecord(value) ||
    !isNonEmptyString(value.identity) ||
    !isNonEmptyString(value.label) ||
    !isNullableString(value.host) ||
    !isNullableString(value.process) ||
    !isNullableString(value.destination) ||
    !isUnsignedInteger(value.value)
  ) {
    throw new Error(`连接页 ${name} 无效`);
  }
  return {
    identity: value.identity,
    label: value.label,
    host: value.host,
    process: value.process,
    destination: value.destination,
    value: value.value
  };
}

function hasOwn(value: Record<string, unknown>, key: string): boolean {
  return Object.prototype.hasOwnProperty.call(value, key);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isNonEmptyString(value: unknown): value is string {
  return typeof value === "string" && value.length > 0;
}

function isNullableString(value: unknown): value is string | null {
  return value === null || typeof value === "string";
}

function isNullableSafeInteger(value: unknown): value is number | null {
  return value === null || (typeof value === "number" && Number.isSafeInteger(value));
}

function isUnsignedInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
}

export async function fetchTraySummary(): Promise<TraySummaryDto> {
  return invoke<TraySummaryDto>("tray_summary");
}
