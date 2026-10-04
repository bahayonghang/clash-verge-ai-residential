import type { ReactNode } from "react";
import { inspectKeysMatch, rankingInspectKey } from "../../../format/report-inspect";
import { formatSharePct, type ShareModel } from "../../../format/report-view";
import { formatBytes } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { cn } from "../../../lib/utils";
import {
  DEFAULT_RANK_SORT,
  nextRankSort,
  rankSortAria,
  type RankSortField,
  type RankSortSpec
} from "../../../rank-sort";
import { SortableTh } from "../../common/sortable-th";
import { useReportInspect } from "./inspect-context";

export function RankingTable({
  locale,
  share,
  sort = DEFAULT_RANK_SORT,
  onSortChange
}: {
  locale: UiLocale;
  share: ShareModel | null;
  sort?: RankSortSpec;
  onSortChange?: (next: RankSortSpec) => void;
}) {
  const inspect = useReportInspect();
  const unknown = t(locale, "common.unknown");
  const rows = share?.rows ?? [];

  const head = (id: RankSortField, label: string, numeric: boolean): ReactNode => (
    <SortableTh
      label={label}
      ariaSort={rankSortAria(id, sort)}
      onClick={() => onSortChange?.(nextRankSort(id, sort))}
      numeric={numeric}
      className="px-2"
    />
  );

  return (
    <section className="space-y-2">
      <h3 className="text-sm font-semibold uppercase tracking-wider">{t(locale, "report.topn")}</h3>
      <div className="overflow-auto rounded-md border">
        <table className="w-full text-sm">
          <thead className="bg-muted/40">
            <tr>
              {head("name", t(locale, "report.col.name"), false)}
              {head("upload", t(locale, "report.col.upload"), true)}
              {head("download", t(locale, "report.col.download"), true)}
              <th className="px-2 py-2 text-left font-semibold">{t(locale, "report.col.share")}</th>
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
                    <td className="px-2 py-1.5 tabular-nums">{formatSharePct(row.share, unknown)}</td>
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
