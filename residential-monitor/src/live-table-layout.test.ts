import { describe, expect, it } from "vitest";
import {
  ACTION_COLUMN,
  DATA_COLUMNS,
  defaultLiveTableLayout,
  parseLiveTableLayout,
  sanitizeLiveTableLayout,
  setColumnHidden,
  setColumnWidth,
  tablePixelWidth,
  visibleDataColumns
} from "./live-table-layout";

describe("live table layout", () => {
  it("defaults to every data column visible with template widths", () => {
    const layout = defaultLiveTableLayout();
    expect(Object.keys(layout.widths)).toHaveLength(DATA_COLUMNS.length);
    expect(layout.hidden).toEqual([]);
    expect(layout.widths.host).toBe(180);
    expect(layout.widths.rateDownload).toBe(104);
    expect(layout.widths[ACTION_COLUMN]).toBeUndefined();
  });

  it("drops action and unknown keys and clamps width", () => {
    const layout = sanitizeLiveTableLayout({
      widths: { host: 10, download: 9000, action: 40, nope: 200 },
      hidden: ["action", "nope", "host"]
    });
    expect(layout.widths.host).toBe(140);
    expect(layout.widths.download).toBe(640);
    expect(layout.widths.action).toBeUndefined();
    expect(layout.hidden).toEqual(["host"]);
  });

  it("keeps one data column visible", () => {
    const allHidden = sanitizeLiveTableLayout({ widths: {}, hidden: [...DATA_COLUMNS] });
    const visible = visibleDataColumns(allHidden);
    expect(visible).toHaveLength(1);
    const again = setColumnHidden(allHidden, visible[0], true);
    expect(visibleDataColumns(again)).toEqual(visible);
  });

  it("parses missing bootstrap payload as default", () => {
    expect(parseLiveTableLayout(undefined)).toEqual(defaultLiveTableLayout());
    expect(parseLiveTableLayout("bad")).toEqual(defaultLiveTableLayout());
  });

  it("adds action width into the table pixel total", () => {
    const layout = setColumnWidth(defaultLiveTableLayout(), "host", 200);
    expect(tablePixelWidth(layout)).toBeGreaterThan(200);
  });
});
