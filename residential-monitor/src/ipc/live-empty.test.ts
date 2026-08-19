import { describe, expect, it } from "vitest";
import { liveEmptyCopy, liveEmptyKind, type LiveEmptyInput } from "./live-empty";

function base(overrides: Partial<LiveEmptyInput> = {}): LiveEmptyInput {
  return {
    address: "127.0.0.1:9097",
    session: "connected",
    collectorRunning: true,
    coverageKind: null,
    coverageReason: null,
    rowCount: 0,
    needResync: false,
    frozen: false,
    errorZh: null,
    ...overrides
  };
}

describe("liveEmptyKind", () => {
  it("未配置控制器", () => {
    expect(liveEmptyKind(base({ address: "", session: "no_data" }))).toBe("unconfigured");
    expect(liveEmptyCopy("unconfigured")).toContain("设置页");
  });

  it("未连接与鉴权失败", () => {
    expect(liveEmptyKind(base({ session: "disconnected" }))).toBe("disconnected");
    expect(liveEmptyKind(base({ session: "tcp_unauthorized" }))).toBe("disconnected");
    expect(liveEmptyKind(base({ session: "endpoint_missing" }))).toBe("disconnected");
  });

  it("采集暂停不依赖 health.session", () => {
    expect(liveEmptyKind(base({ collectorRunning: false, session: "connected" }))).toBe("paused");
    expect(
      liveEmptyKind(
        base({
          collectorRunning: null,
          coverageKind: "closed",
          coverageReason: "pause_or_shutdown"
        })
      )
    ).toBe("paused");
    expect(liveEmptyCopy("paused")).toContain("托盘");
  });

  it("已连接且查询为空", () => {
    expect(liveEmptyKind(base({ session: "connected", rowCount: 0 }))).toBe("connectedEmpty");
    expect(liveEmptyCopy("connectedEmpty")).toBe("当前没有活跃连接");
  });

  it("订阅缺口与协议冻结", () => {
    expect(liveEmptyKind(base({ needResync: true, rowCount: 2 }))).toBe("needResync");
    expect(liveEmptyKind(base({ frozen: true, errorZh: "实时协议版本不兼容，请升级或重载窗口。" }))).toBe(
      "needResync"
    );
    expect(liveEmptyCopy("needResync")).toMatch(/重新订阅|重载/);
  });

  it("有行时不挡住表格", () => {
    expect(liveEmptyKind(base({ rowCount: 3, collectorRunning: false }))).toBe("hasRows");
  });
});
