import type { ReactNode } from "react";
import type { ObservationPhase } from "../../../dto";
import { formatBytes } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";

export function CaliberCard({
  locale,
  icon,
  label,
  color,
  upload,
  download,
  phase,
  uploadField,
  downloadField
}: {
  locale: UiLocale;
  icon: ReactNode;
  label: string;
  color: string;
  upload: number | null;
  download: number | null;
  phase: ObservationPhase;
  uploadField: string;
  downloadField: string;
}) {
  const unknown = t(locale, "common.unknown");
  const displayValue = (value: number | null): string => {
    if (phase === "current") {
      return formatBytes(value, unknown);
    }
    if (["paused", "disconnected", "resyncRequired", "decodeFailed"].includes(phase) && value !== null) {
      return `${formatBytes(value, unknown)} · ${t(locale, "overview.phase.stale")}`;
    }
    return t(locale, `overview.phase.${phase}`);
  };
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
            title={displayValue(upload)}
          >
            {displayValue(upload)}
          </dd>
        </div>
        <div className="flex items-baseline justify-between gap-2">
          <dt className="text-xs text-muted-foreground">{t(locale, "overview.dir.down")}</dt>
          <dd
            data-field={downloadField}
            className="truncate text-base font-semibold tabular-nums"
            title={displayValue(download)}
          >
            {displayValue(download)}
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
  phase,
  coverage,
  healthTitle,
  healthAction
}: {
  locale: UiLocale;
  icon: ReactNode;
  color: string;
  activeCount: number;
  phase: ObservationPhase;
  coverage: string;
  healthTitle: string;
  healthAction: string;
}) {
  const active = phase === "current" ? String(activeCount) : "—";
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
        {active}
      </p>
      <p className="mt-1.5 text-xs text-muted-foreground">{coverage}</p>
      <p className="mt-1.5 text-xs text-muted-foreground">
        {healthTitle}。{t(locale, "common.next")}：{healthAction}
      </p>
    </article>
  );
}
