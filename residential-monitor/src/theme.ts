export type UiTheme = "latte" | "frappe" | "macchiato" | "mocha";

export function parseUiTheme(value: unknown): UiTheme {
  return value === "latte" || value === "frappe" || value === "macchiato" || value === "mocha"
    ? value
    : "mocha";
}

export function applyTheme(theme: UiTheme): void {
  document.documentElement.dataset.theme = theme;
  document.documentElement.style.colorScheme = theme === "latte" ? "light" : "dark";
}
