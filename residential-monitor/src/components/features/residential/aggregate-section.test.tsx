import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { ReportResult } from "../../../dto";
import { AggregateSection, AggregateStatus } from "./aggregate-section";

describe("家宽地址排名界面", () => {
  it("默认下行方向可见且排序表头具有可访问状态", () => {
    const html = renderToStaticMarkup(
      <AggregateSection
        locale="zh"
        timeRange={{ preset: "24h", startUtc: 0, endUtc: 86_400_000 }}
        autoRefresh={true}
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
        autoRefresh={true}
        share={null}
        shareLoading={false}
        shareError={null}
      />
    );
    expect(html).toContain("Residential destination ranking");
    expect(html).toContain("Ranking direction");
    expect(html).toContain("Download share");
  });

  it("暂停状态只显示一个共享状态，不增加第二个刷新按钮", () => {
    const html = renderToStaticMarkup(
      <AggregateSection
        locale="zh"
        timeRange={{ preset: "24h", startUtc: 0, endUtc: 86_400_000 }}
        autoRefresh={false}
        share={null}
        shareLoading={false}
        shareError={null}
      />
    );
    expect(html).toContain('data-state="paused"');
    expect(html).toContain("统计窗口保持为当前快照");
    expect(html).not.toContain("刷新历史统计");
  });

  it("状态元信息取自实际 queryEcho 与 generatedUtc，错误只出现一次", () => {
    const start = 1_725_081_600;
    const end = start + 3_600;
    const generated = end + 5;
    const result = {
      queryEcho: { rangeStartUtc: start, rangeEndUtc: end },
      generatedUtc: generated
    } as ReportResult;
    const html = renderToStaticMarkup(
      <AggregateStatus
        locale="en"
        result={result}
        state="error"
        errorZh="query failed"
        autoRefresh={true}
      />
    );
    expect(html).toContain("Statistics window");
    expect(html).toContain(formatUtcForTest(start));
    expect(html).toContain(formatUtcForTest(end));
    expect(html).toContain("Last updated");
    expect(html).toContain(formatUtcForTest(generated));
    expect(html.match(/query failed/g)).toHaveLength(1);
  });
});

function formatUtcForTest(value: number): string {
  return new Date(value * 1000).toLocaleString();
}
