import { describe, expect, it } from "vitest";
import { EN } from "./en";
import { ZH } from "./zh";
import { HEALTH_KEYS, healthTitle, parseUiLocale, t } from "./index";

describe("ui locale catalogs", () => {
  it("zh and en have the same keys", () => {
    expect(Object.keys(ZH).sort()).toEqual(Object.keys(EN).sort());
  });

  it("parses only en as English", () => {
    expect(parseUiLocale("en")).toBe("en");
    expect(parseUiLocale("zh")).toBe("zh");
    expect(parseUiLocale("fr")).toBe("zh");
    expect(parseUiLocale(undefined)).toBe("zh");
  });

  it("health keys resolve in both locales", () => {
    for (const key of HEALTH_KEYS) {
      const zh = healthTitle("zh", key);
      const en = healthTitle("en", key);
      expect(zh).not.toBe(`health.${key}`);
      expect(en).not.toBe(`health.${key}`);
      expect(zh).not.toBe(en);
    }
  });

  it("product display name follows the approved English copy", () => {
    expect(t("en", "product.display_name")).toBe("Residential Traffic Monitor");
    expect(t("en", "product.slogan")).toContain("Observed lower bound, not a bill");
    expect(t("en", "product.slogan_sidebar")).toBe("Observed lower bound, not a bill.");
    expect(t("zh", "product.display_name")).toBe("家宽流量监控");
    expect(t("zh", "product.slogan_sidebar")).toContain("观测下界");
  });
});
