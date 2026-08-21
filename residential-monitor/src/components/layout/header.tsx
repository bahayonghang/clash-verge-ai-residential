import { RefreshCw } from "lucide-react";
import { t, type UiLocale } from "../../i18n";
import type { TimeRange, TimeRangePreset } from "../../lib/time-range";
import { cn } from "../../lib/utils";
import type { UiDensity, UiFont, UiFontSize, UiTheme } from "../../theme";
import { AppearanceMenu } from "../common/appearance-menu";
import { LanguageSwitcher } from "../common/language-switcher";
import { StatusDot } from "../common/status-dot";
import { ThemeToggle } from "../common/theme-toggle";
import { TimeRangePicker } from "../common/time-range-picker";
import { Button } from "../ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "../ui/tooltip";

export function Header({
  locale,
  theme,
  font,
  fontSize,
  density,
  fonts,
  fontsError,
  healthSession,
  healthLabel,
  healthAction,
  autoRefresh,
  timeRange,
  onLocaleChange,
  onThemeChange,
  onFontChange,
  onFontSizeChange,
  onDensityChange,
  onAutoRefreshToggle,
  onTimeRangeChange
}: {
  locale: UiLocale;
  theme: UiTheme;
  font: UiFont;
  fontSize: UiFontSize;
  density: UiDensity;
  fonts: string[];
  fontsError: string | null;
  healthSession: string;
  healthLabel: string;
  healthAction: string;
  autoRefresh: boolean;
  timeRange: TimeRange;
  onLocaleChange: (locale: UiLocale) => void;
  onThemeChange: (theme: UiTheme) => void;
  onFontChange: (font: UiFont) => void;
  onFontSizeChange: (size: UiFontSize) => void;
  onDensityChange: (density: UiDensity) => void;
  onAutoRefreshToggle: () => void;
  onTimeRangeChange: (preset: TimeRangePreset) => void;
}) {
  return (
    <header className="flex h-14 shrink-0 items-center justify-between gap-2 overflow-hidden border-b border-border/40 px-4">
      <div className="flex min-w-0 items-center gap-2">
        <StatusDot session={healthSession} label={healthLabel} ping={false} />
        <div className="min-w-0">
          <p className="truncate text-sm font-medium" aria-label={t(locale, "header.status")}>
            {healthLabel}
          </p>
          <p className="truncate text-xs text-muted-foreground">{healthAction}</p>
        </div>
      </div>
      <div className="flex shrink-0 items-center gap-1">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className={cn(
                "h-9 w-9 rounded-full",
                autoRefresh ? "text-emerald-600" : "text-muted-foreground"
              )}
              aria-label={autoRefresh ? t(locale, "header.auto_refresh") : t(locale, "header.paused")}
              onClick={onAutoRefreshToggle}
            >
              <RefreshCw className={cn("h-4 w-4", autoRefresh && "text-emerald-500")} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <p className="font-medium">
              {autoRefresh ? t(locale, "header.auto_refresh") : t(locale, "header.paused")}
            </p>
            <p className="opacity-80">
              {autoRefresh ? t(locale, "header.click_to_pause") : t(locale, "header.click_to_resume")}
            </p>
          </TooltipContent>
        </Tooltip>
        <TimeRangePicker locale={locale} value={timeRange} onChange={onTimeRangeChange} />
        <LanguageSwitcher locale={locale} onLocaleChange={onLocaleChange} />
        <ThemeToggle locale={locale} theme={theme} onThemeChange={onThemeChange} />
        <AppearanceMenu
          locale={locale}
          font={font}
          fontSize={fontSize}
          density={density}
          fonts={fonts}
          fontsError={fontsError}
          onFontChange={onFontChange}
          onFontSizeChange={onFontSizeChange}
          onDensityChange={onDensityChange}
        />
      </div>
    </header>
  );
}
