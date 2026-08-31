import { afterEach, describe, expect, it, vi } from "vitest";
import appSource from "../app.tsx?raw";
import {
  defaultTimeRange,
  rollTimeRange,
  startRollingTimeRange,
  TIME_RANGE_ROLL_INTERVAL_MS,
  timeRangeFromPreset
} from "./time-range";

afterEach(() => {
  vi.useRealTimers();
});

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

  it("24h 自动窗口按分钟滚动，暂停冻结并在恢复时立即追平", () => {
    vi.useFakeTimers();
    const started = Date.UTC(2026, 7, 31, 12, 0, 30);
    vi.setSystemTime(started);
    let range = timeRangeFromPreset("24h", Date.now());
    const stop = startRollingTimeRange(() => {
      range = rollTimeRange(range);
    });
    vi.advanceTimersByTime(3 * TIME_RANGE_ROLL_INTERVAL_MS);
    expect(range.endUtc).toBe(started + 3 * TIME_RANGE_ROLL_INTERVAL_MS);
    expect(range.endUtc - range.startUtc).toBe(24 * 60 * 60 * 1000);

    stop();
    const pausedEnd = range.endUtc;
    vi.advanceTimersByTime(5 * TIME_RANGE_ROLL_INTERVAL_MS);
    expect(range.endUtc).toBe(pausedEnd);

    const resume = startRollingTimeRange(() => {
      range = rollTimeRange(range);
    });
    expect(range.endUtc).toBe(Date.now());
    resume();
  });

  it("today 跨本地午夜后重算零点，预设切换立即使用当前时间", () => {
    vi.useFakeTimers();
    const beforeMidnight = new Date(2026, 7, 31, 23, 59, 30).getTime();
    vi.setSystemTime(beforeMidnight);
    let range = timeRangeFromPreset("today", Date.now());
    const stop = startRollingTimeRange(() => {
      range = rollTimeRange(range);
    });
    vi.advanceTimersByTime(TIME_RANGE_ROLL_INTERVAL_MS);
    const afterMidnight = new Date(beforeMidnight + TIME_RANGE_ROLL_INTERVAL_MS);
    const expectedStart = new Date(afterMidnight);
    expectedStart.setHours(0, 0, 0, 0);
    expect(range.startUtc).toBe(expectedStart.getTime());
    expect(range.endUtc).toBe(afterMidnight.getTime());
    stop();

    const sevenDays = timeRangeFromPreset("7d", Date.now());
    expect(sevenDays.endUtc).toBe(Date.now());
    expect(sevenDays.endUtc - sevenDays.startUtc).toBe(7 * 24 * 60 * 60 * 1000);
  });

  it("App 只建立一个全局滚动时钟并由 effect 清理", () => {
    expect(appSource).toContain("startRollingTimeRange");
    expect(appSource).toContain("setTimeRange((current) => rollTimeRange(current))");
    expect(appSource).not.toContain("setTimeout");
  });
});
