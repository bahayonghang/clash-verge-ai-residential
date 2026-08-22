import { describe, expect, it } from "vitest";
import { defaultTimeRange, timeRangeFromPreset } from "./time-range";

describe("时间范围", () => {
  it("默认 24 小时，today 从本地零点起", () => {
    const now = Date.UTC(2026, 7, 21, 12, 0, 0);
    const day = timeRangeFromPreset("24h", now);
    expect(day.endUtc - day.startUtc).toBe(24 * 60 * 60 * 1000);
    expect(defaultTimeRange(now).preset).toBe("24h");
    const local = new Date(now);
    local.setHours(0, 0, 0, 0);
    const today = timeRangeFromPreset("today", now);
    expect(today.startUtc).toBe(local.getTime());
    expect(today.endUtc).toBe(now);
  });
});
