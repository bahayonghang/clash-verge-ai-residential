import type { FocusEvent } from "react";
import { formatTemplate } from "../../../lib/utils";
import { t, type UiLocale } from "../../../i18n";
import type { LiveEmptyKind } from "../../../ipc/live-empty";
import {
  shouldApplyFilterEditorOnBlur,
  type LiveFilterState
} from "../../../live-filter-workspace";
import { Button } from "../../ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "../../ui/card";
import { Switch } from "../../ui/switch";
import { FilterChip } from "./filter-chip";
import { FilterEditor } from "./filter-editor";

export type LiveFilterStatus = "idle" | "applying" | "failed";

export function liveFilterStatusText(
  locale: UiLocale,
  status: LiveFilterStatus,
  applied: LiveFilterState,
  pageCount: number,
  matchedCount: number | null,
  emptyKind: LiveEmptyKind
): string {
  if (status === "applying") {
    return t(locale, "live.filter.applying");
  }
  if (status === "failed") {
    return t(locale, "live.filter.failed");
  }
  const hasActiveFilter = applied.residentialOnly || applied.clauses.length > 0;
  if (hasActiveFilter && matchedCount === 0 && emptyKind === "connectedEmpty") {
    return t(locale, "live.filter.no_match");
  }
  return formatTemplate(t(locale, "live.filter.current_page"), { count: pageCount });
}

/** 仅在筛选提交后的 loading 边沿结束 applying，排序/增量刷新不得改写该状态。 */
export function settleFilterStatus(
  status: LiveFilterStatus,
  loading: boolean,
  queryFailed: boolean,
  sawLoading: boolean
): { status: LiveFilterStatus; sawLoading: boolean } {
  if (loading) {
    return { status, sawLoading: status === "applying" ? true : sawLoading };
  }
  if (sawLoading && status === "applying") {
    return { status: queryFailed ? "failed" : "idle", sawLoading: false };
  }
  return { status, sawLoading };
}

export function FilterWorkspace({
  locale,
  applied,
  draft,
  editorIndex,
  filterStatus,
  pageCount,
  matchedCount,
  emptyKind,
  onDraftChange,
  onApply,
  onCancel,
  onAdd,
  onClear,
  onEdit,
  onRemove,
  onResidential
}: {
  locale: UiLocale;
  applied: LiveFilterState;
  draft: LiveFilterState;
  editorIndex: number | null;
  filterStatus: LiveFilterStatus;
  pageCount: number;
  matchedCount: number | null;
  emptyKind: LiveEmptyKind;
  onDraftChange: (draft: LiveFilterState) => void;
  onApply: () => void;
  onCancel: () => void;
  onAdd: () => void;
  onClear: () => void;
  onEdit: (index: number) => void;
  onRemove: (index: number) => void;
  onResidential: (value: boolean) => void;
}) {
  const editorClause = editorIndex !== null ? draft.clauses[editorIndex] : undefined;
  const onEditorBlur = (event: FocusEvent<HTMLDivElement>): void => {
    if (editorIndex === null) {
      return;
    }
    const next = event.relatedTarget;
    const focusInsideEditor = next instanceof Node && event.currentTarget.contains(next);
    if (
      shouldApplyFilterEditorOnBlur({
        editorConnected: event.currentTarget.isConnected,
        focusInsideEditor,
        editorIndex,
        openEditor: editorIndex
      })
    ) {
      onApply();
    }
  };

  return (
    <Card className="min-w-0">
      <CardHeader className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <CardTitle id="live-filter-title" className="text-sm">
            {t(locale, "live.filter.title")}
          </CardTitle>
          <p className="text-xs text-muted-foreground">
            {formatTemplate(t(locale, "live.filter.applied_count"), { count: applied.clauses.length })}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <label className="inline-flex items-center gap-2 text-sm">
            <Switch
              checked={draft.residentialOnly}
              onCheckedChange={onResidential}
              aria-label={t(locale, "live.filter.residential")}
            />
            {t(locale, "live.filter.residential")}
          </label>
          <Button type="button" variant="outline" size="sm" disabled={draft.clauses.length >= 8} onClick={onAdd}>
            {t(locale, "live.filter.add")}
          </Button>
          <Button type="button" variant="outline" size="sm" disabled={applied.clauses.length === 0} onClick={onClear}>
            {t(locale, "live.filter.clear")}
          </Button>
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <p className="text-xs" data-state={filterStatus} role="status" aria-live="polite">
          {liveFilterStatusText(locale, filterStatus, applied, pageCount, matchedCount, emptyKind)}
        </p>
        <div>
          <h3 className="mb-2 text-xs font-medium">{t(locale, "live.filter.applied_heading")}</h3>
          {applied.clauses.length > 0 ? (
            <ul className="flex flex-wrap gap-2">
              {applied.clauses.map((clause, index) => (
                <FilterChip
                  key={`${clause.field}-${index}`}
                  locale={locale}
                  clause={clause}
                  onEdit={() => onEdit(index)}
                  onRemove={() => onRemove(index)}
                />
              ))}
            </ul>
          ) : (
            <p className="text-xs text-muted-foreground">{t(locale, "live.filter.empty")}</p>
          )}
        </div>
        {editorClause && editorIndex !== null ? (
          <div onBlur={onEditorBlur}>
            <FilterEditor
              locale={locale}
              index={editorIndex}
              clause={editorClause}
              onChange={(clause) => {
                const clauses = draft.clauses.map((item, itemIndex) => (itemIndex === editorIndex ? clause : item));
                onDraftChange({ ...draft, clauses });
              }}
              onApply={onApply}
              onCancel={onCancel}
            />
          </div>
        ) : null}
      </CardContent>
    </Card>
  );
}
