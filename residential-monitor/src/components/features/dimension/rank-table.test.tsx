import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { ReportResult } from "../../../dto";
import { UNKNOWN_RANK_IDENTITY } from "../../../format/rank";
import { RankTable } from "./rank-table";

function report(over: Partial<ReportResult> = {}): ReportResult {
  return {
    schemaVersion: 1,
    dataVersion: 1,
    reportSnapshotToken: "tok",
    queryEcho: {
      rangeStartUtc: 0,
      rangeEndUtc: 60,
      displayTimezone: "local",
      granularity: "minute1",
      filters: { category: null, host: null, process: null, rule: null, chain: null, network: null },
      grouping: "host",
      targetPolicy: "historical",
      comparison: null,
      sort: { field: "download", descending: true },
      page: { limit: 200, after: null },
      topN: 10,
      includeSessions: false
    },
    totals: {
      upload: 100,
      download: 400,
      connectionCount: 4,
      activeDurationSec: 1,
      previousUpload: null,
      previousDownload: null
    },
    series: [],
    rankings: [
      {
        identity: "a.example",
        label: "a.example",
        upload: 10,
        download: 90,
        connectionCount: 1,
        activeDurationSec: 1
      },
      {
        identity: UNKNOWN_RANK_IDENTITY,
        label: "未知",
        upload: 5,
        download: 20,
        connectionCount: 1,
        activeDurationSec: 1
      }
    ],
    coverage: { status: "ok", coveredSec: 60, gapSec: 0, slices: [] },
    attributionQuality: {
      knownUpload: 1,
      knownDownload: 2,
      missingUpload: 0,
      missingDownload: 0,
      knownConnections: 1,
      missingConnections: 0,
      status: "complete"
    },
    drilldownCapability: {
      sessions: true,
      currentPolicy: true,
      crossDimension: true,
      exactTopN: true,
      noteZh: ""
    },
    policyMetadata: { targetPolicy: "historical", policyVersion: null, noteZh: "" },
    dataTier: "Raw",
    namedSql: ["rank_raw"],
    unit: "byte",
    generatedUtc: 1,
    ...over
  };
}

describe("排名表未知行与能力", () => {
  it("主机维 __unknown__ 可下钻检查组成", () => {
    const html = renderToStaticMarkup(
      <RankTable
        locale="zh"
        kind="host"
        result={report()}
        loading={false}
        errorZh={null}
        selectedIdentity={null}
        onSelect={() => undefined}
      />
    );
    expect(html).toContain(`data-identity="${UNKNOWN_RANK_IDENTITY}"`);
    expect(html).toContain('data-unknown="1"');
    expect(html).toContain("未归因主机");
    expect(html).toMatch(/data-unknown="1"[\s\S]*data-drill="1"/);
    expect(html).not.toContain("未知行不能下钻");
  });

  it("进程维 __unknown__ 可下钻", () => {
    const html = renderToStaticMarkup(
      <RankTable
        locale="zh"
        kind="process"
        result={report()}
        loading={false}
        errorZh={null}
        selectedIdentity={null}
        onSelect={() => undefined}
      />
    );
    expect(html).toMatch(/data-unknown="1"[\s\S]*data-drill="1"/);
    expect(html).toContain("控制器未报告进程");
    expect(html).not.toContain("未知行不能下钻");
  });

  it("规则维 __unknown__ 无下钻入口", () => {
    const html = renderToStaticMarkup(
      <RankTable
        locale="zh"
        kind="rule"
        result={report()}
        loading={false}
        errorZh={null}
        selectedIdentity={null}
        onSelect={() => undefined}
      />
    );
    expect(html).toMatch(/data-unknown="1"[\s\S]*data-drill="0"/);
    expect(html).toContain("未知行不能下钻");
  });

  it("exactTopN 为 false 时显示能力说明，不渲染假装无流量的空表", () => {
    const html = renderToStaticMarkup(
      <RankTable
        locale="zh"
        kind="process"
        result={report({
          rankings: [],
          drilldownCapability: {
            sessions: false,
            currentPolicy: false,
            crossDimension: false,
            exactTopN: false,
            noteZh: "该维度尚未五维物化，精确 Top N 不可用。"
          }
        })}
        loading={false}
        errorZh={null}
        selectedIdentity={null}
        onSelect={() => undefined}
      />
    );
    expect(html).toContain("data-capability-note");
    expect(html).toContain("该维度尚未五维物化，精确 Top N 不可用。");
    expect(html).not.toContain("<table");
    expect(html).not.toContain("该区间没有排名。");
  });

  it("占比分母取 totals.download，不取可见行之和", () => {
    const html = renderToStaticMarkup(
      <RankTable
        locale="zh"
        kind="host"
        result={report()}
        loading={false}
        errorZh={null}
        selectedIdentity={null}
        onSelect={() => undefined}
      />
    );
    expect(html).toContain("22.5%");
    expect(html).toContain("5.0%");
    expect(html).not.toContain("81.8%");
  });

  it("crossDimension 为 false 时不渲染下钻列", () => {
    const html = renderToStaticMarkup(
      <RankTable
        locale="zh"
        kind="host"
        result={report({
          drilldownCapability: {
            sessions: false,
            currentPolicy: false,
            crossDimension: false,
            exactTopN: true,
            noteZh: "超出 raw 期，跨维下钻不可用。"
          }
        })}
        loading={false}
        errorZh={null}
        selectedIdentity={null}
        onSelect={() => undefined}
      />
    );
    expect(html).toContain("<table");
    expect(html).not.toContain("data-drill=");
  });

  it("默认下行降序带图标，不可排序列无按钮，不含字段归因", () => {
    const html = renderToStaticMarkup(
      <RankTable
        locale="zh"
        kind="rule"
        result={report()}
        loading={false}
        errorZh={null}
        selectedIdentity={null}
        onSelect={() => undefined}
      />
    );
    expect(html).not.toContain("字段归因");
    expect(html).toContain("bg-muted/40");
    const thead = html.slice(html.indexOf("<thead"), html.indexOf("</thead>"));
    const download = thead.split("</th>").find((chunk) => chunk.includes(">下行<") || chunk.endsWith(">下行"));
    expect(download).toBeDefined();
    expect(download).toContain('aria-sort="descending"');
    expect(download).toContain('data-sort-icon="descending"');
    expect((thead.match(/data-sort-icon=/g) ?? []).length).toBe(4);
    expect(thead).toMatch(/aria-sort="none"/);
    expect((thead.match(/<button/g) ?? []).length).toBe(4);
    expect(thead).toContain(">排名</th>");
    expect(thead).toContain(">份额</th>");
    expect(thead).toContain(">下钻</th>");
  });
});
