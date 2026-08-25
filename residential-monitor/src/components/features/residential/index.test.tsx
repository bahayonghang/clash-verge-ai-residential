import { describe, expect, it } from "vitest";
import { residentialHealthState } from "./monitor-section";

const sources = import.meta.glob(["../../../app.tsx", "./*.tsx"], {
  query: "?raw",
  eager: true,
  import: "default"
}) as Record<string, string>;

function source(suffix: string): string {
  return Object.entries(sources).find(([file]) => file.endsWith(suffix))?.[1] ?? "";
}

describe("家宽页装配", () => {
  it("app 只增加 residential 分支", () => {
    const app = source("app.tsx");
    expect(app).toContain('case "residential"');
    expect(app).toContain("<ResidentialPage");
    expect(app).toContain('className={route === "live" ? "contents" : "hidden"}');
  });

  it("实时段 residential_only，聚合与报告按家宽子集拆分 Host", () => {
    const monitor = source("monitor-section.tsx");
    const aggregate = source("aggregate-section.tsx");
    const report = source("report-section.tsx");
    expect(monitor).toContain("residentialOnly: true");
    expect(monitor).toContain("useLivePage");
    expect(aggregate).toContain('grouping: "host"');
    expect(aggregate).toContain("residentialReportFilters()");
    expect(aggregate).toContain("matchesResidentialRankQuery");
    expect(report).toContain('grouping: "host"');
    expect(report).toContain("residentialReportFilters()");
    expect(report).toContain('currentOn ? "current" : "historical"');
    expect(report).toContain("drilldownCapability.currentPolicy");
    expect(report).toContain("residential.report.current_off");
  });

  it("趋势图保留旧到新输入，趋势表使用独立新到旧投影", () => {
    const aggregate = source("aggregate-section.tsx");
    const table = source("trend-table.tsx");
    expect(aggregate).toContain("data={series}");
    expect(aggregate).toContain("<TrendTable");
    expect(table).toContain("newestFirstSeries(series)");
    expect(table).toContain("sticky top-0");
    expect(table).toContain("overflow-auto rounded-md border");
  });

  it("未配置 targets 走中文下一步，不画 0", () => {
    const page = source("index.tsx");
    expect(page).toContain("targetCount === 0");
    expect(page).toContain("TargetEmpty");
    expect(source("target-empty.tsx")).toContain("residential.targets.next");
  });

  it("口径说明出现在实时段与聚合段", () => {
    expect(source("monitor-section.tsx")).toContain('kind="filter"');
    expect(source("aggregate-section.tsx")).toContain('kind="accounting"');
    expect(source("caliber-note.tsx")).toContain("residential.caliber.filter");
    expect(source("caliber-note.tsx")).toContain("residential.caliber.accounting");
  });

  it("暂停、未连接、缺口互斥，且不把占比未知绑到实时缺口", () => {
    expect(residentialHealthState("paused")).toBe("paused");
    expect(residentialHealthState("disconnected")).toBe("disconnected");
    expect(residentialHealthState("gap")).toBe("gap");
    expect(residentialHealthState("ready")).toBeNull();
    expect(residentialHealthState("unconfigured")).toBeNull();
    const monitor = source("monitor-section.tsx");
    expect(monitor).toContain("residential.state.${healthState}");
    expect(source("index.tsx")).not.toContain("residential.state.gap");
  });
});
