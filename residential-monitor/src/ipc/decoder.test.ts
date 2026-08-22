import { describe, expect, it } from "vitest";
import { EMPTY_METADATA_COVERAGE } from "../dto";
import { decodeMonitorMessage, decodeOverview } from "./decoder";

const overview = {
  schemaVersion: 1,
  observationPhase: "current",
  meterUpload: 10,
  meterDownload: 20,
  attributedUpload: 8,
  attributedDownload: 16,
  categoryUpload: { 家宽: 8 },
  categoryDownload: { 家宽: 16 },
  otherUpload: 0,
  otherDownload: 0,
  gapUpload: 2,
  gapDownload: 4,
  overUpload: 0,
  overDownload: 0,
  activeCount: 1,
  lastSampleUtc: 1,
  coverageKind: null,
  coverageReason: null,
  health: { session: "connected", storageOk: true, storageReason: null },
  metadataCoverage: EMPTY_METADATA_COVERAGE
};

describe("decodeMonitorMessage", () => {
  it("接受 bootstrap", () => {
    const decoded = decodeMonitorMessage({
      kind: "bootstrap",
      schemaVersion: 1,
      subscriptionId: 1,
      snapshot: overview,
      baseSeq: 3,
      backendTime: 1
    });
    expect(decoded.kind).toBe("bootstrap");
  });

  it("拒绝未知 kind", () => {
    expect(() =>
      decodeMonitorMessage({
        kind: "mystery",
        schemaVersion: 1,
        subscriptionId: 1
      })
    ).toThrow(/未知/);
  });

  it("缺口字段保持未知而不是 0", () => {
    const decoded = decodeOverview({
      ...overview,
      meterUpload: null,
      attributedUpload: null,
      gapUpload: null
    });
    expect(decoded.meterUpload).toBeNull();
    expect(decoded.gapUpload).toBeNull();
  });

  it("Overview 区分显式 null 与字段缺失", () => {
    expect(decodeOverview({ ...overview, meterUpload: null }).meterUpload).toBeNull();
    const missing = { ...overview } as Record<string, unknown>;
    delete missing.meterUpload;
    expect(() => decodeOverview(missing)).toThrow(/meterUpload/);
    expect(() => decodeOverview({ ...overview, observationPhase: "mystery" })).toThrow(
      /observationPhase/
    );
  });

  it("connectionDelta 必须携带同 tick 的 snapshot", () => {
    expect(() =>
      decodeMonitorMessage({
        kind: "connectionDelta",
        schemaVersion: 1,
        subscriptionId: 1,
        seq: 2,
        upserts: [],
        removes: [],
        backendTime: 2
      })
    ).toThrow(/概览|对象/);
    const decoded = decodeMonitorMessage({
      kind: "connectionDelta",
      schemaVersion: 1,
      subscriptionId: 1,
      seq: 2,
      snapshot: { ...overview, observationPhase: "baselinePending" },
      upserts: [],
      removes: [],
      backendTime: 2
    });
    expect(decoded.kind === "connectionDelta" && decoded.snapshot.observationPhase).toBe(
      "baselinePending"
    );
  });

  it("拒绝静默丢弃无效 upsert 或补造 health 字段", () => {
    expect(() =>
      decodeMonitorMessage({
        kind: "connectionDelta",
        schemaVersion: 1,
        subscriptionId: 1,
        seq: 2,
        snapshot: overview,
        upserts: [null],
        removes: [],
        backendTime: 2
      })
    ).toThrow(/upsert/);
    expect(() =>
      decodeMonitorMessage({
        kind: "healthChanged",
        schemaVersion: 1,
        subscriptionId: 1,
        seq: 2,
        health: { session: "connected", storageReason: null },
        backendTime: 2
      })
    ).toThrow(/storageOk/);
  });
});
