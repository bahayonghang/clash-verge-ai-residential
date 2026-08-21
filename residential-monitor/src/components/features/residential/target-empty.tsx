import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";

export function TargetEmpty({
  locale,
  onGoSettings
}: {
  locale: UiLocale;
  onGoSettings: () => void;
}) {
  return (
    <div
      className="rounded-xl border border-dashed border-border/60 bg-muted/20 px-4 py-3 text-sm"
      data-state="targets-empty"
      role="status"
    >
      <p className="font-medium text-foreground">{t(locale, "residential.targets.empty")}</p>
      <p className="mt-1 text-muted-foreground">{t(locale, "residential.targets.next")}</p>
      <Button type="button" className="mt-3" onClick={onGoSettings}>
        {t(locale, "live.go_settings")}
      </Button>
    </div>
  );
}
