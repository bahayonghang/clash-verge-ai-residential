import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { liveHotspotStatus, type LiveHotspotStatusInput } from "../../../format/live-hotspot";
import type { LiveConnectionPage } from "../../../ipc/live-session";
import { HOTSPOT_STATUSES, HotspotCards } from "./hotspot-cards";

const hotspot = {
  identity: "0:download",
  label: "downloads.example",
  host: "downloads.example",
  process: "browser.exe",
  destination: "203.0.113.10:443",
  value: 4096
};

const page: LiveConnectionPage = {
  rows: [],
  nextCursor: null,
  matchedCount: 2,
  sampleUtc: 1_723_456_789,
  summary: { topDownload: hotspot, topUpload: hotspot }
};

function input(overrides: Partial<LiveHotspotStatusInput> = {}): LiveHotspotStatusInput {
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

function html(statusInput: LiveHotspotStatusInput, data: LiveConnectionPage | null = page): string {
  return renderToStaticMarkup(<HotspotCards locale="zh" page={data} statusInput={statusInput} />);
}

describe("HotspotCards", () => {
  it("七种状态均有 live.hotspot.status 文案；暂停与缺口不显示数值或 0", () => {
    const ready = html(input());
    expect(liveHotspotStatus(input())).toBe("ready");
    expect(ready).toContain("当前快照");
    expect(ready).toContain("4.0 KiB");
    expect(ready).toContain("downloads.example");

    const noMatch = html(input({ page: { ...page, matchedCount: 0, summary: { topDownload: null, topUpload: null } } }), {
      ...page,
      matchedCount: 0,
      summary: { topDownload: null, topUpload: null }
    });
    expect(noMatch).toContain("没有匹配连接");
    expect(noMatch).not.toContain("4.0 KiB");

    const paused = html(input({ collectorRunning: false }));
    expect(paused).toContain("采集已暂停");
    expect(paused).not.toContain("4.0 KiB");
    expect(paused).not.toMatch(/>0</);
    expect(paused).not.toContain("0 B");

    const gap = html(input({ needResync: true }));
    expect(gap).toContain("订阅存在缺口");
    expect(gap).not.toContain("4.0 KiB");
    expect(gap).not.toContain("0 B");

    expect(html(input({ address: "" }))).toContain("尚未配置控制器");
    expect(html(input({ session: "disconnected" }))).toContain("控制器未连接");
    expect(html(input({ collectorRunning: null }))).toContain("热点能力或采样未知");
    expect(HOTSPOT_STATUSES).toHaveLength(7);
  });
});
