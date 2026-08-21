import { describe, expect, it } from "vitest";
import { decodeOverview } from "./decoder";
import {
  emptyMonitorState,
  markCloseAccepted,
  reduceMonitor,
  visibleRows
} from "./reducer";
import type { LiveConnectionView, LiveOverview, MonitorStreamMessage } from "../dto";

const snapshot = decodeOverview({
  schemaVersion: 1,
  observationPhase: "current",
  meterUpload: 1,
  meterDownload: 1,
  attributedUpload: 1,
  attributedDownload: 1,
  categoryUpload: {},
  categoryDownload: {},
  otherUpload: 0,
  otherDownload: 0,
  gapUpload: 0,
  gapDownload: 0,
  overUpload: 0,
  overDownload: 0,
  activeCount: 0,
  lastSampleUtc: 1,
  coverageKind: null,
  coverageReason: null,
  health: { session: "connected", storageOk: true, storageReason: null }
});

function bootstrap(id: number, baseSeq: number): MonitorStreamMessage {
  return {
    kind: "bootstrap",
    schemaVersion: 1,
    subscriptionId: id,
    snapshot: snapshot as LiveOverview,
    baseSeq,
    backendTime: 1
  };
}

function row(id: string): LiveConnectionView {
  return {
    identity: id,
    connectionId: id,
    epoch: 0,
    upload: 1,
    download: 1,
    rateUpload: null,
    rateDownload: null,
    durationMs: null,
    primary: null,
    tags: [],
    host: `${id}.test`,
    sourceIp: null,
    destinationIp: null,
    processName: null,
    processPath: null,
    network: "tcp",
    rule: null,
    rulePayload: null,
    chains: []
  };
}

describe("reduceMonitor", () => {
  it("丢弃旧订阅迟到消息", () => {
    let state = reduceMonitor(emptyMonitorState(), bootstrap(2, 4));
    state = reduceMonitor(state, {
      kind: "connectionDelta",
      schemaVersion: 1,
      subscriptionId: 1,
      seq: 5,
      snapshot,
      upserts: [row("late")],
      removes: [],
      backendTime: 2
    });
    expect(state.connections.size).toBe(0);
  });

  it("序号缺口时冻结并要求 resync", () => {
    let state = reduceMonitor(emptyMonitorState(), bootstrap(1, 1));
    state = reduceMonitor(state, {
      kind: "connectionDelta",
      schemaVersion: 1,
      subscriptionId: 1,
      seq: 4,
      snapshot,
      upserts: [row("x")],
      removes: [],
      backendTime: 2
    });
    expect(state.frozen).toBe(true);
    expect(state.needResync).toBe(true);
    expect(state.connections.size).toBe(0);
  });

  it("重复序号不回放", () => {
    let state = reduceMonitor(emptyMonitorState(), bootstrap(1, 1));
    const delta: MonitorStreamMessage = {
      kind: "connectionDelta",
      schemaVersion: 1,
      subscriptionId: 1,
      seq: 2,
      snapshot,
      upserts: [row("a")],
      removes: [],
      backendTime: 2
    };
    state = reduceMonitor(state, delta);
    state = reduceMonitor(state, delta);
    expect(state.connections.size).toBe(1);
    expect(state.lastSeq).toBe(2);
  });

  it("关闭 204 后要等 remove 才显示已关闭", () => {
    let state = reduceMonitor(emptyMonitorState(), bootstrap(1, 1));
    state = reduceMonitor(state, {
      kind: "connectionDelta",
      schemaVersion: 1,
      subscriptionId: 1,
      seq: 2,
      snapshot,
      upserts: [row("0:a")],
      removes: [],
      backendTime: 2
    });
    state = markCloseAccepted(state, "0:a");
    expect(state.closeMarks.get("0:a")).toBe("accepted");
    state = reduceMonitor(state, {
      kind: "connectionDelta",
      schemaVersion: 1,
      subscriptionId: 1,
      seq: 3,
      snapshot,
      upserts: [],
      removes: ["0:a"],
      backendTime: 3
    });
    expect(state.closeMarks.get("0:a")).toBe("closed");
  });

  it("虚拟化只取窗口", () => {
    const map = new Map<string, LiveConnectionView>();
    for (let index = 0; index < 40; index += 1) {
      map.set(`0:${index}`, row(`0:${index}`));
    }
    expect(visibleRows(map, 10, 5, 2).length).toBe(9);
  });
});
