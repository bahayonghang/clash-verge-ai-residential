import { useCallback, useEffect, useRef, type KeyboardEvent, type PointerEvent } from "react";
import { t, type UiLocale } from "../../i18n";
import { formatTemplate } from "../../lib/utils";

type DragSession = {
  pointerId: number;
  startX: number;
  startW: number;
  current: number;
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

function writeCol(col: HTMLTableColElement | null, width: number): void {
  if (col) {
    col.style.width = `${width}px`;
  }
}

export function ColResizer({
  id,
  width,
  min,
  max,
  label,
  locale,
  colRef,
  onDraft,
  onCommit
}: {
  id: string;
  width: number;
  min: number;
  max: number;
  label: string;
  locale: UiLocale;
  tableRef: { current: HTMLTableElement | null };
  colRef: { current: HTMLTableColElement | null };
  onDraft: (nextWidth: number) => void;
  onCommit: (nextWidth: number) => void;
}) {
  const dragRef = useRef<DragSession | null>(null);
  const widthRef = useRef(width);
  widthRef.current = width;

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
        writeCol(colRef.current, drag.startW);
        onDraft(drag.startW);
        return;
      }
      if (persistOnRelease(drag.changed, commit)) {
        onCommit(drag.current);
      }
    },
    [colRef, onCommit, onDraft]
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
    dragRef.current = {
      pointerId: event.pointerId,
      startX: event.clientX,
      startW: widthRef.current,
      current: widthRef.current,
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
    const next = Math.min(max, Math.max(min, Math.round(drag.startW + (event.clientX - drag.startX))));
    drag.current = next;
    drag.changed ||= next !== drag.startW;
    writeCol(colRef.current, next);
    onDraft(next);
    event.currentTarget.setAttribute("aria-valuenow", String(next));
    event.currentTarget.setAttribute("aria-valuetext", `${next}px`);
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
    const nextWidth = widthFromColumnKey(event.key, event.shiftKey, widthRef.current, min, max);
    if (nextWidth == null) {
      return;
    }
    event.preventDefault();
    if (nextWidth === widthRef.current) {
      return;
    }
    writeCol(colRef.current, nextWidth);
    onDraft(nextWidth);
    onCommit(nextWidth);
  };

  return (
    <span
      data-col-resize={id}
      role="separator"
      tabIndex={0}
      aria-orientation="vertical"
      aria-label={formatTemplate(t(locale, "live.resize"), { column: label })}
      aria-valuemin={min}
      aria-valuemax={max}
      aria-valuenow={width}
      aria-valuetext={`${width}px`}
      aria-keyshortcuts="ArrowLeft ArrowRight Home End"
      className="absolute top-0 right-0 h-full w-2.5 cursor-col-resize touch-none hover:bg-ring/40 focus-visible:bg-ring"
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerCancel={onPointerCancel}
      onLostPointerCapture={onLostPointerCapture}
      onPointerUp={onPointerUp}
      onKeyDown={onKeyDown}
    />
  );
}
