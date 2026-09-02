import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { ReportResult } from "../../../dto";
import { filtersForDrilldown, UNKNOWN_RANK_IDENTITY } from "../../../format/rank";
import { DrilldownPanel } from "./drilldown-panel";

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
      grouping: "host",
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
    rankings: [
      {
        identity: "sql-rule-key",
        label: "AI",
        upload: 1,
        download: 2,
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

describe("下钻面板由 drilldownCapability 驱动", () => {
  it("crossDimension 为 false 时隐藏下钻入口并显示 noteZh", () => {
    const html = renderToStaticMarkup(
      <DrilldownPanel
        locale="zh"
        kind="host"
        selected={{ identity: "a.example", label: "a.example" }}
        targetKind="rule"
        onTargetKindChange={() => undefined}
        onClear={() => undefined}
        parentResult={report({
          drilldownCapability: {
            sessions: false,
            currentPolicy: false,
            crossDimension: false,
            exactTopN: true,
            noteZh: "超出 raw 期，跨维下钻不可用。"
          }
        })}
        drillResult={null}
        drillLoading={false}
        drillErrorZh={null}
      />
    );
    expect(html).toContain("超出 raw 期，跨维下钻不可用。");
    expect(html).not.toContain("data-drill-target");
    expect(html).not.toContain("已选");
  });

  it("下钻 filters 使用排名 identity，不拼 rule(payload)", () => {
    const filters = filtersForDrilldown("rule", "sql-rule-key");
    expect(filters.rule).toBe("sql-rule-key");
    expect(filters.rule).not.toContain("(");
    const unknownFilters = filtersForDrilldown("host", UNKNOWN_RANK_IDENTITY);
    expect(unknownFilters.host).toBe(UNKNOWN_RANK_IDENTITY);
    const processUnknown = filtersForDrilldown("process", UNKNOWN_RANK_IDENTITY);
    expect(processUnknown.process).toBe(UNKNOWN_RANK_IDENTITY);
    const ruleUnknown = filtersForDrilldown("rule", UNKNOWN_RANK_IDENTITY);
    expect(ruleUnknown.rule).toBeNull();
  });

  it("crossDimension 为 false 且 noteZh 为空时仍显示能力说明", () => {
    const html = renderToStaticMarkup(
      <DrilldownPanel
        locale="zh"
        kind="host"
        selected={{ identity: "a.example", label: "a.example" }}
        targetKind="rule"
        onTargetKindChange={() => undefined}
        onClear={() => undefined}
        parentResult={report({
          drilldownCapability: {
            sessions: false,
            currentPolicy: false,
            crossDimension: false,
            exactTopN: true,
            noteZh: ""
          }
        })}
        drillResult={null}
        drillLoading={false}
        drillErrorZh={null}
      />
    );
    expect(html).toContain("data-capability-note");
    expect(html).toContain("当前层不可下钻");
    expect(html).not.toContain("data-drill-target");
  });

  it("exactTopN 为 false 时显示能力说明，不渲染下钻图", () => {
    const html = renderToStaticMarkup(
      <DrilldownPanel
        locale="zh"
        kind="host"
        selected={{ identity: "a.example", label: "a.example" }}
        targetKind="rule"
        onTargetKindChange={() => undefined}
        onClear={() => undefined}
        parentResult={report()}
        drillResult={report({
          rankings: [],
          series: [],
          drilldownCapability: {
            sessions: false,
            currentPolicy: false,
            crossDimension: true,
            exactTopN: false,
            noteZh: "该维度尚未五维物化，精确 Top N 不可用。"
          }
        })}
        drillLoading={false}
        drillErrorZh={null}
      />
    );
    expect(html).toContain("data-capability-note");
    expect(html).toContain("该维度尚未五维物化，精确 Top N 不可用。");
    expect(html).not.toContain("recharts");
  });
});
