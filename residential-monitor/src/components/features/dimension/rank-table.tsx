import { useEffect, useMemo, useRef, useState } from "react";
import type { ReportResult } from "../../../dto";
import {
  columnWidth,
  DRILL_COL_WIDTH,
  isNumericRankColumn,
  RANK_COL_WIDTH,
  rankTablePixelWidth,
  setRankColumnWidth,
  visibleRankDataColumns,
  WIDTH_MAX,
  WIDTH_MIN,
  type RankDataColumnId
} from "../../../dimension-rank-table-layout";
import {
  formatRankLabel,
  isUnknownIdentity,
  missingDimensionLabel,
  rankDisplayLabel,
  rankingShare,
  type DimensionKind
} from "../../../format/rank";
import { formatBytes } from "../../../format/units";
import { useDimensionRankTableLayout } from "../../../hooks/use-dimension-rank-table-layout";
import { t, type UiLocale } from "../../../i18n";
import { cn } from "../../../lib/utils";
import { ColResizer } from "../../common/col-resizer";
import { dataTableClasses, DataTableEmptyRow, DataTableTd, DataTableTh } from "../../common/data-table";
import { SortableTh } from "../../common/sortable-th";
import { Button } from "../../ui/button";
import { CapabilityNote, resolvedCapabilityNote } from "./capability-note";

type TableSort = "name" | "upload" | "download" | "connections";

const PAGE_SIZE = 20;

const COLUMN_LABEL: Record<RankDataColumnId, string> = {
  name: "overview.col.name",
  upload: "overview.col.upload",
  download: "overview.col.download",
  connections: "report.metric.connections",
  share: "report.col.share",
  attribution: "dimension.col.attribution"
};

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

function attributionText(
  primaryExit: string | null,
  exitMixed: boolean,
  unknown: string,
  mixedLabel: string
): string {
  if (primaryExit == null) {
    return unknown;
  }
  return exitMixed ? `${primaryExit} · ${mixedLabel}` : primaryExit;
}

