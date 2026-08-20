import { describe, expect, it } from "vitest";
import {
  decodeLiveConnectionPage,
  defaultLiveQuery,
  isTauriRuntime,
  LIST_PAGE_DEFAULT
} from "./live-session";
import liveSessionSource from "./live-session.ts?raw";
import mainSource from "../main.ts?raw";

describe("live-session", () => {
  const hotspot = {
    identity: "0:download",
    label: "downloads.example",
    host: "downloads.example",
    process: "browser.exe",
    destination: "203.0.113.10:443",
    value: 4096
  };

  const page = (overrides: Record<string, unknown> = {}): Record<string, unknown> => ({
    rows: [],
    nextCursor: null,
    matchedCount: 1,
    sampleUtc: 1_723_456_789,
    summary: { topDownload: hotspot, topUpload: null },
    ...overrides
  });

  it("默认查询与 C2 LIST_PAGE_DEFAULT 对齐", () => {
    const query = defaultLiveQuery();
    expect(LIST_PAGE_DEFAULT).toBe(200);
    expect(query.sortField).toBe("identity");
    expect(query.descending).toBe(false);
    expect(query.cursor).toBeNull();
    expect(query.limit).toBe(200);
    expect(query.filter).toEqual({
      category: null,
      host: null,
      process: null,
      rule: null,
      chain: null,
      network: null,
      residentialOnly: true,
      clauses: []
    });
  });

  it("非 Tauri 预览态不伪造运行时", () => {
    expect(isTauriRuntime()).toBe(false);
  });

  it("保留同一后端快照的完整筛选摘要，不从分页 rows 推导", () => {
    const decoded = decodeLiveConnectionPage(
      page({
        rows: [{ identity: "0:page-row", download: 999_999 }],
        summary: { topDownload: null, topUpload: hotspot }
      })
    );
    expect(decoded.matchedCount).toBe(1);
    expect(decoded.sampleUtc).toBe(1_723_456_789);
    expect(decoded.summary.topDownload).toBeNull();
    expect(decoded.summary.topUpload).toEqual(hotspot);
  });

  it("接受空匹配的 null 热点，不伪造零值", () => {
    const decoded = decodeLiveConnectionPage(
      page({ matchedCount: 0, sampleUtc: null, summary: { topDownload: null, topUpload: null } })
    );
    expect(decoded.summary.topDownload).toBeNull();
    expect(decoded.summary.topUpload).toBeNull();
    expect(decoded.sampleUtc).toBeNull();
  });

  it("拒绝旧响应或缺失的摘要字段", () => {
    expect(() => decodeLiveConnectionPage({ rows: [], nextCursor: null })).toThrow(/matchedCount/);
    expect(() => decodeLiveConnectionPage(page({ summary: { topDownload: null } }))).toThrow(/字段缺失/);
    expect(() => decodeLiveConnectionPage(page({ sampleUtc: undefined }))).toThrow(/sampleUtc/);
    expect(() => decodeLiveConnectionPage(page({ sampleUtc: 1.5 }))).toThrow(/sampleUtc/);
  });

  it("不把 processPath 或原始规则载荷拷进热点", () => {
    const decoded = decodeLiveConnectionPage(
      page({
        summary: {
          topDownload: { ...hotspot, processPath: "C:\\secret\\browser.exe", rulePayload: "raw" },
          topUpload: null
        }
      })
    );
    expect(decoded.summary.topDownload).toEqual(hotspot);
    expect(decoded.summary.topDownload).not.toHaveProperty("processPath");
    expect(decoded.summary.topDownload).not.toHaveProperty("rulePayload");
  });

  it("拒绝非法热点、计数和游标", () => {
    expect(() => decodeLiveConnectionPage(page({ matchedCount: -1 }))).toThrow(/matchedCount/);
    expect(() => decodeLiveConnectionPage(page({ summary: { topDownload: { ...hotspot, value: -1 }, topUpload: null } }))).toThrow(
      /topDownload/
    );
    expect(() => decodeLiveConnectionPage(page({ summary: { topDownload: { ...hotspot, host: undefined }, topUpload: null } }))).toThrow(
      /topDownload/
    );
    expect(() => decodeLiveConnectionPage(page({ nextCursor: { sortKey: 3, identity: "0:x" } }))).toThrow(
      /nextCursor/
    );
  });

  it("产品源码不再把 window.message 当 Channel", () => {
    expect(mainSource).not.toMatch(/addEventListener\(\s*["']message["']/);
    expect(mainSource).not.toMatch(/window\.addEventListener\(\s*["']message["']/);
    expect(liveSessionSource).toContain("@tauri-apps/api/core");
    expect(liveSessionSource).toContain("subscribe_monitor");
    expect(liveSessionSource).toContain("query_live_connections");
  });
});
