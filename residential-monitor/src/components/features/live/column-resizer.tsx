import { useCallback, useEffect, useRef, type KeyboardEvent, type PointerEvent } from "react";
import { t, type UiLocale } from "../../../i18n";
import { formatTemplate } from "../../../lib/utils";
import {
  columnWidth,
  minWidth,
  setColumnWidth,
  tablePixelWidth,
  WIDTH_MAX,
  type DataColumnId,
  type LiveTableLayout
} from "../../../live-table-layout";

type DragSession = {
  pointerId: number;
  startX: number;
  startW: number;
  startLayout: LiveTableLayout;
  current: LiveTableLayout;
  changed: boolean;
  handle: HTMLElement;
};

export function persistOnRelease(changed: boolean, commit: boolean): boolean {
  return commit && changed;
}

export function widthFromColumnKey(
  key: string,
  shift: boolean,
  current: number,
  min: number,
  max: number
): number | null {
  const step = shift ? 32 : 8;
  if (key === "ArrowLeft") {
    return Math.max(min, current - step);
  }
  if (key === "ArrowRight") {
    return Math.min(max, current + step);
  }
  if (key === "Home") {
    return min;
  }
  if (key === "End") {
    return max;
  }
  return null;
}

function writeWidths(
  table: HTMLTableElement | null,
  col: HTMLTableColElement | null,
  layout: LiveTableLayout,
  column: DataColumnId
): void {
  if (col) {
    col.style.width = `${columnWidth(layout, column)}px`;
  }
  if (table) {
    const px = `${tablePixelWidth(layout)}px`;
    table.style.width = px;
    table.style.minWidth = px;
  }
}

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
  const dragRef = useRef<DragSession | null>(null);
  const layoutRef = useRef(layout);
  layoutRef.current = layout;
  const width = columnWidth(layout, column);
  const min = minWidth(column);

  const finish = useCallback(
    (commit: boolean, pointerId?: number) => {
      const drag = dragRef.current;
      if (!drag || (pointerId !== undefined && drag.pointerId !== pointerId)) {
        return;
      }
      dragRef.current = null;
      document.documentElement.classList.remove("live-table-resizing");
      if (drag.handle.hasPointerCapture(drag.pointerId)) {
        drag.handle.releasePointerCapture(drag.pointerId);
      }
      if (!commit) {
        writeWidths(tableRef.current, colRef.current, drag.startLayout, column);
        return;
      }
      if (persistOnRelease(drag.changed, commit)) {
        onCommit(drag.current);
      }
    },
    [colRef, column, onCommit, tableRef]
  );

  useEffect(() => {
    const onBlur = (): void => finish(false);
    window.addEventListener("blur", onBlur);
    return () => {
      window.removeEventListener("blur", onBlur);
    };
  }, [finish]);

  const onPointerDown = (event: PointerEvent<HTMLSpanElement>): void => {
    if (!event.isPrimary || event.button !== 0) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    const startLayout = {
      widths: { ...layoutRef.current.widths },
      hidden: [...layoutRef.current.hidden]
    };
    dragRef.current = {
      pointerId: event.pointerId,
      startX: event.clientX,
      startW: columnWidth(startLayout, column),
      startLayout,
      current: startLayout,
      changed: false,
      handle: event.currentTarget
    };
    document.documentElement.classList.add("live-table-resizing");
    event.currentTarget.setPointerCapture(event.pointerId);
  };

  const onPointerMove = (event: PointerEvent<HTMLSpanElement>): void => {
    const drag = dragRef.current;
    if (!drag || event.pointerId !== drag.pointerId) {
      return;
    }
    const next = setColumnWidth(drag.startLayout, column, drag.startW + (event.clientX - drag.startX));
    drag.current = next;
    drag.changed ||= columnWidth(next, column) !== drag.startW;
    writeWidths(tableRef.current, colRef.current, next, column);
    event.currentTarget.setAttribute("aria-valuenow", String(columnWidth(next, column)));
    event.currentTarget.setAttribute("aria-valuetext", `${columnWidth(next, column)}px`);
  };

  const onPointerUp = (event: PointerEvent<HTMLSpanElement>): void => {
    finish(true, event.pointerId);
  };

  const onPointerCancel = (event: PointerEvent<HTMLSpanElement>): void => {
    finish(false, event.pointerId);
  };

  const onLostPointerCapture = (event: PointerEvent<HTMLSpanElement>): void => {
    finish(false, event.pointerId);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLSpanElement>): void => {
    const nextWidth = widthFromColumnKey(
      event.key,
      event.shiftKey,
      columnWidth(layoutRef.current, column),
      min,
      WIDTH_MAX
    );
    if (nextWidth == null) {
      return;
    }
    event.preventDefault();
    const next = setColumnWidth(layoutRef.current, column, nextWidth);
    if (columnWidth(next, column) === columnWidth(layoutRef.current, column)) {
      return;
    }
    writeWidths(tableRef.current, colRef.current, next, column);
    onCommit(next);
  };

  return (
    <span
      data-col-resize={column}
      role="separator"
      tabIndex={0}
      aria-orientation="vertical"
      aria-label={formatTemplate(t(locale, "live.resize"), { column: label })}
      aria-valuemin={min}
      aria-valuemax={WIDTH_MAX}
      aria-valuenow={width}
      aria-valuetext={`${width}px`}
      aria-keyshortcuts="ArrowLeft ArrowRight Home End"
      className="absolute top-0 right-0 h-full w-2.5 cursor-col-resize touch-none hover:bg-ring/40 focus-visible:bg-ring"
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
      onPointerCancel={onPointerCancel}
      onLostPointerCapture={onLostPointerCapture}
      onKeyDown={onKeyDown}
    />
  );
}
