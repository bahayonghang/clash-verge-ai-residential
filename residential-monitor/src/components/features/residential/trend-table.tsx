import type { ReportResult } from "../../../dto";
import { formatBytes, formatUtc } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { newestFirstSeries } from "./aggregate-model";

export function TrendTable({
  locale,
  series,
  loading
}: {
  locale: UiLocale;
  series: ReportResult["series"];
  loading: boolean;
}) {
  const unknown = t(locale, "common.unknown");
  const rows = newestFirstSeries(series);
  return (
    <div className="mt-3 max-h-56 overflow-auto rounded-md border border-border/40">
      <table className="w-full min-w-[36rem] border-collapse text-sm">
        <thead className="sticky top-0 z-10 bg-card">
          <tr className="border-b border-border text-muted-foreground">
            <th scope="col" className="whitespace-nowrap px-3 py-2 text-left font-medium">
              {t(locale, "report.col.time")}
            </th>
            <th scope="col" className="px-3 py-2 text-right font-medium tabular-nums">
              {t(locale, "report.col.upload")}
            </th>
            <th scope="col" className="px-3 py-2 text-right font-medium tabular-nums">
              {t(locale, "report.col.download")}
            </th>
          </tr>
        </thead>
        <tbody>
          {rows.length === 0 ? (
            <tr>
              <td className="px-3 py-3 text-muted-foreground" colSpan={3}>
                {loading ? t(locale, "report.running") : t(locale, "chart.empty")}
              </td>
            </tr>
          ) : (
            rows.map((point) => (
              <tr
                key={point.bucketUtc}
                data-bucket-utc={point.bucketUtc}
                className="border-b border-border/40 transition-colors last:border-0 hover:bg-muted/40"
              >
                <td className="whitespace-nowrap px-3 py-2">{formatUtc(point.bucketUtc)}</td>
                <td className="px-3 py-2 text-right tabular-nums">
                  {formatBytes(point.upload, unknown)}
                </td>
                <td className="px-3 py-2 text-right tabular-nums">
                  {formatBytes(point.download, unknown)}
                </td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}
