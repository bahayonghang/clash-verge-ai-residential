import { describe, expect, it } from "vitest";
import { HEALTH_KEYS, healthTitle } from "../i18n";
import { BRAND_MARK, ROUTE_ICONS, ROUTE_ORDER, isRouteId } from "../nav-icons";

describe("应用壳导航", () => {
  it("十段 route 稳定，顺序固定", () => {
    expect(ROUTE_ORDER).toEqual([
      "overview",
      "live",
      "residential",
      "host",
      "rule",
      "chain",
      "process",
      "reports",
      "alerts",
      "settings-data"
    ]);
    expect(ROUTE_ORDER).toHaveLength(10);
  });

  it("十条 route 与产品标记都有本地图标", () => {
    expect(Object.keys(ROUTE_ICONS).sort()).toEqual([...ROUTE_ORDER].sort());
    expect(Object.values(ROUTE_ICONS).every((src) => src.length > 0)).toBe(true);
    expect(BRAND_MARK.length).toBeGreaterThan(0);
  });

  it("isRouteId 不把原型方法名当成 route", () => {
    expect(isRouteId("overview")).toBe(true);
    expect(isRouteId("settings-data")).toBe(true);
    expect(isRouteId("toString")).toBe(false);
    expect(isRouteId("constructor")).toBe(false);
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
