import type { NotifyCapability } from "../../../dto";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";

export function NotifyPanel({
  locale,
  notify,
  onTest
}: {
  locale: UiLocale;
  notify: NotifyCapability | null;
  onTest: () => void;
}) {
  return (
    <section className="space-y-2">
      {notify ? (
        <dl className="grid gap-1 text-sm sm:grid-cols-2">
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.notify_on")}</dt>
            <dd>{notify.available ? t(locale, "alerts.notify_on") : t(locale, "alerts.notify_off")}</dd>
          </div>
          <div className="sm:col-span-2">
            <dt className="text-muted-foreground">{t(locale, "alerts.notify.reason")}</dt>
            <dd>{notify.reasonZh}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.notify.focus")}</dt>
            <dd>{notify.canFocusApp ? t(locale, "report.yes") : t(locale, "report.no")}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.notify.focus_unknown")}</dt>
            <dd>{notify.focusAssistUnknown ? t(locale, "report.yes") : t(locale, "report.no")}</dd>
          </div>
        </dl>
      ) : (
        <p className="text-sm text-muted-foreground">{t(locale, "alerts.notify_idle")}</p>
      )}
      <Button type="button" variant="outline" onClick={onTest}>
        {t(locale, "alerts.test")}
      </Button>
    </section>
  );
}
