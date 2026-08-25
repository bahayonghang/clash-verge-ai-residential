import { describe, expect, it } from "vitest";
import type { ReportQuery, ReportResult } from "../../../dto";
import { RESIDENTIAL_ACCOUNTING_FILTER } from "../../../format/rank";
import {
  directionTraffic,
  matchesResidentialRankQuery,
  newestFirstSeries,
  residentialReportFilters,
  shouldShowResidentialRankLoading
} from "./aggregate-model";

function queryEcho(): ReportQuery {
  return {
    rangeStartUtc: 0,
    rangeEndUtc: 3_600,
    displayTimezone: "local",
    granularity: "minute10",
    filters: residentialReportFilters(),
    grouping: "host",
    targetPolicy: "historical",
    comparison: { previousEqualWindow: true },
    sort: { field: "download", descending: true },
    page: { limit: 200, after: null },
    topN: 20,
    includeSessions: false
  };
}

function report(query: ReportQuery): ReportResult {
  return {
    schemaVersion: 1,
    dataVersion: 1,
    reportSnapshotToken: "token",
    queryEcho: query,
    totals: {
      upload: 0,
      download: 0,
      connectionCount: 0,
      activeDurationSec: 0,
      previousUpload: null,
      previousDownload: null
    },
    series: [],
    rankings: [],
    coverage: { status: "empty", coveredSec: 0, gapSec: 0, slices: [] },
    attributionQuality: {
      knownUpload: 0,
      knownDownload: 0,
      missingUpload: 0,
      missingDownload: 0,
      knownConnections: 0,
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
    policyMetadata: { targetPolicy: "historical", policyVersion: 1, noteZh: "" },
    dataTier: "raw",
    namedSql: [],
    unit: "bytes",
    generatedUtc: 0
  };
}

describe("家宽聚合查询模型", () => {
  it("使用 category 哨兵筛选家宽，但按 host 分组", () => {
    const filters = residentialReportFilters();
    expect(filters.category).toBe(RESIDENTIAL_ACCOUNTING_FILTER);
    expect(filters.host).toBeNull();
    expect(matchesResidentialRankQuery(report(queryEcho()), "download", 20)).toBe(true);
  });

  it("方向或 Top N 不匹配时拒绝陈旧排名", () => {
    expect(matchesResidentialRankQuery(report(queryEcho()), "upload", 20)).toBe(false);
    expect(matchesResidentialRankQuery(report(queryEcho()), "download", 10)).toBe(false);

    const wrongGrouping = report(queryEcho());
    wrongGrouping.queryEcho.grouping = "category";
    expect(matchesResidentialRankQuery(wrongGrouping, "download", 20)).toBe(false);

    const wrongFilter = report(queryEcho());
    wrongFilter.queryEcho.filters.category = null;
    expect(matchesResidentialRankQuery(wrongFilter, "download", 20)).toBe(false);

    const wrongDirection = report(queryEcho());
    wrongDirection.queryEcho.sort.descending = false;
    expect(matchesResidentialRankQuery(wrongDirection, "download", 20)).toBe(false);
  });

  it("陈旧结果只在等待新响应时显示加载，失败后显示错误空态", () => {
    expect(shouldShowResidentialRankLoading(true, null, true, false)).toBe(true);
    expect(shouldShowResidentialRankLoading(false, null, true, false)).toBe(true);
    expect(shouldShowResidentialRankLoading(false, "查询失败", true, false)).toBe(false);
    expect(shouldShowResidentialRankLoading(false, null, false, false)).toBe(false);
    expect(shouldShowResidentialRankLoading(false, null, true, true)).toBe(false);
  });

  it("条形值跟随当前方向", () => {
    const row = { upload: 101, download: 202 };
    expect(directionTraffic(row, "upload")).toBe(101);
    expect(directionTraffic(row, "download")).toBe(202);
  });

  it("表格新到旧排序不修改图表源数组", () => {
    const series: ReportResult["series"] = [
      { bucketUtc: 10, upload: 1, download: 2, connectionCount: 1, activeDurationSec: 60 },
      { bucketUtc: 30, upload: 3, download: 4, connectionCount: 1, activeDurationSec: 60 },
      { bucketUtc: 20, upload: 5, download: 6, connectionCount: 1, activeDurationSec: 60 }
    ];
    const rows = newestFirstSeries(series);
    expect(rows.map((point) => point.bucketUtc)).toEqual([30, 20, 10]);
    expect(series.map((point) => point.bucketUtc)).toEqual([10, 30, 20]);
    expect(newestFirstSeries([])).toEqual([]);
    expect(newestFirstSeries([series[0]])).toEqual([series[0]]);
  });
});
