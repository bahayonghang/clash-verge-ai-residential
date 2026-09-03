import { describe, expect, it } from "vitest";
import {
  DATA_COLUMNS,
  defaultDimensionRankTableLayout,
  parseDimensionRankTableLayout,
  rankTablePixelWidth,
  sanitizeDimensionRankTableLayout,
  setRankColumnWidth,
  WIDTH_MAX,
  WIDTH_MIN
} from "./dimension-rank-table-layout";

describe("维度排名表列宽", () => {
  it("defaults to every data column with template widths", () => {
    const layout = defaultDimensionRankTableLayout();
    expect(Object.keys(layout.widths)).toHaveLength(DATA_COLUMNS.length);
    expect(layout.widths.name).toBe(280);
    expect(layout.widths.attribution).toBe(160);
    expect(layout.widths.rank).toBeUndefined();
    expect(layout.widths.drill).toBeUndefined();
  });

  it("drops unknown keys and clamps width", () => {
    const layout = sanitizeDimensionRankTableLayout({
      widths: { name: 10, download: 9000, rank: 40, drill: 90, nope: 200 }
    });
    expect(layout.widths.name).toBe(WIDTH_MIN);
    expect(layout.widths.download).toBe(WIDTH_MAX);
    expect(layout.widths.rank).toBeUndefined();
    expect(layout.widths.drill).toBeUndefined();
    expect(layout.widths.nope).toBeUndefined();
    expect(layout.widths.attribution).toBe(160);
  });

  it("parses missing bootstrap payload as default", () => {
    expect(parseDimensionRankTableLayout(undefined)).toEqual(defaultDimensionRankTableLayout());
    expect(parseDimensionRankTableLayout("bad")).toEqual(defaultDimensionRankTableLayout());
    expect(parseDimensionRankTableLayout({ widths: null })).toEqual(defaultDimensionRankTableLayout());
  });

  it("clamps a resized column without changing other widths", () => {
    const original = defaultDimensionRankTableLayout();
    const resized = setRankColumnWidth(original, "name", 999);
    expect(resized.widths.name).toBe(WIDTH_MAX);
    expect(resized.widths.download).toBe(original.widths.download);
    expect(resized.widths.attribution).toBe(original.widths.attribution);
  });

  it("chain page ignores attribution width in the table pixel total", () => {
    const layout = defaultDimensionRankTableLayout();
    const withAttr = rankTablePixelWidth(layout, { attribution: true, drill: true });
    const chain = rankTablePixelWidth(layout, { attribution: false, drill: true });
    expect(withAttr - chain).toBe(layout.widths.attribution);
  });
});
