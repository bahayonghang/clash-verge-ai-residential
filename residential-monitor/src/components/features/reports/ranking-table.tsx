import { useMemo, useState, type ReactNode } from "react";
import { inspectKeysMatch, rankingInspectKey } from "../../../format/report-inspect";
import { formatSharePct, type ShareModel, type ShareRow } from "../../../format/report-view";
import { formatBytes } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { cn } from "../../../lib/utils";
import { SortableTh } from "../../common/sortable-th";
import { useReportInspect } from "./inspect-context";

type SortField = "name" | "upload" | "download" | "share";

function sortRows(rows: ShareRow[], field: SortField, descending: boolean): ShareRow[] {
  const copy = [...rows];
  copy.sort((left, right) => {
    let cmp = 0;
    if (field === "name") {
      cmp = left.label.localeCompare(right.label);
    } else if (field === "upload") {
      cmp = (left.upload ?? -1) - (right.upload ?? -1);
    } else if (field === "download") {
      cmp = left.download - right.download;
    } else {
      cmp = (left.share ?? -1) - (right.share ?? -1);
    }
    return descending ? -cmp : cmp;
  });
  return copy;
}

function ariaSort(active: SortField, field: SortField, descending: boolean): "ascending" | "descending" | "none" {
  if (active !== field) {
    return "none";
  }
  return descending ? "descending" : "ascending";
}

export function RankingTable({ locale, share }: { locale: UiLocale; share: ShareModel | null }) {
  const inspect = useReportInspect();
  const unknown = t(locale, "common.unknown");
  const [field, setField] = useState<SortField>("download");
  const [descending, setDescending] = useState(true);
  const rows = useMemo(
    () => sortRows(share?.rows ?? [], field, descending),
    [share, field, descending]
  );

  const toggle = (next: SortField): void => {
    if (field === next) {
      setDescending((value) => !value);
      return;
    }
    setField(next);
    setDescending(next !== "name");
  };

  const head = (id: SortField, label: string, numeric: boolean): ReactNode => (
    <SortableTh
      label={label}
      ariaSort={ariaSort(field, id, descending)}
      onClick={() => toggle(id)}
      numeric={numeric}
      className="px-2"
    />
  );

  return (
    <section className="space-y-2">
      <h3 className="text-sm font-semibold uppercase tracking-wider">{t(locale, "report.topn")}</h3>
      <div className="overflow-auto rounded-md border">
        <table className="w-full text-sm">
          <thead className="bg-muted/40">
            <tr>
              {head("name", t(locale, "report.col.name"), false)}
              {head("upload", t(locale, "report.col.upload"), true)}
              {head("download", t(locale, "report.col.download"), true)}
              {head("share", t(locale, "report.col.share"), false)}
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 ? (
              <tr>
                <td className="px-2 py-2 text-muted-foreground" colSpan={4}>
                  {t(locale, share?.capabilityUnsupported ? "report.empty_cap" : "report.empty")}
                </td>
              </tr>
            ) : (
              rows.map((row) => {
                const key = rankingInspectKey(row);
                const active = Boolean(inspect.activeKey && inspectKeysMatch(inspect.activeKey, key));
                return (
                  <tr
                    key={key}
                    tabIndex={0}
                    className={cn("cursor-pointer hover:bg-muted/40", active && "bg-primary/15")}
                    onMouseEnter={() => inspect.setHover(key)}
                    onMouseLeave={() => inspect.setHover(null)}
                    onClick={() => inspect.togglePinned(key)}
                    onKeyDown={(event) => {
                      if (event.key === "Enter" || event.key === " ") {
                        event.preventDefault();
                        inspect.togglePinned(key);
                      }
                    }}
                  >
                    <td className="px-2 py-1.5">{row.label}</td>
                    <td className="px-2 py-1.5 text-right tabular-nums">
                      {row.upload === null ? t(locale, "report.dash") : formatBytes(row.upload, unknown)}
                    </td>
                    <td className="px-2 py-1.5 text-right tabular-nums">
                      {formatBytes(row.download, unknown)}
                    </td>
                    <td className="px-2 py-1.5 tabular-nums">{formatSharePct(row.share, unknown)}</td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}
