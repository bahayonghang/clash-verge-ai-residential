import type { BootstrapDto } from "../../dto";
import { t, type UiLocale } from "../../i18n";
import { formatTemplate } from "../../lib/utils";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../ui/card";

export function RecoveryPane({ locale, boot }: { locale: UiLocale; boot: BootstrapDto }) {
  const recovery = boot.recovery;
  if (!recovery) {
    return (
      <Card className="max-w-xl">
        <CardHeader>
          <CardTitle>{t(locale, "recovery.title")}</CardTitle>
          <CardDescription>{t(locale, "recovery.missing")}</CardDescription>
        </CardHeader>
      </Card>
    );
  }
  return (
    <Card className="max-w-xl">
      <CardHeader>
        <CardTitle>{t(locale, "recovery.title")}</CardTitle>
        <CardDescription>
          {formatTemplate(t(locale, "recovery.meta"), {
            app: recovery.appVersion,
            db: recovery.userVersion,
            max: recovery.supportedMax
          })}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-2 text-sm text-muted-foreground">
        <p>{recovery.future ? t(locale, "recovery.future") : t(locale, "recovery.unreadable")}</p>
        <p>{recovery.restoreNoteZh}</p>
        <p>{t(locale, "recovery.log_dir")}</p>
        <p className="font-mono text-foreground">{boot.logDir || t(locale, "settings.log_dir_unknown")}</p>
        <div>
          <h3 className="mb-1 font-medium text-foreground">{t(locale, "recovery.backups")}</h3>
          <ul className="list-disc pl-5">
            {recovery.backups.length > 0 ? (
              recovery.backups.map((item) => <li key={item}>{item}</li>)
            ) : (
              <li>{t(locale, "common.none")}</li>
            )}
          </ul>
        </div>
      </CardContent>
    </Card>
  );
}
