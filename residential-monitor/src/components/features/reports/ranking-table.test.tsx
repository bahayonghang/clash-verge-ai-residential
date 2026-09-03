import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { ShareModel } from "../../../format/report-view";
import { ReportInspectProvider } from "./inspect-context";
import { RankingTable } from "./ranking-table";

const sources = import.meta.glob(["./ranking-table.tsx"], {
  query: "?raw",
  eager: true,
  import: "default"
}) as Record<string, string>;

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
    expect((thead.match(/data-sort-icon=/g) ?? []).length).toBe(3);
    const share = thead.split("</th>").find((chunk) => chunk.includes(">份额<") || chunk.endsWith(">份额"));
    expect(share).toBeDefined();
    expect(share).not.toContain("<button");
  });

  it("源码不对 Top N 做本页重排", () => {
    const source = Object.values(sources)[0] ?? "";
    expect(source).not.toContain("sortRows");
    expect(source).not.toMatch(/copy\.sort\s*\(/);
    expect(source).not.toMatch(/left\.upload/);
    expect(source).not.toMatch(/left\.download/);
  });

  it("行序与后端 rankings 投影一致", () => {
    const share: ShareModel = {
      drawPie: true,
      remainder: 0,
      denominator: 100,
      capabilityUnsupported: false,
      rows: [
        { kind: "rank", identity: "up.host", label: "up.host", upload: 90, download: 10, share: 0.1 },
        { kind: "rank", identity: "down.host", label: "down.host", upload: 10, download: 90, share: 0.9 }
      ]
    };
    const html = renderToStaticMarkup(
      <ReportInspectProvider locale="zh" share={share} series={[]}>
        <RankingTable locale="zh" share={share} sort={{ field: "upload", descending: true }} />
      </ReportInspectProvider>
    );
    expect(html.indexOf("up.host")).toBeGreaterThan(-1);
    expect(html.indexOf("up.host")).toBeLessThan(html.indexOf("down.host"));
  });
});
