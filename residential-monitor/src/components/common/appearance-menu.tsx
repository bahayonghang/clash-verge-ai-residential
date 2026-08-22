import { useMemo, useState } from "react";
import { SlidersHorizontal } from "lucide-react";
import { t, type UiLocale } from "../../i18n";
import type { UiDensity, UiFont, UiFontSize } from "../../theme";
import { UI_DENSITIES, UI_FONT_SIZES } from "../../theme";
import { Button } from "../ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger
} from "../ui/dropdown-menu";

export function AppearanceMenu({
  locale,
  font,
  fontSize,
  density,
  fonts,
  fontsError,
  onFontChange,
  onFontSizeChange,
  onDensityChange
}: {
  locale: UiLocale;
  font: UiFont;
  fontSize: UiFontSize;
  density: UiDensity;
  fonts: string[];
  fontsError: string | null;
  onFontChange: (font: UiFont) => void;
  onFontSizeChange: (size: UiFontSize) => void;
  onDensityChange: (density: UiDensity) => void;
}) {
  const [query, setQuery] = useState("");
  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    const names = fonts.includes("system") ? fonts : ["system", ...fonts];
    if (!needle) {
      return names;
    }
    return names.filter((name) => name.toLowerCase().includes(needle));
  }, [fonts, query]);

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="h-9 w-9 rounded-full"
          aria-label={t(locale, "header.appearance")}
        >
          <SlidersHorizontal className="h-4 w-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-64">
        <DropdownMenuLabel>{t(locale, "settings.font")}</DropdownMenuLabel>
        <div className="px-2 pb-2">
          <input
            type="search"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => event.stopPropagation()}
            placeholder={t(locale, "settings.font_search")}
            className="h-8 w-full rounded-md border bg-background px-2 text-sm"
          />
        </div>
        {fontsError ? <p className="px-2 pb-2 text-xs text-destructive">{fontsError}</p> : null}
        <div className="max-h-40 overflow-auto">
          {filtered.map((name) => (
            <DropdownMenuItem
              key={name}
              onClick={() => onFontChange(name)}
              className={name === font ? "bg-muted" : undefined}
            >
              {name === "system" ? t(locale, "settings.font.system") : name}
            </DropdownMenuItem>
          ))}
        </div>
        <DropdownMenuSeparator />
        <DropdownMenuLabel>{t(locale, "settings.font_size")}</DropdownMenuLabel>
        {UI_FONT_SIZES.map((size) => (
          <DropdownMenuItem
            key={size}
            onClick={() => onFontSizeChange(size)}
            className={size === fontSize ? "bg-muted" : undefined}
          >
            {t(locale, `settings.font_size.${size}`)}
          </DropdownMenuItem>
        ))}
        <DropdownMenuSeparator />
        <DropdownMenuLabel>{t(locale, "settings.density")}</DropdownMenuLabel>
        {UI_DENSITIES.map((item) => (
          <DropdownMenuItem
            key={item}
            onClick={() => onDensityChange(item)}
            className={item === density ? "bg-muted" : undefined}
          >
            {t(locale, `settings.density.${item}`)}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
