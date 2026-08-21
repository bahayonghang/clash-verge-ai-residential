import { useEffect, useMemo, useState } from "react";
import type { ReportResult } from "../../../dto";
import {
  formatRankLabel,
  isUnknownIdentity,
  rankDisplayLabel,
  rankingShare,
  type DimensionKind
} from "../../../format/rank";
import { formatBytes } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { cn } from "../../../lib/utils";
import { Button } from "../../ui/button";
import { CapabilityNote, resolvedCapabilityNote } from "./capability-note";

type TableSort = "name" | "upload" | "download" | "connections";

const PAGE_SIZE = 20;

function ariaSort(
  column: TableSort,
  sort: TableSort,
  descending: boolean
): "ascending" | "descending" | "none" {
  if (column !== sort) {
    return "none";
  }
  return descending ? "descending" : "ascending";
}

export function RankTable({
  locale,
  kind,
  result,
  loading,
  errorZh,
  selectedIdentity,
  onSelect
}: {
  locale: UiLocale;
  kind: DimensionKind;
  result: ReportResult | null;
  loading: boolean;
  errorZh: string | null;
  selectedIdentity: string | null;
  onSelect: (identity: string, label: string) => void;
}) {
  const unknown = t(locale, "common.unknown");
  const [sort, setSort] = useState<TableSort>("download");
  const [descending, setDescending] = useState(true);
  const [page, setPage] = useState(0);
  const exactTopN = result?.drilldownCapability.exactTopN !== false;
  const crossDimension = result?.drilldownCapability.crossDimension === true;
  const totals = result?.totals;
  const sorted = useMemo(() => {
    const rows = [...(result?.rankings ?? [])];
    rows.sort((left, right) => {
      const dir = descending ? -1 : 1;
      if (sort === "name") {
        return dir * rankDisplayLabel(left.identity, left.label, unknown).localeCompare(
          rankDisplayLabel(right.identity, right.label, unknown),
          locale
        );
      }
      const leftValue =
        sort === "upload" ? left.upload : sort === "connections" ? left.connectionCount : left.download;
      const rightValue =
        sort === "upload" ? right.upload : sort === "connections" ? right.connectionCount : right.download;
      return dir * (leftValue - rightValue);
    });
    return rows;
  }, [descending, locale, result?.rankings, sort, unknown]);

  const pageCount = Math.max(1, Math.ceil(sorted.length / PAGE_SIZE));
  const safePage = Math.min(page, pageCount - 1);
  const visible = sorted.slice(safePage * PAGE_SIZE, (safePage + 1) * PAGE_SIZE);

  useEffect(() => {
    setPage(0);
  }, [result?.reportSnapshotToken, sort, descending]);

  function toggleSort(column: TableSort): void {
    if (sort === column) {
      setDescending((current) => !current);
      return;
    }
    setSort(column);
    setDescending(column !== "name");
    setPage(0);
  }

  if (!exactTopN) {
    return (
      <CapabilityNote
        locale={locale}
        noteZh={resolvedCapabilityNote(locale, result?.drilldownCapability.noteZh, "dimension.exact_top_n_off")}
      />
    );
  }

  return (
    <div className="space-y-3">
      {errorZh && !result ? <CapabilityNote locale={locale} noteZh={errorZh} /> : null}
      <div className="overflow-x-auto">
        <table className="w-full text-sm" aria-busy={loading}>
          <thead>
            <tr className="border-b border-border/60 text-left text-muted-foreground">
              <th className="py-2 font-medium">{t(locale, "dimension.col.rank")}</th>
              <th className="py-2 font-medium" aria-sort={ariaSort("name", sort, descending)}>
                <button type="button" className="hover:text-foreground" onClick={() => toggleSort("name")}>
                  {t(locale, "overview.col.name")}
                </button>
              </th>
              <th className="py-2 font-medium" aria-sort={ariaSort("upload", sort, descending)}>
                <button type="button" className="hover:text-foreground" onClick={() => toggleSort("upload")}>
                  {t(locale, "overview.col.upload")}
                </button>
              </th>
              <th className="py-2 font-medium" aria-sort={ariaSort("download", sort, descending)}>
                <button type="button" className="hover:text-foreground" onClick={() => toggleSort("download")}>
                  {t(locale, "overview.col.download")}
                </button>
              </th>
              <th className="py-2 font-medium" aria-sort={ariaSort("connections", sort, descending)}>
                <button type="button" className="hover:text-foreground" onClick={() => toggleSort("connections")}>
                  {t(locale, "report.metric.connections")}
                </button>
              </th>
              <th className="py-2 font-medium">{t(locale, "report.col.share")}</th>
              {crossDimension ? (
                <th className="py-2 font-medium">{t(locale, "dimension.drilldown")}</th>
              ) : null}
            </tr>
          </thead>
          <tbody>
            {visible.length === 0 ? (
              <tr>
                <td className="py-4 text-muted-foreground" colSpan={crossDimension ? 7 : 6}>
                  {loading ? t(locale, "report.running") : t(locale, "dimension.empty")}
                </td>
              </tr>
            ) : (
              visible.map((row, index) => {
                const label = formatRankLabel(row.identity, row.label, unknown);
                const unknownRow = isUnknownIdentity(row.identity);
                const share = rankingShare(row.download, totals?.download ?? 0);
                const canDrill = crossDimension && (!unknownRow || kind === "host");
                return (
                  <tr
                    key={row.identity}
                    data-identity={row.identity}
                    data-unknown={unknownRow ? "1" : "0"}
                    data-kind={kind}
                    className={cn(
                      "border-b border-border/40 last:border-0",
                      selectedIdentity === row.identity ? "bg-muted/40" : undefined
                    )}
                  >
                    <td className="py-2 tabular-nums">{safePage * PAGE_SIZE + index + 1}</td>
                    <td className="py-2">{label}</td>
                    <td className="py-2 tabular-nums">{formatBytes(row.upload, unknown)}</td>
                    <td className="py-2 tabular-nums">{formatBytes(row.download, unknown)}</td>
                    <td className="py-2 tabular-nums">{row.connectionCount}</td>
                    <td className="py-2 tabular-nums">
                      {totals && totals.download > 0 ? `${(share * 100).toFixed(1)}%` : unknown}
                    </td>
                    {crossDimension ? (
                      <td className="py-2">
                        {canDrill ? (
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            data-drill="1"
                            className="h-7 px-2"
                            onClick={() => onSelect(row.identity, label)}
                          >
                            {t(locale, "dimension.drilldown")}
                          </Button>
                        ) : (
                          <span data-drill="0" className="text-xs text-muted-foreground">
                            {t(locale, "dimension.no_drill_unknown")}
                          </span>
                        )}
                      </td>
                    ) : null}
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>
      {sorted.length > PAGE_SIZE ? (
        <div className="flex items-center justify-end gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={safePage === 0}
            onClick={() => setPage(Math.max(0, safePage - 1))}
          >
            {t(locale, "dimension.prev")}
          </Button>
          <span className="text-xs text-muted-foreground">
            {safePage + 1} / {pageCount}
          </span>
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={safePage >= pageCount - 1}
            onClick={() => setPage(safePage + 1)}
          >
            {t(locale, "dimension.next")}
          </Button>
        </div>
      ) : null}
    </div>
  );
}
