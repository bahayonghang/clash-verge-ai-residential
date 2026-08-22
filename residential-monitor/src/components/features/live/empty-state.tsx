import { liveEmptyCopy, type LiveEmptyKind } from "../../../ipc/live-empty";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";

export function emptyCopy(
  kind: LiveEmptyKind,
  locale: UiLocale,
  healthTitle: string,
  healthAction: string
): string {
  if (kind === "disconnected") {
    return `${healthTitle}。${t(locale, "common.next")}：${healthAction}`;
  }
  return liveEmptyCopy(kind, locale) ?? healthTitle;
}

export function LiveRecoveryActions({
  kind,
  locale,
  onGoSettings,
  onResubscribe
}: {
  kind: LiveEmptyKind;
  locale: UiLocale;
  onGoSettings: () => void;
  onResubscribe: () => void;
}) {
  if (kind === "unconfigured") {
    return (
      <Button type="button" onClick={onGoSettings}>
        {t(locale, "live.go_settings")}
      </Button>
    );
  }
  if (kind === "needResync") {
    return (
      <Button type="button" onClick={onResubscribe}>
        {t(locale, "live.resync")}
      </Button>
    );
  }
  return null;
}

export function EmptyState({
  kind,
  locale,
  healthTitle,
  healthAction,
  onGoSettings,
  onResubscribe
}: {
  kind: LiveEmptyKind;
  locale: UiLocale;
  healthTitle: string;
  healthAction: string;
  onGoSettings: () => void;
  onResubscribe: () => void;
}) {
  if (kind === "hasRows") {
    return null;
  }
  const copy = emptyCopy(kind, locale, healthTitle, healthAction);
  return (
    <div className="flex flex-col items-start gap-3 text-sm" data-empty-kind={kind}>
      <p className="text-muted-foreground">{copy}</p>
      <LiveRecoveryActions
        kind={kind}
        locale={locale}
        onGoSettings={onGoSettings}
        onResubscribe={onResubscribe}
      />
    </div>
  );
}
