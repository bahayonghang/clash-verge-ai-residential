import { useCallback, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  decodeReportArchivePage,
  decodeReportResult,
  type ReportArchivePage,
  type ReportArchiveSummary,
  type ReportQuery,
  type ReportResult
} from "../dto";
import {
  applyPresetRange,
  defaultReportForm,
  formFromQueryEcho,
  type ArchiveKindFilter,
  type ReportForm
} from "../format/report-view";
import { t, type UiLocale } from "../i18n";
import { isTauriRuntime } from "../ipc/live-session";
import { formatTemplate, invokeErrorZh } from "../lib/utils";

export type ReportSource = "auto-hour" | "auto-day" | "manual" | null;

export type RedactMode = "none" | "mask";
export type ExportFormat = "csv" | "json" | "html";

export interface ExportSpec {
  format: ExportFormat;
  includeSeries: boolean;
  includeRankings: boolean;
  includeSessions: boolean;
  redactHost: RedactMode;
  redactProcess: RedactMode;
}

export interface ExportPreview {
  format: ExportFormat;
  rowCount: number;
  sampleLabels: string[];
  metadataZh: string;
}

export function defaultReportQuery(nowUtc = Math.floor(Date.now() / 1000)): ReportQuery {
  return {
    rangeStartUtc: nowUtc - 3600,
    rangeEndUtc: nowUtc,
    displayTimezone: "local",
    granularity: "hour",
    filters: { category: null, host: null, process: null, rule: null, chain: null, network: null },
    grouping: "host",
    targetPolicy: "historical",
    comparison: { previousEqualWindow: true },
    sort: { field: "download", descending: true },
    page: { limit: 200, after: null },
    topN: 20,
    includeSessions: false
  };
}

export function defaultExportSpec(): ExportSpec {
  return {
    format: "csv",
    includeSeries: true,
    includeRankings: true,
    includeSessions: false,
    redactHost: "none",
    redactProcess: "none"
  };
}

