import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { decodeReportResult, type ReportFilters, type ReportQuery, type ReportResult } from "../dto";
import { emptyReportFilters } from "../format/rank";
import { t } from "../i18n";
import { isTauriRuntime } from "../ipc/live-session";
import type { TimeRange, TimeRangePreset } from "../lib/time-range";
import { invokeErrorZh } from "../lib/utils";

export { emptyReportFilters, filtersForDrilldown, UNKNOWN_RANK_IDENTITY } from "../format/rank";

export type TrendPreset = "30m" | "1h" | "24h";

export const TREND_PRESETS: TrendPreset[] = ["30m", "1h", "24h"];

const MINUTE_MS = 60 * 1000;

export interface ReportViewState {
  result: ReportResult | null;
  loading: boolean;
  errorZh: string | null;
  seq: number;
}

export interface UseReportInput {
  grouping: ReportQuery["grouping"];
  timeRange: TimeRange;
  granularity: ReportQuery["granularity"];
  topN: number;
  filters?: ReportFilters;
  enabled?: boolean;
  sort?: ReportQuery["sort"];
}

export interface UseReportResult {
  result: ReportResult | null;
  loading: boolean;
  errorZh: string | null;
}

export function snapMsToMinute(ms: number): number {
  return Math.floor(ms / MINUTE_MS) * MINUTE_MS;
}

export function snapTimeRangeToMinute(range: TimeRange): TimeRange {
  return {
    preset: range.preset,
    startUtc: snapMsToMinute(range.startUtc),
    endUtc: snapMsToMinute(range.endUtc)
  };
}

export function granularityForTrendPreset(preset: TrendPreset): ReportQuery["granularity"] {
  if (preset === "30m") {
    return "minute1";
  }
  if (preset === "1h") {
    return "minute2";
  }
  return "minute10";
}

export function granularityForTimeRange(preset: TimeRangePreset): ReportQuery["granularity"] {
  if (preset === "5m" || preset === "30m") {
    return "minute1";
  }
  if (preset === "1h") {
    return "minute2";
  }
  if (preset === "24h" || preset === "today") {
    return "minute10";
  }
  if (preset === "7d") {
    return "hour";
  }
  return "day";
}

export function isTrendPreset(preset: TimeRangePreset): preset is TrendPreset {
  return preset === "30m" || preset === "1h" || preset === "24h";
}

export function buildReportQuery(input: {
  grouping: ReportQuery["grouping"];
  timeRange: TimeRange;
  granularity: ReportQuery["granularity"];
  topN: number;
  filters?: ReportFilters;
  sort?: ReportQuery["sort"];
}): ReportQuery {
  const snapped = snapTimeRangeToMinute(input.timeRange);
  return {
    rangeStartUtc: Math.floor(snapped.startUtc / 1000),
    rangeEndUtc: Math.floor(snapped.endUtc / 1000),
    displayTimezone: "local",
    granularity: input.granularity,
    filters: input.filters ?? emptyReportFilters(),
    grouping: input.grouping,
    targetPolicy: "historical",
    comparison: { previousEqualWindow: true },
    sort: input.sort ?? { field: "download", descending: true },
    page: { limit: 200, after: null },
    topN: input.topN,
    includeSessions: false
  };
}

export function beginReportRequest(state: ReportViewState): ReportViewState {
  return { ...state, seq: state.seq + 1, loading: true };
}

export function finishReportRequest(
  state: ReportViewState,
  seq: number,
  outcome: { ok: true; result: ReportResult } | { ok: false; errorZh: string }
): ReportViewState {
  if (seq !== state.seq) {
    return state;
  }
  if (outcome.ok) {
    return { seq: state.seq, result: outcome.result, loading: false, errorZh: null };
  }
  return { ...state, loading: false, errorZh: outcome.errorZh };
}

export async function runReport(query: ReportQuery): Promise<ReportResult> {
  const raw = await invoke<unknown>("run_report", { query });
  return decodeReportResult(raw);
}

export function useReport(input: UseReportInput): UseReportResult {
  const [result, setResult] = useState<ReportResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [errorZh, setErrorZh] = useState<string | null>(null);
  const seqRef = useRef(0);
  const enabled = input.enabled !== false;
  const startUtc = snapMsToMinute(input.timeRange.startUtc);
  const endUtc = snapMsToMinute(input.timeRange.endUtc);
  const filterKey = JSON.stringify(input.filters ?? emptyReportFilters());
  const sortKey = JSON.stringify(input.sort ?? { field: "download", descending: true });
  const grouping = input.grouping;
  const granularity = input.granularity;
  const topN = input.topN;
  const preset = input.timeRange.preset;
  const query = useMemo(
    () =>
      buildReportQuery({
        grouping,
        timeRange: { preset, startUtc, endUtc },
        granularity,
        topN,
        filters: JSON.parse(filterKey) as ReportFilters,
        sort: JSON.parse(sortKey) as ReportQuery["sort"]
      }),
    [endUtc, filterKey, granularity, grouping, preset, sortKey, startUtc, topN]
  );

  useEffect(() => {
    if (!enabled || !isTauriRuntime()) {
      setLoading(false);
      return;
    }
    const seq = ++seqRef.current;
    setLoading(true);
    let cancelled = false;
    void runReport(query)
      .then((next) => {
        if (cancelled || seq !== seqRef.current) {
          return;
        }
        setResult(next);
        setErrorZh(null);
        setLoading(false);
      })
      .catch((caught: unknown) => {
        if (cancelled || seq !== seqRef.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, t("zh", "report.fail")));
        setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [enabled, query]);

  return { result, loading, errorZh };
}
