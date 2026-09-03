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
        <h3 id="residential-share-title" className="text-sm font-semibold">
          {t(locale, "residential.share")}
        </h3>
        <div className="mt-1 flex flex-wrap gap-x-4 gap-y-1">
          <p className="text-xs text-muted-foreground/80">
            {t(locale, "residential.share.denominator")}
          </p>
          {share?.namedSql.length ? (
            <p className="text-xs text-muted-foreground/80">
              {t(locale, "report.named_sql")} {share.namedSql.join(", ")}
            </p>
          ) : null}
        </div>
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
          colorToken={1}
          loading={loading && share === null}
        />
        <StatCard
          icon={<Layers />}
          label={t(locale, "residential.share.residential")}
          value={formatBytes(readout.residential, unknown)}
          colorToken={2}
          loading={loading && share === null}
        />
        <StatCard
          icon={<Layers />}
          label={t(locale, "residential.share.attributed")}
          value={formatBytes(readout.attributed, unknown)}
          colorToken={3}
          loading={loading && share === null}
        />
      </div>
      {note ? (
        <p className="text-xs text-muted-foreground" data-share-kind={readout.kind} role="status">
          {note}
        </p>
      ) : null}
      <ul className="space-y-1.5">
        {[
          {
            name: t(locale, "residential.share.residential"),
            upload: share?.residentialUpload ?? null,
            download: share?.residentialDownload ?? null
          },
          {
            name: t(locale, "residential.share.attributed"),
            upload: share?.attributedUpload ?? null,
            download: share?.attributedDownload ?? null
          }
        ].map((row) => (
          <li
            key={row.name}
            className="flex flex-wrap items-baseline gap-x-6 gap-y-0.5 text-sm"
          >
            <span className="font-medium">{row.name}</span>
            <span className="text-muted-foreground">
              {t(locale, "overview.col.upload")}{" "}
              <span className="tabular-nums text-foreground">
                {formatBytes(row.upload, unknown)}
              </span>
            </span>
            <span className="text-muted-foreground">
              {t(locale, "overview.col.download")}{" "}
              <span className="tabular-nums text-foreground">
                {formatBytes(row.download, unknown)}
              </span>
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}
