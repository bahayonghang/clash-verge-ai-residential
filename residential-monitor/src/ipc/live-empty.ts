import { t, type UiLocale } from "../i18n";
import type { ObservationPhase } from "../dto";

export type LiveEmptyKind =
  | "unconfigured"
  | "connecting"
  | "disconnected"
  | "paused"
  | "stale"
  | "connectedEmpty"
  | "needResync"
  | "hasRows";

export interface LiveEmptyInput {
  address: string;
  session: string | null;
  observationPhase: ObservationPhase;
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
  "non_loopback"
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
  if (input.observationPhase === "unconfigured" || input.address.trim().length === 0) {
    return "unconfigured";
  }
  if (isCollectorPaused(input)) {
    return input.rowCount > 0 ? "stale" : "paused";
  }
  if (input.observationPhase === "connecting" || input.session === "connecting") {
    return input.rowCount > 0 ? "stale" : "connecting";
  }
  if (
    input.observationPhase === "disconnected" ||
    input.observationPhase === "decodeFailed" ||
    input.observationPhase === "resyncRequired" ||
    (input.session && DISCONNECTED_SESSIONS.has(input.session))
  ) {
    return input.rowCount > 0 ? "stale" : "disconnected";
  }
  if (input.rowCount > 0) {
    return "hasRows";
  }
  return "connectedEmpty";
}

export function liveEmptyCopy(kind: LiveEmptyKind, locale: UiLocale = "zh"): string | null {
  switch (kind) {
    case "unconfigured":
      return t(locale, "live.empty.unconfigured");
    case "paused":
      return t(locale, "live.empty.paused");
    case "connecting":
      return t(locale, "live.empty.connecting");
    case "stale":
      return t(locale, "live.empty.stale");
    case "connectedEmpty":
      return t(locale, "live.empty.connected");
    case "needResync":
      return t(locale, "live.empty.resync");
    case "disconnected":
    case "hasRows":
      return null;
  }
}
