import { describe, expect, it } from "vitest";
import { SHELL_WIDTH_MAX, SHELL_WIDTH_MIN } from "../shell-width";
import { persistOnRelease, widthFromResizeKey } from "./use-sidebar-resize";

describe("侧栏分隔条", () => {
  it("方向键按步长调整并 clamp", () => {
    expect(widthFromResizeKey("ArrowLeft", false, 220)).toBe(212);
    expect(widthFromResizeKey("ArrowRight", false, 220)).toBe(228);
    expect(widthFromResizeKey("ArrowLeft", true, 220)).toBe(188);
    expect(widthFromResizeKey("ArrowRight", true, 220)).toBe(252);
    expect(widthFromResizeKey("Home", false, 220)).toBe(SHELL_WIDTH_MIN);
    expect(widthFromResizeKey("End", false, 220)).toBe(SHELL_WIDTH_MAX);
    expect(widthFromResizeKey("ArrowLeft", false, SHELL_WIDTH_MIN)).toBe(SHELL_WIDTH_MIN);
    expect(widthFromResizeKey("ArrowRight", false, SHELL_WIDTH_MAX)).toBe(SHELL_WIDTH_MAX);
    expect(widthFromResizeKey("Enter", false, 220)).toBe(null);
  });

  it("只在松手且宽度已变时持久化一次", () => {
    expect(persistOnRelease(true, true)).toBe(true);
    expect(persistOnRelease(false, true)).toBe(false);
    expect(persistOnRelease(true, false)).toBe(false);
    expect(persistOnRelease(false, false)).toBe(false);
  });
});
