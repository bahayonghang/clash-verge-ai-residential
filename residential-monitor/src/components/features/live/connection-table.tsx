import { useRef } from "react";
import type { CloseState, LiveConnectionView } from "../../../dto";
import { displayLiveRow } from "../../../format/live-row";
import { t, type UiLocale } from "../../../i18n";
import {
  ACTION_WIDTH,
  columnLabelKey,
  columnWidth,
  isNumericColumn,
  tablePixelWidth,
  visibleDataColumns,
  type DataColumnId,
  type LiveTableLayout
} from "../../../live-table-layout";
import { sortAria, sortMarker, type LiveSortState } from "../../../live-table-sort";
import { Button } from "../../ui/button";
import { TableBody, TableCell, TableHead, TableHeader, TableRow } from "../../ui/table";
import { ColumnResizer } from "./column-resizer";
import { EmptyState } from "./empty-state";
import type { LiveEmptyKind } from "../../../ipc/live-empty";

export type LiveRowView = ReturnType<typeof displayLiveRow>;

export interface LiveCell {
  text: string;
  numeric: boolean;
  title?: string;
}

const CELL_TEXT: Record<DataColumnId, (view: LiveRowView) => string> = {
  host: (view) => view.host,
  download: (view) => view.download,
  upload: (view) => view.upload,
  rateDownload: (view) => view.dlSpeed,
  rateUpload: (view) => view.ulSpeed,
  chain: (view) => view.chains,
  rule: (view) => view.rule,
  process: (view) => view.process,
  duration: (view) => view.time,
  source: (view) => view.source,
  destination: (view) => view.destination,
  type: (view) => view.type
};

export function cellOf(view: LiveRowView, column: DataColumnId): LiveCell {
  const text = CELL_TEXT[column] ? CELL_TEXT[column](view) : view.host;
  return {
    text,
    numeric: isNumericColumn(column),
    title: text
  };
}

function closeLabel(locale: UiLocale, mark: CloseState["mark"] | undefined): string {
  if (mark === "accepted") {
    return t(locale, "live.close_accepted");
  }
  if (mark === "closed") {
    return t(locale, "live.close_done");
  }
  if (mark === "unconfirmed") {
    return t(locale, "live.close_unconfirmed");
  }
  return t(locale, "live.close");
}

export function ConnectionTable({
  locale,
  rows,
  layout,
  sort,
  closeMarks,
  emptyKind,
  healthTitle,
  healthAction,
  onSort,
  onClose,
  onLayoutCommit,
  onGoSettings,
  onResubscribe
}: {
  locale: UiLocale;
  rows: LiveConnectionView[];
  layout: LiveTableLayout;
  sort: LiveSortState;
  closeMarks: ReadonlyMap<string, CloseState["mark"]>;
  emptyKind: LiveEmptyKind;
  healthTitle: string;
  healthAction: string;
  onSort: (column: DataColumnId) => void;
  onClose: (identity: string) => void;
  onLayoutCommit: (layout: LiveTableLayout) => void;
  onGoSettings: () => void;
  onResubscribe: () => void;
}) {
  const tableRef = useRef<HTMLTableElement | null>(null);
  const colRefs = useRef<Partial<Record<DataColumnId, HTMLTableColElement | null>>>({});
  const visible = visibleDataColumns(layout);
  const tableWidth = tablePixelWidth(layout);
  const unknown = t(locale, "common.unknown");

  return (
    <div
      className="live-table-wrap min-h-[12rem] min-w-0 w-full max-h-[min(60vh,36rem)] overflow-auto [scrollbar-gutter:stable]"
      tabIndex={0}
      role="region"
      aria-label={t(locale, "live.table")}
    >
      <table
        ref={tableRef}
        className="live-table table-fixed caption-bottom border-collapse text-sm"
        style={{ width: tableWidth, minWidth: tableWidth }}
      >
        <colgroup>
          {visible.map((column) => (
            <col
              key={column}
              data-col={column}
              ref={(node) => {
                colRefs.current[column] = node;
              }}
              style={{ width: columnWidth(layout, column) }}
            />
          ))}
          <col data-col="action" style={{ width: ACTION_WIDTH }} />
        </colgroup>
        <TableHeader>
          <TableRow className="data-row">
            {visible.map((column) => {
              const label = t(locale, columnLabelKey(column));
              const aria = sortAria(column, sort);
              return (
                <TableHead
                  key={column}
                  data-col={column}
                  aria-sort={aria}
                  className={`relative sticky top-0 z-[1] overflow-hidden bg-card ${isNumericColumn(column) ? "text-right" : ""}`}
                >
                  <button
                    type="button"
                    className="inline-flex max-w-full items-center gap-1 truncate rounded-sm text-left font-medium hover:text-foreground focus-visible:outline-none"
                    onClick={() => onSort(column)}
                  >
                    <span className="truncate">{label}</span>
                    <span aria-hidden="true">{sortMarker(column, sort)}</span>
                  </button>
                  <ColumnResizer
                    column={column}
                    label={label}
                    layout={layout}
                    locale={locale}
                    tableRef={tableRef}
                    colRef={{
                      get current() {
                        return colRefs.current[column] ?? null;
                      }
                    }}
                    onCommit={onLayoutCommit}
                  />
                </TableHead>
              );
            })}
            <TableHead data-col="action" className="sticky top-0 z-[1] bg-card">
              {t(locale, "live.col.action")}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.length === 0 ? (
            <TableRow>
              <TableCell colSpan={visible.length + 1} className="whitespace-normal p-6">
                <EmptyState
                  kind={emptyKind}
                  locale={locale}
                  healthTitle={healthTitle}
                  healthAction={healthAction}
                  onGoSettings={onGoSettings}
                  onResubscribe={onResubscribe}
                />
              </TableCell>
            </TableRow>
          ) : (
            rows.map((row) => {
              const mark = closeMarks.get(row.identity);
              const view = displayLiveRow(row, locale, unknown);
              return (
                <TableRow key={row.identity} className="data-row">
                  {visible.map((column) => {
                    const cell = cellOf(view, column);
                    return (
                      <TableCell
                        key={column}
                        title={cell.title}
                        className={`overflow-hidden text-ellipsis ${cell.numeric ? "text-right tabular-nums" : ""}`}
                      >
                        {cell.text}
                      </TableCell>
                    );
                  })}
                  <TableCell>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      disabled={mark != null}
                      onClick={() => onClose(row.identity)}
                    >
                      {closeLabel(locale, mark)}
                    </Button>
                  </TableCell>
                </TableRow>
              );
            })
          )}
        </TableBody>
      </table>
    </div>
  );
}
