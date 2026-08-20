export type UiTheme = "latte" | "frappe" | "macchiato" | "mocha";
export type UiFont = "system" | "yahei" | "serif" | "mono";
export type UiFontSize = "sm" | "md" | "lg";
export type UiDensity = "comfortable" | "compact";

export const UI_FONTS: UiFont[] = ["system", "yahei", "serif", "mono"];
export const UI_FONT_SIZES: UiFontSize[] = ["sm", "md", "lg"];
export const UI_DENSITIES: UiDensity[] = ["comfortable", "compact"];

export function parseUiTheme(value: unknown): UiTheme {
  return value === "latte" || value === "frappe" || value === "macchiato" || value === "mocha"
    ? value
    : "mocha";
}

export function parseUiFont(value: unknown): UiFont {
  return value === "yahei" || value === "serif" || value === "mono" || value === "system" ? value : "system";
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
  document.documentElement.dataset.font = font;
}

export function applyFontSize(size: UiFontSize): void {
  document.documentElement.dataset.fontSize = size;
}

export function applyDensity(density: UiDensity): void {
  document.documentElement.dataset.density = density;
}
