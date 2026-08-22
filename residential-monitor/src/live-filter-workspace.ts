import type { LiveConnectionQuery, LiveFilterClause } from "./ipc/live-session";

export type LiveFilterState = LiveConnectionQuery["filter"];
export type FilterEditorKeyAction = "apply" | "cancel" | "none";

/** 草稿与已提交查询分离，按键不得触发查询。 */
export function cloneLiveFilter(filter: LiveFilterState): LiveFilterState {
  return {
    ...filter,
    clauses: filter.clauses.map((clause) => ({ ...clause }))
  };
}

export function applyLiveFilterDraft(
  query: LiveConnectionQuery,
  draft: LiveFilterState
): LiveConnectionQuery {
  return {
    ...query,
    cursor: null,
    filter: cloneLiveFilter(draft)
  };
}

export function removeLiveFilterClause(filter: LiveFilterState, index: number): LiveFilterState {
  if (!Number.isInteger(index) || index < 0 || index >= filter.clauses.length) {
    return cloneLiveFilter(filter);
  }
  return {
    ...filter,
    clauses: filter.clauses.filter((_, item) => item !== index)
  };
}

export function clearLiveFilterClauses(filter: LiveFilterState): LiveFilterState {
  return { ...cloneLiveFilter(filter), clauses: [] };
}

export function appendLiveFilterClause(
  filter: LiveFilterState,
  clause: LiveFilterClause,
  maxClauses = 8
): LiveFilterState {
  if (filter.clauses.length >= maxClauses) {
    return cloneLiveFilter(filter);
  }
  return { ...cloneLiveFilter(filter), clauses: [...filter.clauses, { ...clause }] };
}

/** 单调 token，用于丢弃过期的查询响应。 */
export function nextLiveRequestToken(current: number): number {
  return current >= Number.MAX_SAFE_INTEGER ? 1 : current + 1;
}

export function isCurrentLiveRequest(requestToken: number, currentToken: number): boolean {
  return requestToken === currentToken;
}

export function filterEditorKeyAction(key: string, isTextarea: boolean): FilterEditorKeyAction {
  if (key === "Escape") {
    return "cancel";
  }
  if (key === "Enter" && !isTextarea) {
    return "apply";
  }
  return "none";
}

export function shouldApplyFilterEditorOnBlur(input: {
  editorConnected: boolean;
  focusInsideEditor: boolean;
  editorIndex: number;
  openEditor: number | null;
}): boolean {
  return input.editorConnected && !input.focusInsideEditor && input.openEditor === input.editorIndex;
}
