import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ReportQuery, ReportResult } from "../dto";
import { defaultLiveQuery } from "../ipc/live-session";
import { useLivePage, type UseLivePageInput } from "./use-live-page";
import { useReport, type UseReportInput } from "./use-report";
import { defaultExportSpec, useReportArchive } from "./use-report-archive";
import { useResidentialShare } from "./use-residential-share";
import { useSettings } from "./use-settings";

// 无 DOM 依赖的 effect 驱动器：执行真实 hook、真实队列和真实 IPC decoder。
// 原生 WebView/React DOM 行为不由此夹具替代。
const runtime = vi.hoisted(() => ({
  slots: [] as unknown[], index: 0, dirty: false, visible: true,
  effects: [] as (() => void)[], cleanups: new Map<number, (() => void) | void>()
}));
vi.mock("react", async (original) => {
  const react = await original<typeof import("react")>();
  const memo = <T,>(make: () => T, deps: readonly unknown[]): T => {
    const index = runtime.index++;
    const old = runtime.slots[index] as { value: T; deps: readonly unknown[] } | undefined;
    if (!old || deps.some((value, i) => !Object.is(value, old.deps[i]))) {
      runtime.slots[index] = { value: make(), deps };
    }
    return (runtime.slots[index] as { value: T }).value;
  };
  return {
    ...react,
    useContext: () => runtime.visible,
    useRef: <T,>(value: T) => {
      const index = runtime.index++;
      runtime.slots[index] ??= { current: value };
      return runtime.slots[index];
    },
    useState: <T,>(initial: T | (() => T)) => {
      const index = runtime.index++;
      if (!(index in runtime.slots)) {
        runtime.slots[index] = typeof initial === "function" ? (initial as () => T)() : initial;
      }
      return [runtime.slots[index], (next: T | ((old: T) => T)) => {
        const previous = runtime.slots[index] as T;
        const value = typeof next === "function" ? (next as (old: T) => T)(previous) : next;
        if (!Object.is(previous, value)) {
          runtime.slots[index] = value;
          runtime.dirty = true;
        }
      }];
    },
    useMemo: memo,
    useCallback: <T,>(callback: T, deps: readonly unknown[]) => memo(() => callback, deps),
    useEffect: (effect: () => (() => void) | void, deps: readonly unknown[]) => {
      const index = runtime.index++;
      const old = runtime.slots[index] as readonly unknown[] | undefined;
      if (!old || deps.some((value, i) => !Object.is(value, old[i]))) {
        runtime.slots[index] = deps;
        runtime.effects.push(() => {
          runtime.cleanups.get(index)?.();
          runtime.cleanups.set(index, effect());
        });
      }
    }
  };
});
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("../ipc/live-session", async (original) => ({
  ...await original<typeof import("../ipc/live-session")>(), isTauriRuntime: () => true
}));

function mount<T>(hook: () => T) {
  let value!: T;
  const render = (): void => {
    runtime.dirty = false;
    runtime.index = 0;
    value = hook();
    for (const effect of runtime.effects.splice(0)) effect();
  };
  render();
  return {
    get value() { return value; },
    render,
    async flush() {
      for (let i = 0; i < 50; i += 1) {
        await Promise.resolve();
        if (runtime.dirty) render();
      }
    },
    hide() { runtime.visible = false; render(); },
    show() { runtime.visible = true; render(); },
    unmount() { for (const cleanup of runtime.cleanups.values()) cleanup?.(); }
  };
}

function report(query: ReportQuery, token: string): ReportResult {
  return {
    schemaVersion: 1, dataVersion: 1, reportSnapshotToken: token, queryEcho: query,
    totals: { upload: 0, download: 0, connectionCount: 0, activeDurationSec: 0, previousUpload: null, previousDownload: null },
    series: [], rankings: [], coverage: { status: "ok", coveredSec: 60, gapSec: 0, slices: [] },
    attributionQuality: { knownUpload: 0, knownDownload: 0, missingUpload: 0, missingDownload: 0,
      knownConnections: 0, missingConnections: 0, status: "complete" },
    drilldownCapability: { sessions: true, currentPolicy: true, crossDimension: true, exactTopN: true, noteZh: "" },
    policyMetadata: { targetPolicy: "historical", policyVersion: null, noteZh: "" },
    dataTier: "Raw", namedSql: ["rank_raw"], unit: "byte", generatedUtc: 1
  };
}

