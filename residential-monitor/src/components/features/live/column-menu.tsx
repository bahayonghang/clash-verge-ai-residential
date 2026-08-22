import { Columns3 } from "lucide-react";
import { t, type UiLocale } from "../../../i18n";
import {
  columnLabelKey,
  DATA_COLUMNS,
  defaultLiveTableLayout,
  setColumnHidden,
  visibleDataColumns,
  type DataColumnId,
  type LiveTableLayout
} from "../../../live-table-layout";
import { Button } from "../../ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "../../ui/popover";

export function ColumnMenu({
  locale,
  layout,
  onLayoutChange
}: {
  locale: UiLocale;
  layout: LiveTableLayout;
  onLayoutChange: (layout: LiveTableLayout) => void;
}) {
  const visible = visibleDataColumns(layout);
  return (
    <Popover>
      <PopoverTrigger asChild>
        <Button type="button" variant="outline" size="sm">
          <Columns3 className="size-4" />
          {t(locale, "live.columns")}
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="flex w-64 flex-col gap-2">
        <p className="text-sm font-medium">{t(locale, "live.columns.panel")}</p>
        <ul className="flex flex-col gap-1">
          {DATA_COLUMNS.map((column) => {
            const checked = !layout.hidden.includes(column);
            const lastVisible = visible.length === 1 && visible[0] === column;
            return (
              <li key={column}>
                <label className="flex items-center gap-2 text-sm">
                  <input
                    type="checkbox"
                    className="size-4 accent-primary"
                    checked={checked}
                    disabled={lastVisible && checked}
                    onChange={(event) => {
                      const next = setColumnHidden(layout, column as DataColumnId, !event.target.checked);
                      onLayoutChange(next);
                    }}
                  />
                  {t(locale, columnLabelKey(column))}
                </label>
              </li>
            );
          })}
        </ul>
        <Button type="button" variant="secondary" size="sm" onClick={() => onLayoutChange(defaultLiveTableLayout())}>
          {t(locale, "live.columns.reset")}
        </Button>
      </PopoverContent>
    </Popover>
  );
}
