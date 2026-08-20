import { describe, expect, it } from "vitest";
import {
  SHELL_WIDTH_DEFAULT,
  SHELL_WIDTH_MAX,
  SHELL_WIDTH_MIN,
  clampUiSidebarWidth,
  parseUiSidebarWidth
} from "./shell-width";

describe("parseUiSidebarWidth", () => {
  it("clamps, rounds, and falls back to 220", () => {
    expect(parseUiSidebarWidth(220)).toBe(220);
    expect(parseUiSidebarWidth("280")).toBe(280);
    expect(parseUiSidebarWidth(12.9)).toBe(SHELL_WIDTH_MIN);
    expect(parseUiSidebarWidth("12.9")).toBe(SHELL_WIDTH_DEFAULT);
    expect(parseUiSidebarWidth(159)).toBe(SHELL_WIDTH_MIN);
    expect(parseUiSidebarWidth(353)).toBe(SHELL_WIDTH_MAX);
    expect(parseUiSidebarWidth(Number.NaN)).toBe(SHELL_WIDTH_DEFAULT);
    expect(parseUiSidebarWidth("nope")).toBe(SHELL_WIDTH_DEFAULT);
    expect(parseUiSidebarWidth(undefined)).toBe(SHELL_WIDTH_DEFAULT);
    expect(parseUiSidebarWidth(null)).toBe(SHELL_WIDTH_DEFAULT);
    expect(clampUiSidebarWidth(221.4)).toBe(221);
  });
});
