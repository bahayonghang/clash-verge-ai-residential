import { describe, expect, it } from "vitest";
import {
  applyLiveFilterDraft,
  cloneLiveFilter,
  filterEditorKeyAction,
  isCurrentLiveRequest,
  nextLiveRequestToken,
  shouldApplyFilterEditorOnBlur
} from "../../../live-filter-workspace";
import { defaultLiveQuery } from "../../../ipc/live-session";

const sources = import.meta.glob(["./**/*.ts", "./**/*.tsx"], {
  query: "?raw",
  eager: true,
  import: "default"
}) as Record<string, string>;

describe("live 组件约束", () => {
  it("components/features/live 不得 invoke 或插入 HTML", () => {
    const hits = Object.entries(sources).filter(
      ([file, text]) =>
        !file.endsWith(".test.ts") &&
        !file.endsWith(".test.tsx") &&
        (/\binvoke\s*[<(]/.test(text) || text.includes("dangerouslySetInnerHTML"))
    );
    expect(hits.map(([file]) => file)).toEqual([]);
  });

  it("应用 / 取消 / 失焦 / Escape / 重复提交 / 过期响应六条路径", () => {
    const query = defaultLiveQuery();
    const draft = cloneLiveFilter(query.filter);
    draft.clauses.push({ field: "host", mode: "contains", value: "a.test" });
    expect(applyLiveFilterDraft(query, draft).filter.clauses).toHaveLength(1);
    expect(filterEditorKeyAction("Enter", false)).toBe("apply");
    expect(filterEditorKeyAction("Escape", false)).toBe("cancel");
    expect(
      shouldApplyFilterEditorOnBlur({
        editorConnected: true,
        focusInsideEditor: false,
        editorIndex: 0,
        openEditor: 0
      })
    ).toBe(true);
    const first = nextLiveRequestToken(0);
    const second = nextLiveRequestToken(first);
    expect(isCurrentLiveRequest(first, second)).toBe(false);
    expect(isCurrentLiveRequest(second, second)).toBe(true);
  });

  it("固定列宽表格且无关闭全部入口", () => {
    const table = Object.entries(sources).find(([file]) => file.endsWith("connection-table.tsx"))?.[1];
    expect(table).toBeDefined();
    expect(table).toContain("table-fixed");
    expect(table).toContain("live-table-wrap");
    expect(table).toContain("min-w-0");
    const production = Object.entries(sources).filter(
      ([file]) => !file.endsWith(".test.ts") && !file.endsWith(".test.tsx")
    );
    expect(
      production.some(
        ([, text]) => text.includes("关闭全部") || /close_all|closeAll/.test(text)
      )
    ).toBe(false);
  });
});
