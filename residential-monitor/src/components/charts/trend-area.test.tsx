import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { RankBar } from "./rank-bar";
import { TrendArea } from "./trend-area";

describe("图表空态", () => {
  it("无数据时渲染虚线空态，不渲染 0 高度图", () => {
    const trend = renderToStaticMarkup(
      <TrendArea locale="zh" data={[]} loading={false} emptyHint="该区间没有可绘制的序列。" />
    );
    expect(trend).toContain("border-dashed");
    expect(trend).toContain("无趋势数据");
    expect(trend).toContain("h-[200px]");
    expect(trend).not.toContain("recharts");

    const rank = renderToStaticMarkup(
      <RankBar locale="zh" data={[]} loading={false} emptyHint="能力不足" />
    );
    expect(rank).toContain("border-dashed");
    expect(rank).toContain("无排名数据");
    expect(rank).not.toContain("recharts");
  });
});
