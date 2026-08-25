import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { AggregateSection } from "./aggregate-section";

describe("家宽地址排名界面", () => {
  it("默认下行方向可见且排序表头具有可访问状态", () => {
    const html = renderToStaticMarkup(
      <AggregateSection
        locale="zh"
        timeRange={{ preset: "24h", startUtc: 0, endUtc: 86_400_000 }}
        share={null}
        shareLoading={false}
        shareError={null}
      />
    );
    expect(html).toContain("家宽目的地址排名");
    expect(html).toContain('aria-label="排名方向"');
    expect(html).toMatch(/aria-pressed="true"[^>]*>下行<\/button>/);
    expect(html).toMatch(/aria-sort="descending"[^>]*>.*?下行/s);
    expect(html).toMatch(/aria-sort="none"[^>]*>.*?上行/s);
    expect(html).toContain("下行份额");
  });

  it("英文方向与份额文案来自同一键集合", () => {
    const html = renderToStaticMarkup(
      <AggregateSection
        locale="en"
        timeRange={{ preset: "24h", startUtc: 0, endUtc: 86_400_000 }}
        share={null}
        shareLoading={false}
        shareError={null}
      />
    );
    expect(html).toContain("Residential destination ranking");
    expect(html).toContain("Ranking direction");
    expect(html).toContain("Download share");
  });
});