export function RankTable({
  locale,
  kind,
  result,
  loading,
  errorZh,
  selectedIdentity,
  onSelect,
  layoutSeed
}: {
  locale: UiLocale;
  kind: DimensionKind;
  result: ReportResult | null;
  loading: boolean;
  errorZh: string | null;
  selectedIdentity: string | null;
  onSelect: (identity: string, label: string) => void;
  layoutSeed?: unknown;
}) {
  const unknown = t(locale, "common.unknown");
  const mixedLabel = t(locale, "dimension.exit_mixed");
  const missing = missingDimensionLabel(locale, kind);
  const [sort, setSort] = useState<TableSort>("download");
  const [descending, setDescending] = useState(true);
  const [page, setPage] = useState(0);
  const { layout, commitLayout, errorZh: layoutErrorZh } = useDimensionRankTableLayout(
    layoutSeed,
    locale
  );
  const tableRef = useRef<HTMLTableElement | null>(null);
  const colRefs = useRef<Partial<Record<RankDataColumnId, HTMLTableColElement | null>>>({});
  const exactTopN = result?.drilldownCapability.exactTopN !== false;
  const crossDimension = result?.drilldownCapability.crossDimension === true;
  const showAttribution = kind !== "chain";
  const showDrill = crossDimension;
  const dataColumns = visibleRankDataColumns(showAttribution);
  const tableWidth = rankTablePixelWidth(layout, {
    attribution: showAttribution,
    drill: showDrill
  });
  const totals = result?.totals;
  const sorted = useMemo(() => {
    const rows = [...(result?.rankings ?? [])];
    rows.sort((left, right) => {
      const dir = descending ? -1 : 1;
      if (sort === "name") {
        return (
          dir *
          rankDisplayLabel(left.identity, left.label, missing).localeCompare(
            rankDisplayLabel(right.identity, right.label, missing),
            locale
          )
        );
      }
      const leftValue =
        sort === "upload" ? left.upload : sort === "connections" ? left.connectionCount : left.download;
      const rightValue =
        sort === "upload" ? right.upload : sort === "connections" ? right.connectionCount : right.download;
      return dir * (leftValue - rightValue);
    });
    return rows;
  }, [descending, locale, missing, result?.rankings, sort]);

  const pageCount = Math.max(1, Math.ceil(sorted.length / PAGE_SIZE));
  const safePage = Math.min(page, pageCount - 1);
  const visible = sorted.slice(safePage * PAGE_SIZE, (safePage + 1) * PAGE_SIZE);
  const colSpan = 6 + (showAttribution ? 1 : 0) + (showDrill ? 1 : 0);

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

  function renderResizer(column: RankDataColumnId, label: string) {
    const extra = tableWidth - columnWidth(layout, column);
    return (
      <ColResizer
        id={column}
        width={columnWidth(layout, column)}
        min={WIDTH_MIN}
        max={WIDTH_MAX}
        label={label}
        locale={locale}
        tableRef={tableRef}
        colRef={{
          get current() {
            return colRefs.current[column] ?? null;
          }
        }}
        onDraft={(nextWidth) => {
          if (tableRef.current) {
            const px = `${extra + nextWidth}px`;
            tableRef.current.style.width = px;
            tableRef.current.style.minWidth = px;
          }
        }}
        onCommit={(nextWidth) => {
          commitLayout(setRankColumnWidth(layout, column, nextWidth));
        }}
      />
    );
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
      {layoutErrorZh ? <CapabilityNote locale={locale} noteZh={layoutErrorZh} /> : null}
      <div className={dataTableClasses.wrapper}>
        <table
          ref={tableRef}
          className={cn(dataTableClasses.table, "table-fixed max-w-none")}
          style={{ width: tableWidth, minWidth: tableWidth }}
          aria-busy={loading}
        >
          <colgroup>
            <col data-col="rank" style={{ width: RANK_COL_WIDTH }} />
            {dataColumns.map((column) => (
              <col
                key={column}
                data-col={column}
                ref={(node) => {
                  colRefs.current[column] = node;
                }}
                style={{ width: columnWidth(layout, column) }}
              />
            ))}
            {showDrill ? <col data-col="drill" style={{ width: DRILL_COL_WIDTH }} /> : null}
          </colgroup>
          <thead className="bg-muted/40 text-foreground">
            <tr className={dataTableClasses.headRow}>
              <DataTableTh>{t(locale, "dimension.col.rank")}</DataTableTh>
              {dataColumns.map((column) => {
                const label = t(locale, COLUMN_LABEL[column]);
                const numeric = isNumericRankColumn(column);
                if (column === "share" || column === "attribution") {
                  return (
                    <DataTableTh
                      key={column}
                      numeric={numeric}
                      className="relative overflow-hidden"
                      data-col={column}
                    >
                      {label}
                      {renderResizer(column, label)}
                    </DataTableTh>
                  );
                }
                return (
                  <SortableTh
                    key={column}
                    label={label}
                    ariaSort={ariaSort(column, sort, descending)}
                    onClick={() => toggleSort(column)}
                    numeric={numeric}
                    subtle
                    className={cn(
                      numeric ? dataTableClasses.thNumeric : dataTableClasses.th,
                      "relative overflow-hidden"
                    )}
                  >
                    {renderResizer(column, label)}
                  </SortableTh>
                );
              })}
              {showDrill ? <DataTableTh>{t(locale, "dimension.drilldown")}</DataTableTh> : null}
            </tr>
          </thead>
          <tbody>
            {visible.length === 0 ? (
              <DataTableEmptyRow colSpan={colSpan}>
                {loading ? t(locale, "report.running") : t(locale, "dimension.empty")}
              </DataTableEmptyRow>
            ) : (
              visible.map((row, index) => {
                const label = formatRankLabel(row.identity, row.label, unknown, missing);
                const unknownRow = isUnknownIdentity(row.identity);
                const share = rankingShare(row.download, totals?.download ?? 0);
                const canDrill =
                  crossDimension && (!unknownRow || kind === "host" || kind === "process");
                const exitText = attributionText(
                  row.primaryExit,
                  row.exitMixed,
                  unknown,
                  mixedLabel
                );
                return (
                  <tr
                    key={row.identity}
                    data-identity={row.identity}
                    data-unknown={unknownRow ? "1" : "0"}
                    data-kind={kind}
                    className={cn(
                      dataTableClasses.row,
                      selectedIdentity === row.identity ? "bg-muted/40" : undefined
                    )}
                  >
                    <DataTableTd className="tabular-nums">{safePage * PAGE_SIZE + index + 1}</DataTableTd>
                    <DataTableTd>{label}</DataTableTd>
                    <DataTableTd numeric>{formatBytes(row.upload, unknown)}</DataTableTd>
                    <DataTableTd numeric>{formatBytes(row.download, unknown)}</DataTableTd>
                    <DataTableTd numeric>{row.connectionCount}</DataTableTd>
                    <DataTableTd numeric>
                      {totals && totals.download > 0 ? `${(share * 100).toFixed(1)}%` : unknown}
                    </DataTableTd>
                    {showAttribution ? (
                      <DataTableTd
                        className="max-w-0 truncate"
                        title={exitText}
                        {...(row.primaryExit != null ? { "data-exit": row.primaryExit } : {})}
                        data-exit-mixed={row.exitMixed ? "1" : "0"}
                      >
                        {exitText}
                      </DataTableTd>
                    ) : null}
                    {showDrill ? (
                      <DataTableTd>
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
                      </DataTableTd>
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
