import { describe, expect, it } from "vitest";
import { visibleFontChoices } from "./font-picker";

describe("visibleFontChoices", () => {
  it("保留 system 与当前字体，并按查询过滤", () => {
    const labelOf = (font: string): string => (font === "yahei" ? "微软雅黑" : font);
    const list = visibleFontChoices(["Arial", "yahei"], "Consolas", "", labelOf);
    expect(list[0]).toBe("system");
    expect(list).toContain("Consolas");
    expect(list).toContain("Arial");
    expect(visibleFontChoices(["Arial", "yahei"], "system", "雅", labelOf)).toEqual(["yahei"]);
    expect(visibleFontChoices(['foo";color:red'], "system", "", (font) => font)).toEqual(["system"]);
  });
});
