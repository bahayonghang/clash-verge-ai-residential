import { describe, expect, it } from "vitest";
import { BUSINESS_ROUTES } from "./nav-icons";
import { BUSINESS_NAV_TINTS, businessNavTint } from "./nav-tints";

describe("businessNavTint", () => {
  it("九段业务路由各有独立 CSS 变量，设置无 tint", () => {
    const values = BUSINESS_ROUTES.map((id) => BUSINESS_NAV_TINTS[id]);
    expect(new Set(values).size).toBe(BUSINESS_ROUTES.length);
    for (const id of BUSINESS_ROUTES) {
      expect(businessNavTint(id)).toBe(`var(--nav-${id})`);
    }
    expect(businessNavTint("settings-data")).toBeNull();
  });
});
