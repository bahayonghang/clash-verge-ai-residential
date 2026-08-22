import { useEffect, useState } from "react";
import type { RetentionPreview } from "../../../dto";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";

export function DataSection({
  locale,
  logDir,
  dataDir,
  retention,
  onLoad,
  onOpenLog,
  onPreviewRetention,
  onRunRetention,
  onBackup,
  onRestore,
  onValidate,
  onVacuum
}: {
  locale: UiLocale;
  logDir: string;
  dataDir: string;
  retention: RetentionPreview | null;
  onLoad: () => void;
  onOpenLog: () => void;
  onPreviewRetention: () => void;
  onRunRetention: () => void;
  onBackup: () => void;
  onRestore: () => void;
  onValidate: () => Promise<boolean | null>;
  onVacuum: () => void;
}) {
  const [validateNote, setValidateNote] = useState<string | null>(null);
  useEffect(() => {
    onLoad();
  }, [onLoad]);

  return (
    <section className="space-y-4">
      <div>
        <h2 className="text-base font-semibold">{t(locale, "settings.data")}</h2>
        <p className="text-sm text-muted-foreground">{t(locale, "settings.data_help")}</p>
      </div>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h3 className="text-sm font-medium">{t(locale, "settings.log_dir")}</h3>
          <p className="font-mono text-sm">{logDir || t(locale, "settings.log_dir_unknown")}</p>
        </div>
        <Button type="button" variant="outline" disabled={!logDir} onClick={onOpenLog}>
          {t(locale, "settings.open_log_dir")}
        </Button>
      </div>
      <div>
        <h3 className="text-sm font-medium">{t(locale, "settings.data_dir")}</h3>
        <p className="font-mono text-sm">{dataDir || t(locale, "common.unknown")}</p>
      </div>
      <div className="flex flex-wrap gap-2">
        <Button type="button" variant="outline" onClick={onBackup}>
          {t(locale, "settings.backup")}
        </Button>
        <Button type="button" variant="outline" onClick={onRestore}>
          {t(locale, "settings.restore")}
        </Button>
        <Button
          type="button"
          variant="outline"
          onClick={() => {
            void onValidate().then((ok) => {
              setValidateNote(ok === true ? t(locale, "settings.validate_ok") : null);
            });
          }}
        >
          {t(locale, "settings.validate")}
        </Button>
        <Button type="button" variant="outline" onClick={onPreviewRetention}>
          {t(locale, "settings.retention_preview")}
        </Button>
        <Button type="button" variant="outline" onClick={onRunRetention}>
          {t(locale, "settings.retention_run")}
        </Button>
        <Button type="button" variant="outline" onClick={onVacuum}>
          {t(locale, "settings.vacuum")}
        </Button>
      </div>
      <p className="text-sm text-muted-foreground">
        {retention
          ? `${retention.noteZh} raw ${retention.rawRows} / hourly ${retention.hourlyRows} / dailyDim ${retention.dailyDimRows} / dailyCore ${retention.dailyCoreRows}. ${t(locale, "settings.retention.auto_delete")}=${retention.autoDeleteEnabled}`
          : t(locale, "settings.retention_note")}
      </p>
      {validateNote ? <p className="text-sm">{validateNote}</p> : null}
    </section>
  );
}
