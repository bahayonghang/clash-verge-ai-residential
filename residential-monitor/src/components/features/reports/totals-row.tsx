import type { ReportResult } from "../../../dto";
import { formatSharePct } from "../../../format/report-view";
import { formatBytes } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";

function deltaPct(current: number, previous: number | null, unknown: string): string {
  if (previous === null) {
    return unknown;
  }
  if (previous === 0) {
    return current === 0 ? "0%" : unknown;
  }
  return formatSharePct((current - previous) / previous, unknown);
}

export function TotalsRow({ locale, report }: { locale: UiLocale; report: ReportResult | null }) {
  const unknown = t(locale, "common.unknown");
  if (!report) {
    return <p className="text-sm text-muted-foreground">{t(locale, "report.none")}</p>;
  }
  const upShare = formatSharePct(
    report.totals.upload + report.totals.download > 0
      ? report.totals.download / (report.totals.upload + report.totals.download)
      : null,
    unknown
  );
  return (
    <section aria-label={t(locale, "report.numbers")} className="space-y-2">
      <h2 className="text-sm font-semibold uppercase tracking-wider">{t(locale, "report.totals")}</h2>
      <dl className="grid gap-3 sm:grid-cols-3">
        <div className="rounded-xl border bg-card p-3">
          <dt className="text-xs uppercase tracking-wider text-muted-foreground">
            {t(locale, "report.metric.upload")}
          </dt>
          <dd className="font-mono text-lg tabular-nums">{formatBytes(report.totals.upload, unknown)}</dd>
          <p className="text-xs text-muted-foreground">
            {t(locale, "report.comparison_line")} {report.totals.previousUpload == null
              ? unknown
              : formatBytes(report.totals.previousUpload, unknown)}{" "}
            ({deltaPct(report.totals.upload, report.totals.previousUpload, unknown)})
          </p>
        </div>
        <div className="rounded-xl border bg-card p-3">
          <dt className="text-xs uppercase tracking-wider text-muted-foreground">
            {t(locale, "report.metric.download")}
          </dt>
          <dd className="font-mono text-lg tabular-nums">{formatBytes(report.totals.download, unknown)}</dd>
          <p className="text-xs text-muted-foreground">
            {t(locale, "report.comparison_line")} {report.totals.previousDownload == null
              ? unknown
              : formatBytes(report.totals.previousDownload, unknown)}{" "}
            ({deltaPct(report.totals.download, report.totals.previousDownload, unknown)})
          </p>
        </div>
        <div className="rounded-xl border bg-card p-3">
          <dt className="text-xs uppercase tracking-wider text-muted-foreground">
            {t(locale, "report.metric.connections")}
          </dt>
          <dd className="font-mono text-lg tabular-nums">{report.totals.connectionCount}</dd>
          <p className="text-xs text-muted-foreground">
            {t(locale, "report.col.share")} {upShare}
          </p>
        </div>
      </dl>
    </section>
  );
}
