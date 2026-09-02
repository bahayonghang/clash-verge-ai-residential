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
        activeDurationSec: 1,
        primaryExit: "DIRECT",
        exitMixed: false
      },
      {
        identity: UNKNOWN_RANK_IDENTITY,
        label: "未知",
        upload: 5,
        download: 20,
        connectionCount: 1,
        activeDurationSec: 1,
        primaryExit: null,
        exitMixed: false
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
    expect(thead).toContain("份额");
    expect(thead).toContain("归属");
    expect(thead).toContain(">下钻</th>");
  });
});

describe("排名表归属列", () => {
  it("仅 DIRECT 时归属为 DIRECT 且无混合", () => {
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
    expect(html).toContain('data-exit="DIRECT"');
    expect(html).toContain('data-exit-mixed="0"');
    expect(html).not.toContain("混合");
    expect(html).toContain("归属");
    const tableClass = html.match(/<table[^>]*class="([^"]*)"/)?.[1] ?? "";
    expect(tableClass.split(/\s+/)).not.toContain("w-full");
    expect(html).toContain("tabular-nums");
    expect(html).toContain("hover:bg-muted/40");
  });

  it("混合出口在主出口后标混合", () => {
    const html = renderToStaticMarkup(
      <RankTable
        locale="zh"
        kind="host"
        result={report({
          rankings: [
            {
              identity: "mix.example",
              label: "mix.example",
              upload: 1,
              download: 90,
              connectionCount: 1,
              activeDurationSec: 1,
              primaryExit: "PROXY",
              exitMixed: true
            }
          ]
        })}
        loading={false}
        errorZh={null}
        selectedIdentity={null}
        onSelect={() => undefined}
      />
    );
    expect(html).toContain('data-exit="PROXY"');
    expect(html).toContain('data-exit-mixed="1"');
    expect(html).toContain("PROXY · 混合");
  });

  it("无非空 chain_key 时归属为未知而不是 DIRECT", () => {
    const html = renderToStaticMarkup(
      <RankTable
        locale="zh"
        kind="host"
        result={report({
          rankings: [
            {
              identity: "blank.example",
              label: "blank.example",
              upload: 1,
              download: 90,
              connectionCount: 1,
              activeDurationSec: 1,
              primaryExit: null,
              exitMixed: false
            }
          ]
        })}
        loading={false}
        errorZh={null}
        selectedIdentity={null}
        onSelect={() => undefined}
      />
    );
    expect(html).not.toContain('data-exit="DIRECT"');
    expect(html).toContain('data-exit-mixed="0"');
    expect(html).toContain("未知");
  });

  it("链路表不渲染归属列，主机规则进程有归属且在下钻左侧", () => {
    const chain = renderToStaticMarkup(
      <RankTable
        locale="zh"
        kind="chain"
        result={report()}
        loading={false}
        errorZh={null}
        selectedIdentity={null}
        onSelect={() => undefined}
      />
    );
    expect(chain).not.toContain("归属");
    expect(chain).toContain(">下钻</th>");
    for (const kind of ["host", "rule", "process"] as const) {
      const html = renderToStaticMarkup(
        <RankTable
          locale="zh"
          kind={kind}
          result={report()}
          loading={false}
          errorZh={null}
          selectedIdentity={null}
          onSelect={() => undefined}
        />
      );
      const head = html.slice(html.indexOf("<thead"), html.indexOf("</thead>"));
      expect(head.indexOf("归属")).toBeGreaterThan(-1);
      expect(head.indexOf("归属")).toBeLessThan(head.indexOf("下钻"));
    }
  });

  it("Hourly 无出口时归属未知，表格仍显示", () => {
    const html = renderToStaticMarkup(
      <RankTable
        locale="zh"
        kind="host"
        result={report({
          dataTier: "HourlyDimension",
          rankings: [
            {
              identity: "old.example",
              label: "old.example",
              upload: 1,
              download: 90,
              connectionCount: 1,
              activeDurationSec: 1,
              primaryExit: null,
              exitMixed: false
            }
          ]
        })}
        loading={false}
        errorZh={null}
        selectedIdentity={null}
        onSelect={() => undefined}
      />
    );
    expect(html).toContain("<table");
    expect(html).not.toContain('data-exit="DIRECT"');
    expect(html).toContain("未知");
    expect(html).toContain('data-identity="old.example"');
  });
});

