import { describe, expect, it } from "vitest";

const routes = [
  { id: "overview", available: true, unavailableUntil: null },
  { id: "live", available: true, unavailableUntil: null },
  { id: "reports", available: true, unavailableUntil: null },
  { id: "alerts", available: false, unavailableUntil: "C4" },
  { id: "settings-data", available: true, unavailableUntil: null }
];

describe("应用壳导航", () => {
  it("五段 route 稳定，告警页仍禁用", () => {
    expect(routes.map((item) => item.id)).toEqual([
      "overview",
      "live",
      "reports",
      "alerts",
      "settings-data"
    ]);
    expect(routes.find((item) => item.id === "reports")?.available).toBe(true);
    expect(routes.find((item) => item.id === "alerts")?.unavailableUntil).toBe("C4");
  });
});
