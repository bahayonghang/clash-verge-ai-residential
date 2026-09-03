import { useCallback, useEffect, useId, useRef, useState, type RefObject } from "react";
import type { AutostartRequestState } from "../../../hooks/autostart-request";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";
import { Switch } from "../../ui/switch";

export type StartupToggleAction = "confirm-enable" | "disable" | "none";

export function startupToggleAction(
  enabled: boolean,
  nextEnabled: boolean,
  busy: boolean
): StartupToggleAction {
  if (busy || enabled === nextEnabled) {
    return "none";
  }
  return nextEnabled ? "confirm-enable" : "disable";
}

export function commitStartupConfirmation(
  confirmed: boolean,
  onSetEnabled: (enabled: boolean) => void
): void {
  if (confirmed) {
    onSetEnabled(true);
  }
}

export function scheduleStartupFocus(
  getTarget: () => Pick<HTMLElement, "focus"> | null,
  schedule: (task: () => void) => void = queueMicrotask
): void {
  schedule(() => getTarget()?.focus());
}

export function StartupConfirmation({
  locale,
  titleId,
  descriptionId,
  confirmButtonRef,
  confirmDisabled = false,
  onConfirm,
  onCancel
}: {
  locale: UiLocale;
  titleId: string;
  descriptionId: string;
  confirmButtonRef: RefObject<HTMLButtonElement | null>;
  confirmDisabled?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  return (
    <div
      className="space-y-3 rounded-lg border border-primary/40 bg-muted/30 p-3"
      role="alertdialog"
      aria-modal="false"
      aria-labelledby={titleId}
      aria-describedby={descriptionId}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          onCancel();
        }
      }}
    >
      <h3 className="text-sm font-semibold" id={titleId}>
        {t(locale, "settings.autostart.confirm_title")}
      </h3>
      <p className="text-sm text-muted-foreground" id={descriptionId}>
        {t(locale, "settings.autostart.confirm_help")}
      </p>
      <div className="flex flex-wrap gap-2">
        <Button type="button" ref={confirmButtonRef} disabled={confirmDisabled} onClick={onConfirm}>
          {t(locale, "settings.autostart.confirm")}
        </Button>
        <Button type="button" variant="outline" onClick={onCancel}>
          {t(locale, "settings.autostart.cancel")}
        </Button>
      </div>
    </div>
  );
}

export function StartupSection({
  locale,
  state,
  onRefresh,
  onSetEnabled
}: {
  locale: UiLocale;
  state: AutostartRequestState;
  onRefresh: () => void;
  onSetEnabled: (enabled: boolean) => void;
}) {
  const [confirming, setConfirming] = useState(false);
  const switchRef = useRef<HTMLButtonElement>(null);
  const confirmButtonRef = useRef<HTMLButtonElement>(null);
  const id = useId();
  const descriptionId = `${id}-description`;
  const dialogTitleId = `${id}-confirm-title`;
  const dialogDescriptionId = `${id}-confirm-description`;
  const busy = state.loading || state.saving;
  const disabled = busy || confirming || !state.loaded;
  const statusKey = state.loading
    ? "settings.autostart.loading"
    : state.saving
      ? "settings.autostart.saving"
      : state.loaded
        ? state.enabled
          ? "settings.autostart.enabled"
          : "settings.autostart.disabled"
        : "settings.autostart.unknown";

  useEffect(() => {
    if (confirming && !busy && state.loaded && !state.enabled) {
      confirmButtonRef.current?.focus();
    }
  }, [busy, confirming, state.enabled, state.loaded]);

  const restoreSwitchFocus = useCallback(() => {
    scheduleStartupFocus(() => switchRef.current);
  }, []);

  useEffect(() => {
    if (confirming && (busy || !state.loaded || state.enabled)) {
      setConfirming(false);
      restoreSwitchFocus();
    }
  }, [busy, confirming, restoreSwitchFocus, state.enabled, state.loaded]);

  const cancelConfirmation = () => {
    commitStartupConfirmation(false, onSetEnabled);
    setConfirming(false);
    restoreSwitchFocus();
  };
  const confirmEnable = () => {
    setConfirming(false);
    restoreSwitchFocus();
    commitStartupConfirmation(true, onSetEnabled);
  };

  return (
    <section className="space-y-3 rounded-xl border bg-card p-4" data-autostart-section>
      <div className="flex items-start justify-between gap-3">
        <div>
          <h2 className="text-base font-semibold">{t(locale, "settings.autostart.title")}</h2>
          <p className="text-sm text-muted-foreground" id={descriptionId}>
            {t(locale, "settings.autostart.help")}
          </p>
        </div>
        <span className="text-sm" aria-live="polite">
          {t(locale, statusKey)}
        </span>
      </div>
      <div className="flex items-center justify-between gap-4 rounded-lg border p-3">
        <label className="text-sm font-medium" htmlFor={`${id}-switch`}>
          {t(locale, "settings.autostart.label")}
        </label>
        <Switch
          ref={switchRef}
          id={`${id}-switch`}
          checked={state.enabled}
          disabled={disabled}
          aria-label={t(locale, "settings.autostart.label")}
          aria-describedby={descriptionId}
          onCheckedChange={(nextEnabled) => {
            const action = startupToggleAction(state.enabled, nextEnabled, disabled);
            if (action === "confirm-enable") {
              setConfirming(true);
            } else if (action === "disable") {
              onSetEnabled(false);
            }
          }}
        />
      </div>
      {confirming ? (
        <StartupConfirmation
          locale={locale}
          titleId={dialogTitleId}
          descriptionId={dialogDescriptionId}
          confirmButtonRef={confirmButtonRef}
          confirmDisabled={busy || !state.loaded || state.enabled}
          onConfirm={confirmEnable}
          onCancel={cancelConfirmation}
        />
      ) : null}
      {state.errorZh ? (
        <div className="flex flex-wrap items-center gap-2" role="alert">
          <span className="text-sm text-destructive">{state.errorZh}</span>
          <Button type="button" variant="outline" disabled={busy || confirming} onClick={onRefresh}>
            {t(locale, "settings.autostart.retry")}
          </Button>
        </div>
      ) : null}
    </section>
  );
}
