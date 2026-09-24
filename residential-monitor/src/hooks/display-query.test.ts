import { describe, expect, it, vi } from "vitest";
import { DisplayQuery, type DisplayRequest } from "./display-query";

const flush = async (): Promise<void> => {
  for (let i = 0; i < 6; i += 1) await Promise.resolve();
};

describe("展示查询生命周期", () => {
  it("慢查询只保留最新意图，旧选择不提交，最大一个在途", async () => {
    const queue = new DisplayQuery();
    queue.setActive(true);
    let finish!: () => void;
    let first!: DisplayRequest;
    const applied: string[] = [];
    const run = vi.fn(async (request: DisplayRequest) => {
      first = request;
      await new Promise<void>((resolve) => { finish = resolve; });
      if (request.isCurrent()) applied.push("old");
    });
    void queue.request("old", run);
    await flush();
    void queue.request("discarded", async () => { applied.push("discarded"); });
    void queue.request("latest", async () => { applied.push("latest"); });
    await flush();
    expect(first.signal.aborted).toBe(true);
    expect(run).toHaveBeenCalledTimes(1);
    expect(applied).toEqual([]);
    finish();
    await flush();
    expect(applied).toEqual(["latest"]);
  });

  it("持续同查询 tick 不饿死当前结果，突发刷新只补查一次", async () => {
    const queue = new DisplayQuery();
    queue.setActive(true);
    let finish!: () => void;
    let first!: DisplayRequest;
    let applied = 0;
    void queue.request("same", async (request) => {
      first = request;
      await new Promise<void>((resolve) => { finish = resolve; });
      if (request.isCurrent()) applied += 1;
    });
    await flush();
    const next = vi.fn(async () => { applied += 1; });
    for (let i = 0; i < 100; i += 1) void queue.request("same", next);
    expect(first.signal.aborted).toBe(false);
    finish();
    await flush();
    expect(applied).toBe(2);
    expect(next).toHaveBeenCalledTimes(1);
  });

  it("隐藏撤销在途结果，隐藏期间零查询，恢复合并对齐后的请求", async () => {
    const queue = new DisplayQuery();
    queue.setActive(true);
    let finish!: () => void;
    let old!: DisplayRequest;
    const calls: string[] = [];
    void queue.request("before", async (request) => {
      old = request;
      calls.push("before");
      await new Promise<void>((resolve) => { finish = resolve; });
    });
    await flush();
    queue.setActive(false);
    expect(old.signal.aborted).toBe(true);
    void queue.request("hidden", async () => { calls.push("hidden"); });
    finish();
    await flush();
    expect(old.isCurrent()).toBe(false);
    expect(calls).toEqual(["before"]);
    queue.setActive(true);
    queue.setActive(true);
    void queue.request("aligned", async () => { calls.push("aligned"); });
    await flush();
    expect(calls).toEqual(["before", "aligned"]);
    queue.clear();
    queue.setActive(true);
    await flush();
    expect(calls).toHaveLength(2);
  });
});
