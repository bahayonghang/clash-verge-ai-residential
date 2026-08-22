import type { AlertInstance, ReportQuery } from "../../../dto";
import { formatUtc } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";

export function EvidencePanel({
  locale,
  item,
  onJump
}: {
  locale: UiLocale;
  item: AlertInstance | null;
  onJump?: (query: ReportQuery) => void;
}) {
  if (!item) {
    return null;
  }
  const unknown = t(locale, "common.unknown");
  const evidence = item.evidence;
  return (
    <section className="space-y-2 rounded-xl border bg-card p-3 text-sm">
      <h3 className="font-semibold">{t(locale, "alerts.evidence")}</h3>
      <p>
        {t(locale, "alerts.col.observed")}{" "}
        {evidence.observedValue === null ? unknown : String(evidence.observedValue)}
      </p>
      <p>
        {t(locale, "alerts.col.coverage")} {evidence.coverageSummary}
      </p>
      {item.status === "not-evaluable" ? (
        <p role="status">{evidence.notEvaluableReason ?? t(locale, "alerts.not_evaluable_note")}</p>
      ) : null}
      <p>
        {t(locale, "alerts.evidence.window")}{" "}
        {evidence.windowStartUtc == null ? unknown : formatUtc(evidence.windowStartUtc)} →{" "}
        {evidence.windowEndUtc == null ? unknown : formatUtc(evidence.windowEndUtc)}
      </p>
      {evidence.reportQuery && onJump ? (
        <Button type="button" variant="outline" onClick={() => onJump(evidence.reportQuery as ReportQuery)}>
          {t(locale, "alerts.jump_report")}
        </Button>
      ) : null}
    </section>
  );
}
