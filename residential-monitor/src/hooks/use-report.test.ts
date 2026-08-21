import { describe, expect, it } from "vitest";
import type { ReportQuery, ReportResult } from "../dto";
import {
  beginReportRequest,
  buildReportQuery,
  finishReportRequest,
  granularityForTimeRange,
  granularityForTrendPreset,
  snapTimeRangeToMinute,
  type ReportViewState
} from "./use-report";

function emptyResult(over: Partial<ReportResult> = {}): ReportResult {
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
      comparison: { previousEqualWindow: true },
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

function state(over: Partial<ReportViewState> = {}): ReportViewState {
  return { result: null, loading: false, errorZh: null, seq: 0, ...over };
}

describe("趋势图档位与粒度映射", () => {
  it("30 分钟 / 1 小时 / 24 小时分别请求 minute1 / minute2 / minute10", () => {
    expect(granularityForTrendPreset("30m")).toBe("minute1");
    expect(granularityForTrendPreset("1h")).toBe("minute2");
    expect(granularityForTrendPreset("24h")).toBe("minute10");
  });

  it("顶栏其余预设不升粒度、不偷偷换成分钟档", () => {
    expect(granularityForTimeRange("5m")).toBe("minute1");
    expect(granularityForTimeRange("today")).toBe("minute10");
    expect(granularityForTimeRange("7d")).toBe("hour");
    expect(granularityForTimeRange("30d")).toBe("day");
  });
});

describe("timeRange 归整到分钟边界", () => {
  it("起止时刻向下归整到整分，查询秒值随整分变化", () => {
    const range = {
      preset: "24h" as const,
      startUtc: 1_200_030,
      endUtc: 1_260_590
    };
    const snapped = snapTimeRangeToMinute(range);
    expect(snapped.startUtc).toBe(1_200_000);
    expect(snapped.endUtc).toBe(1_260_000);
    expect(snapped.startUtc % 60_000).toBe(0);
    expect(snapped.endUtc % 60_000).toBe(0);
    const query = buildReportQuery({
      grouping: "host",
      timeRange: range,
      granularity: "minute10",
      topN: 10
    });
    expect(query.rangeStartUtc).toBe(1_200);
    expect(query.rangeEndUtc).toBe(1_260);
    const sameMinute: ReportQuery = buildReportQuery({
      grouping: "host",
      timeRange: { ...range, startUtc: 1_200_999, endUtc: 1_260_001 },
      granularity: "minute10",
      topN: 10
    });
    expect(sameMinute.rangeStartUtc).toBe(query.rangeStartUtc);
    expect(sameMinute.rangeEndUtc).toBe(query.rangeEndUtc);
  });
});

describe("报告请求竞态", () => {
  it("过期响应被丢弃", () => {
    const started = beginReportRequest(beginReportRequest(state()));
    expect(started.seq).toBe(2);
    const stale = emptyResult({ reportSnapshotToken: "old" });
    const dropped = finishReportRequest(started, 1, { ok: true, result: stale });
    expect(dropped).toBe(started);
    expect(dropped.result).toBeNull();
    const fresh = emptyResult({ reportSnapshotToken: "new" });
    const applied = finishReportRequest(started, 2, { ok: true, result: fresh });
    expect(applied.result?.reportSnapshotToken).toBe("new");
    expect(applied.loading).toBe(false);
  });

  it("失败保留上次结果，并单独暴露 errorZh", () => {
    const previous = emptyResult({ reportSnapshotToken: "keep" });
    const loaded = finishReportRequest(beginReportRequest(state()), 1, { ok: true, result: previous });
    const retrying = beginReportRequest(loaded);
    const failed = finishReportRequest(retrying, 2, { ok: false, errorZh: "分钟粒度只在 raw 保留期内可用" });
    expect(failed.result?.reportSnapshotToken).toBe("keep");
    expect(failed.errorZh).toBe("分钟粒度只在 raw 保留期内可用");
    expect(failed.loading).toBe(false);
  });

  it("成功响应整份替换，不沿用上次 drilldownCapability", () => {
    const first = emptyResult({
      drilldownCapability: {
        sessions: true,
        currentPolicy: true,
        crossDimension: true,
        exactTopN: true,
        noteZh: ""
      }
    });
    const second = emptyResult({
      reportSnapshotToken: "later",
      drilldownCapability: {
        sessions: false,
        currentPolicy: false,
        crossDimension: false,
        exactTopN: false,
        noteZh: "超出 raw 期，跨维下钻不可用。"
      }
    });
    const loaded = finishReportRequest(beginReportRequest(state()), 1, { ok: true, result: first });
    const next = finishReportRequest(beginReportRequest(loaded), 2, { ok: true, result: second });
    expect(next.result?.drilldownCapability.crossDimension).toBe(false);
    expect(next.result?.drilldownCapability.noteZh).toContain("跨维下钻");
  });
});
