import type { ReportResult } from "../../../dto";
import { formatUtc } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { formatTemplate } from "../../../lib/utils";

export function CoveragePanel({ locale, report }: { locale: UiLocale; report: ReportResult | null }) {
  if (!report) {
    return null;
  }
  return (
    <section className="space-y-2">
      <h3 className="text-sm font-semibold uppercase tracking-wider">{t(locale, "report.coverage_title")}</h3>
      <p className="text-sm">
        {formatTemplate(t(locale, "report.coverage"), {
          status: report.coverage.status,
          gap: report.coverage.gapSec,
          unit: report.unit
        })}
      </p>
      <p className="text-sm text-muted-foreground">
        {t(locale, "report.coverage.covered")} {report.coverage.coveredSec}
      </p>
      <div className="overflow-auto rounded-md border">
        <table className="w-full text-sm">
          <thead className="bg-muted/40">
            <tr>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "report.coverage.slice")}</th>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "alerts.col.coverage")}</th>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "report.col.time")}</th>
            </tr>
          </thead>
          <tbody>
            {report.coverage.slices.length === 0 ? (
              <tr>
                <td className="px-2 py-2 text-muted-foreground" colSpan={3}>
                  {t(locale, "common.none")}
                </td>
              </tr>
            ) : (
              report.coverage.slices.map((slice, index) => (
                <tr key={`${slice.kind}-${slice.startedUtc}-${index}`}>
                  <td className="px-2 py-1.5">{slice.kind}</td>
                  <td className="px-2 py-1.5">{slice.reason}</td>
                  <td className="px-2 py-1.5">
                    {formatUtc(slice.startedUtc)} → {slice.endedUtc == null ? t(locale, "report.dash") : formatUtc(slice.endedUtc)}
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}
