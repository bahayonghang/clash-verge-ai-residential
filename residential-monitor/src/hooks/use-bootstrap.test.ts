import { describe, expect, it } from "vitest";
import { ROUTE_ORDER } from "../nav-icons";
import { decodeBootstrap, previewBootstrap } from "./use-bootstrap";

describe("decodeBootstrap", () => {
  it("预览引导含十段 route", () => {
    expect(previewBootstrap().routes.map((item) => item.id)).toEqual(ROUTE_ORDER);
  });

  it("接受引导里的既有五段 route", () => {
    const raw = previewBootstrap();
    raw.routes = raw.routes.filter((item) =>
      item.id === "overview" ||
      item.id === "live" ||
      item.id === "reports" ||
      item.id === "alerts" ||
      item.id === "settings-data"
    );
    expect(decodeBootstrap(raw).routes).toHaveLength(5);
  });

  it("未知 schema 或未知 route 失败，不猜测", () => {
    expect(() => decodeBootstrap({ schemaVersion: 2 })).toThrow();
    const raw = previewBootstrap();
    const poisoned: unknown = {
      ...raw,
      routes: [...raw.routes, { id: "toString", titleZh: "x", available: true, unavailableUntil: null }]
    };
    expect(() => decodeBootstrap(poisoned)).toThrow(/未知或无效/);
  });
});
