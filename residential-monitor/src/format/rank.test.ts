import { describe, expect, it } from "vitest";
import {
  ellipsizeLabel,
  filtersForDrilldown,
  formatRankLabel,
  looksLikeIp,
  rankAxisWidth,
  UNKNOWN_RANK_IDENTITY
} from "./rank";

describe("rank helpers", () => {
  it("looksLikeIp accepts v4 and v6", () => {
    expect(looksLikeIp("1.2.3.4")).toBe(true);
    expect(looksLikeIp("::1")).toBe(true);
    expect(looksLikeIp("[2001:db8::1]")).toBe(true);
    expect(looksLikeIp("a.test")).toBe(false);
    expect(looksLikeIp("__unknown__")).toBe(false);
  });

  it("formatRankLabel marks IP identities", () => {
    expect(formatRankLabel("8.8.8.8", "8.8.8.8", "未知")).toBe("8.8.8.8  IP");
    expect(formatRankLabel("a.test", "a.test", "未知")).toBe("a.test");
    expect(formatRankLabel(UNKNOWN_RANK_IDENTITY, "未知", "未知")).toBe("未知");
  });

  it("ellipsizeLabel keeps the right-hand side", () => {
    expect(ellipsizeLabel("short", 12)).toBe("short");
    expect(ellipsizeLabel("static.rust-lang.org", 12)).toBe("…st-lang.org");
  });

  it("rankAxisWidth clamps between 96 and 220", () => {
    expect(rankAxisWidth(["a"])).toBe(96);
    expect(rankAxisWidth(["release-assets.githubusercontent.com"])).toBeLessThanOrEqual(220);
    expect(rankAxisWidth(["release-assets.githubusercontent.com"])).toBeGreaterThan(96);
  });

  it("host unknown drilldown sets host sentinel", () => {
    expect(filtersForDrilldown("host", UNKNOWN_RANK_IDENTITY).host).toBe(UNKNOWN_RANK_IDENTITY);
    expect(filtersForDrilldown("rule", UNKNOWN_RANK_IDENTITY).rule).toBeNull();
  });
});
