import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { CloseState } from "../dto";
import { toQueryClause } from "../format/live-filter-units";
import { t, type UiLocale } from "../i18n";
import {
  fetchTraySummary,
  isTauriRuntime,
  LIST_PAGE_DEFAULT,
  queryLiveConnections,
  type LiveConnectionPage,
  type LiveConnectionQuery
} from "../ipc/live-session";
import { invokeErrorZh } from "../lib/utils";
import {
  isCurrentLiveRequest,
  nextLiveRequestToken,
  type LiveFilterState
} from "../live-filter-workspace";
import { parseLiveTableLayout, type LiveTableLayout } from "../live-table-layout";
import type { LiveSortState } from "../live-table-sort";

export type LiveQueryTrigger = "view" | "delta";

export interface LiveQuerySlice {
  page: LiveConnectionPage | null;
  loading: boolean;
  errorZh: string | null;
  queryFailed: boolean;
  trigger: LiveQueryTrigger | null;
  seq: number;
}

export interface UseLivePageInput {
  applied: LiveFilterState;
  sort: LiveSortState;
  cursor?: LiveConnectionQuery["cursor"];
  refreshSignal: number | null;
  locale: UiLocale;
  active?: boolean;
}

export function buildLiveQuery(
  applied: LiveFilterState,
  sort: LiveSortState,
  cursor: LiveConnectionQuery["cursor"] = null
): LiveConnectionQuery {
  return {
    filter: {
      ...applied,
      clauses: applied.clauses.map(toQueryClause)
    },
    sortField: sort.sortField,
    descending: sort.descending,
    cursor,
    limit: LIST_PAGE_DEFAULT
  };
}

export function startLiveQuery(state: LiveQuerySlice, trigger: LiveQueryTrigger): LiveQuerySlice {
  return {
    ...state,
    loading: true,
    trigger,
    seq: nextLiveRequestToken(state.seq)
  };
}

export function applyLiveQuerySuccess(
  state: LiveQuerySlice,
  seq: number,
  page: LiveConnectionPage
): LiveQuerySlice {
  if (!isCurrentLiveRequest(seq, state.seq)) {
    return state;
  }
  return { ...state, loading: false, page, errorZh: null, queryFailed: false };
}

export function applyLiveQueryFailure(
  state: LiveQuerySlice,
  seq: number,
  errorZh: string
): LiveQuerySlice {
  if (!isCurrentLiveRequest(seq, state.seq)) {
    return state;
  }
  return { ...state, loading: false, errorZh, queryFailed: true };
}

export function decodeCloseState(value: unknown, identity: string): CloseState {
  if (!value || typeof value !== "object") {
    throw new Error("关闭结果无效");
  }
  const rec = value as Record<string, unknown>;
  const mark = rec.mark;
  if (mark !== "accepted" && mark !== "closed" && mark !== "unconfirmed") {
    throw new Error("关闭结果无效");
  }
  return {
    requestId: typeof rec.requestId === "string" ? rec.requestId : "",
    identity: typeof rec.identity === "string" ? rec.identity : identity,
    mark
  };
}

const EMPTY_SLICE: LiveQuerySlice = {
  page: null,
  loading: false,
  errorZh: null,
  queryFailed: false,
  trigger: null,
  seq: 0
};

export function useLivePage(input: UseLivePageInput): {
  page: LiveConnectionPage | null;
  loading: boolean;
  errorZh: string | null;
  queryFailed: boolean;
  trigger: LiveQueryTrigger | null;
  collectorRunning: boolean | null;
  closeConnection: (identity: string) => Promise<CloseState>;
  saveLayout: (layout: LiveTableLayout) => Promise<void>;
} {
  const inputRef = useRef(input);
  inputRef.current = input;
  const [slice, setSlice] = useState<LiveQuerySlice>(EMPTY_SLICE);
  const sliceRef = useRef(slice);
  sliceRef.current = slice;
  const [collectorRunning, setCollectorRunning] = useState<boolean | null>(null);

  const run = useCallback(async (trigger: LiveQueryTrigger): Promise<void> => {
    const started = startLiveQuery(sliceRef.current, trigger);
    sliceRef.current = started;
    setSlice(started);
    const seq = started.seq;
    const { applied, sort, cursor, locale } = inputRef.current;
    if (!isTauriRuntime()) {
      const next: LiveQuerySlice = { ...started, loading: false };
      if (isCurrentLiveRequest(seq, sliceRef.current.seq)) {
        sliceRef.current = next;
        setSlice(next);
      }
      return;
    }
    try {
      const page = await queryLiveConnections(buildLiveQuery(applied, sort, cursor ?? null));
      setSlice((current) => {
        const next = applyLiveQuerySuccess(current, seq, page);
        sliceRef.current = next;
        return next;
      });
    } catch (caught: unknown) {
      const errorZh = invokeErrorZh(caught, t(locale, "live.filter.failed"));
      setSlice((current) => {
        const next = applyLiveQueryFailure(current, seq, errorZh);
        sliceRef.current = next;
        return next;
      });
    }
    try {
      const tray = await fetchTraySummary();
      if (isCurrentLiveRequest(seq, sliceRef.current.seq)) {
        setCollectorRunning(tray.collectorRunning);
      }
    } catch {
      if (isCurrentLiveRequest(seq, sliceRef.current.seq)) {
        setCollectorRunning(null);
      }
    }
  }, []);

  const appliedKey = JSON.stringify(input.applied);
  const cursorKey = JSON.stringify(input.cursor ?? null);

  const active = input.active !== false;

  useEffect(() => {
    if (!active) {
      return;
    }
    void run("view");
  }, [run, appliedKey, input.sort.sortField, input.sort.descending, cursorKey, active]);

  useEffect(() => {
    if (!active || input.refreshSignal == null) {
      return;
    }
    void run("delta");
  }, [run, input.refreshSignal, active]);

  const closeConnection = useCallback(async (identity: string): Promise<CloseState> => {
    const locale = inputRef.current.locale;
    if (!isTauriRuntime()) {
      const errorZh = t(locale, "alerts.close_fail");
      setSlice((current) => ({ ...current, errorZh }));
      throw new Error(errorZh);
    }
    try {
      const raw = await invoke<unknown>("close_connection", {
        identity,
        requestId: `ui-${Date.now()}`
      });
      return decodeCloseState(raw, identity);
    } catch (caught: unknown) {
      const errorZh = invokeErrorZh(caught, t(locale, "alerts.close_fail"));
      setSlice((current) => ({ ...current, errorZh }));
      throw caught;
    }
  }, []);

  const saveLayout = useCallback(async (layout: LiveTableLayout): Promise<void> => {
    const locale = inputRef.current.locale;
    if (!isTauriRuntime()) {
      return;
    }
    try {
      parseLiveTableLayout(await invoke<unknown>("save_live_table_layout", { layout }));
    } catch (caught: unknown) {
      const errorZh = invokeErrorZh(caught, t(locale, "live.layout_save_fail"));
      setSlice((current) => ({ ...current, errorZh }));
    }
  }, []);

  return {
    page: slice.page,
    loading: slice.loading,
    errorZh: slice.errorZh,
    queryFailed: slice.queryFailed,
    trigger: slice.trigger,
    collectorRunning,
    closeConnection,
    saveLayout
  };
}
