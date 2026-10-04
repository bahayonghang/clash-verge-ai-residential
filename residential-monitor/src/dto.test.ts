import { describe, expect, it } from "vitest";
import {
  decodeAbout,
  decodeAlertCenter,
  decodeAlertRule,
  decodeAlertSummary,
  decodeAutostartState,
  decodeDeleteReport,
  decodeDiagnostics,
  decodeLiveRow,
  decodeReportArchivePage,
  decodeReportResult,
  decodeResidentialShare,
  decodeShellStatus,
  EMPTY_METADATA_COVERAGE
} from "./dto";

function validReportPayload() {
  return {
    schemaVersion: 1,
    dataVersion: 1,
    reportSnapshotToken: "abc",
    queryEcho: {
      rangeStartUtc: 0,
      rangeEndUtc: 60,
      displayTimezone: "local",
      granularity: "minute1",
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
      activeDurationSec: 60,
      previousUpload: null,
      previousDownload: null
    },
    series: [],
    rankings: [
      {
        identity: "__unknown__",
        label: "",
        upload: 1,
        download: 2,
        connectionCount: 1,
        activeDurationSec: 60
      }
    ],
    coverage: { status: "covered", coveredSec: 60, gapSec: 0, slices: [] },
    attributionQuality: {
      knownUpload: 0,
      knownDownload: 0,
      missingUpload: 1,
      missingDownload: 2,
      knownConnections: 0,
      missingConnections: 1,
      status: "unavailable"
    },
    drilldownCapability: {
      sessions: true,
      currentPolicy: true,
      crossDimension: true,
      exactTopN: true,
      noteZh: ""
    },
    policyMetadata: { targetPolicy: "historical", policyVersion: null, noteZh: "" },
    dataTier: "raw",
    namedSql: ["rank_raw_attr"],
    unit: "byte",
    generatedUtc: 60
  };
}

