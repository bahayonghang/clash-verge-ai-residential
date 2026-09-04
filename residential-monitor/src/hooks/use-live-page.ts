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
  type ConnectionSummary,
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
  refreshSignal: number | null;
  locale: UiLocale;
  active?: boolean;
}

export type LiveCursor = LiveConnectionQuery["cursor"];

export interface LivePager {
  cursor: LiveCursor;
  history: LiveCursor[];
  pageNumber: number;
}

export function firstLivePager(): LivePager {
  return { cursor: null, history: [], pageNumber: 1 };
}

export function cursorsEqual(left: LiveCursor, right: LiveCursor): boolean {
  if (left === right) {
    return true;
  }
  if (left == null || right == null) {
    return false;
  }
  return left.sortKey === right.sortKey && left.identity === right.identity;
}

export function advanceLivePager(pager: LivePager, nextCursor: LiveCursor): LivePager {
  if (nextCursor == null || cursorsEqual(pager.cursor, nextCursor)) {
    return pager;
  }
  return {
    cursor: nextCursor,
    history: [...pager.history, pager.cursor],
    pageNumber: pager.pageNumber + 1
  };
}

export function rewindLivePager(pager: LivePager): LivePager {
  if (pager.history.length === 0) {
    return pager.pageNumber === 1 && pager.cursor == null ? pager : firstLivePager();
  }
  const cursor = pager.history[pager.history.length - 1] ?? null;
  return {
    cursor,
    history: pager.history.slice(0, -1),
    pageNumber: Math.max(1, pager.pageNumber - 1)
  };
}

export function loadNextLivePage(pager: LivePager, page: LiveConnectionPage | null): LivePager {
  return advanceLivePager(pager, page?.nextCursor ?? null);
}

export function loadPrevLivePage(pager: LivePager): LivePager {
  return rewindLivePager(pager);
}

export function pinLiveSummary(
  pinned: ConnectionSummary | null,
  page: LiveConnectionPage,
  cursor: LiveCursor
): ConnectionSummary {
  if (cursor == null || pinned == null) {
    return page.summary;
  }
  return pinned;
}

export function withPinnedSummary(
  page: LiveConnectionPage,
  summary: ConnectionSummary
): LiveConnectionPage {
  if (page.summary === summary) {
    return page;
  }
  return { ...page, summary };
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
  pageNumber: number;
  canLoadNext: boolean;
  canLoadPrev: boolean;
  loadNext: () => void;
  loadPrev: () => void;
  closeConnection: (identity: string) => Promise<CloseState>;
  saveLayout: (layout: LiveTableLayout) => Promise<void>;
} {
  const inputRef = useRef(input);
  inputRef.current = input;
  const [slice, setSlice] = useState<LiveQuerySlice>(EMPTY_SLICE);
  const sliceRef = useRef(slice);
  sliceRef.current = slice;
  const [collectorRunning, setCollectorRunning] = useState<boolean | null>(null);
  const appliedKey = JSON.stringify(input.applied);
  const filterKey = `${appliedKey}|${input.sort.sortField}|${input.sort.descending ? "d" : "a"}`;
  const [pager, setPager] = useState<LivePager>(firstLivePager);
  const pagerFilterKeyRef = useRef(filterKey);
  const pinnedRef = useRef<ConnectionSummary | null>(null);
  if (pagerFilterKeyRef.current !== filterKey) {
    pagerFilterKeyRef.current = filterKey;
    setPager(firstLivePager());
    pinnedRef.current = null;
  }
  const pagerRef = useRef(pager);
  pagerRef.current = pager;

  const run = useCallback(async (trigger: LiveQueryTrigger): Promise<void> => {
    const started = startLiveQuery(sliceRef.current, trigger);
    sliceRef.current = started;
    setSlice(started);
    const seq = started.seq;
    const { applied, sort, locale } = inputRef.current;
    const cursor = pagerRef.current.cursor;
    if (!isTauriRuntime()) {
      const next: LiveQuerySlice = { ...started, loading: false };
      if (isCurrentLiveRequest(seq, sliceRef.current.seq)) {
        sliceRef.current = next;
        setSlice(next);
      }
      return;
    }
    try {
      const page = await queryLiveConnections(buildLiveQuery(applied, sort, cursor));
      const summary = pinLiveSummary(pinnedRef.current, page, cursor);
      pinnedRef.current = summary;
      const display = withPinnedSummary(page, summary);
      setSlice((current) => {
        const next = applyLiveQuerySuccess(current, seq, display);
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

  const cursorKey = JSON.stringify(pager.cursor);
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

  const loadNext = useCallback((): void => {
    if (sliceRef.current.loading) {
      return;
    }
    setPager((current) => loadNextLivePage(current, sliceRef.current.page));
  }, []);

  const loadPrev = useCallback((): void => {
    if (sliceRef.current.loading) {
      return;
    }
    setPager((current) => loadPrevLivePage(current));
  }, []);

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
    pageNumber: pager.pageNumber,
    canLoadNext: Boolean(slice.page?.nextCursor) && !slice.loading,
    canLoadPrev: pager.pageNumber > 1 && !slice.loading,
    loadNext,
    loadPrev,
    closeConnection,
    saveLayout
  };
}
