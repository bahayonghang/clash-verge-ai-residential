import { useEffect, useRef, useState } from "react";
import type { BootstrapDto } from "../dto";
import { t, type UiLocale } from "../i18n";
import { decodeMonitorMessage } from "../ipc/decoder";
import { isTauriRuntime, resyncMonitor, subscribeMonitor } from "../ipc/live-session";
import { emptyMonitorState, reduceMonitor, type MonitorState } from "../ipc/reducer";

export function useMonitorStream(
  boot: BootstrapDto | null,
  locale: UiLocale,
  resyncTick = 0
): MonitorState {
  const [state, setState] = useState<MonitorState>(emptyMonitorState);
  const resyncInFlight = useRef(false);
  const generation = useRef(0);
  const onRawRef = useRef<(raw: unknown) => void>(() => undefined);
  const localeRef = useRef(locale);
  const subscriptionIdRef = useRef<number | null>(null);

  useEffect(() => {
    localeRef.current = locale;
  }, [locale]);

  useEffect(() => {
    if (!boot) {
      return;
    }
    setState((prev) => (prev.snapshot ? prev : { ...prev, snapshot: boot.overview }));
  }, [boot]);

  useEffect(() => {
    if (!boot || !isTauriRuntime()) {
      return;
    }
    const gen = ++generation.current;
    let cancelled = false;

    const onRaw = (raw: unknown): void => {
      if (cancelled || generation.current !== gen) {
        return;
      }
      try {
        const message = decodeMonitorMessage(raw);
        setState((prev) => reduceMonitor(prev, message));
      } catch {
        setState((prev) => ({
          ...prev,
          frozen: true,
          errorZh: t(localeRef.current, "stream.decode_fail")
        }));
      }
    };
    onRawRef.current = onRaw;

    void subscribeMonitor(onRaw).catch(() => {
      if (!cancelled && generation.current === gen) {
        setState((prev) => ({ ...prev, errorZh: t(localeRef.current, "stream.resync_fail") }));
      }
    });

    return () => {
      cancelled = true;
    };
  }, [boot]);

  useEffect(() => {
    if (!state.needResync || resyncInFlight.current || state.subscriptionId === null) {
      return;
    }
    const gen = generation.current;
    resyncInFlight.current = true;
    void resyncMonitor(state.subscriptionId, (raw) => onRawRef.current(raw))
      .catch(() => {
        if (generation.current === gen) {
          setState((current) => ({
            ...current,
            errorZh: t(localeRef.current, "stream.resync_fail")
          }));
        }
      })
      .finally(() => {
        resyncInFlight.current = false;
      });
  }, [state.needResync, state.subscriptionId]);

  useEffect(() => {
    subscriptionIdRef.current = state.subscriptionId;
  }, [state.subscriptionId]);

  useEffect(() => {
    if (resyncTick < 1 || resyncInFlight.current || subscriptionIdRef.current === null) {
      return;
    }
    const gen = generation.current;
    const subscriptionId = subscriptionIdRef.current;
    resyncInFlight.current = true;
    void resyncMonitor(subscriptionId, (raw) => onRawRef.current(raw))
      .catch(() => {
        if (generation.current === gen) {
          setState((current) => ({
            ...current,
            errorZh: t(localeRef.current, "stream.resync_fail")
          }));
        }
      })
      .finally(() => {
        resyncInFlight.current = false;
      });
  }, [resyncTick]);

  return state;
}
