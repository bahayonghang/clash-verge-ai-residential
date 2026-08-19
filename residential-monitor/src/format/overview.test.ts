import { describe, expect, it } from "vitest";
import { categoryRows } from "./overview";

describe("categoryRows", () => {
  it("unions upload and download keys and keeps a missing side as null", () => {
    const rows = categoryRows({ 家宽: 10, 其他: 2 }, { 家宽: 20, 办公: 4 });
    const byName = Object.fromEntries(rows.map((row) => [row.name, row]));
    expect(Object.keys(byName).sort()).toEqual(["其他", "办公", "家宽"].sort());
    expect(byName["家宽"]).toEqual({ name: "家宽", upload: 10, download: 20 });
    expect(byName["其他"]).toEqual({ name: "其他", upload: 2, download: null });
    expect(byName["办公"]).toEqual({ name: "办公", upload: null, download: 4 });
  });

  it("returns an empty list when both maps are empty", () => {
    expect(categoryRows({}, {})).toEqual([]);
  });

  it("preserves zero instead of turning it into null", () => {
    expect(categoryRows({ 家宽: 0 }, { 家宽: 0 })).toEqual([
      { name: "家宽", upload: 0, download: 0 }
    ]);
  });
});
