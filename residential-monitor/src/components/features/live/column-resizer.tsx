import { useRef } from "react";
import type { UiLocale } from "../../../i18n";
import {
  columnWidth,
  minWidth,
  setColumnWidth,
  tablePixelWidth,
  WIDTH_MAX,
  type DataColumnId,
  type LiveTableLayout
} from "../../../live-table-layout";
import { ColResizer } from "../../common/col-resizer";

export { persistOnRelease, widthFromColumnKey } from "../../common/col-resizer";

export function ColumnResizer({
  column,
  label,
  layout,
  locale,
  tableRef,
  colRef,
  onCommit
}: {
  column: DataColumnId;
  label: string;
  layout: LiveTableLayout;
  locale: UiLocale;
  tableRef: { current: HTMLTableElement | null };
  colRef: { current: HTMLTableColElement | null };
  onCommit: (next: LiveTableLayout) => void;
}) {
  const layoutRef = useRef(layout);
  layoutRef.current = layout;
  const extra = tablePixelWidth(layout) - columnWidth(layout, column);

  return (
    <ColResizer
      id={column}
      width={columnWidth(layout, column)}
      min={minWidth(column)}
      max={WIDTH_MAX}
      label={label}
      locale={locale}
      tableRef={tableRef}
      colRef={colRef}
      onDraft={(nextWidth) => {
        if (tableRef.current) {
          const px = `${extra + nextWidth}px`;
          tableRef.current.style.width = px;
          tableRef.current.style.minWidth = px;
        }
      }}
      onCommit={(nextWidth) => {
        onCommit(setColumnWidth(layoutRef.current, column, nextWidth));
      }}
    />
  );
}
