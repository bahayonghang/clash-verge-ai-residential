import { describe, expect, it } from "vitest";
import { reportPieSvg, reportTrendSvg } from "./report-svg";
import { reportTrendModel } from "./report-view";

describe("reportPieSvg", () => {
  it("draws a full circle for a single slice", () => {
    const svg = reportPieSvg([{ kind: "remainder", value: 10 }], "下行份额");
    expect(svg).toContain("<circle");
    expect(svg).toContain("pie-remainder");
    expect(svg).toContain("aria-label=\"下行份额\"");
  });

  it("draws two rank slices plus remainder", () => {
    const svg = reportPieSvg(
      [
        { kind: "rank", value: 50 },
        { kind: "rank", value: 30 },
        { kind: "remainder", value: 20 }
      ],
      "share"
    );
    expect(svg.match(/<path /g)?.length).toBe(3);
    expect(svg).toContain("pie-slice-0");
    expect(svg).toContain("pie-slice-1");
    expect(svg).toContain("pie-remainder");
    expect(svg).not.toContain("<circle");
  });

  it("returns empty markup when every slice is zero", () => {
    expect(reportPieSvg([{ kind: "rank", value: 0 }], "x")).toBe("");
  });
});

describe("reportTrendSvg", () => {
  it("draws paired bars for a single bucket", () => {
    const svg = reportTrendSvg(reportTrendModel([{ bucketUtc: 1, upload: 2, download: 4 }]), "趋势");
    expect(svg.match(/<rect /g)?.length).toBe(2);
    expect(svg).toContain("trend-up");
    expect(svg).toContain("trend-down");
    expect(svg).not.toContain("<polyline");
  });

  it("draws two polylines for multiple buckets", () => {
    const svg = reportTrendSvg(
      reportTrendModel([
        { bucketUtc: 1, upload: 1, download: 2 },
        { bucketUtc: 2, upload: 2, download: 4 }
      ]),
      "趋势"
    );
    expect(svg.match(/<polyline /g)?.length).toBe(2);
    expect(svg).toContain("8.00,");
    expect(svg).toContain("312.00,");
  });

  it("returns empty markup for an empty series", () => {
    expect(reportTrendSvg(reportTrendModel([]), "趋势")).toBe("");
  });
});