beforeEach(() => {
  runtime.slots = []; runtime.index = 0; runtime.dirty = false; runtime.visible = true;
  runtime.effects = []; runtime.cleanups.clear();
  vi.mocked(invoke).mockReset().mockResolvedValue(undefined);
});

describe("真实展示 hook 的 IPC 生命周期", () => {
  it("report 慢响应只补最新窗口；相同 token 的每次获取都有独立释放", async () => {
    let input: UseReportInput = { grouping: "host", timeRange: { preset: "1h", startUtc: 0, endUtc: 60_000 }, granularity: "minute1", topN: 10 };
    const pending: { query: ReportQuery; finish: (raw: unknown) => void }[] = [];
    vi.mocked(invoke).mockImplementation((cmd, args) => cmd === "run_report"
      ? new Promise((finish) => pending.push({ query: (args as { query: ReportQuery }).query, finish }))
      : Promise.resolve());
    const hook = mount(() => useReport(input));
    await hook.flush();
    for (const endUtc of [120_000, 180_000]) {
      input = { ...input, timeRange: { ...input.timeRange, endUtc } };
      hook.render();
    }
    await hook.flush();
    expect(pending).toHaveLength(1);
    pending[0].finish(report(pending[0].query, "abandoned"));
    await hook.flush();
    expect(hook.value.result).toBeNull();
    expect(pending).toHaveLength(2);
    expect(pending[1].query.rangeEndUtc).toBe(180);
    pending[1].finish(report(pending[1].query, "shared"));
    await hook.flush();
    hook.hide();
    input = { ...input, timeRange: { ...input.timeRange, endUtc: 240_000 } };
    hook.render();
    await hook.flush();
    expect(pending).toHaveLength(2);
    hook.show(); hook.show();
    await hook.flush();
    expect(pending).toHaveLength(3);
    pending[2].finish(report(pending[2].query, "shared"));
    await hook.flush();
    hook.unmount();
    await hook.flush();
    expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "release_report").map(([, args]) => (args as { token: string }).token))
      .toEqual(["abandoned", "shared", "shared"]);
  });

  it("live 同时激活与 delta 只查询一次；隐藏后的旧页不更新热点或发 tray 查询", async () => {
    let input: UseLivePageInput = { applied: defaultLiveQuery().filter, sort: { sortField: "identity", descending: false }, refreshSignal: 1, locale: "zh" };
    const pending: ((raw: unknown) => void)[] = [];
    vi.mocked(invoke).mockImplementation((cmd) => cmd === "query_live_connections"
      ? new Promise((finish) => pending.push(finish))
      : Promise.resolve({ collectorRunning: true, health: "connected", windowVisible: true }));
    const hook = mount(() => useLivePage(input));
    await hook.flush();
    expect(pending).toHaveLength(1);
    input = { ...input, refreshSignal: 2 }; hook.render();
    hook.hide();
    pending[0]({ rows: [], nextCursor: null, matchedCount: 0, sampleUtc: 1, summary: { topUpload: null, topDownload: null } });
    await hook.flush();
    expect(hook.value.page).toBeNull();
    expect(vi.mocked(invoke).mock.calls.map(([cmd]) => cmd)).toEqual(["query_live_connections"]);
    hook.show();
    await hook.flush();
    expect(pending).toHaveLength(2);
    hook.unmount();
    pending[1]({ rows: [], nextCursor: null, matchedCount: 0, sampleUtc: 2, summary: { topUpload: null, topDownload: null } });
    await hook.flush();
  });

  it("share 保留旧值，慢 IPC 合并最新窗口且隐藏不拉取", async () => {
    let range = { preset: "1h" as const, startUtc: 0, endUtc: 60_000 };
    const pending: ((raw: unknown) => void)[] = [];
    vi.mocked(invoke).mockImplementation((cmd) => cmd === "residential_share"
      ? new Promise((finish) => pending.push(finish)) : Promise.resolve());
    const hook = mount(() => useResidentialShare(range));
    await hook.flush();
    range = { ...range, endUtc: 120_000 }; hook.render();
    range = { ...range, endUtc: 180_000 }; hook.render();
    const payload = { schemaVersion: 1, residentialUpload: 1, residentialDownload: 2,
      attributedUpload: 3, attributedDownload: 4, coverageStatus: "covered", namedSql: [],
      generatedUtc: 1, targetCount: 1, policyVersion: 1 };
    pending[0](payload); await hook.flush();
    expect(hook.value.share).toBeNull();
    expect(pending).toHaveLength(2);
    pending[1](payload); await hook.flush();
    expect(hook.value.share?.residentialDownload).toBe(2);
    hook.hide(); await hook.flush();
    expect(pending).toHaveLength(2);
    hook.unmount();
  });

  it("手动报告恢复不重复持久化，HTML 串行且卸载释放租约", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd === "run_report") return report((args as { query: ReportQuery }).query, "manual");
      if (cmd === "list_report_archives") return { schemaVersion: 1, items: [], next: null };
      if (cmd === "render_report_html") return { html: "<html>report</html>" };
      return undefined;
    });
    const hook = mount(() => useReportArchive("zh", true));
    void hook.value.runManual(); await hook.flush();
    expect(hook.value.report?.reportSnapshotToken).toBe("manual");
    expect(hook.value.html).toContain("report");
    void hook.value.runManual(); await hook.flush();
    expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "release_report")).toHaveLength(1);
    hook.hide(); await hook.flush();
    hook.show(); hook.show(); await hook.flush();
    expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "run_report").map(([, args]) => (args as { persistManual: boolean }).persistManual)).toEqual([true, true, false]);
    hook.unmount(); await hook.flush();
    expect(vi.mocked(invoke).mock.calls.filter(([cmd]) => cmd === "release_report")).toHaveLength(3);
  });

  it("旧档案 HTML 晚返回不能覆盖用户的新表单选择", async () => {
    let html!: (raw: unknown) => void;
    let nextReport!: (raw: unknown) => void;
    let nextQuery!: ReportQuery;
    const summary = { archiveId: "archive", kind: "day", rangeStartUtc: 0, rangeEndUtc: 60,
      displayTimezone: "local", grouping: "host", status: "ok", generatedUtc: 1, dataVersion: 1,
      coverageStatus: "ok", totalsUpload: 0, totalsDownload: 0, connectionCount: 0, errorCode: null, noteZh: null };
    vi.mocked(invoke).mockImplementation((cmd, args) => {
      if (cmd === "list_report_archives") return Promise.resolve({ schemaVersion: 1, items: [summary], next: null });
      if (cmd === "get_report_archive") return Promise.resolve(report({
        rangeStartUtc: 0, rangeEndUtc: 60, displayTimezone: "local", granularity: "minute1",
        filters: { category: null, host: null, process: null, rule: null, chain: null, network: null },
        grouping: "host", targetPolicy: "historical", comparison: null, sort: { field: "download", descending: true },
        page: { limit: 200, after: null }, topN: 10, includeSessions: false
      }, "archive"));
      if (cmd === "render_report_html" && (args as { token: string }).token === "archive") {
        return new Promise((resolve) => { html = resolve; });
      }
      if (cmd === "render_report_html") return Promise.resolve({ html: "<html>new</html>" });
      if (cmd === "run_report") {
        nextQuery = (args as { query: ReportQuery }).query;
        return new Promise((resolve) => { nextReport = resolve; });
      }
      return Promise.resolve();
    });
    const hook = mount(() => useReportArchive("zh", true));
    void hook.value.loadArchives(true); await hook.flush();
    expect(hook.value.report?.reportSnapshotToken).toBe("archive");
    hook.value.setForm({ ...hook.value.form, grouping: "chain" });
    void hook.value.runQuery({ ...hook.value.report!.queryEcho, grouping: "chain" });
    await hook.flush();
    html({ html: "<html>old</html>" }); await hook.flush();
    expect(hook.value.form.grouping).toBe("chain");
    expect(hook.value.html).toBeNull();
    nextReport(report(nextQuery, "new")); await hook.flush();
    hook.unmount(); await hook.flush();
  });

  it("导出对话框期间隐藏仍使用原快照，导出结束才释放本视图租约", async () => {
    let pick!: (path: unknown) => void;
    vi.mocked(invoke).mockImplementation((cmd, args) => {
      if (cmd === "run_report") return Promise.resolve(report((args as { query: ReportQuery }).query, "export-held"));
      if (cmd === "list_report_archives") return Promise.resolve({ schemaVersion: 1, items: [], next: null });
      if (cmd === "pick_file") return new Promise((resolve) => { pick = resolve; });
      return Promise.resolve();
    });
    const hook = mount(() => useReportArchive("zh"));
    void hook.value.runManual(); await hook.flush();
    void hook.value.exportReport(defaultExportSpec()); await hook.flush();
    hook.hide(); await hook.flush();
    expect(vi.mocked(invoke).mock.calls.some(([cmd]) => cmd === "release_report")).toBe(false);
    pick("fixture.csv"); await hook.flush();
    const calls = vi.mocked(invoke).mock.calls;
    expect(calls.find(([cmd]) => cmd === "export_report")?.[1]).toMatchObject({ token: "export-held", path: "fixture.csv" });
    expect(calls.findIndex(([cmd]) => cmd === "release_report")).toBeGreaterThan(calls.findIndex(([cmd]) => cmd === "export_report"));
    hook.show(); await hook.flush();
    expect(calls.filter(([cmd]) => cmd === "pick_file")).toHaveLength(1);
    hook.unmount(); await hook.flush();
  });

  it.each([
    ["runRetention", "run_retention"],
    ["createBackup", "create_backup"],
    ["restoreBackup", "restore_backup"]
  ] as const)("设置 %s 将取消 ID 传给实际工作，并在结束后清理", async (action, command) => {
    let reject!: (reason: unknown) => void;
    const progress = (operationId: string, canCancel = true) => ({
      schemaVersion: 1, operationId, kind: "fixture", phase: "running", current: 0, total: 100,
      unit: "percent", status: canCancel ? "running" : "cancelled", canCancel, redactedError: null
    });
    vi.mocked(invoke).mockImplementation((cmd, args) => {
      const id = (args as { operationId?: string } | undefined)?.operationId ?? "";
      if (cmd === "start_operation") return Promise.resolve(progress(id));
      if (cmd === "cancel_operation") return Promise.resolve(progress(id, false));
      if (cmd === "pick_file") return Promise.resolve("fixture.sqlite3");
      if (cmd === command) return new Promise((_resolve, fail) => { reject = fail; });
      return Promise.resolve();
    });
    const hook = mount(() => useSettings("zh", null));
    void hook.value[action](); await hook.flush();
    const id = hook.value.progress?.operationId;
    expect(id).toMatch(/^op-/);
    expect(vi.mocked(invoke).mock.calls.find(([cmd]) => cmd === command)?.[1]).toMatchObject({ operationId: id });
    void hook.value.cancelOperation(); await hook.flush();
    expect(vi.mocked(invoke).mock.calls.find(([cmd]) => cmd === "cancel_operation")?.[1]).toEqual({ operationId: id });
    expect(vi.mocked(invoke).mock.calls.some(([cmd]) => cmd === "finish_operation")).toBe(false);
    reject({ code: "cancelled" }); await hook.flush();
    expect(vi.mocked(invoke).mock.calls.find(([cmd]) => cmd === "finish_operation")?.[1]).toEqual({ operationId: id });
    hook.unmount();
  });
});
