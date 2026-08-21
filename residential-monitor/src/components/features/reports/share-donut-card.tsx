import { inspectKeysMatch, rankingInspectKey } from "../../../format/report-inspect";
import { formatSharePct, type ShareModel } from "../../../format/report-view";
import { formatBytes } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { cn } from "../../../lib/utils";
import { ShareDonut } from "../../charts/share-donut";
import { useReportInspect } from "./inspect-context";

export function ShareDonutCard({ locale, share }: { locale: UiLocale; share: ShareModel | null }) {
  const inspect = useReportInspect();
  const unknown = t(locale, "common.unknown");
  const rows = share?.rows ?? [];
  return (
    <section className="space-y-2">
      <h3 className="text-sm font-semibold uppercase tracking-wider">{t(locale, "report.pie")}</h3>
      {share && !share.drawPie && !share.capabilityUnsupported ? (
        <p className="text-sm text-muted-foreground">{t(locale, "report.pie.unavailable")}</p>
      ) : null}
      <ShareDonut
        data={rows.map((row) => ({
          label: row.label,
          value: row.download,
          inspectKey: rankingInspectKey(row),
          remainder: row.kind === "remainder"
        }))}
        loading={false}
        emptyHint={t(locale, "report.empty_cap")}
        activeKey={inspect.activeKey}
        onHover={inspect.setHover}
        onSelect={inspect.togglePinned}
      />
      <div className="max-h-56 overflow-auto rounded-md border">
        <table className="w-full text-sm">
          <thead className="bg-muted/40">
            <tr>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "report.col.name")}</th>
              <th className="px-2 py-1.5 text-right font-medium">{t(locale, "report.col.upload")}</th>
              <th className="px-2 py-1.5 text-right font-medium">{t(locale, "report.col.download")}</th>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "report.col.share")}</th>
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 ? (
              <tr>
                <td className="px-2 py-2 text-muted-foreground" colSpan={4}>
                  {t(locale, share?.capabilityUnsupported ? "report.empty_cap" : "report.empty")}
                </td>
              </tr>
            ) : (
              rows.map((row) => {
                const key = rankingInspectKey(row);
                const active = Boolean(inspect.activeKey && inspectKeysMatch(inspect.activeKey, key));
                const width = row.share === null ? 0 : Math.max(0, Math.round(row.share * 100));
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
                    <td className="px-2 py-1.5">{row.label}</td>
                    <td className="px-2 py-1.5 text-right tabular-nums">
                      {row.upload === null ? t(locale, "report.dash") : formatBytes(row.upload, unknown)}
                    </td>
                    <td className="px-2 py-1.5 text-right tabular-nums">
                      {formatBytes(row.download, unknown)}
                    </td>
                    <td className="px-2 py-1.5">
                      <span className="mr-2 tabular-nums">{formatSharePct(row.share, unknown)}</span>
                      {row.share === null ? null : (
                        <span className="inline-block h-1.5 w-16 overflow-hidden rounded-full bg-muted">
                          <span className="block h-full bg-primary" style={{ width: `${width}%` }} />
                        </span>
                      )}
                    </td>
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
