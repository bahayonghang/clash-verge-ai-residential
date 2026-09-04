import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { defaultLiveQuery, LIST_PAGE_DEFAULT, queryLiveConnections } from "../ipc/live-session";
import {
  applyLiveQueryFailure,
  applyLiveQuerySuccess,
  buildLiveQuery,
  decodeCloseState,
  firstLivePager,
  loadNextLivePage,
  loadPrevLivePage,
  pinLiveSummary,
  startLiveQuery,
  withPinnedSummary,
  type LiveQuerySlice
} from "./use-live-page";
import useLivePageSource from "./use-live-page.ts?raw";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn()
}));

const page = {
  rows: [],
  nextCursor: null,
  matchedCount: 0,
  sampleUtc: 1,
  summary: { topDownload: null, topUpload: null }
};

function slice(overrides: Partial<LiveQuerySlice> = {}): LiveQuerySlice {
  return {
    page: null,
    loading: false,
    errorZh: null,
    queryFailed: false,
    trigger: null,
    seq: 0,
    ...overrides
  };
}

describe("useLivePage 查询信封与竞态", () => {
  it("入参沿用既有字段、单位换算、默认 limit", () => {
    const applied = {
      ...defaultLiveQuery().filter,
      residentialOnly: true,
      clauses: [
        { field: "host", mode: "contains" as const, value: "example.com" },
        { field: "download", mode: "gte" as const, value: "1", unit: "KiB" },
        { field: "process", mode: "exact" as const, value: "" }
      ]
    };
    const query = buildLiveQuery(applied, { sortField: "download", descending: true }, null);
    expect(query.limit).toBe(200);
    expect(query.sortField).toBe("download");
    expect(query.descending).toBe(true);
    expect(query.cursor).toBeNull();
    expect(query.filter.residentialOnly).toBe(true);
    expect(query.filter.clauses).toEqual([
      { field: "host", mode: "contains", value: "example.com" },
      { field: "download", mode: "gte", value: "1024" },
      { field: "process", mode: "exact", value: "" }
    ]);
  });

  it("重复提交与过期响应不得覆盖最新结果，失败保留上次页", () => {
    const first = startLiveQuery(slice({ page }), "view");
    const second = startLiveQuery(first, "view");
    expect(second.seq).toBe(first.seq + 1);
    expect(second.loading).toBe(true);
    const stale = applyLiveQuerySuccess(second, first.seq, {
      ...page,
      matchedCount: 9
    });
    expect(stale.page).toEqual(page);
    expect(stale.loading).toBe(true);
    const failed = applyLiveQueryFailure(second, second.seq, "筛选请求失败。请检查条件后重试。");
    expect(failed.page).toEqual(page);
    expect(failed.loading).toBe(false);
    expect(failed.queryFailed).toBe(true);
    expect(failed.errorZh).toContain("失败");
    const ok = applyLiveQuerySuccess(second, second.seq, { ...page, matchedCount: 3 });
    expect(ok.page?.matchedCount).toBe(3);
    expect(ok.errorZh).toBeNull();
  });

  it("四个触发源共用 requestSeq，IPC 只在 hook 内", () => {
    expect(useLivePageSource).toContain("nextLiveRequestToken");
    expect(useLivePageSource).toContain("queryLiveConnections");
    expect(useLivePageSource).toContain("close_connection");
    expect(useLivePageSource).toContain("save_live_table_layout");
    expect(useLivePageSource).toContain("fetchTraySummary");
    expect(useLivePageSource).toMatch(/trigger: LiveQueryTrigger/);
    expect(useLivePageSource).toContain("loadNext");
    expect(useLivePageSource).toContain("loadPrev");
    expect(useLivePageSource).toContain("pinLiveSummary");
  });

  it("解码 CloseState 三态，拒绝未知 mark", () => {
    expect(decodeCloseState({ requestId: "a", identity: "0:1", mark: "accepted" }, "0:1").mark).toBe(
      "accepted"
    );
    expect(decodeCloseState({ requestId: "a", identity: "0:1", mark: "closed" }, "0:1").mark).toBe("closed");
    expect(decodeCloseState({ requestId: "a", identity: "0:1", mark: "unconfirmed" }, "0:1").mark).toBe(
      "unconfirmed"
    );
    expect(() => decodeCloseState({ mark: "pending" }, "0:1")).toThrow(/关闭结果无效/);
  });
});

describe("useLivePage 游标翻页", () => {
  const hotspot = (identity: string, value: number) => ({
    identity,
    label: identity,
    host: identity,
    process: null,
    destination: null,
    value
  });

  const rawPage = (identity: string, next: { sortKey: string; identity: string } | null, summaryId: string) => ({
    rows: [{ identity, processPath: `C:\\${identity}.exe` }],
    nextCursor: next,
    matchedCount: LIST_PAGE_DEFAULT + 1,
    sampleUtc: 1,
    summary: {
      topDownload: hotspot(summaryId, summaryId === "hot-1" ? 100 : 200),
      topUpload: null
    }
  });

  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd !== "query_live_connections") {
        throw new Error(String(cmd));
      }
      const cursor = (args as { query: { cursor: { sortKey: string; identity: string } | null } }).query.cursor;
      if (cursor == null) {
        return rawPage("0:page-1", { sortKey: "0:page-1", identity: "0:page-1" }, "hot-1");
      }
      return rawPage("0:page-2", null, "hot-2");
    });
  });

  it("matchedCount > limit 时请求 nextCursor，得到不同 identity，summary 钉在第一页", async () => {
    const applied = defaultLiveQuery().filter;
    const sort = { sortField: "identity" as const, descending: false };
    const firstQuery = buildLiveQuery(applied, sort, firstLivePager().cursor);
    expect(firstQuery.cursor).toBeNull();
    expect(firstQuery.limit).toBe(LIST_PAGE_DEFAULT);
    const first = await queryLiveConnections(firstQuery);
    expect(first.matchedCount).toBeGreaterThan(LIST_PAGE_DEFAULT);
    expect(first.nextCursor).not.toBeNull();

    const pager = loadNextLivePage(firstLivePager(), first);
    expect(pager.pageNumber).toBe(2);
    expect(pager.cursor).toEqual(first.nextCursor);
    const secondQuery = buildLiveQuery(applied, sort, pager.cursor);
    expect(secondQuery.cursor).toEqual(first.nextCursor);
    const second = await queryLiveConnections(secondQuery);

    const firstIds = new Set(first.rows.map((row) => row.identity));
    const secondIds = new Set(second.rows.map((row) => row.identity));
    expect(firstIds).not.toEqual(secondIds);
    expect(second.matchedCount).toBe(first.matchedCount);
    expect(second.summary.topDownload?.identity).toBe("hot-2");
    const pinned = pinLiveSummary(first.summary, second, pager.cursor);
    expect(pinned).toEqual(first.summary);
    expect(withPinnedSummary(second, pinned).summary).toEqual(first.summary);
    expect(loadPrevLivePage(pager).cursor).toBeNull();
    expect(vi.mocked(invoke).mock.calls.map((call) => (call[1] as { query: { cursor: unknown } }).query.cursor)).toEqual(
      [null, first.nextCursor]
    );
  });
});