describe("decodeShellStatus", () => {
  it("严格解码 Windows 登录自启动状态", () => {
    expect(decodeAutostartState({ enabled: true })).toEqual({ enabled: true });
    expect(decodeAutostartState({ enabled: false, extra: "ignored" })).toEqual({ enabled: false });
    expect(() => decodeAutostartState({})).toThrow(/AutostartStateDto/);
    expect(() => decodeAutostartState({ enabled: "true" })).toThrow(/AutostartStateDto/);
    expect(() => decodeAutostartState({ enabled: null })).toThrow(/AutostartStateDto/);
  });

  it("接受完整 DTO", () => {
    const decoded = decodeShellStatus({
      schemaVersion: 1,
      kind: "shellStatus",
      identifier: "io.github.bahayonghang.residential-monitor",
      phase: "c0-skeleton",
      messageZh: "骨架"
    });
    expect(decoded.identifier).toContain("residential-monitor");
  });

  it("拒绝缺少 kind 的载荷", () => {
    expect(() =>
      decodeShellStatus({
        schemaVersion: 1,
        identifier: "x",
        phase: "c0-skeleton",
        messageZh: "骨架"
      })
    ).toThrow(/kind/);
  });

  it("接受四个 Option 的家宽份额", () => {
    const decoded = decodeResidentialShare({
      schemaVersion: 1,
      residentialUpload: null,
      residentialDownload: null,
      attributedUpload: null,
      attributedDownload: null,
      coverageStatus: "uncovered",
      namedSql: ["coverage_raw"],
      generatedUtc: 1,
      targetCount: 0,
      policyVersion: null
    });
    expect(decoded.residentialUpload).toBeNull();
    expect(decoded.coverageStatus).toBe("uncovered");
    expect(decoded.targetCount).toBe(0);
  });

  it("拒绝缺少 schemaVersion 的家宽份额", () => {
    expect(() =>
      decodeResidentialShare({
        residentialUpload: 0,
        coverageStatus: "covered",
        namedSql: [],
        generatedUtc: 1,
        targetCount: 0
      })
    ).toThrow(/ResidentialShare/);
  });

  it("拒绝缺少 token 的报告结果", () => {
    expect(() => decodeReportResult({ schemaVersion: 1, totals: {}, coverage: {} })).toThrow(/缺失/);
  });

  it("接受带 token 的报告结果", () => {
    const decoded = decodeReportResult(validReportPayload());
    expect(decoded.reportSnapshotToken).toBe("abc");
    expect(decoded.attributionQuality.status).toBe("unavailable");
    expect(decoded.rankings[0]?.identity).toBe("__unknown__");
    expect(decoded.rankings[0]?.primaryExit).toBeNull();
    expect(decoded.rankings[0]?.exitMixed).toBe(false);
  });

  it("旧档案缺出口字段按未知解码，非法类型拒绝", () => {
    const decoded = decodeReportResult(validReportPayload());
    expect(decoded.rankings[0]?.primaryExit).toBeNull();
    expect(decoded.rankings[0]?.exitMixed).toBe(false);
    const withExit = validReportPayload();
    Object.assign(withExit.rankings[0]!, { primaryExit: "DIRECT", exitMixed: true });
    const ok = decodeReportResult(withExit);
    expect(ok.rankings[0]?.primaryExit).toBe("DIRECT");
    expect(ok.rankings[0]?.exitMixed).toBe(true);
    const badExit = validReportPayload();
    Object.assign(badExit.rankings[0]!, { primaryExit: 0 });
    expect(() => decodeReportResult(badExit)).toThrow(/primaryExit/);
    const badMixed = validReportPayload();
    Object.assign(badMixed.rankings[0]!, { exitMixed: "true" });
    expect(() => decodeReportResult(badMixed)).toThrow(/exitMixed/);
  });

  it("拒绝缺失 attributionQuality 或非法负数排名", () => {
    const missing = validReportPayload();
    delete (missing as Partial<typeof missing>).attributionQuality;
    expect(() => decodeReportResult(missing)).toThrow(/attributionQuality|缺失/);
    const negative = validReportPayload();
    negative.rankings[0]!.download = -1;
    expect(() => decodeReportResult(negative)).toThrow(/rankings/);
    const nonConserving = validReportPayload();
    nonConserving.attributionQuality.missingDownload = 1;
    expect(() => decodeReportResult(nonConserving)).toThrow(/不守恒/);
    const inconsistentStatus = validReportPayload();
    inconsistentStatus.attributionQuality.status = "complete";
    expect(() => decodeReportResult(inconsistentStatus)).toThrow(/status 与计数不一致/);
  });

  it("拒绝缺少 checksum 的诊断", () => {
    expect(() => decodeDiagnostics({ schemaVersion: 1 })).toThrow(/缺失|无效/);
  });

  it("拒绝缺少 items 的告警中心", () => {
    expect(() => decodeAlertCenter({ schemaVersion: 1 })).toThrow(/缺失|无效/);
  });

  it("拒绝把未签名 about 标成 signed", () => {
    expect(() =>
      decodeAbout({
        schemaVersion: 1,
        releasesUrl: "https://github.com/bahayonghang/clash-verge-ai-residential/releases",
        signed: true
      })
    ).toThrow(/signed/);
  });

  it("接受未签名 about", () => {
    const decoded = decodeAbout({
      schemaVersion: 1,
      productName: "ResiWatch",
      binaryName: "residential-monitor",
      identifier: "io.github.bahayonghang.residential-monitor",
      aumid: "io.github.bahayonghang.residential-monitor",
      version: "0.1.0",
      releasesUrl: "https://github.com/bahayonghang/clash-verge-ai-residential/releases",
      signed: false,
      updaterPlugin: false,
      windowsService: false,
      signatureNoteZh: "本候选未做 Authenticode 签名。"
    });
    expect(decoded.signed).toBe(false);
    expect(decoded.updaterPlugin).toBe(false);
    expect(decoded.windowsService).toBe(false);
    expect(decoded.releasesUrl).toContain("/releases");
  });

  it("部分删除不得被解码成全部成功以外的字段缺失", () => {
    const decoded = decodeDeleteReport({
      schemaVersion: 1,
      allDeclaredOk: false,
      items: [],
      summaryZh: "部分失败"
    });
    expect(decoded.allDeclaredOk).toBe(false);
  });

  it("接受告警中心分页", () => {
    const decoded = decodeAlertCenter({ schemaVersion: 1, items: [], nextCursor: null });
    expect(decoded.items).toEqual([]);
  });

  it("拒绝缺少 schemaVersion 的档案页", () => {
    expect(() => decodeReportArchivePage({ items: [] })).toThrow(/无效/);
  });

  it("拒绝缺少 items 的档案页", () => {
    expect(() => decodeReportArchivePage({ schemaVersion: 1 })).toThrow(/无效/);
  });

  it("拒绝缺少 next 的档案页", () => {
    expect(() => decodeReportArchivePage({ schemaVersion: 1, items: [] })).toThrow(/无效/);
  });

  it("拒绝缺少 archiveId 的档案项", () => {
    expect(() =>
      decodeReportArchivePage({
        schemaVersion: 1,
        items: [{ kind: "hour", status: "ok", rangeStartUtc: 1, rangeEndUtc: 2 }],
        next: null
      })
    ).toThrow(/无效/);
  });

  it("接受档案分页", () => {
    const decoded = decodeReportArchivePage({ schemaVersion: 1, items: [], next: null });
    expect(decoded.items).toEqual([]);
    expect(decoded.next).toBeNull();
  });

  it("接受含档案项的分页", () => {
    const decoded = decodeReportArchivePage({
      schemaVersion: 1,
      items: [
        {
          archiveId: "a1",
          kind: "day",
          status: "ok",
          rangeStartUtc: 1,
          rangeEndUtc: 2
        }
      ],
      next: "1|a1"
    });
    expect(decoded.items[0]?.archiveId).toBe("a1");
    expect(decoded.next).toBe("1|a1");
  });

  it("毒化告警与诊断 payload 被拒绝，不进视图对象", () => {
    expect(() => decodeAlertCenter({ items: [], nextCursor: null })).toThrow(/AlertCenterPage|无效/);
    expect(() => decodeAlertSummary({ activeCount: 1, notEvaluableCount: 0, outboxBacklog: 0, lastEventUtc: null })).toThrow(
      /AlertSummary/
    );
    expect(() =>
      decodeAlertRule({
        ruleId: "rate-home",
        version: 1,
        enabled: true,
        kind: "mystery",
        selectorKind: "primary-category",
        selectorValue: "家宽",
        direction: "download",
        thresholdValue: 1,
        recoveryThreshold: null,
        period: null,
        timezone: "Asia/Shanghai",
        cooldownSec: 1,
        quietStartMin: null,
        quietEndMin: null,
        createdUtc: 0,
        updatedUtc: 0
      })
    ).toThrow(/kind/);
    expect(() =>
      decodeAlertCenter({
        schemaVersion: 1,
        items: [
          {
            instanceId: "i1",
            ruleId: "r1",
            ruleVersion: 1,
            selectorIdentity: "家宽",
            status: "mystery",
            startedUtc: null,
            resolvedUtc: null,
            lastEvalUtc: 1,
            lastObserved: null,
            evidence: {}
          }
        ],
        nextCursor: null
      })
    ).toThrow(/status|无效/);
    expect(() => decodeDiagnostics({ schemaVersion: 1, c4Checksum: "x" })).toThrow(/缺失|无效/);
  });

  it("连接行剥离 processPath，缺字段拒绝", () => {
    const row = {
      identity: "0:a",
      connectionId: "a",
      epoch: 0,
      upload: 1,
      download: 1,
      rateUpload: null,
      rateDownload: null,
      durationMs: null,
      primary: null,
      tags: [],
      host: null,
      sourceIp: null,
      destinationIp: null,
      processName: null,
      network: "tcp",
      inbound: null,
      sourcePort: null,
      destinationPort: null,
      start: null,
      rule: null,
      rulePayload: null,
      chains: [],
      processPath: "C:\\secret\\a.exe"
    };
    const decoded = decodeLiveRow(row);
    expect(decoded.identity).toBe("0:a");
    expect(decoded).not.toHaveProperty("processPath");
    expect(JSON.stringify(decoded)).not.toContain("processPath");
    expect(JSON.stringify(decoded)).not.toContain("C:\\\\secret");
    const missing = { ...row } as Record<string, unknown>;
    delete missing.identity;
    expect(() => decodeLiveRow(missing)).toThrow(/identity/);
  });

  it("接受完整诊断快照", () => {
    const decoded = decodeDiagnostics({
      schemaVersion: 1,
      appVersion: "0.3.0",
      sqliteUserVersion: 4,
      supportedSchema: 4,
      c4Checksum: "abc",
      journalMode: "wal",
      synchronous: "NORMAL",
      controllerTransportStatus: "connected",
      coverageSummary: "ok",
      writerWatermark: 1,
      writerReceipts: 1,
      lastFrameUtc: null,
      reconnectHintZh: "",
      databaseOk: true,
      walCheckpointOk: true,
      backupRetentionNoteZh: "note",
      alertActive: 0,
      outboxBacklog: 0,
      recentRedactedErrorClasses: [],
      metadataCoverage: EMPTY_METADATA_COVERAGE
    });
    expect(decoded.c4Checksum).toBe("abc");
    expect(decoded.metadataCoverage.processPathOnly).toBe(0);
  });

  it("接受手动档案 kind", () => {
    const decoded = decodeReportArchivePage({
      schemaVersion: 1,
      items: [
        {
          archiveId: "m1",
          kind: "manual",
          status: "ok",
          rangeStartUtc: 1,
          rangeEndUtc: 2
        }
      ],
      next: null
    });
    expect(decoded.items[0]?.kind).toBe("manual");
  });
});
