import { describe, expect, it } from "vitest";
import { decodeReportResult, decodeShellStatus } from "./dto";

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
});
