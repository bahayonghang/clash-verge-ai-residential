import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { RankBarHover, ShareSliceHover } from "./chart-hover";

const chartSources = import.meta.glob("./*.tsx", {
  query: "?raw",
  eager: true,
  import: "default"
}) as Record<string, string>;

describe("图表浮层", () => {
  it("条形图浮层用条目名和格式化数值，不含英文 value", () => {
    const html = renderToStaticMarkup(
      <RankBarHover
        active
        payload={[{ value: 4939212390, payload: { label: "IPCIDR" } }]}
        label={4939212390}
        formatValue={() => "4.6 GiB"}
      />
    );
    expect(html).toContain("IPCIDR");
    expect(html).toContain("4.6 GiB");
    expect(html).toContain("bg-popover");
    expect(html).toContain("text-popover-foreground");
    expect(html).not.toContain("value :");
    expect(html).not.toContain("value:");
  });

  it("扇形图浮层用切片名，不含系列名 value", () => {
    const html = renderToStaticMarkup(
      <ShareSliceHover active payload={[{ name: "value", value: 12, payload: { label: "GPT" } }]} />
    );
    expect(html).toContain("GPT");
    expect(html).toContain("12");
    expect(html).toContain("bg-popover");
    expect(html).not.toContain("value :");
  });

  it("RankBar Y 轴不用 #888888，ShareDonut 无 inspect 时挂同一浮层", () => {
    const rankBar = Object.entries(chartSources).find(([file]) => file.endsWith("rank-bar.tsx"))?.[1] ?? "";
    const shareDonut = Object.entries(chartSources).find(([file]) => file.endsWith("share-donut.tsx"))?.[1] ?? "";
    expect(rankBar).toContain("RankBarHover");
    expect(rankBar).toContain("var(--muted-foreground)");
    expect(rankBar).not.toContain("#888888");
    expect(shareDonut).toContain("ShareSliceHover");
    expect(shareDonut).toContain("onHover ? null");
  });
});
