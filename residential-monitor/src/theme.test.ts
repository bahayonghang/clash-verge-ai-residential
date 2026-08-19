import { describe, expect, it } from "vitest";
import { parseUiTheme } from "./theme";

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
