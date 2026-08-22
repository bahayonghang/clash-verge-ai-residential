import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { LiveOverview } from "../../../dto";
import { EMPTY_METADATA_COVERAGE } from "../../../dto";
import { SCHEMA_VERSION } from "../../../dto";
import { categoryRows } from "../../../format/overview";
import { CaliberGrid } from "./caliber-grid";

function overview(over: Partial<LiveOverview> = {}): LiveOverview {
  return {
    schemaVersion: SCHEMA_VERSION,
    observationPhase: "current",
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
    metadataCoverage: EMPTY_METADATA_COVERAGE,
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

  it("连接中与基线阶段不把 null 或 activeCount 0 显示为真实值", () => {
    const connecting = renderToStaticMarkup(
      <CaliberGrid
        locale="zh"
        overview={overview({
          observationPhase: "connecting",
          meterUpload: null,
          meterDownload: null,
          activeCount: 0
        })}
      />
    );
    expect(fieldValue(connecting, "meter-upload")).toBe("等待控制器连接");
    expect(fieldValue(connecting, "active-count")).toBe("—");
    const baseline = renderToStaticMarkup(
      <CaliberGrid locale="zh" overview={overview({ observationPhase: "baselinePending" })} />
    );
    expect(fieldValue(baseline, "meter-upload")).toBe("正在建立差分基线");
    expect(fieldValue(baseline, "active-count")).toBe("—");
  });

  it("current、暂停、断连、重同步与解码失败使用各自语义", () => {
    const current = renderToStaticMarkup(
      <CaliberGrid locale="zh" overview={overview({ meterUpload: 0, activeCount: 0 })} />
    );
    expect(fieldValue(current, "meter-upload")).toBe("0 B");
    expect(fieldValue(current, "active-count")).toBe("0");

    const disconnected = renderToStaticMarkup(
      <CaliberGrid locale="zh" overview={overview({ observationPhase: "disconnected" })} />
    );
    expect(fieldValue(disconnected, "meter-upload")).toBe("10 B · 上次值");
    expect(fieldValue(disconnected, "active-count")).toBe("—");

    for (const [phase, copy] of [
      ["paused", "采集已暂停，当前值不可用"],
      ["resyncRequired", "需要重新同步，当前值不可用"],
      ["decodeFailed", "控制器响应无法解码，当前值不可用"]
    ] as const) {
      const html = renderToStaticMarkup(
        <CaliberGrid
          locale="zh"
          overview={overview({ observationPhase: phase, meterUpload: null })}
        />
      );
      expect(fieldValue(html, "meter-upload")).toBe(copy);
      expect(fieldValue(html, "active-count")).toBe("—");
    }
  });
});
