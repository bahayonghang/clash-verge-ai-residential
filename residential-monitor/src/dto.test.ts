import { describe, expect, it } from "vitest";
import {
  decodeAbout,
  decodeAlertCenter,
  decodeDeleteReport,
  decodeDiagnostics,
  decodeReportArchivePage,
  decodeReportResult,
  decodeShellStatus
} from "./dto";

describe("decodeShellStatus", () => {
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

  it("拒绝缺少 token 的报告结果", () => {
    expect(() => decodeReportResult({ schemaVersion: 1, totals: {}, coverage: {} })).toThrow(/缺失/);
  });

  it("接受带 token 的报告结果", () => {
    const decoded = decodeReportResult({
      schemaVersion: 1,
      reportSnapshotToken: "abc",
      totals: { upload: 1, download: 2 },
      coverage: { status: "empty" }
    });
    expect(decoded.reportSnapshotToken).toBe("abc");
  });

  it("拒绝缺少 checksum 的诊断", () => {
    expect(() => decodeDiagnostics({ schemaVersion: 1 })).toThrow(/无效/);
  });

  it("拒绝缺少 items 的告警中心", () => {
    expect(() => decodeAlertCenter({ schemaVersion: 1 })).toThrow(/无效/);
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
      productName: "家宽流量监控",
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
});
