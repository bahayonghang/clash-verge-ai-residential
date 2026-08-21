import { describe, expect, it } from "vitest";
import { defaultLiveQuery } from "../ipc/live-session";
import {
  applyLiveQueryFailure,
  applyLiveQuerySuccess,
  buildLiveQuery,
  decodeCloseState,
  startLiveQuery,
  type LiveQuerySlice
} from "./use-live-page";
import useLivePageSource from "./use-live-page.ts?raw";

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
