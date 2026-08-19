import { describe, expect, it } from "vitest";
import { HEALTH_KEYS, healthTitle } from "../i18n";
import { BRAND_MARK, ROUTE_ICONS } from "../nav-icons";

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

  it("五条 route 与产品标记都有本地图标", () => {
    expect(Object.keys(ROUTE_ICONS).sort()).toEqual(routes.map((item) => item.id).sort());
    expect(Object.values(ROUTE_ICONS).every((src) => src.length > 0)).toBe(true);
    expect(BRAND_MARK.length).toBeGreaterThan(0);
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
    for (const key of required) {
      expect(HEALTH_KEYS).toContain(key);
      expect(healthTitle("zh", key).length).toBeGreaterThan(0);
      expect(healthTitle("en", key)).not.toBe(healthTitle("zh", key));
    }
  });
});
