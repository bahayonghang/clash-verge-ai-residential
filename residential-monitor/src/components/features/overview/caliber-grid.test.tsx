import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { LiveOverview } from "../../../dto";
import { SCHEMA_VERSION } from "../../../dto";
import { categoryRows } from "../../../format/overview";
import { CaliberGrid } from "./caliber-grid";

function overview(over: Partial<LiveOverview> = {}): LiveOverview {
  return {
    schemaVersion: SCHEMA_VERSION,
    meterUpload: 10,
    meterDownload: 20,
    attributedUpload: 8,
    attributedDownload: 16,
    categoryUpload: { 家宽: 8 },
    categoryDownload: { 家宽: 16 },
    otherUpload: 1,
    otherDownload: 2,
    gapUpload: 1,
    gapDownload: 2,
    overUpload: 0,
    overDownload: 0,
    activeCount: 3,
    lastSampleUtc: 1_700_000_000,
    coverageKind: null,
    coverageReason: null,
    health: { session: "connected", storageOk: true, storageReason: null },
    ...over
  };
}

function fieldValue(html: string, field: string): string {
  const match = html.match(new RegExp(`data-field="${field}"[^>]*>([^<]*)<`));
  if (!match || match[1] === undefined) {
    throw new Error(`missing ${field}`);
  }
  return match[1];
}

describe("概览口径卡", () => {
  it("meterUpload / gapUpload / overUpload 为 null 时显示未知，不显示 0", () => {
    const html = renderToStaticMarkup(
      <CaliberGrid
        locale="zh"
        overview={overview({
          meterUpload: null,
          gapUpload: null,
          overUpload: null,
          meterDownload: 20,
          gapDownload: 2,
          overDownload: 0
        })}
      />
    );
    expect(fieldValue(html, "meter-upload")).toBe("未知");
    expect(fieldValue(html, "gap-upload")).toBe("未知");
    expect(fieldValue(html, "over-upload")).toBe("未知");
    expect(fieldValue(html, "meter-upload")).not.toBe("0 B");
    expect(fieldValue(html, "over-download")).toBe("0 B");
  });

  it("categoryRows 缺失键仍为 null", () => {
    const rows = categoryRows({ 家宽: 8 }, { 家宽: 16, 办公: 4 });
    const office = rows.find((row) => row.name === "办公");
    expect(office?.upload).toBeNull();
    expect(office?.download).toBe(4);
  });
});
