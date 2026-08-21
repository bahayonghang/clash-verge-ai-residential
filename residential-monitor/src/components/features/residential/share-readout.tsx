import { Gauge, Layers } from "lucide-react";
import type { ResidentialShare } from "../../../dto";
import { formatBytes } from "../../../format/units";
import { shareReadout } from "../../../hooks/use-residential-share";
import { t, type UiLocale } from "../../../i18n";
import { StatCard } from "../../common/stat-card";

function formatPercent(value: number | null, unknown: string): string {
  if (value === null) {
    return unknown;
  }
  return `${value.toFixed(1)}%`;
}

export function ShareReadout({
  locale,
  share,
  loading,
  errorZh
}: {
  locale: UiLocale;
  share: ResidentialShare | null;
  loading: boolean;
  errorZh: string | null;
}) {
  const unknown = t(locale, "common.unknown");
  const readout = shareReadout(share);
  const percentUnavailable = readout.kind === "unknown" || readout.kind === "zero-denominator";
  let note = "";
  if (readout.kind === "unknown") {
    note = t(locale, "residential.share.uncovered");
  } else if (readout.kind === "zero-denominator") {
    note = t(locale, "residential.share.zero_den");
  } else if (readout.kind === "zero-residential") {
    note = t(locale, "residential.share.zero_traffic");
  }
  return (
    <section className="space-y-3" aria-labelledby="residential-share-title">
      <div>
        <h2 id="residential-share-title" className="text-sm font-semibold">
          {t(locale, "residential.share")}
        </h2>
        <p className="text-xs text-muted-foreground">{t(locale, "residential.share.denominator")}</p>
      </div>
      {errorZh ? (
        <p className="text-sm text-destructive" role="alert">
          {errorZh}
        </p>
      ) : null}
      <div className="grid gap-2.5 sm:grid-cols-2 lg:grid-cols-3">
        <StatCard
          icon={<Gauge />}
          label={t(locale, "residential.share.ratio")}
          value={percentUnavailable ? unknown : formatPercent(readout.percent, unknown)}
          subvalue={t(locale, "residential.share.denominator")}
          color="#3B82F6"
          loading={loading && share === null}
        />
        <StatCard
          icon={<Layers />}
          label={t(locale, "residential.share.residential")}
          value={formatBytes(readout.residential, unknown)}
          color="#8B5CF6"
          loading={loading && share === null}
        />
        <StatCard
          icon={<Layers />}
          label={t(locale, "residential.share.attributed")}
          value={formatBytes(readout.attributed, unknown)}
          color="#06B6D4"
          loading={loading && share === null}
        />
      </div>
      {note ? (
        <p className="text-xs text-muted-foreground" data-share-kind={readout.kind} role="status">
          {note}
        </p>
      ) : null}
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
            <tr className="border-b border-border/40">
              <td className="py-2">{t(locale, "residential.share.residential")}</td>
              <td className="py-2 tabular-nums">{formatBytes(share?.residentialUpload ?? null, unknown)}</td>
              <td className="py-2 tabular-nums">{formatBytes(share?.residentialDownload ?? null, unknown)}</td>
            </tr>
            <tr>
              <td className="py-2">{t(locale, "residential.share.attributed")}</td>
              <td className="py-2 tabular-nums">{formatBytes(share?.attributedUpload ?? null, unknown)}</td>
              <td className="py-2 tabular-nums">{formatBytes(share?.attributedDownload ?? null, unknown)}</td>
            </tr>
          </tbody>
        </table>
      </div>
      {share?.namedSql.length ? (
        <p className="text-xs text-muted-foreground">
          {t(locale, "report.named_sql")} {share.namedSql.join(", ")}
        </p>
      ) : null}
    </section>
  );
}
