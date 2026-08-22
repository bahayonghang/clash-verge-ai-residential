import { t, type UiLocale } from "../../../i18n";

export function resolvedCapabilityNote(
  locale: UiLocale,
  noteZh: string | null | undefined,
  fallbackKey: string
): string {
  return noteZh && noteZh.length > 0 ? noteZh : t(locale, fallbackKey);
}

export function CapabilityNote({ locale, noteZh }: { locale: UiLocale; noteZh: string }) {
  if (noteZh.length === 0) {
    return null;
  }
  return (
    <div
      role="status"
      data-capability-note="1"
      className="rounded-xl border border-dashed border-border/60 bg-muted/20 px-4 py-3 text-sm text-muted-foreground"
    >
      <p className="font-medium text-foreground">{t(locale, "dimension.capability")}</p>
      <p className="mt-1">{noteZh}</p>
    </div>
  );
}
