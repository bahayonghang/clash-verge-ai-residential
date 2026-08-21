import { t, type UiLocale } from "../../../i18n";
import {
  SHELL_WIDTH_MAX,
  SHELL_WIDTH_MIN,
  SHELL_WIDTH_STEP
} from "../../../shell-width";
import {
  UI_DENSITIES,
  UI_FONT_SIZES,
  type UiDensity,
  type UiFont,
  type UiFontSize,
  type UiTheme
} from "../../../theme";
import { cn } from "../../../lib/utils";
import { fieldClass } from "../form-styles";
import { FontPicker } from "./font-picker";

const THEMES: UiTheme[] = ["latte", "frappe", "macchiato", "mocha"];

export function AppearanceSection({
  locale,
  theme,
  font,
  fontSize,
  density,
  fonts,
  fontsError,
  sidebarWidth,
  onLocale,
  onTheme,
  onFont,
  onFontSize,
  onDensity,
  onSidebarWidth
}: {
  locale: UiLocale;
  theme: UiTheme;
  font: UiFont;
  fontSize: UiFontSize;
  density: UiDensity;
  fonts: string[];
  fontsError: string | null;
  sidebarWidth: number;
  onLocale: (locale: UiLocale) => void;
  onTheme: (theme: UiTheme) => void;
  onFont: (font: UiFont) => void;
  onFontSize: (size: UiFontSize) => void;
  onDensity: (density: UiDensity) => void;
  onSidebarWidth: (width: number) => void;
}) {
  return (
    <section className="space-y-6">
      <div>
        <h2 className="text-base font-semibold">{t(locale, "settings.appearance.title")}</h2>
        <p className="text-sm text-muted-foreground">{t(locale, "settings.appearance.help")}</p>
      </div>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h3 className="text-sm font-medium">{t(locale, "settings.locale")}</h3>
          <p className="text-xs text-muted-foreground">{t(locale, "settings.locale_help")}</p>
        </div>
        <div className="flex rounded-md border" role="group" aria-label={t(locale, "settings.locale")}>
          {(["zh", "en"] as UiLocale[]).map((item) => (
            <button
              key={item}
              type="button"
              className={cn("px-3 py-1.5 text-sm", locale === item && "bg-primary text-primary-foreground")}
              onClick={() => onLocale(item)}
            >
              {t(locale, `settings.locale.${item}`)}
            </button>
          ))}
        </div>
      </div>
      <div className="space-y-2">
        <h3 className="text-sm font-medium">{t(locale, "settings.theme")}</h3>
        <p className="text-xs text-muted-foreground">{t(locale, "settings.theme_help")}</p>
        <div className="grid gap-2 sm:grid-cols-2">
          {THEMES.map((item) => (
            <button
              key={item}
              type="button"
              className={cn(
                "flex items-center justify-between rounded-md border px-3 py-2 text-sm",
                theme === item && "border-primary bg-primary/10"
              )}
              onClick={() => onTheme(item)}
            >
              {t(locale, `settings.theme.${item}`)}
              {theme === item ? <span aria-hidden="true">✓</span> : null}
            </button>
          ))}
        </div>
      </div>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h3 className="text-sm font-medium">{t(locale, "settings.font")}</h3>
          <p className="text-xs text-muted-foreground">{t(locale, "settings.font_help")}</p>
        </div>
        <FontPicker
          locale={locale}
          font={font}
          fonts={fonts}
          fontsError={fontsError}
          onChange={onFont}
        />
      </div>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h3 className="text-sm font-medium">{t(locale, "settings.font_size")}</h3>
          <p className="text-xs text-muted-foreground">{t(locale, "settings.font_size_help")}</p>
        </div>
        <div className="flex rounded-md border" role="group" aria-label={t(locale, "settings.font_size")}>
          {UI_FONT_SIZES.map((item) => (
            <button
              key={item}
              type="button"
              className={cn("px-3 py-1.5 text-sm", fontSize === item && "bg-primary text-primary-foreground")}
              onClick={() => onFontSize(item)}
            >
              {t(locale, `settings.font_size.${item}`)}
            </button>
          ))}
        </div>
      </div>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h3 className="text-sm font-medium">{t(locale, "settings.density")}</h3>
          <p className="text-xs text-muted-foreground">{t(locale, "settings.density_help")}</p>
        </div>
        <div className="flex rounded-md border" role="group" aria-label={t(locale, "settings.density")}>
          {UI_DENSITIES.map((item) => (
            <button
              key={item}
              type="button"
              className={cn("px-3 py-1.5 text-sm", density === item && "bg-primary text-primary-foreground")}
              onClick={() => onDensity(item)}
            >
              {t(locale, `settings.density.${item}`)}
            </button>
          ))}
        </div>
      </div>
      <label className="flex max-w-xs flex-col gap-1 text-sm">
        {t(locale, "settings.sidebar_width")}
        <input
          className={fieldClass}
          type="number"
          min={SHELL_WIDTH_MIN}
          max={SHELL_WIDTH_MAX}
          step={SHELL_WIDTH_STEP}
          value={sidebarWidth}
          onChange={(event) => onSidebarWidth(Number(event.target.value))}
        />
        <span className="text-xs text-muted-foreground">{t(locale, "settings.sidebar_width_help")}</span>
      </label>
    </section>
  );
}
