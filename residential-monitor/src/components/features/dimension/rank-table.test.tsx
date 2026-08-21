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
  it("identity 为 __unknown__ 时按未知渲染且无下钻入口", () => {
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
    expect(html).toContain("未知");
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
});
