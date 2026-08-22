import { useCallback, useEffect, useRef, useState, type KeyboardEvent, type PointerEvent } from "react";
import {
  applyShellWidth,
  clampUiSidebarWidth,
  SHELL_WIDTH_MAX,
  SHELL_WIDTH_MIN,
  SHELL_WIDTH_STEP
} from "../shell-width";

export function widthFromResizeKey(key: string, shift: boolean, current: number): number | null {
  const step = shift ? 32 : SHELL_WIDTH_STEP;
  if (key === "ArrowLeft") {
    return clampUiSidebarWidth(current - step);
  }
  if (key === "ArrowRight") {
    return clampUiSidebarWidth(current + step);
  }
  if (key === "Home") {
    return SHELL_WIDTH_MIN;
  }
  if (key === "End") {
    return SHELL_WIDTH_MAX;
  }
  return null;
}

export function persistOnRelease(changed: boolean, commit: boolean): boolean {
  return commit && changed;
}

type DragSession = {
  pointerId: number;
  startX: number;
  startW: number;
  changed: boolean;
};

export function useSidebarResize(
  width: number,
  onCommit: (next: number) => void
): {
  displayWidth: number;
  onPointerDown: (event: PointerEvent<HTMLElement>) => void;
  onPointerMove: (event: PointerEvent<HTMLElement>) => void;
  onPointerUp: (event: PointerEvent<HTMLElement>) => void;
  onPointerCancel: (event: PointerEvent<HTMLElement>) => void;
  onKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
  onKeyUp: (event: KeyboardEvent<HTMLElement>) => void;
  onBlur: () => void;
} {
  const [displayWidth, setDisplayWidth] = useState(width);
  const dragRef = useRef<DragSession | null>(null);
  const keyboardDirty = useRef(false);
  const displayRef = useRef(displayWidth);
  const commitRef = useRef(onCommit);

  useEffect(() => {
    commitRef.current = onCommit;
  }, [onCommit]);

  useEffect(() => {
    if (!dragRef.current) {
      displayRef.current = width;
      setDisplayWidth(width);
    }
  }, [width]);

  const applyDraft = useCallback((next: number) => {
    displayRef.current = next;
    setDisplayWidth(next);
    applyShellWidth(next);
  }, []);

  const finishDrag = useCallback((commit: boolean, pointerId?: number) => {
    const drag = dragRef.current;
    if (!drag || (pointerId !== undefined && drag.pointerId !== pointerId)) {
      return;
    }
    dragRef.current = null;
    document.documentElement.classList.remove("shell-resizing");
    if (persistOnRelease(drag.changed, commit)) {
      commitRef.current(displayRef.current);
      return;
    }
    if (!commit) {
      applyDraft(drag.startW);
    }
  }, [applyDraft]);

  const onPointerDown = useCallback(
    (event: PointerEvent<HTMLElement>) => {
      if (!event.isPrimary || event.button !== 0) {
        return;
      }
      event.preventDefault();
      event.stopPropagation();
      if (keyboardDirty.current) {
        keyboardDirty.current = false;
        commitRef.current(displayRef.current);
      }
      dragRef.current = {
        pointerId: event.pointerId,
        startX: event.clientX,
        startW: displayRef.current,
        changed: false
      };
      document.documentElement.classList.add("shell-resizing");
      event.currentTarget.setPointerCapture(event.pointerId);
    },
    []
  );

  const onPointerMove = useCallback(
    (event: PointerEvent<HTMLElement>) => {
      const drag = dragRef.current;
      if (!drag || event.pointerId !== drag.pointerId) {
        return;
      }
      const next = clampUiSidebarWidth(drag.startW + (event.clientX - drag.startX));
      drag.changed ||= next !== drag.startW;
      applyDraft(next);
    },
    [applyDraft]
  );

  const onPointerUp = useCallback(
    (event: PointerEvent<HTMLElement>) => {
      finishDrag(true, event.pointerId);
    },
    [finishDrag]
  );

  const onPointerCancel = useCallback(
    (event: PointerEvent<HTMLElement>) => {
      finishDrag(false, event.pointerId);
    },
    [finishDrag]
  );

  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLElement>) => {
      const next = widthFromResizeKey(event.key, event.shiftKey, displayRef.current);
      if (next == null) {
        return;
      }
      event.preventDefault();
      if (next === displayRef.current) {
        return;
      }
      applyDraft(next);
      keyboardDirty.current = true;
    },
    [applyDraft]
  );

  const onKeyUp = useCallback(() => {
    if (!keyboardDirty.current) {
      return;
    }
    keyboardDirty.current = false;
    commitRef.current(displayRef.current);
  }, []);

  const onBlur = useCallback(() => {
    finishDrag(false);
    if (keyboardDirty.current) {
      keyboardDirty.current = false;
      commitRef.current(displayRef.current);
    }
  }, [finishDrag]);

  return {
    displayWidth,
    onPointerDown,
    onPointerMove,
    onPointerUp,
    onPointerCancel,
    onKeyDown,
    onKeyUp,
    onBlur
  };
}
