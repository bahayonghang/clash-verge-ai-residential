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

  it("falls back safely for malformed layout fields", () => {
    expect(sanitizeLiveTableLayout({ widths: undefined, hidden: undefined })).toEqual(
      defaultLiveTableLayout()
    );
    expect(parseLiveTableLayout({ widths: null, hidden: { nope: true } })).toEqual(
      defaultLiveTableLayout()
    );
  });

  it("clamps a resized column without changing other widths", () => {
    const original = defaultLiveTableLayout();
    const resized = setColumnWidth(original, "host", 999);
    expect(resized.widths.host).toBe(640);
    expect(resized.widths.download).toBe(original.widths.download);
  });

  it("changes the table pixel width by only the resized visible column delta", () => {
    const original = defaultLiveTableLayout();
    const resized = setColumnWidth(original, "host", original.widths.host + 37);
    expect(tablePixelWidth(resized)).toBe(tablePixelWidth(original) + 37);
    expect(resized.widths.upload).toBe(original.widths.upload);
  });

  it("excludes hidden columns from the fixed table pixel width", () => {
    const original = defaultLiveTableLayout();
    const hidden = setColumnHidden(original, "host", true);
    expect(tablePixelWidth(hidden)).toBe(tablePixelWidth(original) - original.widths.host);
  });

  it("adds action width into the table pixel total", () => {
    const layout = setColumnWidth(defaultLiveTableLayout(), "host", 200);
    expect(tablePixelWidth(layout)).toBeGreaterThan(200);
  });
});