export function pickLatestArchive(page: ReportArchivePage): ReportArchiveSummary | null {
  return (
    page.items.find((item) => item.kind === "day" && item.status === "ok") ??
    page.items.find((item) => item.kind === "hour" && item.status === "ok") ??
    null
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function decodePreview(value: unknown): ExportPreview {
  if (!isRecord(value) || typeof value.rowCount !== "number" || !Array.isArray(value.sampleLabels)) {
    throw new Error("ExportPreview 无效");
  }
  return {
    format: value.format === "json" || value.format === "html" ? value.format : "csv",
    rowCount: value.rowCount,
    sampleLabels: value.sampleLabels.map((item) => String(item)),
    metadataZh: typeof value.metadataZh === "string" ? value.metadataZh : ""
  };
}

export function useReportArchive(locale: UiLocale): {
  form: ReportForm;
  topN: number;
  compare: boolean;
  archives: ReportArchivePage | null;
  archiveKindFilter: ArchiveKindFilter;
  selectedArchiveId: string | null;
  report: ReportResult | null;
  reportSource: ReportSource;
  statusZh: string;
  loading: boolean;
  errorZh: string | null;
  exportPreview: ExportPreview | null;
  setForm: (form: ReportForm) => void;
  setTopN: (value: number) => void;
  setCompare: (value: boolean) => void;
  setArchiveKindFilter: (filter: ArchiveKindFilter) => void;
  loadArchives: (selectLatest: boolean) => Promise<void>;
  selectArchive: (archiveId: string) => Promise<void>;
  runManual: () => Promise<void>;
  runQuery: (query: ReportQuery) => Promise<void>;
  previewExport: (spec: ExportSpec) => Promise<void>;
  exportReport: (spec: ExportSpec) => Promise<void>;
  getStored: (token: string) => Promise<void>;
  release: () => Promise<void>;
} {
  const seq = useRef(0);
  const listSeq = useRef(0);
  const [form, setForm] = useState<ReportForm>(defaultReportForm);
  const [topN, setTopN] = useState(20);
  const [compare, setCompare] = useState(true);
  const [archives, setArchives] = useState<ReportArchivePage | null>(null);
  const [archiveKindFilter, setArchiveKindFilterState] = useState<ArchiveKindFilter>("all");
  const [selectedArchiveId, setSelectedArchiveId] = useState<string | null>(null);
  const [report, setReport] = useState<ReportResult | null>(null);
  const [reportSource, setReportSource] = useState<ReportSource>(null);
  const [statusZh, setStatusZh] = useState(() => t(locale, "report.idle"));
  const [loading, setLoading] = useState(false);
  const [errorZh, setErrorZh] = useState<string | null>(null);
  const [exportPreview, setExportPreview] = useState<ExportPreview | null>(null);

  const applyDecoded = useCallback(
    (next: ReportResult, status: string, source: ReportSource, archiveId: string | null): void => {
      setReport(next);
      setStatusZh(status);
      setReportSource(source);
      setSelectedArchiveId(archiveId);
      setForm((current) => formFromQueryEcho(next.queryEcho, current));
      setErrorZh(null);
    },
    []
  );

  const buildQuery = useCallback((): ReportQuery => {
    const archiveRange =
      form.windowSource === "archive" && report
        ? {
            start: report.queryEcho.rangeStartUtc,
            end: report.queryEcho.rangeEndUtc,
            timezone: report.queryEcho.displayTimezone
          }
        : undefined;
    const query = applyPresetRange(defaultReportQuery(), form, Math.floor(Date.now() / 1000), archiveRange);
    return {
      ...query,
      topN,
      comparison: compare ? { previousEqualWindow: true } : null
    };
  }, [compare, form, report, topN]);

  const runQuery = useCallback(
    async (query: ReportQuery): Promise<void> => {
      const token = ++seq.current;
      setLoading(true);
      setStatusZh(t(locale, "report.running"));
      const fallback = t(locale, "report.fail");
      if (!isTauriRuntime()) {
        setLoading(false);
        setStatusZh(fallback);
        setErrorZh(fallback);
        return;
      }
      try {
        const decoded = decodeReportResult(await invoke<unknown>("run_report", { query }));
        if (token !== seq.current) {
          return;
        }
        applyDecoded(
          decoded,
          formatTemplate(t(locale, "report.done"), { token: decoded.reportSnapshotToken.slice(0, 8) }),
          "manual",
          null
        );
      } catch (caught: unknown) {
        if (token !== seq.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, fallback));
        setStatusZh(fallback);
      } finally {
        if (token === seq.current) {
          setLoading(false);
        }
      }
    },
    [applyDecoded, locale]
  );

  const runManual = useCallback(async (): Promise<void> => {
    await runQuery(buildQuery());
  }, [buildQuery, runQuery]);

  const loadArchiveList = useCallback(async (): Promise<ReportArchivePage> => {
    const kind = archiveKindFilter === "all" ? null : archiveKindFilter;
    return decodeReportArchivePage(
      await invoke<unknown>("list_report_archives", { kind, after: null, limit: 50 })
    );
  }, [archiveKindFilter]);

  const loadArchives = useCallback(
    async (selectLatest: boolean): Promise<void> => {
      const fallback = t(locale, "report.archive.unavailable");
      if (!selectLatest) {
        const token = ++listSeq.current;
        if (!isTauriRuntime()) {
          return;
        }
        try {
          const page = await loadArchiveList();
          if (token !== listSeq.current) {
            return;
          }
          setArchives(page);
        } catch (caught: unknown) {
          if (token !== listSeq.current) {
            return;
          }
          setErrorZh(invokeErrorZh(caught, fallback));
        }
        return;
      }
      const token = ++seq.current;
      setLoading(true);
      setStatusZh(t(locale, "report.archive.catchup"));
      if (!isTauriRuntime()) {
        setLoading(false);
        setStatusZh(fallback);
        setErrorZh(fallback);
        return;
      }
      try {
        const page = await loadArchiveList();
        if (token !== seq.current) {
          return;
        }
        setArchives(page);
        const latest = pickLatestArchive(page);
        if (!latest) {
          setSelectedArchiveId(null);
          const hasFailed = page.items.some((item) => item.status === "failed");
          if (hasFailed) {
            setStatusZh(t(locale, "report.archive.failed"));
          } else if (page.items.length === 0) {
            setStatusZh(`${t(locale, "report.archive.empty")} ${t(locale, "report.archive.catchup")}`);
          } else {
            setStatusZh(t(locale, "report.archive.none_closed"));
          }
          setErrorZh(null);
          setLoading(false);
          return;
        }
        const decoded = decodeReportResult(
          await invoke<unknown>("get_report_archive", { archiveId: latest.archiveId })
        );
        if (token !== seq.current) {
          return;
        }
        const source: ReportSource = latest.kind === "day" ? "auto-day" : "auto-hour";
        applyDecoded(
          decoded,
          latest.kind === "day" ? t(locale, "report.archive.loaded_day") : t(locale, "report.archive.loaded_hour"),
          source,
          latest.archiveId
        );
        setForm((current) => ({ ...formFromQueryEcho(decoded.queryEcho, current), windowSource: "archive" }));
      } catch (caught: unknown) {
        if (token !== seq.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, fallback));
        setStatusZh(fallback);
      } finally {
        if (token === seq.current) {
          setLoading(false);
        }
      }
    },
    [applyDecoded, loadArchiveList, locale]
  );

  const selectArchive = useCallback(
    async (archiveId: string): Promise<void> => {
      const item = archives?.items.find((row) => row.archiveId === archiveId);
      setSelectedArchiveId(archiveId);
      if (!item || item.status !== "ok") {
        setStatusZh(item?.noteZh ? item.noteZh : t(locale, "report.archive.failed"));
        return;
      }
      const token = ++seq.current;
      setLoading(true);
      const fallback = t(locale, "report.archive.unavailable");
      if (!isTauriRuntime()) {
        setLoading(false);
        setStatusZh(fallback);
        setErrorZh(fallback);
        return;
      }
      try {
        const decoded = decodeReportResult(
          await invoke<unknown>("get_report_archive", { archiveId })
        );
        if (token !== seq.current) {
          return;
        }
        const source: ReportSource = item.kind === "day" ? "auto-day" : "auto-hour";
        applyDecoded(
          decoded,
          item.kind === "day" ? t(locale, "report.archive.loaded_day") : t(locale, "report.archive.loaded_hour"),
          source,
          archiveId
        );
        setForm((current) => ({ ...formFromQueryEcho(decoded.queryEcho, current), windowSource: "archive" }));
      } catch (caught: unknown) {
        if (token !== seq.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, fallback));
        setStatusZh(fallback);
      } finally {
        if (token === seq.current) {
          setLoading(false);
        }
      }
    },
    [applyDecoded, archives, locale]
  );

  const setArchiveKindFilter = useCallback((filter: ArchiveKindFilter): void => {
    setArchiveKindFilterState(filter);
  }, []);

  const previewExport = useCallback(
    async (spec: ExportSpec): Promise<void> => {
      const token = ++seq.current;
      const fallback = t(locale, "report.export_fail");
      if (!report) {
        setStatusZh(t(locale, "report.need_run"));
        return;
      }
      if (!isTauriRuntime()) {
        setErrorZh(fallback);
        return;
      }
      try {
        const next = decodePreview(
          await invoke<unknown>("preview_export", {
            token: report.reportSnapshotToken,
            spec
          })
        );
        if (token !== seq.current) {
          return;
        }
        setExportPreview(next);
        setErrorZh(null);
      } catch (caught: unknown) {
        if (token !== seq.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, fallback));
      }
    },
    [locale, report]
  );

  const getStored = useCallback(
    async (tokenStr: string): Promise<void> => {
      const token = ++seq.current;
      const fallback = t(locale, "report.fail");
      if (!isTauriRuntime()) {
        setErrorZh(fallback);
        return;
      }
      try {
        const decoded = decodeReportResult(await invoke<unknown>("get_report", { token: tokenStr }));
        if (token !== seq.current) {
          return;
        }
        applyDecoded(
          decoded,
          formatTemplate(t(locale, "report.done"), { token: decoded.reportSnapshotToken.slice(0, 8) }),
          reportSource,
          selectedArchiveId
        );
      } catch (caught: unknown) {
        if (token !== seq.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, fallback));
      }
    },
    [applyDecoded, locale, reportSource, selectedArchiveId]
  );

  const release = useCallback(async (): Promise<void> => {
    if (!report || !isTauriRuntime()) {
      return;
    }
    const token = ++seq.current;
    try {
      await invoke("release_report", { token: report.reportSnapshotToken });
      if (token !== seq.current) {
        return;
      }
      setErrorZh(null);
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, t(locale, "report.fail")));
    }
  }, [locale, report]);

  const exportReport = useCallback(
    async (spec: ExportSpec): Promise<void> => {
      const token = ++seq.current;
      const fallback = t(locale, "report.export_fail");
      if (!report) {
        setStatusZh(t(locale, "report.need_run"));
        return;
      }
      if (!isTauriRuntime()) {
        setErrorZh(fallback);
        return;
      }
      try {
        const picked = await invoke<string | null>("pick_file", {
          purpose: "report-export",
          mode: "save"
        });
        if (!picked) {
          if (token === seq.current) {
            setStatusZh(t(locale, "report.export_cancel"));
          }
          return;
        }
        await invoke("export_report", {
          token: report.reportSnapshotToken,
          spec,
          path: picked
        });
        if (token !== seq.current) {
          return;
        }
        setErrorZh(null);
        setStatusZh(formatTemplate(t(locale, "report.exported"), { format: spec.format.toUpperCase() }));
      } catch (caught: unknown) {
        if (token !== seq.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, fallback));
        setStatusZh(fallback);
      }
    },
    [locale, report]
  );

  return {
    form,
    topN,
    compare,
    archives,
    archiveKindFilter,
    selectedArchiveId,
    report,
    reportSource,
    statusZh,
    loading,
    errorZh,
    exportPreview,
    setForm,
    setTopN,
    setCompare,
    setArchiveKindFilter,
    loadArchives,
    selectArchive,
    runManual,
    runQuery,
    previewExport,
    exportReport,
    getStored,
    release
  };
}
