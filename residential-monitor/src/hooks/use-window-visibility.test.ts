import { describe, expect, it, vi } from "vitest";
import { decodeWindowVisibility, observeWindowVisibility } from "./use-window-visibility";

describe("原生窗口可见性边界", () => {
  it("只接受自有 boolean 字段，拒绝缺失或猜测值", () => {
    expect(decodeWindowVisibility({ visible: false })).toBe(false);
    expect(decodeWindowVisibility({ visible: true })).toBe(true);
    for (const raw of [null, {}, { visible: 1 }, { visible: null }, Object.create({ visible: true })]) {
      expect(() => decodeWindowVisibility(raw)).toThrow();
    }
  });

  it("先监听再读初值，迟到初值不能覆盖隐藏事件", async () => {
    const order: string[] = [];
    let receive!: (raw: unknown) => void;
    let finish!: (raw: unknown) => void;
    const stop = vi.fn();
    const visible = vi.fn();
    const error = vi.fn();
    const unlisten = await observeWindowVisibility(visible, error, {
      listen: async (callback) => {
        order.push("listen");
        receive = callback;
        return stop;
      },
      read: () => {
        order.push("read");
        return new Promise((resolve) => { finish = resolve; });
      }
    });
    receive({ visible: false });
    finish({ visible: true });
    await Promise.resolve();
    expect(order).toEqual(["listen", "read"]);
    expect(visible.mock.calls).toEqual([[false]]);
    expect(error).not.toHaveBeenCalled();
    receive({ visible: "true" });
    expect(error).toHaveBeenCalledOnce();
    unlisten();
    expect(stop).toHaveBeenCalledOnce();
  });
});
