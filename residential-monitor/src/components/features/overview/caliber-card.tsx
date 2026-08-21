import type { ReactNode } from "react";
import { formatBytes } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";

export function CaliberCard({
  locale,
  icon,
  label,
  color,
  upload,
  download,
  uploadField,
  downloadField
}: {
  locale: UiLocale;
  icon: ReactNode;
  label: string;
  color: string;
  upload: number | null;
  download: number | null;
  uploadField: string;
  downloadField: string;
}) {
  const unknown = t(locale, "common.unknown");
  return (
    <article className="flex flex-col rounded-xl border bg-card p-3.5 shadow-xs">
      <div
        className="mb-2.5 flex h-8 w-8 items-center justify-center rounded-md"
        style={{ backgroundColor: `${color}15` }}
      >
        <span
          className="flex h-4 w-4 items-center justify-center [&_svg]:h-4 [&_svg]:w-4"
          style={{ color }}
        >
          {icon}
        </span>
      </div>
      <h3 className="truncate text-[11px] font-medium uppercase tracking-[0.14em] text-muted-foreground">
        {label}
      </h3>
      <dl className="mt-2.5 space-y-1.5">
        <div className="flex items-baseline justify-between gap-2">
          <dt className="text-xs text-muted-foreground">{t(locale, "overview.dir.up")}</dt>
          <dd
            data-field={uploadField}
            className="truncate text-base font-semibold tabular-nums"
            title={formatBytes(upload, unknown)}
          >
            {formatBytes(upload, unknown)}
          </dd>
        </div>
        <div className="flex items-baseline justify-between gap-2">
          <dt className="text-xs text-muted-foreground">{t(locale, "overview.dir.down")}</dt>
          <dd
            data-field={downloadField}
            className="truncate text-base font-semibold tabular-nums"
            title={formatBytes(download, unknown)}
          >
            {formatBytes(download, unknown)}
          </dd>
        </div>
      </dl>
    </article>
  );
}

export function SessionCaliberCard({
  locale,
  icon,
  color,
  activeCount,
  coverage,
  healthTitle,
  healthAction
}: {
  locale: UiLocale;
  icon: ReactNode;
  color: string;
  activeCount: number;
  coverage: string;
  healthTitle: string;
  healthAction: string;
}) {
  return (
    <article className="flex flex-col rounded-xl border bg-card p-3.5 shadow-xs">
      <div
        className="mb-2.5 flex h-8 w-8 items-center justify-center rounded-md"
        style={{ backgroundColor: `${color}15` }}
      >
        <span
          className="flex h-4 w-4 items-center justify-center [&_svg]:h-4 [&_svg]:w-4"
          style={{ color }}
        >
          {icon}
        </span>
      </div>
      <h3 className="truncate text-[11px] font-medium uppercase tracking-[0.14em] text-muted-foreground">
        {t(locale, "overview.active")}
      </h3>
      <p data-field="active-count" className="mt-2.5 text-lg font-semibold leading-none tabular-nums">
        {activeCount}
      </p>
      <p className="mt-1.5 text-xs text-muted-foreground">{coverage}</p>
      <p className="mt-1.5 text-xs text-muted-foreground">
        {healthTitle}。{t(locale, "common.next")}：{healthAction}
      </p>
    </article>
  );
}
