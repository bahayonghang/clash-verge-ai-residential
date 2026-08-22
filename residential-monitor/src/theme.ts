export type UiTheme = "latte" | "frappe" | "macchiato" | "mocha";
export type UiFont = string;
export type LegacyUiFont = "system" | "yahei" | "serif" | "mono";
export type UiFontSize = "sm" | "md" | "lg";
export type UiDensity = "comfortable" | "compact";

export const UI_FONT_SIZES: UiFontSize[] = ["sm", "md", "lg"];
export const UI_DENSITIES: UiDensity[] = ["comfortable", "compact"];

const FONT_FAMILY_MAX_UNITS = 31;
const FONT_FORBIDDEN_MARKS = "\"';{}<>\\";

export function parseUiTheme(value: unknown): UiTheme {
  return value === "latte" || value === "frappe" || value === "macchiato" || value === "mocha"
    ? value
    : "mocha";
}

export function isLegacyUiFont(font: string): font is LegacyUiFont {
  return font === "system" || font === "yahei" || font === "serif" || font === "mono";
}

export function isUiFontFamilyName(value: string): boolean {
  if (!value || value.startsWith("@") || value.length > FONT_FAMILY_MAX_UNITS) {
    return false;
  }
  for (const char of value) {
    const code = char.charCodeAt(0);
    if (code < 32 || code === 127 || FONT_FORBIDDEN_MARKS.includes(char)) {
      return false;
    }
  }
  return true;
}

export function parseUiFont(value: unknown): UiFont {
  if (typeof value !== "string") {
    return "system";
  }
  const name = value.trim();
  if (!name) {
    return "system";
  }
  if (isLegacyUiFont(name) || isUiFontFamilyName(name)) {
    return name;
  }
  return "system";
}

export function quoteFontFamily(name: string): string {
  return `"${name.replaceAll("\\", "\\\\").replaceAll('"', '\\"')}"`;
}

export function fontStack(font: string): string {
  switch (font) {
    case "system":
      return '"Segoe UI", "Microsoft YaHei", sans-serif';
    case "yahei":
      return '"Microsoft YaHei UI", "Microsoft YaHei", sans-serif';
    case "serif":
      return '"Source Han Serif SC", "Noto Serif CJK SC", "Songti SC", SimSun, serif';
    case "mono":
      return '"Cascadia Mono", "Sarasa Mono SC", ui-monospace, monospace';
    default:
      return `${quoteFontFamily(font)}, sans-serif`;
  }
}

export function parseUiFontSize(value: unknown): UiFontSize {
  return value === "sm" || value === "lg" || value === "md" ? value : "md";
}

export function parseUiDensity(value: unknown): UiDensity {
  return value === "compact" || value === "comfortable" ? value : "comfortable";
}

export function applyTheme(theme: UiTheme): void {
  document.documentElement.dataset.theme = theme;
  document.documentElement.style.colorScheme = theme === "latte" ? "light" : "dark";
}

export function applyFont(font: UiFont): void {
  document.documentElement.dataset.font = isLegacyUiFont(font) ? font : "custom";
  document.documentElement.style.setProperty("--ui-font", fontStack(font));
}

export function applyFontSize(size: UiFontSize): void {
  document.documentElement.dataset.fontSize = size;
}

export function applyDensity(density: UiDensity): void {
  document.documentElement.dataset.density = density;
}
