import type { AlertInstance } from "../../../dto";
import type { AlertStatusFilter } from "../../../hooks/use-alerts";
import { formatUtc } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { cn } from "../../../lib/utils";
import { Button } from "../../ui/button";
import { fieldClass, labelClass } from "../form-styles";

const STATUSES: AlertStatusFilter[] = [
  "all",
  "inactive",
  "active",
  "not-evaluable",
  "resolved",
  "superseded"
];

function statusLabel(locale: UiLocale, status: string): string {
  if (status === "all") {
    return t(locale, "alerts.filter.all");
  }
  return t(locale, `alerts.status.${status}`);
}

export function CenterList({
  locale,
  items,
  filter,
  selectedId,
  hasMore,
  onFilter,
  onSelect,
  onMore
}: {
  locale: UiLocale;
  items: AlertInstance[];
  filter: AlertStatusFilter;
  selectedId: string | null;
  hasMore: boolean;
  onFilter: (filter: AlertStatusFilter) => void;
  onSelect: (item: AlertInstance) => void;
  onMore: () => void;
}) {
  const unknown = t(locale, "common.unknown");
  return (
    <section className="space-y-3">
      <div className="flex flex-wrap items-end gap-3">
        <label className={labelClass}>
          {t(locale, "alerts.filter.status")}
          <select
            className={cn(fieldClass, "max-w-48")}
            value={filter}
            onChange={(event) => onFilter(event.target.value as AlertStatusFilter)}
          >
            {STATUSES.map((item) => (
              <option key={item} value={item}>
                {statusLabel(locale, item)}
              </option>
            ))}
          </select>
        </label>
      </div>
      <div className="overflow-auto rounded-md border">
        <table className="w-full text-sm">
          <thead className="bg-muted/40">
            <tr>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "alerts.col.rule")}</th>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "alerts.col.status")}</th>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "alerts.col.target")}</th>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "alerts.col.observed")}</th>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "alerts.col.coverage")}</th>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "alerts.col.resolved")}</th>
              <th className="px-2 py-1.5 text-left font-medium">{t(locale, "alerts.col.noteval")}</th>
            </tr>
          </thead>
          <tbody>
            {items.length === 0 ? (
              <tr>
                <td className="px-2 py-2 text-muted-foreground" colSpan={7}>
                  {t(locale, "alerts.empty")}
                </td>
              </tr>
            ) : (
              items.map((item) => (
                <tr
                  key={item.instanceId}
                  tabIndex={0}
                  className={cn(
                    "cursor-pointer hover:bg-muted/40",
                    selectedId === item.instanceId && "bg-primary/10"
                  )}
                  onClick={() => onSelect(item)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter" || event.key === " ") {
                      event.preventDefault();
                      onSelect(item);
                    }
                  }}
                >
                  <td className="px-2 py-1.5">
                    {item.ruleId} v{item.ruleVersion}
                  </td>
                  <td className="px-2 py-1.5">{statusLabel(locale, item.status)}</td>
                  <td className="px-2 py-1.5">{item.selectorIdentity}</td>
                  <td className="px-2 py-1.5 tabular-nums">
                    {item.evidence.observedValue === null ? unknown : String(item.evidence.observedValue)}
                  </td>
                  <td className="px-2 py-1.5">{item.evidence.coverageSummary}</td>
                  <td className="px-2 py-1.5">
                    {item.resolvedUtc === null ? t(locale, "report.dash") : formatUtc(item.resolvedUtc)}
                  </td>
                  <td className="px-2 py-1.5">{item.evidence.notEvaluableReason ?? t(locale, "report.dash")}</td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
      {hasMore ? (
        <Button type="button" variant="outline" onClick={onMore}>
          {t(locale, "alerts.load_more")}
        </Button>
      ) : null}
    </section>
  );
}
