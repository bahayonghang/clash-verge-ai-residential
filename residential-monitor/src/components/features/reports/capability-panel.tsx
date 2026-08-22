import { useState } from "react";
import type { ReportResult } from "../../../dto";
import { t, type UiLocale } from "../../../i18n";
import { formatTemplate } from "../../../lib/utils";

function flag(locale: UiLocale, value: boolean): string {
  return value ? t(locale, "report.yes") : t(locale, "report.no");
}

export function CapabilityPanel({ locale, report }: { locale: UiLocale; report: ReportResult | null }) {
  const [open, setOpen] = useState(false);
  if (!report) {
    return null;
  }
  return (
    <details
      className="rounded-xl border bg-card p-3"
      open={open}
      onToggle={(event) => setOpen(event.currentTarget.open)}
    >
      <summary className="cursor-pointer text-sm font-semibold">{t(locale, "report.notes")}</summary>
      <div className="mt-3 space-y-2 text-sm">
        <p>{report.drilldownCapability.noteZh}</p>
        <p>{formatTemplate(t(locale, "report.tier"), { tier: report.dataTier })}</p>
        <p>{report.policyMetadata.noteZh}</p>
        <dl className="grid gap-1 sm:grid-cols-2">
          <div>
            <dt className="text-muted-foreground">{t(locale, "report.capability.sessions")}</dt>
            <dd>{flag(locale, report.drilldownCapability.sessions)}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "report.capability.policy")}</dt>
            <dd>{flag(locale, report.drilldownCapability.currentPolicy)}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "report.capability.cross")}</dt>
            <dd>{flag(locale, report.drilldownCapability.crossDimension)}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "report.capability.exact")}</dt>
            <dd>{flag(locale, report.drilldownCapability.exactTopN)}</dd>
          </div>
        </dl>
        <p className="text-muted-foreground">
          {t(locale, "report.named_sql")} {report.namedSql.length > 0 ? report.namedSql.join(", ") : t(locale, "common.none")}
        </p>
      </div>
    </details>
  );
}
