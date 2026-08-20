import { describe, expect, it } from "vitest";
import { defaultLiveQuery, type LiveFilterClause } from "./ipc/live-session";
import {
  appendLiveFilterClause,
  applyLiveFilterDraft,
  clearLiveFilterClauses,
  cloneLiveFilter,
  filterEditorKeyAction,
  isCurrentLiveRequest,
  nextLiveRequestToken,
  removeLiveFilterClause,
  shouldApplyFilterEditorOnBlur
} from "./live-filter-workspace";

const hostClause: LiveFilterClause = { field: "host", mode: "contains", value: "example.com" };

describe("live filter workspace", () => {
  it("keeps a draft detached until apply resets the cursor", () => {
    const query = { ...defaultLiveQuery(), cursor: { sortKey: "later", identity: "connection-1" } };
    const draft = cloneLiveFilter(query.filter);
    draft.clauses.push(hostClause);

    expect(query.filter.clauses).toEqual([]);
    expect(applyLiveFilterDraft(query, draft)).toMatchObject({
      cursor: null,
      filter: { clauses: [hostClause] }
    });
  });

  it("removes or clears clauses without changing the quick residential switch", () => {
    const filter = {
      ...defaultLiveQuery().filter,
      residentialOnly: false,
      clauses: [hostClause, { field: "process", mode: "exact" as const, value: "mihomo" }]
    };

    expect(removeLiveFilterClause(filter, 0).clauses).toEqual([filter.clauses[1]]);
    expect(removeLiveFilterClause(filter, 9)).toEqual(filter);
    expect(clearLiveFilterClauses(filter)).toEqual({ ...filter, clauses: [] });
  });

  it("caps draft clauses at eight and issues monotonic response tokens", () => {
    const full = {
      ...defaultLiveQuery().filter,
      clauses: Array.from({ length: 8 }, (_, index) => ({
        field: "host",
        mode: "contains" as const,
        value: String(index)
      }))
    };

    expect(appendLiveFilterClause(full, hostClause)).toEqual(full);
    expect(nextLiveRequestToken(4)).toBe(5);
    expect(nextLiveRequestToken(Number.MAX_SAFE_INTEGER)).toBe(1);
    expect(isCurrentLiveRequest(4, 5)).toBe(false);
    expect(isCurrentLiveRequest(5, 5)).toBe(true);
  });

  it("applies on Enter or blur, and restores the draft on Escape", () => {
    expect(filterEditorKeyAction("Enter", false)).toBe("apply");
    expect(filterEditorKeyAction("Enter", true)).toBe("none");
    expect(filterEditorKeyAction("Escape", false)).toBe("cancel");
    expect(filterEditorKeyAction("a", false)).toBe("none");
    expect(
      shouldApplyFilterEditorOnBlur({
        editorConnected: true,
        focusInsideEditor: false,
        editorIndex: 0,
        openEditor: 0
      })
    ).toBe(true);
    expect(
      shouldApplyFilterEditorOnBlur({
        editorConnected: true,
        focusInsideEditor: true,
        editorIndex: 0,
        openEditor: 0
      })
    ).toBe(false);
    expect(
      shouldApplyFilterEditorOnBlur({
        editorConnected: true,
        focusInsideEditor: false,
        editorIndex: 0,
        openEditor: null
      })
    ).toBe(false);
  });
});
