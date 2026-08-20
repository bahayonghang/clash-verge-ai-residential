import { describe, expect, it } from "vitest";
import { fontStack, parseUiDensity, parseUiFont, parseUiFontSize, parseUiTheme } from "./theme";

describe("parseUiTheme", () => {
  it("accepts the four flavors and falls back to mocha", () => {
    expect(parseUiTheme("latte")).toBe("latte");
    expect(parseUiTheme("frappe")).toBe("frappe");
    expect(parseUiTheme("macchiato")).toBe("macchiato");
    expect(parseUiTheme("mocha")).toBe("mocha");
    expect(parseUiTheme("dark")).toBe("mocha");
    expect(parseUiTheme(" mocha ")).toBe("mocha");
    expect(parseUiTheme("Latte")).toBe("mocha");
    expect(parseUiTheme(undefined)).toBe("mocha");
    expect(parseUiTheme(null)).toBe("mocha");
  });
});

describe("appearance prefs", () => {
  it("parses font stacks, families, and injection attempts", () => {
    expect(parseUiFont("system")).toBe("system");
    expect(parseUiFont("yahei")).toBe("yahei");
    expect(parseUiFont("serif")).toBe("serif");
    expect(parseUiFont("mono")).toBe("mono");
    expect(parseUiFont("Comic Sans")).toBe("Comic Sans");
    expect(parseUiFont("Microsoft YaHei")).toBe("Microsoft YaHei");
    expect(parseUiFont('foo";color:red')).toBe("system");
    expect(parseUiFont("@SimSun")).toBe("system");
    expect(parseUiFont("a".repeat(32))).toBe("system");
    expect(parseUiFont(undefined)).toBe("system");
    expect(fontStack("system")).toContain("Segoe UI");
    expect(fontStack("Microsoft YaHei")).toBe('"Microsoft YaHei", sans-serif');
  });

  it("parses font sizes and falls back to md", () => {
    expect(parseUiFontSize("sm")).toBe("sm");
    expect(parseUiFontSize("md")).toBe("md");
    expect(parseUiFontSize("lg")).toBe("lg");
    expect(parseUiFontSize("20")).toBe("md");
    expect(parseUiFontSize(undefined)).toBe("md");
  });

  it("parses density and falls back to comfortable", () => {
    expect(parseUiDensity("compact")).toBe("compact");
    expect(parseUiDensity("comfortable")).toBe("comfortable");
    expect(parseUiDensity("tight")).toBe("comfortable");
    expect(parseUiDensity(undefined)).toBe("comfortable");
  });
});
