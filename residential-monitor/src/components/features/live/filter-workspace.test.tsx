import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { defaultLiveQuery } from "../../../ipc/live-session";
import {
  FilterWorkspace,
  liveFilterStatusText,
  settleFilterStatus
} from "./filter-workspace";

const noop = (): void => undefined;
const applied = defaultLiveQuery().filter;

describe("FilterWorkspace 状态文案", () => {
  it("筛选无匹配使用专门文案，不合并成无数据", () => {
    const filtered = {
      ...applied,
      clauses: [{ field: "host", mode: "contains" as const, value: "nope.example" }]
    };
    expect(
      liveFilterStatusText("zh", "idle", filtered, 0, 0, "connectedEmpty")
    ).toBe("没有连接匹配已应用条件。");
    const html = renderToStaticMarkup(
      <FilterWorkspace
        locale="zh"
        applied={filtered}
        draft={filtered}
        editorIndex={null}
        filterStatus="idle"
        pageCount={0}
        matchedCount={0}
        emptyKind="connectedEmpty"
        onDraftChange={noop}
        onApply={noop}
        onCancel={noop}
        onAdd={noop}
        onClear={noop}
        onEdit={noop}
        onRemove={noop}
        onResidential={noop}
      />
    );
    expect(html).toContain("没有连接匹配已应用条件");
    expect(html).toContain("只看家宽");
  });

  it("排序加载不得把筛选状态写成 applying", () => {
    expect(settleFilterStatus("idle", true, false, false)).toEqual({
      status: "idle",
      sawLoading: false
    });
    expect(settleFilterStatus("applying", true, false, false)).toEqual({
      status: "applying",
      sawLoading: true
    });
    expect(settleFilterStatus("applying", false, false, true)).toEqual({
      status: "idle",
      sawLoading: false
    });
    expect(settleFilterStatus("applying", false, true, true)).toEqual({
      status: "failed",
      sawLoading: false
    });
    expect(settleFilterStatus("applying", false, false, false)).toEqual({
      status: "applying",
      sawLoading: false
    });
  });
});
