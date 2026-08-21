import type { ReportArchivePage } from "../../../dto";
import type { ArchiveKindFilter } from "../../../format/report-view";
import { formatBytes, formatUtc } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { cn } from "../../../lib/utils";
import { fieldClass, labelClass } from "../form-styles";

function kindLabel(locale: UiLocale, kind: string): string {
  if (kind === "day") {
    return t(locale, "report.archive.kind.day");
  }
  if (kind === "manual") {
    return t(locale, "report.archive.kind.saved");
  }
  return t(locale, "report.archive.kind.hour");
}

function statusLabel(locale: UiLocale, status: string): string {
  return status === "ok" ? t(locale, "report.archive.status.ok") : t(locale, "report.archive.status.failed");
}

export function ArchiveList({
  locale,
  archives,
  filter,
  selectedId,
  onFilter,
  onSelect
}: {
  locale: UiLocale;
  archives: ReportArchivePage | null;
  filter: ArchiveKindFilter;
  selectedId: string | null;
  onFilter: (filter: ArchiveKindFilter) => void;
  onSelect: (archiveId: string) => void;
}) {
  const unknown = t(locale, "common.unknown");
  const items = archives?.items ?? [];
  return (
    <section className="space-y-3">
      <h2 className="text-sm font-semibold uppercase tracking-wider">{t(locale, "report.archive.list")}</h2>
      <p className="text-sm text-muted-foreground">{t(locale, "report.archive.failed_retry")}</p>
      <label className={labelClass}>
        {t(locale, "report.archive.filter")}
        <select
          className={cn(fieldClass, "max-w-48")}
          value={filter}
          onChange={(event) => onFilter(event.target.value as ArchiveKindFilter)}
        >
          <option value="all">{t(locale, "report.archive.filter.all")}</option>
          <option value="day">{t(locale, "report.archive.kind.day")}</option>
          <option value="hour">{t(locale, "report.archive.kind.hour")}</option>
          <option value="manual">{t(locale, "report.archive.kind.saved")}</option>
        </select>
      </label>
      <div className="max-h-56 overflow-auto rounded-md border">
        <table className="w-full text-sm">
          <thead className="bg-muted/40">
            <tr>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "report.archive.col.time")}</th>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "report.archive.col.kind")}</th>
              <th className="px-2 py-1.5 text-right font-medium">{t(locale, "report.archive.col.download")}</th>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "report.archive.col.status")}</th>
            </tr>
          </thead>
          <tbody>
            {items.length === 0 ? (
              <tr>
                <td className="px-2 py-2 text-muted-foreground" colSpan={4}>
                  {t(locale, "report.archive.empty")}
                </td>
              </tr>
            ) : (
              items.map((item) => (
                <tr
                  key={item.archiveId}
                  tabIndex={0}
                  aria-current={item.archiveId === selectedId ? "true" : undefined}
                  className={cn(
                    "cursor-pointer hover:bg-muted/40",
                    item.archiveId === selectedId && "bg-primary/10"
                  )}
                  onClick={() => onSelect(item.archiveId)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      onSelect(item.archiveId);
                    }
                  }}
                >
                  <td className="px-2 py-1.5">{formatUtc(item.rangeStartUtc)}</td>
                  <td className="px-2 py-1.5">{kindLabel(locale, item.kind)}</td>
                  <td className="px-2 py-1.5 text-right tabular-nums">
                    {item.totalsDownload == null ? unknown : formatBytes(item.totalsDownload, unknown)}
                  </td>
                  <td className="px-2 py-1.5">{statusLabel(locale, item.status)}</td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}
