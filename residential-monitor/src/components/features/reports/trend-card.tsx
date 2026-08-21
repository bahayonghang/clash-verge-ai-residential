import type { ReportResult } from "../../../dto";
import { inspectKeysMatch, trendInspectKey } from "../../../format/report-inspect";
import { formatBytes, formatUtc } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { cn } from "../../../lib/utils";
import { TrendArea } from "../../charts/trend-area";
import { useReportInspect } from "./inspect-context";

export function TrendCard({
  locale,
  series,
  loading
}: {
  locale: UiLocale;
  series: ReportResult["series"];
  loading: boolean;
}) {
  const inspect = useReportInspect();
  const unknown = t(locale, "common.unknown");
  const points = series.map((point) => ({
    bucketUtc: point.bucketUtc,
    upload: point.upload,
    download: point.download,
    inspectKey: trendInspectKey(point.bucketUtc)
  }));
  return (
    <section className="space-y-2" aria-label={t(locale, "report.chart_table")}>
      <h3 className="text-sm font-semibold uppercase tracking-wider">{t(locale, "report.trend")}</h3>
      <TrendArea
        data={points}
        loading={loading}
        emptyHint={t(locale, "report.empty")}
        locale={locale}
        activeKey={inspect.activeKey}
        onHover={inspect.setHover}
        onSelect={inspect.togglePinned}
      />
      <div className="max-h-56 overflow-auto rounded-md border">
        <table className="w-full text-sm">
          <thead className="bg-muted/40">
            <tr>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "report.col.time")}</th>
              <th className="px-2 py-1.5 text-right font-medium">{t(locale, "report.col.upload")}</th>
              <th className="px-2 py-1.5 text-right font-medium">{t(locale, "report.col.download")}</th>
            </tr>
          </thead>
          <tbody>
            {series.length === 0 ? (
              <tr>
                <td className="px-2 py-2 text-muted-foreground" colSpan={3}>
                  {t(locale, "report.empty")}
                </td>
              </tr>
            ) : (
              series.map((point) => {
                const key = trendInspectKey(point.bucketUtc);
                const active = Boolean(inspect.activeKey && inspectKeysMatch(inspect.activeKey, key));
                return (
                  <tr
                    key={key}
                    tabIndex={0}
                    className={cn("cursor-pointer hover:bg-muted/40", active && "bg-primary/15")}
                    onMouseEnter={() => inspect.setHover(key)}
                    onMouseLeave={() => inspect.setHover(null)}
                    onClick={() => inspect.togglePinned(key)}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        inspect.togglePinned(key);
                      }
                    }}
                  >
                    <td className="px-2 py-1.5">{formatUtc(point.bucketUtc)}</td>
                    <td className="px-2 py-1.5 text-right tabular-nums">{formatBytes(point.upload, unknown)}</td>
                    <td className="px-2 py-1.5 text-right tabular-nums">{formatBytes(point.download, unknown)}</td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}
