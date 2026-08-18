import { describe, expect, it } from "vitest";
import { decodeMonitorMessage, decodeOverview } from "./decoder";

const overview = {
  schemaVersion: 1,
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
  health: { session: "connected", storageOk: true, storageReason: null }
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
});
