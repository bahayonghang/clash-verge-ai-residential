import { Languages } from "lucide-react";
import { t, type UiLocale } from "../../i18n";
import { Button } from "../ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger
} from "../ui/dropdown-menu";

const LOCALES: UiLocale[] = ["zh", "en"];

export function LanguageSwitcher({
  locale,
  onLocaleChange
}: {
  locale: UiLocale;
  onLocaleChange: (locale: UiLocale) => void;
}) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" className="h-9 w-9 rounded-full" aria-label={t(locale, "settings.locale")}>
          <Languages className="h-4 w-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {LOCALES.map((item) => (
          <DropdownMenuItem
            key={item}
            onClick={() => onLocaleChange(item)}
            className={item === locale ? "bg-muted" : undefined}
          >
            {t(locale, `settings.locale.${item}`)}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
