import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { ReportResult } from "../../../dto";
import { RankBarCard } from "./rank-bar-card";

function report(over: Partial<ReportResult> = {}): ReportResult {
  return {
    schemaVersion: 1,
    dataVersion: 1,
    reportSnapshotToken: "tok",
    queryEcho: {
      rangeStartUtc: 0,
      rangeEndUtc: 60,
      displayTimezone: "local",
      granularity: "hour",
      filters: { category: null, host: null, process: null, rule: null, chain: null, network: null },
      grouping: "process",
      targetPolicy: "historical",
      comparison: null,
      sort: { field: "download", descending: true },
      page: { limit: 200, after: null },
      topN: 10,
      includeSessions: false
    },
    totals: {
      upload: 1,
      download: 2,
      connectionCount: 1,
      activeDurationSec: 1,
      previousUpload: null,
      previousDownload: null
    },
    series: [],
    rankings: [],
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

describe("排行条形图能力说明", () => {
  it("exactTopN 为 false 时显示能力说明，不渲染空表或条形图", () => {
    const html = renderToStaticMarkup(
      <RankBarCard
        locale="zh"
        title="进程"
        kind="process"
        result={report({
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
        topN={20}
        onTopNChange={() => undefined}
      />
    );
    expect(html).toContain("data-capability-note");
    expect(html).toContain("该维度尚未五维物化，精确 Top N 不可用。");
    expect(html).not.toContain("recharts");
    expect(html).not.toContain("无排名数据");
  });

  it("exactTopN 为 false 且 noteZh 为空时仍显示能力说明", () => {
    const html = renderToStaticMarkup(
      <RankBarCard
        locale="zh"
        title="进程"
        kind="process"
        result={report({
          drilldownCapability: {
            sessions: false,
            currentPolicy: false,
            crossDimension: false,
            exactTopN: false,
            noteZh: ""
          }
        })}
        loading={false}
        errorZh={null}
        topN={20}
        onTopNChange={() => undefined}
      />
    );
    expect(html).toContain("data-capability-note");
    expect(html).toContain("精确 Top N 不可用");
  });
});
