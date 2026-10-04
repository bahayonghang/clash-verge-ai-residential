import { createContext, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { t, type UiLocale } from "../i18n";
import { isTauriRuntime } from "../ipc/live-session";

export const WindowVisibilityContext = createContext(true);

export function decodeWindowVisibility(raw: unknown): boolean {
  if (raw === null || typeof raw !== "object" ||
      !Object.hasOwn(raw, "visible") || typeof (raw as { visible: unknown }).visible !== "boolean") {
    throw new Error("窗口可见状态无效");
  }
  return (raw as { visible: boolean }).visible;
}

export async function observeWindowVisibility(
  onVisible: (visible: boolean) => void,
  onError: () => void,
  port = {
    listen: (receive: (raw: unknown) => void) =>
      listen<unknown>("window-visibility", (event) => receive(event.payload)),
    read: () => invoke<unknown>("get_window_visibility")
  }
): Promise<() => void> {
  let eventSeen = false;
  const receive = (raw: unknown): void => {
    try {
      onVisible(decodeWindowVisibility(raw));
    } catch {
      onError();
    }
  };
  const unlisten = await port.listen((raw) => {
    eventSeen = true;
    receive(raw);
  });
  // 先订阅再读初值，避免迟到的初值覆盖隐藏/恢复事件。
  void port.read().then((raw) => {
    if (!eventSeen) receive(raw);
  }).catch(() => {
    if (!eventSeen) onError();
  });
  return unlisten;
}

export function useWindowVisibility(locale: UiLocale, onResume: () => void): {
  visible: boolean;
  errorZh: string | null;
} {
  const [visible, setVisible] = useState(() => !isTauriRuntime());
  const [failed, setFailed] = useState(false);
  const resumeRef = useRef(onResume);
  resumeRef.current = onResume;
  useEffect(() => {
    if (!isTauriRuntime()) return;
    let stopped = false;
    let previous = false;
    let stop: (() => void) | undefined;
    const fail = (): void => {
      if (stopped) return;
      previous = false;
      setVisible(false);
      setFailed(true);
    };
    void observeWindowVisibility((next) => {
      if (stopped) return;
      if (next && !previous) resumeRef.current();
      previous = next;
      setVisible(next);
      setFailed(false);
    }, fail).then((unlisten) => {
      if (stopped) unlisten();
      else stop = unlisten;
    }).catch(fail);
    return () => {
      stopped = true;
      stop?.();
    };
  }, []);
  return { visible, errorZh: failed ? t(locale, "window.visibility_fail") : null };
}
