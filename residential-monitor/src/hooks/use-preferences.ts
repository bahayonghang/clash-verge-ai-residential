import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { BootstrapDto } from "../dto";
import { parseUiLocale, t, type UiLocale } from "../i18n";
import { isTauriRuntime } from "../ipc/live-session";
import {
  applyShellWidth,
  clampUiSidebarWidth,
  parseUiSidebarWidth,
  SHELL_WIDTH_DEFAULT
} from "../shell-width";
import {
  applyDensity,
  applyFont,
  applyFontSize,
  applyTheme,
  parseUiDensity,
  parseUiFont,
  parseUiFontSize,
  parseUiTheme,
  type UiDensity,
  type UiFont,
  type UiFontSize,
  type UiTheme
} from "../theme";
import { invokeErrorZh } from "../lib/utils";

export interface PreferencesState {
  locale: UiLocale;
  theme: UiTheme;
  font: UiFont;
  fontSize: UiFontSize;
  density: UiDensity;
  sidebarWidth: number;
}

function prefsFromBoot(boot: BootstrapDto): PreferencesState {
  return {
    locale: parseUiLocale(boot.uiLocale),
    theme: parseUiTheme(boot.uiTheme),
    font: parseUiFont(boot.uiFont),
    fontSize: parseUiFontSize(boot.uiFontSize),
    density: parseUiDensity(boot.uiDensity),
    sidebarWidth: parseUiSidebarWidth(boot.uiSidebarWidth)
  };
}

function applyAll(prefs: PreferencesState): void {
  applyTheme(prefs.theme);
  applyFont(prefs.font);
  applyFontSize(prefs.fontSize);
  applyDensity(prefs.density);
  applyShellWidth(prefs.sidebarWidth);
  document.documentElement.lang = prefs.locale === "en" ? "en" : "zh-CN";
}

const DEFAULT_PREFS: PreferencesState = {
  locale: "zh",
  theme: "mocha",
  font: "system",
  fontSize: "md",
  density: "comfortable",
  sidebarWidth: SHELL_WIDTH_DEFAULT
};

export function usePreferences(boot: BootstrapDto | null): {
  prefs: PreferencesState;
  fonts: string[];
  fontsError: string | null;
  errorZh: string | null;
  setLocale: (locale: UiLocale) => Promise<void>;
  setTheme: (theme: UiTheme) => Promise<void>;
  setFont: (font: UiFont) => Promise<void>;
  setFontSize: (size: UiFontSize) => Promise<void>;
  setDensity: (density: UiDensity) => Promise<void>;
  commitSidebarWidth: (width: number) => Promise<void>;
} {
  const [prefs, setPrefs] = useState<PreferencesState>(DEFAULT_PREFS);
  const [fonts, setFonts] = useState<string[]>(["system"]);
  const [fontsError, setFontsError] = useState<string | null>(null);
  const [errorZh, setErrorZh] = useState<string | null>(null);

  useEffect(() => {
    if (!boot) {
      return;
    }
    const next = prefsFromBoot(boot);
    setPrefs(next);
    applyAll(next);
  }, [boot]);

  useEffect(() => {
    if (!isTauriRuntime()) {
      return;
    }
    let cancelled = false;
    void invoke<string[]>("list_ui_fonts")
      .then((list) => {
        if (cancelled) {
          return;
        }
        const unique = ["system", ...list.filter((name) => name && name !== "system")];
        setFonts(unique);
        setFontsError(null);
      })
      .catch((caught: unknown) => {
        if (cancelled) {
          return;
        }
        setFonts(["system"]);
        setFontsError(invokeErrorZh(caught, t(prefs.locale, "settings.font_list_failed")));
      });
    return () => {
      cancelled = true;
    };
  }, [boot, prefs.locale]);

  const persist = useCallback(
    async <K extends keyof PreferencesState>(
      key: K,
      next: PreferencesState[K],
      command: string,
      argName: string,
      apply: (value: PreferencesState[K]) => void,
      parse: (raw: unknown) => PreferencesState[K]
    ): Promise<void> => {
      const previous = prefs[key];
      apply(next);
      setPrefs((current) => ({ ...current, [key]: next }));
      if (!isTauriRuntime()) {
        return;
      }
      try {
        const saved = parse(await invoke<unknown>(command, { [argName]: next }));
        apply(saved);
        setPrefs((current) => ({ ...current, [key]: saved }));
        setErrorZh(null);
      } catch (caught: unknown) {
        apply(previous);
        setPrefs((current) => ({ ...current, [key]: previous }));
        setErrorZh(invokeErrorZh(caught, t(prefs.locale, "prefs.save_fail")));
      }
    },
    [prefs]
  );

  const setLocale = useCallback(
    (locale: UiLocale) =>
      persist(
        "locale",
        locale,
        "save_ui_locale",
        "locale",
        (next) => {
          document.documentElement.lang = next === "en" ? "en" : "zh-CN";
        },
        parseUiLocale
      ),
    [persist]
  );

  const setTheme = useCallback(
    (theme: UiTheme) => persist("theme", theme, "save_ui_theme", "theme", applyTheme, parseUiTheme),
    [persist]
  );

  const setFont = useCallback(
    (font: UiFont) => persist("font", font, "save_ui_font", "font", applyFont, parseUiFont),
    [persist]
  );

  const setFontSize = useCallback(
    (size: UiFontSize) =>
      persist("fontSize", size, "save_ui_font_size", "size", applyFontSize, parseUiFontSize),
    [persist]
  );

  const setDensity = useCallback(
    (density: UiDensity) =>
      persist("density", density, "save_ui_density", "density", applyDensity, parseUiDensity),
    [persist]
  );

  const commitSidebarWidth = useCallback(
    async (width: number): Promise<void> => {
      const previous = prefs.sidebarWidth;
      const next = clampUiSidebarWidth(width);
      applyShellWidth(next);
      setPrefs((current) => ({ ...current, sidebarWidth: next }));
      if (!isTauriRuntime()) {
        return;
      }
      try {
        const saved = parseUiSidebarWidth(await invoke<number>("save_ui_sidebar_width", { width: next }));
        applyShellWidth(saved);
        setPrefs((current) => ({ ...current, sidebarWidth: saved }));
        setErrorZh(null);
      } catch (caught: unknown) {
        applyShellWidth(previous);
        setPrefs((current) => ({ ...current, sidebarWidth: previous }));
        setErrorZh(invokeErrorZh(caught, t(prefs.locale, "prefs.sidebar_save_fail")));
      }
    },
    [prefs.locale, prefs.sidebarWidth]
  );

  useEffect(() => {
    applyAll(prefs);
  }, [prefs]);

  return {
    prefs,
    fonts,
    fontsError,
    errorZh,
    setLocale,
    setTheme,
    setFont,
    setFontSize,
    setDensity,
    commitSidebarWidth
  };
}
