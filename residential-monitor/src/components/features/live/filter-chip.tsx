import type { LiveFilterClause } from "../../../ipc/live-session";
import { unitLabelKey } from "../../../format/live-filter-units";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";

export function clauseLabel(locale: UiLocale, clause: LiveFilterClause): string {
  const unit = clause.unit ? ` ${t(locale, unitLabelKey(clause.field, clause.unit))}` : "";
  const value = clause.value.trim() || t(locale, "common.none");
  return `${t(locale, `live.filter.field.${clause.field}`)} ${t(locale, `live.filter.${clause.mode}`)} ${value}${unit}`;
}

export function FilterChip({
  locale,
  clause,
  onEdit,
  onRemove
}: {
  locale: UiLocale;
  clause: LiveFilterClause;
  onEdit: () => void;
  onRemove: () => void;
}) {
  const label = clauseLabel(locale, clause);
  return (
    <li className="flex max-w-full items-center gap-1 rounded-full border bg-muted/40 px-2 py-1 text-xs">
      <span className="truncate">{label}</span>
      <Button type="button" variant="ghost" size="sm" className="h-6 px-2" onClick={onEdit} aria-label={`${t(locale, "live.filter.edit")} ${label}`}>
        {t(locale, "live.filter.edit")}
      </Button>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        className="h-6 px-2"
        onClick={onRemove}
        aria-label={`${t(locale, "live.filter.remove")} ${label}`}
      >
        {t(locale, "live.filter.remove")}
      </Button>
    </li>
  );
}
