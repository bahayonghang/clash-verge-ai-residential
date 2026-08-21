import { useMemo, useState } from "react";
import { t, type UiLocale } from "../../../i18n";
import { fontStack, isLegacyUiFont, parseUiFont, type UiFont } from "../../../theme";
import { cn } from "../../../lib/utils";
import { fieldClass } from "../form-styles";

export function visibleFontChoices(
  fonts: string[],
  current: string,
  query: string,
  labelOf: (font: string) => string
): string[] {
  const items: string[] = [];
  const seen = new Set<string>();
  const add = (font: string): void => {
    const key = font.toLowerCase();
    if (!font || seen.has(key)) {
      return;
    }
    seen.add(key);
    items.push(font);
  };
  add("system");
  if (isLegacyUiFont(current) || parseUiFont(current) === current) {
    add(current);
  }
  for (const font of fonts) {
    if (isLegacyUiFont(font) || parseUiFont(font) === font) {
      add(font);
    }
  }
  const needle = query.trim().toLowerCase();
  if (!needle) {
    return items;
  }
  return items.filter(
    (font) => font.toLowerCase().includes(needle) || labelOf(font).toLowerCase().includes(needle)
  );
}

export function FontPicker({
  locale,
  font,
  fonts,
  fontsError,
  onChange
}: {
  locale: UiLocale;
  font: UiFont;
  fonts: string[];
  fontsError: string | null;
  onChange: (font: UiFont) => void;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const choices = useMemo(() => {
    const labelOf = (name: string): string =>
      isLegacyUiFont(name) ? t(locale, `settings.font.${name}`) : name;
    return visibleFontChoices(fonts, font, query, labelOf);
  }, [fonts, font, query, locale]);
  const labelOf = (name: string): string =>
    isLegacyUiFont(name) ? t(locale, `settings.font.${name}`) : name;

  return (
    <div className="relative min-w-56">
      <button
        type="button"
        className={cn(fieldClass, "flex items-center justify-between")}
        style={{ fontFamily: fontStack(font) }}
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={() => setOpen((value) => !value)}
      >
        <span>{labelOf(font)}</span>
      </button>
      {fontsError ? <p className="mt-1 text-xs text-destructive">{fontsError}</p> : null}
      {open ? (
        <div className="absolute z-20 mt-1 w-full rounded-md border bg-popover p-2 shadow-md">
          <input
            className={fieldClass}
            type="search"
            autoComplete="off"
            placeholder={t(locale, "settings.font_search")}
            aria-label={t(locale, "settings.font_search")}
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
          <ul className="mt-2 max-h-40 overflow-auto" role="listbox" aria-label={t(locale, "settings.font")}>
            {choices.map((name) => (
              <li key={name}>
                <button
                  type="button"
                  role="option"
                  aria-selected={name === font}
                  className={cn(
                    "flex w-full items-center justify-between rounded-md px-2 py-1 text-left text-sm hover:bg-muted/40",
                    name === font && "bg-muted"
                  )}
                  style={{ fontFamily: fontStack(name) }}
                  onClick={() => {
                    onChange(name);
                    setOpen(false);
                  }}
                >
                  <span>{labelOf(name)}</span>
                  {name === font ? <span aria-hidden="true">✓</span> : null}
                </button>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  );
}
