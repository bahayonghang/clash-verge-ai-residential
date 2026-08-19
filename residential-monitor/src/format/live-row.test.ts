import { describe, expect, it } from "vitest";
import { displayLiveRow, formatRelative, formatRule, formatType, joinHostPort } from "./live-row";
import type { LiveConnectionView } from "../dto";

function row(overrides: Partial<LiveConnectionView> = {}): LiveConnectionView {
  return {
    identity: "0:a",
    connectionId: "a",
    epoch: 0,
    upload: 10,
    download: 20,
    rateUpload: null,
    rateDownload: 0,
    durationMs: 180_000,
    primary: null,
    tags: [],
    host: "api.example",
    sourceIp: "198.18.0.1",
    destinationIp: "1.1.1.1",
    processName: "app.exe",
    processPath: null,
    network: "tcp",
    inbound: "Tun",
    sourcePort: "1546",
    destinationPort: "443",
    start: null,
    rule: "DomainSuffix",
    rulePayload: "example",
    chains: ["家宽-SOCKS5", "AI-家宽"],
    ...overrides
  };
}

describe("live-row display", () => {
  it("joins host and ports like Clash", () => {
    expect(joinHostPort("a.test", "443")).toBe("a.test:443");
    expect(formatRule("DomainSuffix", "chatgpt.com")).toBe("DomainSuffix(chatgpt.com)");
    expect(formatType("Tun", "tcp")).toBe("Tun(tcp)");
  });

  it("keeps unknown for missing rate and does not write 0 B/s", () => {
    const view = displayLiveRow(row({ rateDownload: null, rateUpload: null }), "zh", "未知");
    expect(view.dlSpeed).toBe("未知");
    expect(view.ulSpeed).toBe("未知");
    expect(view.host).toBe("api.example:443");
    expect(view.chains).toBe("家宽-SOCKS5 / AI-家宽");
  });

  it("shows measured zero rate", () => {
    const view = displayLiveRow(row({ rateDownload: 0 }), "en", "Unknown");
    expect(view.dlSpeed).toBe("0 B/s");
  });

  it("formats relative time in both locales", () => {
    expect(formatRelative(180_000, "zh")).toContain("分钟");
    expect(formatRelative(180_000, "en")).toContain("minutes");
  });
});
