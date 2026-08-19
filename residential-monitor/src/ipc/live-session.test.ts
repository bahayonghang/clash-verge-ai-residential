import { describe, expect, it } from "vitest";
import { defaultLiveQuery, isTauriRuntime, LIST_PAGE_DEFAULT } from "./live-session";
import liveSessionSource from "./live-session.ts?raw";
import mainSource from "../main.ts?raw";

describe("live-session", () => {
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

  it("产品源码不再把 window.message 当 Channel", () => {
    expect(mainSource).not.toMatch(/addEventListener\(\s*["']message["']/);
    expect(mainSource).not.toMatch(/window\.addEventListener\(\s*["']message["']/);
    expect(liveSessionSource).toContain("@tauri-apps/api/core");
    expect(liveSessionSource).toContain("subscribe_monitor");
    expect(liveSessionSource).toContain("query_live_connections");
  });
});
