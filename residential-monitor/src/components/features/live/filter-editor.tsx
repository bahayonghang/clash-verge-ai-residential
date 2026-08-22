import type { KeyboardEvent } from "react";
import {
  clauseForField,
  defaultFilterUnit,
  isNumericFilterField,
  NUMERIC_FILTER_FIELDS,
  NUMERIC_MODES,
  unitLabelKey,
  unitsForField
} from "../../../format/live-filter-units";
import type { LiveFilterClause } from "../../../ipc/live-session";
import { t, type UiLocale } from "../../../i18n";
import { filterEditorKeyAction } from "../../../live-filter-workspace";
import { Button } from "../../ui/button";
import { Input } from "../../ui/input";

export const FILTER_FIELDS = [
  "host",
  "chain",
  "rule",
  "process",
  "source",
  "destination",
  "type",
  ...NUMERIC_FILTER_FIELDS
] as const;

const TEXT_MODES = ["contains", "exact"] as const;

const SELECT_CLASS =
  "border-input flex h-9 min-w-[8rem] rounded-md border bg-transparent px-2 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]";

export function FilterEditor({
  locale,
  index,
  clause,
  onChange,
  onApply,
  onCancel
}: {
  locale: UiLocale;
  index: number;
  clause: LiveFilterClause;
  onChange: (clause: LiveFilterClause) => void;
  onApply: () => void;
  onCancel: () => void;
}) {
  const numeric = isNumericFilterField(clause.field);
  const modes = numeric ? NUMERIC_MODES : TEXT_MODES;
  const unit = clause.unit ?? defaultFilterUnit(clause.field);

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>): void => {
    const action = filterEditorKeyAction(event.key, event.target instanceof HTMLTextAreaElement);
    if (action === "cancel") {
      event.preventDefault();
      onCancel();
      return;
    }
    if (action === "apply") {
      event.preventDefault();
      onApply();
    }
  };

  return (
    <div
      className="live-filter-editor flex flex-col gap-3 rounded-lg border bg-muted/20 p-3"
      data-filter-editor={index}
      onKeyDown={onKeyDown}
    >
      <p className="text-xs text-muted-foreground">{t(locale, "live.filter.draft")}</p>
      <div className="flex flex-wrap items-end gap-3">
        <label className="flex min-w-[8rem] flex-col gap-1 text-xs">
          {t(locale, "live.filter.field")}
          <select
            id={`live-filter-field-${index}`}
            className={SELECT_CLASS}
            value={clause.field}
            onChange={(event) => onChange(clauseForField(event.target.value))}
          >
            {FILTER_FIELDS.map((field) => (
              <option key={field} value={field}>
                {t(locale, `live.filter.field.${field}`)}
              </option>
            ))}
          </select>
        </label>
        <label className="flex min-w-[8rem] flex-col gap-1 text-xs">
          {t(locale, "live.filter.mode")}
          <select
            id={`live-filter-mode-${index}`}
            className={SELECT_CLASS}
            value={clause.mode}
            onChange={(event) =>
              onChange({ ...clause, mode: event.target.value as LiveFilterClause["mode"] })
            }
          >
            {modes.map((mode) => (
              <option key={mode} value={mode}>
                {t(locale, `live.filter.${mode}`)}
              </option>
            ))}
          </select>
        </label>
        <label className="flex min-w-[10rem] flex-1 flex-col gap-1 text-xs">
          {t(locale, "live.filter.value")}
          <Input
            id={`live-filter-value-${index}`}
            type={numeric ? "number" : "text"}
            min={numeric ? 0 : undefined}
            step={numeric ? "any" : undefined}
            value={clause.value}
            onChange={(event) => onChange({ ...clause, value: event.target.value })}
          />
        </label>
        {numeric ? (
          <label className="flex min-w-[8rem] flex-col gap-1 text-xs">
            {t(locale, "live.filter.unit")}
            <select
              id={`live-filter-unit-${index}`}
              className={SELECT_CLASS}
              value={unit}
              onChange={(event) => onChange({ ...clause, unit: event.target.value })}
            >
              {unitsForField(clause.field).map((item) => (
                <option key={item} value={item}>
                  {t(locale, unitLabelKey(clause.field, item))}
                </option>
              ))}
            </select>
          </label>
        ) : null}
      </div>
      <div className="flex gap-2">
        <Button type="button" size="sm" onClick={onApply}>
          {t(locale, "live.filter.apply")}
        </Button>
        <Button type="button" variant="secondary" size="sm" onClick={onCancel}>
          {t(locale, "live.filter.cancel")}
        </Button>
      </div>
    </div>
  );
}
