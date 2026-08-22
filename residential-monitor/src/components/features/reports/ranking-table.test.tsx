import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { ReportInspectProvider } from "./inspect-context";
import { RankingTable } from "./ranking-table";

describe("报告排名表头", () => {
  it("可排序列带 lucide 图标，默认下行为降序", () => {
    const html = renderToStaticMarkup(
      <ReportInspectProvider locale="zh" share={null} series={[]}>
        <RankingTable locale="zh" share={null} />
      </ReportInspectProvider>
    );
    const thead = html.slice(html.indexOf("<thead"), html.indexOf("</thead>"));
    const download = thead.split("</th>").find((chunk) => chunk.includes(">下行<") || chunk.endsWith(">下行"));
    expect(download).toBeDefined();
    expect(download).toContain('aria-sort="descending"');
    expect(download).toContain('data-sort-icon="descending"');
    expect((thead.match(/data-sort-icon=/g) ?? []).length).toBe(4);
  });
});
