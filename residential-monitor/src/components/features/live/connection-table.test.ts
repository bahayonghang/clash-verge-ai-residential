import { describe, expect, it } from "vitest";
import { displayLiveRow } from "../../../format/live-row";
import type { LiveConnectionView } from "../../../dto";
import { DATA_COLUMNS, type DataColumnId } from "../../../live-table-layout";
import { cellOf } from "./connection-table";

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

describe("cellOf", () => {
  it("覆盖每个 DataColumnId", () => {
    const view = displayLiveRow(row(), "zh", "未知");
    expect(cellOf(view, "host")).toEqual({ text: "api.example:443", numeric: false, title: "api.example:443" });
    expect(cellOf(view, "download").numeric).toBe(true);
    expect(cellOf(view, "upload").numeric).toBe(true);
    expect(cellOf(view, "rateDownload")).toEqual({ text: "0 B/s", numeric: true, title: "0 B/s" });
    expect(cellOf(view, "rateUpload").text).toBe("未知");
    expect(cellOf(view, "chain").text).toContain("家宽-SOCKS5");
    expect(cellOf(view, "rule").text).toBe("DomainSuffix(example)");
    expect(cellOf(view, "process").text).toBe("app.exe");
    expect(cellOf(view, "duration").numeric).toBe(true);
    expect(cellOf(view, "source").text).toBe("198.18.0.1:1546");
    expect(cellOf(view, "destination").text).toBe("1.1.1.1:443");
    expect(cellOf(view, "type").text).toBe("Tun(tcp)");
    expect(DATA_COLUMNS.map((column) => cellOf(view, column).text).every((text) => text.length > 0)).toBe(true);
  });

  it("未知值回退为传入的 unknown 文案", () => {
    const view = displayLiveRow(
      row({
        host: null,
        sourceIp: null,
        destinationIp: null,
        processName: null,
        inbound: null,
        network: null,
        rule: null,
        chains: [],
        rateDownload: null,
        rateUpload: null,
        durationMs: null
      }),
      "zh",
      "未知"
    );
    expect(cellOf(view, "host").text).toBe("未知");
    expect(cellOf(view, "process").text).toBe("未知");
    expect(cellOf(view, "rule").text).toBe("未知");
    expect(cellOf(view, "chain").text).toBe("未知");
    expect(cellOf(view, "source").text).toBe("未知");
    expect(cellOf(view, "destination").text).toBe("未知");
    expect(cellOf(view, "type").text).toBe("未知");
    expect(cellOf(view, "rateDownload").text).toBe("未知");
    expect(cellOf(view, "duration").text).toBe("未知");
    expect(cellOf(view, "nope" as DataColumnId).text).toBe(view.host);
  });
});
