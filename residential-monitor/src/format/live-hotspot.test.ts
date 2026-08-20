import { describe, expect, it } from "vitest";
import type { ConnectionHotspot, LiveConnectionPage } from "../ipc/live-session";
import {
  canShowHotspotSnapshotFacts,
  canShowHotspotValue,
  hotspotDisplayDetail,
  hotspotDisplayLabel,
  liveHotspotStatus
} from "./live-hotspot";

const hotspot: ConnectionHotspot = {
  identity: "0:example",
  label: "downloads.example",
  host: "downloads.example",
  process: "browser.exe",
  destination: "203.0.113.10:443",
  value: 4096
};

const page: LiveConnectionPage = {
  rows: [],
  nextCursor: null,
  matchedCount: 1,
  sampleUtc: 1_723_456_789,
  summary: { topDownload: hotspot, topUpload: null }
};

function input(overrides: Partial<Parameters<typeof liveHotspotStatus>[0]> = {}) {
  return {
    page,
    address: "127.0.0.1:9097",
    session: "connected",
    collectorRunning: true,
    coverageKind: null,
    coverageReason: null,
    needResync: false,
    frozen: false,
    ...overrides
  };
}

describe("live hotspot display", () => {
  it("uses only the decoded summary snapshot, including an explicit no-match result", () => {
    expect(liveHotspotStatus(input())).toBe("ready");
    expect(liveHotspotStatus(input({ page: { ...page, matchedCount: 0 } }))).toBe("noMatch");
  });

  it("hides a prior snapshot for paused, gap, disconnected, and unavailable states", () => {
    expect(liveHotspotStatus(input({ collectorRunning: false }))).toBe("paused");
    expect(liveHotspotStatus(input({ coverageKind: "closed", coverageReason: "pause_or_shutdown" }))).toBe("paused");
    expect(liveHotspotStatus(input({ needResync: true }))).toBe("gap");
    expect(liveHotspotStatus(input({ frozen: true }))).toBe("gap");
    expect(liveHotspotStatus(input({ session: "disconnected" }))).toBe("disconnected");
    expect(liveHotspotStatus(input({ collectorRunning: null }))).toBe("unknown");
    expect(liveHotspotStatus(input({ address: "" }))).toBe("unconfigured");
    expect(liveHotspotStatus(input({ coverageKind: "gap", coverageReason: "disconnect_or_sleep" }))).toBe("gap");
    expect(liveHotspotStatus(input({ coverageKind: "future", coverageReason: "new_reason" }))).toBe("unknown");
    expect(liveHotspotStatus(input({ page: null }))).toBe("unknown");
    expect(liveHotspotStatus(input({ page: { ...page, sampleUtc: null } }))).toBe("unknown");
  });

  it("hides hotspot values and snapshot facts except on a current ready or no-match page", () => {
    expect(canShowHotspotValue("ready")).toBe(true);
    expect(canShowHotspotSnapshotFacts("ready")).toBe(true);
    expect(canShowHotspotValue("noMatch")).toBe(false);
    expect(canShowHotspotSnapshotFacts("noMatch")).toBe(true);
    for (const status of ["paused", "gap", "unconfigured", "disconnected", "unknown"] as const) {
      expect(canShowHotspotValue(status)).toBe(false);
      expect(canShowHotspotSnapshotFacts(status)).toBe(false);
    }
  });

  it("uses safe label fields as fallbacks without looking at paginated rows", () => {
    expect(hotspotDisplayLabel(hotspot, "Unknown")).toBe("downloads.example");
    expect(hotspotDisplayLabel({ ...hotspot, label: "", host: null }, "Unknown")).toBe("browser.exe");
    expect(hotspotDisplayDetail(hotspot, "Unknown")).toBe("browser.exe");
    expect(hotspotDisplayDetail({ ...hotspot, host: null, process: null }, "Unknown")).toBe("203.0.113.10:443");
  });
});
