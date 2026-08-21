import { useState } from "react";
import type { DeletePreview, DeleteReport } from "../../../dto";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";
import { fieldClass, labelClass } from "../form-styles";

export function DangerSection({
  locale,
  preview,
  report,
  onPreview,
  onConfirm
}: {
  locale: UiLocale;
  preview: DeletePreview | null;
  report: DeleteReport | null;
  onPreview: () => void;
  onConfirm: (phrase: string) => void;
}) {
  const [phrase, setPhrase] = useState("");
  const items = preview?.items ?? [];
  return (
    <section className="space-y-4">
      <div>
        <h2 className="text-base font-semibold">{t(locale, "settings.delete_title")}</h2>
        <p className="text-sm text-muted-foreground">{preview?.noteZh ?? t(locale, "settings.delete_help")}</p>
      </div>
      {locale === "en" ? <p className="text-sm text-muted-foreground">{t(locale, "settings.delete_phrase_en")}</p> : null}
      <ul className="list-disc pl-5 text-sm">
        {items.length === 0 ? (
          <li>{t(locale, "settings.preview_idle")}</li>
        ) : (
          items.map((item) => (
            <li key={item.id}>
              <strong>{item.id}</strong>：{item.noteZh}{" "}
              {item.exists ? t(locale, "settings.exists") : t(locale, "settings.missing")}
            </li>
          ))
        )}
      </ul>
      <label className={labelClass}>
        {t(locale, "settings.delete_phrase")}
        <input
          id="delete-phrase"
          className={fieldClass}
          autoComplete="off"
          value={phrase}
          onChange={(event) => setPhrase(event.target.value)}
        />
      </label>
      <div className="flex flex-wrap gap-2">
        <Button type="button" variant="outline" onClick={onPreview}>
          {t(locale, "settings.preview_delete")}
        </Button>
        <Button type="button" variant="destructive" onClick={() => onConfirm(phrase)}>
          {t(locale, "settings.confirm_delete")}
        </Button>
      </div>
      {report ? (
        <div className="space-y-2">
          <p role="status">{report.summaryZh}</p>
          <ul className="list-disc pl-5 text-sm">
            {report.items.map((item) => (
              <li key={item.id}>
                {item.id}：{item.messageZh}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </section>
  );
}
