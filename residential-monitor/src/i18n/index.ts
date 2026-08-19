import { EN } from "./en";
import { ZH } from "./zh";

export type UiLocale = "zh" | "en";

export const HEALTH_KEYS = [
  "connecting",
  "connected",
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
  "storage_failure",
  "storage_backpressure",
  "sleeping_or_clock_gap",
  "paused",
  "coverage_gap",
  "capability_expired",
  "notification_unavailable",
  "migration_failed",
  "restore_failed",
  "no_data"
] as const;

const TABLES: Record<UiLocale, Record<string, string>> = { zh: ZH, en: EN };

export function parseUiLocale(value: unknown): UiLocale {
  return value === "en" ? "en" : "zh";
}

export function t(locale: UiLocale, key: string): string {
  return TABLES[locale][key] ?? TABLES.zh[key] ?? key;
}

export function healthTitle(locale: UiLocale, session: string): string {
  return t(locale, `health.${session}`);
}

export function healthAction(locale: UiLocale, session: string): string {
  return t(locale, `health.${session}.action`);
}
