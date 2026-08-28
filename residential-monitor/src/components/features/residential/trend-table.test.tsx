import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { ReportResult } from "../../../dto";
import { TrendTable } from "./trend-table";

function point(bucketUtc: number): ReportResult["series"][number] {
  return {
    bucketUtc,
    upload: bucketUtc,
    download: bucketUtc * 2,
    connectionCount: 1,
    activeDurationSec: 60
  };
}

describe("家宽趋势明细", () => {
  it("多桶最新置顶并带 sticky、对齐和窄屏滚动样式", () => {
    const html = renderToStaticMarkup(
      <TrendTable locale="zh" series={[point(10), point(30), point(20)]} loading={false} />
    );
    expect(html.indexOf('data-bucket-utc="30"')).toBeLessThan(
      html.indexOf('data-bucket-utc="20"')
    );
    expect(html.indexOf('data-bucket-utc="20"')).toBeLessThan(
      html.indexOf('data-bucket-utc="10"')
    );
    expect(html).toContain("overflow-auto");
    expect(html).toContain("rounded-md");
    expect(html).toContain("sticky top-0");
    expect(html).toContain("min-w-[36rem]");
    expect(html).toContain("text-right tabular-nums");
    expect(html).toContain("hover:bg-muted/40");
    expect(html).toContain("时间");
    expect(html).toContain("上行");
    expect(html).toContain("下行");
  });

  it("空态和单桶在中英文下保持语义表头", () => {
    const empty = renderToStaticMarkup(<TrendTable locale="en" series={[]} loading={false} />);
    expect(empty).toContain("Time");
    expect(empty).toContain("Upload");
    expect(empty).toContain("Download");
    expect(empty).toContain("No trend data");

    const single = renderToStaticMarkup(
      <TrendTable locale="zh" series={[point(99)]} loading={false} />
    );
    expect(single.match(/data-bucket-utc=/g)).toHaveLength(1);
    expect(single).toContain('data-bucket-utc="99"');
  });
});
