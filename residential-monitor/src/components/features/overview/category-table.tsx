import type { LiveOverview } from "../../../dto";
import { categoryRows } from "../../../format/overview";
import { formatBytes } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { OverviewCard } from "../../common/overview-card";

export function CategoryTable({ locale, overview }: { locale: UiLocale; overview: LiveOverview }) {
  const unknown = t(locale, "common.unknown");
  const rows = categoryRows(overview.categoryUpload, overview.categoryDownload);
  return (
    <OverviewCard title={t(locale, "overview.categories")} icon={null}>
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-border/60 text-left text-muted-foreground">
              <th className="py-2 font-medium">{t(locale, "overview.col.name")}</th>
              <th className="py-2 font-medium">{t(locale, "overview.col.upload")}</th>
              <th className="py-2 font-medium">{t(locale, "overview.col.download")}</th>
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 ? (
              <tr>
                <td className="py-3 text-muted-foreground" colSpan={3}>
                  {t(locale, "common.none")}
                </td>
              </tr>
            ) : (
              rows.map((row) => (
                <tr key={row.name} className="border-b border-border/40 last:border-0">
                  <td className="py-2">{row.name}</td>
                  <td className="py-2 tabular-nums">{formatBytes(row.upload, unknown)}</td>
                  <td className="py-2 tabular-nums">{formatBytes(row.download, unknown)}</td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </OverviewCard>
  );
}
