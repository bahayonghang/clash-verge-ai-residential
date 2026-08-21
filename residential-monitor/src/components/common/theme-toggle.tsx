import { Palette } from "lucide-react";
import { t, type UiLocale } from "../../i18n";
import type { UiTheme } from "../../theme";
import { Button } from "../ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger
} from "../ui/dropdown-menu";

const THEMES: UiTheme[] = ["latte", "frappe", "macchiato", "mocha"];

export function ThemeToggle({
  locale,
  theme,
  onThemeChange
}: {
  locale: UiLocale;
  theme: UiTheme;
  onThemeChange: (theme: UiTheme) => void;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" className="h-9 w-9 rounded-full" aria-label={t(locale, "settings.theme")}>
          <Palette className="h-4 w-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {THEMES.map((item) => (
          <DropdownMenuItem
            key={item}
            onClick={() => onThemeChange(item)}
            className={item === theme ? "bg-muted" : undefined}
          >
            {t(locale, `settings.theme.${item}`)}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
