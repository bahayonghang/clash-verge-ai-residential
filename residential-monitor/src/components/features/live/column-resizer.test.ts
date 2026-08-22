import { describe, expect, it } from "vitest";
import { WIDTH_MAX, minWidth } from "../../../live-table-layout";
import { persistOnRelease, widthFromColumnKey } from "./column-resizer";

describe("列宽键盘把手", () => {
  it("四键调整并 clamp", () => {
    const min = minWidth("host");
    expect(widthFromColumnKey("ArrowLeft", false, 180, min, WIDTH_MAX)).toBe(172);
    expect(widthFromColumnKey("ArrowRight", false, 180, min, WIDTH_MAX)).toBe(188);
    expect(widthFromColumnKey("ArrowLeft", true, 180, min, WIDTH_MAX)).toBe(148);
    expect(widthFromColumnKey("ArrowRight", true, 180, min, WIDTH_MAX)).toBe(212);
    expect(widthFromColumnKey("Home", false, 180, min, WIDTH_MAX)).toBe(min);
    expect(widthFromColumnKey("End", false, 180, min, WIDTH_MAX)).toBe(WIDTH_MAX);
    expect(widthFromColumnKey("ArrowLeft", false, min, min, WIDTH_MAX)).toBe(min);
    expect(widthFromColumnKey("ArrowRight", false, WIDTH_MAX, min, WIDTH_MAX)).toBe(WIDTH_MAX);
    expect(widthFromColumnKey("Enter", false, 180, min, WIDTH_MAX)).toBeNull();
  });

  it("松手且有变化才持久化一次", () => {
    expect(persistOnRelease(true, true)).toBe(true);
    expect(persistOnRelease(false, true)).toBe(false);
    expect(persistOnRelease(true, false)).toBe(false);
    expect(persistOnRelease(false, false)).toBe(false);
  });
});
