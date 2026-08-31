import { t, type UiLocale } from "../../../i18n";

export type CaliberKind = "filter" | "accounting";

export function CaliberNote({
  locale,
  kind
}: {
  locale: UiLocale;
  kind: CaliberKind;
}) {
  const key = kind === "filter" ? "residential.caliber.filter" : "residential.caliber.accounting";
  return (
    <p data-caliber={kind} className="text-xs text-muted-foreground/80" role="note">
      {t(locale, key)}
    </p>
  );
}
