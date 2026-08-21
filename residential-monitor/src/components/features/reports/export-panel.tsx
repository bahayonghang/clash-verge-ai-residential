import { useState } from "react";
import type { ExportFormat, ExportPreview, ExportSpec, RedactMode } from "../../../hooks/use-report-archive";
import { defaultExportSpec } from "../../../hooks/use-report-archive";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";
import { fieldClass, labelClass } from "../form-styles";

export function ExportPanel({
  locale,
  preview,
  disabled,
  onPreview,
  onExport
}: {
  locale: UiLocale;
  preview: ExportPreview | null;
  disabled: boolean;
  onPreview: (spec: ExportSpec) => void;
  onExport: (spec: ExportSpec) => void;
}) {
  const [spec, setSpec] = useState<ExportSpec>(defaultExportSpec);
  return (
    <section className="space-y-3">
      <h3 className="text-sm font-semibold uppercase tracking-wider">{t(locale, "report.export_preview")}</h3>
      <div className="flex flex-wrap items-end gap-3">
        <label className={labelClass}>
          {t(locale, "report.export.spec")}
          <select
            className={fieldClass}
            value={spec.format}
            onChange={(event) => setSpec({ ...spec, format: event.target.value as ExportFormat })}
          >
            <option value="csv">CSV</option>
            <option value="json">JSON</option>
            <option value="html">HTML</option>
          </select>
        </label>
        <label className={labelClass}>
          {t(locale, "report.redact_host")}
          <select
            className={fieldClass}
            value={spec.redactHost}
            onChange={(event) => setSpec({ ...spec, redactHost: event.target.value as RedactMode })}
          >
            <option value="none">{t(locale, "report.redact.none")}</option>
            <option value="mask">{t(locale, "report.redact.mask")}</option>
          </select>
        </label>
        <label className={labelClass}>
          {t(locale, "report.redact_process")}
          <select
            className={fieldClass}
            value={spec.redactProcess}
            onChange={(event) => setSpec({ ...spec, redactProcess: event.target.value as RedactMode })}
          >
            <option value="none">{t(locale, "report.redact.none")}</option>
            <option value="mask">{t(locale, "report.redact.mask")}</option>
          </select>
        </label>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={spec.includeSeries}
            onChange={(event) => setSpec({ ...spec, includeSeries: event.target.checked })}
          />
          {t(locale, "report.include_series")}
        </label>
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={spec.includeRankings}
            onChange={(event) => setSpec({ ...spec, includeRankings: event.target.checked })}
          />
          {t(locale, "report.include_rankings")}
        </label>
        <Button type="button" variant="outline" disabled={disabled} onClick={() => onPreview(spec)}>
          {t(locale, "report.export_preview")}
        </Button>
        <Button type="button" disabled={disabled} onClick={() => onExport(spec)}>
          {t(locale, `report.export_${spec.format}`)}
        </Button>
      </div>
      {preview ? (
        <p className="text-sm text-muted-foreground">
          {preview.metadataZh} · {preview.rowCount} · {preview.sampleLabels.join(", ")}
        </p>
      ) : null}
    </section>
  );
}
