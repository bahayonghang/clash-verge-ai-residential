import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { withDisplayOperation } from "./display-operation";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("展示查询独立取消", () => {
  beforeEach(() => vi.mocked(invoke).mockReset());

  it("取消只发给自己的操作，完成后释放注册项", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);
    const abort = new AbortController();
    let finish!: () => void;
    let started!: string;
    const task = withDisplayOperation(abort.signal, async (id) => {
      started = id;
      await new Promise<void>((resolve) => { finish = resolve; });
      return 1;
    });
    await Promise.resolve();
    abort.abort();
    finish();
    expect(await task).toBe(1);
    expect(vi.mocked(invoke).mock.calls.map(([cmd, args]) => [cmd, (args as { operationId: string }).operationId])).toEqual([
      ["start_operation", started], ["cancel_operation", started], ["finish_operation", started]
    ]);
  });

  it("start 期间隐藏不启动 SQL，查询错误仍清理", async () => {
    let start!: () => void;
    vi.mocked(invoke).mockImplementation((cmd) => cmd === "start_operation"
      ? new Promise<void>((resolve) => { start = resolve; })
      : Promise.resolve());
    const abort = new AbortController();
    const run = vi.fn(async () => 1);
    const task = withDisplayOperation(abort.signal, run);
    abort.abort();
    start();
    await expect(task).rejects.toThrow("取消");
    expect(run).not.toHaveBeenCalled();
    vi.mocked(invoke).mockResolvedValue(undefined);
    await expect(withDisplayOperation(new AbortController().signal, async () => {
      throw new Error("query failed");
    })).rejects.toThrow("query failed");
    expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "finish_operation")).toHaveLength(2);
  });
});
