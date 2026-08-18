import { describe, expect, it } from "vitest";

const routes = [
  { id: "overview", available: true, unavailableUntil: null },
  { id: "live", available: true, unavailableUntil: null },
  { id: "reports", available: true, unavailableUntil: null },
  { id: "alerts", available: true, unavailableUntil: null },
  { id: "settings-data", available: true, unavailableUntil: null }
];

describe("应用壳导航", () => {
  it("五段 route 稳定，告警页已启用", () => {
    expect(routes.map((item) => item.id)).toEqual([
      "overview",
      "live",
      "reports",
      "alerts",
      "settings-data"
    ]);
    expect(routes.find((item) => item.id === "reports")?.available).toBe(true);
    expect(routes.find((item) => item.id === "alerts")?.available).toBe(true);
    expect(routes.find((item) => item.id === "alerts")?.unavailableUntil).toBe(null);
  });
});

describe("发布硬化状态", () => {
  it("规定状态都有中文标题和恢复动作", () => {
    const required = [
      "connecting",
      "connected",
      "disconnected",
      "tcp_unauthorized",
      "pipe_access_denied",
      "pipe_busy_timeout",
      "protocol_incompatible",
      "storage_failure",
      "coverage_gap",
      "capability_expired",
      "notification_unavailable",
      "migration_failed",
      "no_data"
    ];
    const titles: Record<string, string> = {
      connecting: "正在连接控制器",
      connected: "已连接",
      disconnected: "控制器已断开",
      tcp_unauthorized: "TCP 鉴权失败",
      pipe_access_denied: "管道访问被拒绝",
      pipe_busy_timeout: "管道忙超时",
      protocol_incompatible: "协议不兼容",
      storage_failure: "存储故障",
      coverage_gap: "存在采集缺口",
      capability_expired: "数据能力已过期",
      notification_unavailable: "系统通知不可用",
      migration_failed: "迁移失败",
      no_data: "暂无采样"
    };
    for (const key of required) {
      expect(titles[key].length).toBeGreaterThan(0);
    }
  });
});
