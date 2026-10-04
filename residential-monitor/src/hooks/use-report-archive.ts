import { useCallback, useEffect, useRef, useState } from "react";
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
import { DEFAULT_RANK_SORT, type RankSortSpec } from "../rank-sort";
import { releaseReportToken, runReport } from "./use-report";
import { useDisplayQuery } from "./use-display-query";
import type { DisplayRequest } from "./display-query";

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
    sort: DEFAULT_RANK_SORT,
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

export function decodeHtmlDocument(value: unknown): string {
  if (!value || typeof value !== "object" || !("html" in value)) {
    throw new Error("HtmlDocument 无效");
  }
  const html = (value as { html: unknown }).html;
  if (typeof html !== "string" || html.length === 0) {
    throw new Error("HtmlDocument.html 无效");
  }
  return html;
}

async function renderReportHtml(token: string): Promise<string> {
  return decodeHtmlDocument(
    await invoke<unknown>("render_report_html", {
      token,
      spec: { ...defaultExportSpec(), format: "html" as const }
    })
  );
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

export function useReportArchive(locale: UiLocale, previewHtml = false): {
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
  html: string | null;
  htmlError: string | null;
  exportPreview: ExportPreview | null;
  setForm: (form: ReportForm) => void;
  setTopN: (value: number) => void;
  setCompare: (value: boolean) => void;
  setArchiveKindFilter: (filter: ArchiveKindFilter) => void;
  loadArchives: (selectLatest: boolean) => Promise<void>;
  selectArchive: (archiveId: string) => Promise<void>;
  sort: RankSortSpec;
  runManual: () => Promise<void>;
  runQuery: (query: ReportQuery) => Promise<void>;
  applyRankSort: (next: RankSortSpec) => Promise<void>;
  restoreResidentialManual: () => Promise<void>;
  previewExport: (spec: ExportSpec) => Promise<void>;
  exportReport: (spec: ExportSpec) => Promise<void>;
  getStored: (token: string) => Promise<void>;
  release: () => Promise<void>;
} {
  const seq = useRef(0);
  const listSeq = useRef(0);
  const { queue, active } = useDisplayQuery();
  const tokenRef = useRef<string | null>(null);
  const exportingToken = useRef<string | null>(null);
  const deferredRelease = useRef<string | null>(null);
  const [form, setForm] = useState<ReportForm>(defaultReportForm);
  const [topN, setTopN] = useState(20);
  const [compare, setCompare] = useState(true);
  const [sort, setSort] = useState<RankSortSpec>(DEFAULT_RANK_SORT);
  const [archives, setArchives] = useState<ReportArchivePage | null>(null);
  const [archiveKindFilter, setArchiveKindFilterState] = useState<ArchiveKindFilter>("all");
  const [selectedArchiveId, setSelectedArchiveId] = useState<string | null>(null);
  const [report, setReport] = useState<ReportResult | null>(null);
  const [reportSource, setReportSource] = useState<ReportSource>(null);
  const [statusZh, setStatusZh] = useState(() => t(locale, "report.idle"));
  const [loading, setLoading] = useState(false);
  const [errorZh, setErrorZh] = useState<string | null>(null);
  const [html, setHtml] = useState<string | null>(null);
  const [htmlError, setHtmlError] = useState<string | null>(null);
  const [exportPreview, setExportPreview] = useState<ExportPreview | null>(null);
  const retained = useRef({ report, reportSource, selectedArchiveId, statusZh });
  retained.current = { report, reportSource, selectedArchiveId, statusZh };

  const applyDecoded = useCallback(
    async (next: ReportResult, status: string, source: ReportSource, archiveId: string | null,
      request: DisplayRequest, acquired = true): Promise<void> => {
      if (!request.isCurrent()) {
        if (acquired) await releaseReportToken(next.reportSnapshotToken);
        return;
      }
      const previous = tokenRef.current;
      tokenRef.current = next.reportSnapshotToken || null;
      if (previous && acquired) {
        void releaseReportToken(previous);
      }
      setReport(next);
      setStatusZh(status);
      setReportSource(source);
      setSelectedArchiveId(archiveId);
      setForm((current) => formFromQueryEcho(next.queryEcho, current));
      setSort(next.queryEcho.sort);
      setErrorZh(null);
      if (previewHtml) {
        setHtml(null);
        setHtmlError(null);
        try {
          const nextHtml = await renderReportHtml(next.reportSnapshotToken);
          if (request.isCurrent()) setHtml(nextHtml);
        } catch (caught: unknown) {
          if (request.isCurrent()) setHtmlError(invokeErrorZh(caught, t(locale, "report.export_fail")));
        }
      }
    },
    [locale, previewHtml]
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
      comparison: compare ? { previousEqualWindow: true } : null,
      sort
    };
  }, [compare, form, report, sort, topN]);

  const loadArchiveList = useCallback(async (): Promise<ReportArchivePage> => {
    const kind = archiveKindFilter === "all" ? null : archiveKindFilter;
    return decodeReportArchivePage(
      await invoke<unknown>("list_report_archives", { kind, after: null, limit: 50 })
    );
  }, [archiveKindFilter]);

  const loadArchives = useCallback(
    (selectLatest: boolean): Promise<void> => queue.request(`list:${archiveKindFilter}:${selectLatest}`, async (request) => {
      const fallback = t(locale, "report.archive.unavailable");
      if (!selectLatest) {
        const token = ++listSeq.current;
        if (!isTauriRuntime()) {
          return;
        }
        try {
          const page = await loadArchiveList();
          if (!request.isCurrent() || token !== listSeq.current) {
            return;
          }
          setArchives(page);
          const previous = retained.current;
          if (!tokenRef.current && previous.report) {
            const next = previous.selectedArchiveId
              ? decodeReportResult(await invoke<unknown>("get_report_archive", { archiveId: previous.selectedArchiveId }))
              : await runReport(previous.report.queryEcho, false, request.signal);
            await applyDecoded(next, previous.statusZh, previous.reportSource, previous.selectedArchiveId, request);
          }
        } catch (caught: unknown) {
          if (!request.isCurrent() || token !== listSeq.current) {
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
        if (!request.isCurrent() || token !== seq.current) {
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
        try {
          const decoded = decodeReportResult(
            await invoke<unknown>("get_report_archive", { archiveId: latest.archiveId })
          );
          if (!request.isCurrent() || token !== seq.current) {
            void releaseReportToken(decoded.reportSnapshotToken);
            return;
          }
          const source: ReportSource = latest.kind === "day" ? "auto-day" : "auto-hour";
          await applyDecoded(
            decoded,
            latest.kind === "day" ? t(locale, "report.archive.loaded_day") : t(locale, "report.archive.loaded_hour"),
            source,
            latest.archiveId,
            request
          );
          if (!request.isCurrent()) return;
          setForm((current) => ({ ...formFromQueryEcho(decoded.queryEcho, current), windowSource: "archive" }));
        } catch (caught: unknown) {
          if (!request.isCurrent() || token !== seq.current) {
            return;
          }
          setErrorZh(invokeErrorZh(caught, t(locale, "report.fail")));
          setStatusZh(invokeErrorZh(caught, t(locale, "report.fail")));
        }
      } catch (caught: unknown) {
        if (!request.isCurrent() || token !== seq.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, fallback));
        setStatusZh(fallback);
      } finally {
        if (request.isCurrent() && token === seq.current) {
          setLoading(false);
        }
      }
    }),
    [applyDecoded, archiveKindFilter, loadArchiveList, locale, queue]
  );

  const runQuery = useCallback(
    async (query: ReportQuery): Promise<void> => {
      let persistManual = true;
      return queue.request(`query:${JSON.stringify(query)}`, async (request) => {
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
          const persist = persistManual;
          persistManual = false;
          const decoded = await runReport(query, persist, request.signal);
          if (!request.isCurrent() || token !== seq.current) {
            void releaseReportToken(decoded.reportSnapshotToken);
            return;
          }
          await applyDecoded(
            decoded,
            formatTemplate(t(locale, "report.done"), { token: decoded.reportSnapshotToken.slice(0, 8) }),
            "manual",
            null,
            request
          );
          if (request.isCurrent()) {
            try {
              const page = await loadArchiveList();
              if (request.isCurrent()) setArchives(page);
            } catch (caught: unknown) {
              if (request.isCurrent()) setErrorZh(invokeErrorZh(caught, t(locale, "report.archive.unavailable")));
            }
          }
        } catch (caught: unknown) {
          if (!request.isCurrent() || token !== seq.current) {
            return;
          }
          setErrorZh(invokeErrorZh(caught, fallback));
          setStatusZh(fallback);
        } finally {
          if (request.isCurrent() && token === seq.current) {
            setLoading(false);
          }
        }
      });
    },
    [applyDecoded, loadArchiveList, locale, queue]
  );

  const runManual = useCallback(async (): Promise<void> => {
    await runQuery(buildQuery());
  }, [buildQuery, runQuery]);

  const applyRankSort = useCallback(
    async (next: RankSortSpec): Promise<void> => {
      setSort(next);
      await runQuery({ ...buildQuery(), sort: next });
    },
    [buildQuery, runQuery]
  );

  const restoreResidentialManual = useCallback((): Promise<void> => queue.request("residential-manual", async (request) => {
    const token = ++seq.current;
    const fallback = t(locale, "report.fail");
    if (!isTauriRuntime()) {
      return;
    }
    setLoading(true);
    setStatusZh(t(locale, "report.running"));
    try {
      const raw = await invoke<unknown>("get_latest_residential_manual");
      if (!request.isCurrent() || token !== seq.current) {
        if (raw != null) {
          try {
            void releaseReportToken(decodeReportResult(raw).reportSnapshotToken);
          } catch {
            /* 过期响应释放失败不覆盖 */
          }
        }
        return;
      }
      if (raw == null) {
        setLoading(false);
        return;
      }
      const decoded = decodeReportResult(raw);
      await applyDecoded(decoded, t(locale, "residential.report.ready"), "manual", null, request);
    } catch (caught: unknown) {
      if (!request.isCurrent() || token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, fallback));
      setStatusZh(fallback);
    } finally {
      if (request.isCurrent() && token === seq.current) {
        setLoading(false);
      }
    }
  }), [applyDecoded, locale, queue]);

  const selectArchive = useCallback(
    (archiveId: string): Promise<void> => queue.request(`archive:${archiveId}`, async (request) => {
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
        if (!request.isCurrent() || token !== seq.current) {
          void releaseReportToken(decoded.reportSnapshotToken);
          return;
        }
        const source: ReportSource =
          item.kind === "day" ? "auto-day" : item.kind === "manual" ? "manual" : "auto-hour";
        const loaded =
          item.kind === "day"
            ? t(locale, "report.archive.loaded_day")
            : item.kind === "manual"
              ? t(locale, "report.archive.loaded_manual")
              : t(locale, "report.archive.loaded_hour");
        await applyDecoded(decoded, loaded, source, archiveId, request);
        if (!request.isCurrent()) return;
        setForm((current) => ({ ...formFromQueryEcho(decoded.queryEcho, current), windowSource: "archive" }));
      } catch (caught: unknown) {
        if (!request.isCurrent() || token !== seq.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, t(locale, "report.fail")));
        setStatusZh(invokeErrorZh(caught, t(locale, "report.fail")));
      } finally {
        if (request.isCurrent() && token === seq.current) {
          setLoading(false);
        }
      }
    }),
    [applyDecoded, archives, locale, queue]
  );

  const setArchiveKindFilter = useCallback((filter: ArchiveKindFilter): void => {
    setArchiveKindFilterState(filter);
  }, []);

  const previewExport = useCallback(
    (spec: ExportSpec): Promise<void> => queue.request(`preview:${JSON.stringify(spec)}`, async (request) => {
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
        if (!request.isCurrent() || token !== seq.current) {
          return;
        }
        setExportPreview(next);
        setErrorZh(null);
      } catch (caught: unknown) {
        if (!request.isCurrent() || token !== seq.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, fallback));
      }
    }, false),
    [locale, queue, report]
  );

  const getStored = useCallback(
    (tokenStr: string): Promise<void> => queue.request(`stored:${tokenStr}`, async (request) => {
      const token = ++seq.current;
      const fallback = t(locale, "report.fail");
      if (!isTauriRuntime() || tokenStr !== tokenRef.current) {
        setErrorZh(fallback);
        return;
      }
      try {
        const decoded = decodeReportResult(await invoke<unknown>("get_report", { token: tokenStr }));
        if (!request.isCurrent() || token !== seq.current) {
          return;
        }
        await applyDecoded(
          decoded,
          formatTemplate(t(locale, "report.done"), { token: decoded.reportSnapshotToken.slice(0, 8) }),
          reportSource,
          selectedArchiveId,
          request,
          false
        );
      } catch (caught: unknown) {
        if (!request.isCurrent() || token !== seq.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, fallback));
      }
    }, false),
    [applyDecoded, locale, queue, reportSource, selectedArchiveId]
  );

  const release = useCallback(async (): Promise<void> => {
    const token = tokenRef.current;
    tokenRef.current = null;
    if (token && token === exportingToken.current) {
      deferredRelease.current = token;
      return;
    }
    await releaseReportToken(token);
  }, []);

  useEffect(() => {
    if (!active) {
      setLoading(false);
      void release();
    }
  }, [active, release]);

  useEffect(() => {
    return () => { void release(); };
  }, [release]);

  const exportReport = useCallback(
    (spec: ExportSpec): Promise<void> => queue.request(`export:${JSON.stringify(spec)}`, async (request) => {
      const token = ++seq.current;
      const fallback = t(locale, "report.export_fail");
      if (!report || !tokenRef.current) {
        setStatusZh(t(locale, "report.need_run"));
        return;
      }
      if (!isTauriRuntime()) {
        setErrorZh(fallback);
        return;
      }
      const snapshotToken = tokenRef.current;
      exportingToken.current = snapshotToken;
      try {
        const picked = await invoke<string | null>("pick_file", {
          purpose: "report-export",
          mode: "save",
          locale
        });
        if (!picked) {
          if (request.isCurrent() && token === seq.current) {
            setStatusZh(t(locale, "report.export_cancel"));
          }
          return;
        }
        await invoke("export_report", {
          token: snapshotToken,
          spec,
          path: picked
        });
        if (!request.isCurrent() || token !== seq.current) {
          return;
        }
        setErrorZh(null);
        setStatusZh(formatTemplate(t(locale, "report.exported"), { format: spec.format.toUpperCase() }));
      } catch (caught: unknown) {
        if (!request.isCurrent() || token !== seq.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, fallback));
        setStatusZh(fallback);
      } finally {
        exportingToken.current = null;
        const abandoned = deferredRelease.current;
        deferredRelease.current = null;
        await releaseReportToken(abandoned);
      }
    }, false),
    [locale, queue, report]
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
    html,
    htmlError,
    exportPreview,
    sort,
    setForm,
    setTopN,
    setCompare,
    setArchiveKindFilter,
    loadArchives,
    selectArchive,
    runManual,
    runQuery,
    applyRankSort,
    restoreResidentialManual,
    previewExport,
    exportReport,
    getStored,
    release
  };
}
