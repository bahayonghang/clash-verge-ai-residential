import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  decodeAlertCenter,
  decodeDiagnostics,
  type AlertCenterPage,
  type AlertInstance,
  type AlertRule,
  type AlertSummary,
  type DiagnosticsSnapshot,
  type NotifyCapability
} from "../dto";
import { t, type UiLocale } from "../i18n";
import { isTauriRuntime } from "../ipc/live-session";
import { formatTemplate, invokeErrorZh } from "../lib/utils";

export type AlertStatusFilter = "all" | AlertInstance["status"];

export function emptyAlertDraft(): AlertRule {
  return {
    ruleId: "rate-home",
    version: 1,
    enabled: true,
    kind: "rate",
    selectorKind: "primary-category",
    selectorValue: "家宽",
    direction: "download",
    thresholdValue: 1000000,
    recoveryThreshold: 400000,
    period: null,
    timezone: "Asia/Shanghai",
    cooldownSec: 300,
    quietStartMin: null,
    quietEndMin: null,
    createdUtc: 0,
    updatedUtc: 0
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function decodeNotify(value: unknown): NotifyCapability {
  if (!isRecord(value) || typeof value.available !== "boolean" || typeof value.reasonZh !== "string") {
    throw new Error("NotifyCapability 无效");
  }
  return {
    available: value.available,
    reasonZh: value.reasonZh,
    canFocusApp: value.canFocusApp === true,
    focusAssistUnknown: value.focusAssistUnknown === true
  };
}

function decodeRules(value: unknown): AlertRule[] {
  if (!Array.isArray(value)) {
    throw new Error("告警规则列表无效");
  }
  return value as AlertRule[];
}

function decodeSummary(value: unknown): AlertSummary {
  if (!isRecord(value) || value.schemaVersion !== 1) {
    throw new Error("AlertSummary 无效");
  }
  return value as unknown as AlertSummary;
}

export function useAlerts(locale: UiLocale, active: boolean): {
  rules: AlertRule[];
  page: AlertCenterPage | null;
  summary: AlertSummary | null;
  notify: NotifyCapability | null;
  diagnostics: DiagnosticsSnapshot | null;
  outboxCount: number | null;
  selected: AlertInstance | null;
  statusFilter: AlertStatusFilter;
  loading: boolean;
  errorZh: string | null;
  statusZh: string;
  setStatusFilter: (filter: AlertStatusFilter) => void;
  setSelected: (item: AlertInstance | null) => void;
  refresh: () => Promise<void>;
  loadMore: () => Promise<void>;
  upsertRule: (rule: AlertRule) => Promise<void>;
  testNotify: () => Promise<void>;
  loadDiagnostics: () => Promise<void>;
  exportDiagnostics: () => Promise<void>;
  scanOutbox: () => Promise<void>;
} {
  const seq = useRef(0);
  const [rules, setRules] = useState<AlertRule[]>([]);
  const [page, setPage] = useState<AlertCenterPage | null>(null);
  const [summary, setSummary] = useState<AlertSummary | null>(null);
  const [notify, setNotify] = useState<NotifyCapability | null>(null);
  const [diagnostics, setDiagnostics] = useState<DiagnosticsSnapshot | null>(null);
  const [outboxCount, setOutboxCount] = useState<number | null>(null);
  const [selected, setSelected] = useState<AlertInstance | null>(null);
  const [statusFilter, setStatusFilterState] = useState<AlertStatusFilter>("all");
  const [loading, setLoading] = useState(false);
  const [errorZh, setErrorZh] = useState<string | null>(null);
  const [statusZh, setStatusZh] = useState(() => t(locale, "alerts.idle"));

  const refresh = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    setLoading(true);
    const fallback = t(locale, "alerts.refresh.fail");
    if (!isTauriRuntime()) {
      setLoading(false);
      setStatusZh(t(locale, "alerts.unavailable"));
      return;
    }
    try {
      const status = statusFilter === "all" ? null : statusFilter;
      const [nextPage, nextRules, nextSummary, nextDiag] = await Promise.all([
        invoke<unknown>("list_alert_center", { status, after: null }),
        invoke<unknown>("list_alert_rules"),
        invoke<unknown>("alert_summary"),
        invoke<unknown>("get_diagnostics")
      ]);
      if (token !== seq.current) {
        return;
      }
      const decodedPage = decodeAlertCenter(nextPage);
      setPage(decodedPage);
      setRules(decodeRules(nextRules));
      setSummary(decodeSummary(nextSummary));
      setDiagnostics(decodeDiagnostics(nextDiag));
      setSelected((current) =>
        current ? (decodedPage.items.find((item) => item.instanceId === current.instanceId) ?? null) : null
      );
      setErrorZh(null);
      setStatusZh(formatTemplate(t(locale, "alerts.loaded"), { count: decodedPage.items.length }));
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, fallback));
      setStatusZh(fallback);
    } finally {
      if (token === seq.current) {
        setLoading(false);
      }
    }
  }, [locale, statusFilter]);

  const loadMore = useCallback(async (): Promise<void> => {
    const cursor = page?.nextCursor;
    if (!cursor || !isTauriRuntime()) {
      return;
    }
    const token = ++seq.current;
    const fallback = t(locale, "alerts.refresh.fail");
    try {
      const status = statusFilter === "all" ? null : statusFilter;
      const raw = await invoke<unknown>("list_alert_center", { status, after: cursor });
      if (token !== seq.current) {
        return;
      }
      const next = decodeAlertCenter(raw);
      setPage((current) =>
        current
          ? { ...next, items: [...current.items, ...next.items] }
          : next
      );
      setErrorZh(null);
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, fallback));
    }
  }, [locale, page?.nextCursor, statusFilter]);

  const upsertRule = useCallback(
    async (rule: AlertRule): Promise<void> => {
      const token = ++seq.current;
      const fallback = t(locale, "alerts.rule.invalid");
      if (!isTauriRuntime()) {
        setErrorZh(fallback);
        return;
      }
      try {
        await invoke("upsert_alert_rule", { rule });
        if (token !== seq.current) {
          return;
        }
        setErrorZh(null);
        setStatusZh(t(locale, "alerts.rule.saved"));
        await refresh();
      } catch (caught: unknown) {
        if (token !== seq.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, fallback));
        setStatusZh(fallback);
      }
    },
    [locale, refresh]
  );

  const testNotify = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    const fallback = t(locale, "alerts.test.fail");
    if (!isTauriRuntime()) {
      setErrorZh(fallback);
      return;
    }
    try {
      const cap = decodeNotify(await invoke<unknown>("test_notification"));
      if (token !== seq.current) {
        return;
      }
      setNotify(cap);
      setErrorZh(null);
      setStatusZh(cap.available ? t(locale, "alerts.test.ok") : cap.reasonZh);
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, fallback));
      setStatusZh(fallback);
    }
  }, [locale]);

  const loadDiagnostics = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    const fallback = t(locale, "alerts.diag_idle");
    if (!isTauriRuntime()) {
      return;
    }
    try {
      const next = decodeDiagnostics(await invoke<unknown>("get_diagnostics"));
      if (token !== seq.current) {
        return;
      }
      setDiagnostics(next);
      setErrorZh(null);
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, fallback));
    }
  }, [locale]);

  const exportDiagnostics = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    const fallback = t(locale, "alerts.diag.export_fail");
    if (!isTauriRuntime()) {
      setErrorZh(fallback);
      return;
    }
    try {
      const path = await invoke<string | null>("pick_file", {
        purpose: "diagnostics-export",
        mode: "save",
        locale
      });
      if (!path) {
        if (token === seq.current) {
          setStatusZh(t(locale, "report.export_cancel"));
        }
        return;
      }
      await invoke("export_diagnostics", { path });
      if (token !== seq.current) {
        return;
      }
      setErrorZh(null);
      setStatusZh(t(locale, "alerts.diag.export_ok"));
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, fallback));
      setStatusZh(fallback);
    }
  }, [locale]);

  const scanOutbox = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    const fallback = t(locale, "alerts.refresh.fail");
    if (!isTauriRuntime()) {
      return;
    }
    try {
      const count = await invoke<number>("scan_outbox");
      if (token !== seq.current) {
        return;
      }
      setOutboxCount(count);
      setErrorZh(null);
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, fallback));
    }
  }, [locale]);

  const setStatusFilter = useCallback((filter: AlertStatusFilter): void => {
    setStatusFilterState(filter);
    setPage(null);
  }, []);

  useEffect(() => {
    if (!active) {
      return;
    }
    void refresh();
  }, [active, refresh]);

  return {
    rules,
    page,
    summary,
    notify,
    diagnostics,
    outboxCount,
    selected,
    statusFilter,
    loading,
    errorZh,
    statusZh,
    setStatusFilter,
    setSelected,
    refresh,
    loadMore,
    upsertRule,
    testNotify,
    loadDiagnostics,
    exportDiagnostics,
    scanOutbox
  };
}
