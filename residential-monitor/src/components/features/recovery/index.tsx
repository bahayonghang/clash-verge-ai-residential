import { useEffect } from "react";
import type { BootstrapDto } from "../../../dto";
import { t, type UiLocale } from "../../../i18n";
import { formatTemplate } from "../../../lib/utils";
import { useSettings } from "../../../hooks/use-settings";
import { Button } from "../../ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../ui/card";

export function RecoveryPage({ locale, boot }: { locale: UiLocale; boot: BootstrapDto }) {
  const settings = useSettings(locale, boot);
  const recovery = boot.recovery;
  const loadDataDir = settings.loadDataDir;

  useEffect(() => {
    void loadDataDir();
  }, [loadDataDir]);

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
      <CardContent className="space-y-3 text-sm text-muted-foreground">
        <p>{recovery.future ? t(locale, "recovery.future") : t(locale, "recovery.unreadable")}</p>
        <p>{recovery.restoreNoteZh}</p>
        <p>{t(locale, "recovery.log_dir")}</p>
        <p className="font-mono text-foreground">{boot.logDir || t(locale, "settings.log_dir_unknown")}</p>
        <div className="flex flex-wrap gap-2">
          <Button type="button" variant="outline" disabled={!boot.logDir} onClick={() => void settings.openLogDir()}>
            {t(locale, "recovery.open_log_dir")}
          </Button>
          <Button
            type="button"
            disabled={!recovery.restoreAvailable}
            onClick={() => void settings.restoreBackup()}
          >
            {t(locale, "recovery.run")}
          </Button>
        </div>
        {settings.errorZh ? (
          <p className="text-destructive" role="alert">
            {settings.errorZh}
          </p>
        ) : null}
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
