export const SHELL_WIDTH_DEFAULT = 220;
export const SHELL_WIDTH_MIN = 160;
export const SHELL_WIDTH_MAX = 352;
export const SHELL_WIDTH_STEP = 8;

export function clampUiSidebarWidth(value: number): number {
  if (!Number.isFinite(value)) {
    return SHELL_WIDTH_DEFAULT;
  }
  return Math.min(SHELL_WIDTH_MAX, Math.max(SHELL_WIDTH_MIN, Math.round(value)));
}

export function parseUiSidebarWidth(value: unknown): number {
  if (typeof value === "number") {
    return clampUiSidebarWidth(value);
  }
  if (typeof value === "string") {
    const trimmed = value.trim();
    if (!/^-?\d+$/.test(trimmed)) {
      return SHELL_WIDTH_DEFAULT;
    }
    return clampUiSidebarWidth(Number(trimmed));
  }
  return SHELL_WIDTH_DEFAULT;
}

export function applyShellWidth(width: number): void {
  document.documentElement.style.setProperty("--shell-width", `${clampUiSidebarWidth(width)}px`);
}
