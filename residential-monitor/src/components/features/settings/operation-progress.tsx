import type { OperationProgress } from "../../../dto";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";

export function OperationProgressView({
  locale,
  progress,
  onCancel
}: {
  locale: UiLocale;
  progress: OperationProgress | null;
  onCancel: () => void;
}) {
  if (!progress) {
    return null;
  }
  return (
    <section className="space-y-2 rounded-xl border bg-card p-3 text-sm" role="status">
      <p>
        {t(locale, "settings.progress.phase")} {progress.phase} · {progress.status}
      </p>
      <p className="tabular-nums">
        {progress.current} / {progress.total} {progress.unit}
      </p>
      {progress.redactedError ? <p className="text-destructive">{progress.redactedError}</p> : null}
      {progress.canCancel ? (
        <Button type="button" variant="outline" onClick={onCancel}>
          {t(locale, "settings.progress.cancel")}
        </Button>
      ) : null}
    </section>
  );
}
