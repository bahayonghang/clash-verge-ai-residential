import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import {
  inspectKeysMatch,
  reportInspectModel,
  type ReportInspectModel
} from "../../../format/report-inspect";
import { formatSharePct, type ShareModel } from "../../../format/report-view";
import { formatBytes, formatUtc } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { formatTemplate } from "../../../lib/utils";

export function retainInspectKey(
  key: string | null,
  share: ShareModel | null,
  series: Array<{ bucketUtc: number; upload: number; download: number }>
): string | null {
  if (!key) {
    return null;
  }
  return reportInspectModel(key, share, series) ? key : null;
}

export function formatInspectTip(locale: UiLocale, model: ReportInspectModel): string {
  const unknown = t(locale, "common.unknown");
  if (model.surface === "pie") {
    const up = model.upload === null ? t(locale, "report.dash") : formatBytes(model.upload, unknown);
    return formatTemplate(t(locale, "report.inspect.pie"), {
      name: model.label,
      up,
      down: formatBytes(model.download, unknown),
      share: formatSharePct(model.share, unknown)
    });
  }
  const time = formatUtc(model.bucketUtc);
  const up = formatBytes(model.upload, unknown);
  const down = formatBytes(model.download, unknown);
  if (model.direction === "up") {
    return formatTemplate(t(locale, "report.inspect.trend_up"), { time, up });
  }
  if (model.direction === "down") {
    return formatTemplate(t(locale, "report.inspect.trend_down"), { time, down });
  }
  return formatTemplate(t(locale, "report.inspect.trend"), { time, up, down });
}

interface InspectState {
  pinned: string | null;
  hover: string | null;
  activeKey: string | null;
  setPinned: (key: string | null) => void;
  setHover: (key: string | null) => void;
  togglePinned: (key: string | null) => void;
}

const InspectContext = createContext<InspectState | null>(null);

export function ReportInspectProvider({
  locale,
  share,
  series,
  children
}: {
  locale: UiLocale;
  share: ShareModel | null;
  series: Array<{ bucketUtc: number; upload: number; download: number }>;
  children: ReactNode;
}) {
  const [pinned, setPinnedState] = useState<string | null>(null);
  const [hover, setHoverState] = useState<string | null>(null);

  useEffect(() => {
    setPinnedState((key) => retainInspectKey(key, share, series));
    setHoverState((key) => retainInspectKey(key, share, series));
  }, [share, series]);

  const setPinned = useCallback((key: string | null) => {
    setPinnedState(key);
  }, []);
  const setHover = useCallback((key: string | null) => {
    setHoverState(key);
  }, []);
  const togglePinned = useCallback((key: string | null) => {
    setPinnedState((current) => (current && key && inspectKeysMatch(current, key) ? null : key));
  }, []);

  useEffect(() => {
    const onKey = (event: KeyboardEvent): void => {
      if (event.key === "Escape") {
        setPinnedState(null);
        setHoverState(null);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const activeKey = hover ?? pinned;
  const model = activeKey ? reportInspectModel(activeKey, share, series) : null;
  const value = useMemo(
    () => ({ pinned, hover, activeKey, setPinned, setHover, togglePinned }),
    [pinned, hover, activeKey, setPinned, setHover, togglePinned]
  );

  return (
    <InspectContext.Provider value={value}>
      {children}
      {model ? (
        <p className="rounded-md border bg-card px-3 py-2 text-sm" role="status">
          {formatInspectTip(locale, model)}
          {pinned ? <span className="ml-2 text-muted-foreground">{t(locale, "report.inspect.pinned")}</span> : null}
        </p>
      ) : null}
    </InspectContext.Provider>
  );
}

export function useReportInspect(): InspectState {
  const value = useContext(InspectContext);
  if (!value) {
    throw new Error("useReportInspect 必须在 ReportInspectProvider 内使用");
  }
  return value;
}
