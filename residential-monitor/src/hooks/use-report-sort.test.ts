import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ReportQuery, ReportResult } from "../dto";
import { DEFAULT_RANK_SORT, nextRankSort } from "../rank-sort";
import { buildReportQuery, runReport } from "./use-report";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn()
}));

function ranking(identity: string, upload: number, download: number): ReportResult["rankings"][number] {
  return {
    identity,
    label: identity,
    upload,
    download,
    connectionCount: 1,
    activeDurationSec: 1,
    primaryExit: "DIRECT",
    exitMixed: false
  };
}

function payload(query: ReportQuery, row: ReportResult["rankings"][number]): ReportResult {
  return {
    schemaVersion: 1,
    dataVersion: 1,
    reportSnapshotToken: `tok-${query.sort.field}`,
    queryEcho: query,
    totals: {
      upload: 100,
      download: 100,
      connectionCount: 2,
      activeDurationSec: 1,
      previousUpload: null,
      previousDownload: null
    },
    series: [],
    rankings: [row],
    coverage: { status: "ok", coveredSec: 60, gapSec: 0, slices: [] },
    attributionQuality: {
      knownUpload: 100,
      knownDownload: 100,
      missingUpload: 0,
      missingDownload: 0,
      knownConnections: 2,
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
    generatedUtc: 1
  };
}

describe("表头 sort 走 run_report 白名单字段", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd !== "run_report") {
        throw new Error(String(cmd));
      }
      const query = (args as { query: ReportQuery }).query;
      const champion =
        query.sort.field === "upload"
          ? ranking("up.host", 90, 10)
          : ranking("down.host", 10, 90);
      return payload(query, champion);
    });
  });

  it("同一窗口 top_n=1：download desc 与 upload desc 可返回不同 identity", async () => {
    const timeRange = { preset: "1h" as const, startUtc: 1_000_000, endUtc: 1_060_000 };
    const downloadQuery = buildReportQuery({
      grouping: "host",
      timeRange,
      granularity: "minute2",
      topN: 1,
      sort: DEFAULT_RANK_SORT
    });
    const uploadQuery = buildReportQuery({
      grouping: "host",
      timeRange,
      granularity: "minute2",
      topN: 1,
      sort: nextRankSort("upload", DEFAULT_RANK_SORT)
    });
    expect(downloadQuery.sort).toEqual({ field: "download", descending: true });
    expect(uploadQuery.sort).toEqual({ field: "upload", descending: true });
    expect(downloadQuery.topN).toBe(1);
    expect(uploadQuery.topN).toBe(1);

    const down = await runReport(downloadQuery);
    const up = await runReport(uploadQuery);
    expect(down.rankings[0]?.identity).toBe("down.host");
    expect(up.rankings[0]?.identity).toBe("up.host");
    expect(down.rankings[0]?.identity).not.toBe(up.rankings[0]?.identity);
    expect(vi.mocked(invoke).mock.calls.map((call) => (call[1] as { query: ReportQuery }).query.sort)).toEqual([
      { field: "download", descending: true },
      { field: "upload", descending: true }
    ]);
  });
});
