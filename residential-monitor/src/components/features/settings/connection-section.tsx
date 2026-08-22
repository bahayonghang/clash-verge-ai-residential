import { useEffect, useLayoutEffect, useRef, useState } from "react";
import type { ControllerSettings } from "../../../dto";
import { t, type UiLocale } from "../../../i18n";
import { applySecretField } from "../../../ipc/secret-field";
import { healthOf } from "../../../lib/health";
import { formatTemplate } from "../../../lib/utils";
import { Button } from "../../ui/button";
import { fieldClass, labelClass } from "../form-styles";

export function ConnectionSection({
  locale,
  address,
  targets,
  secret,
  settings,
  session,
  collectorRunning,
  probeMessage,
  probeState,
  wizardComplete,
  onAddress,
  onTargets,
  onSecret,
  onSave,
  onTest,
  onDisconnect,
  onReconnect,
  onPause,
  onResume,
  onCompleteWizard,
  onEnter
}: {
  locale: UiLocale;
  address: string;
  targets: string;
  secret: string;
  settings: ControllerSettings | null;
  session: string;
  collectorRunning: boolean | null;
  probeMessage: string;
  probeState: string;
  wizardComplete: boolean;
  onAddress: (value: string) => void;
  onTargets: (value: string) => void;
  onSecret: (value: string) => void;
  onSave: () => void;
  onTest: () => void;
  onDisconnect: () => void;
  onReconnect: () => void;
  onPause: () => void;
  onResume: () => void;
  onCompleteWizard: () => void;
  onEnter: () => void;
}) {
  const rootRef = useRef<HTMLDivElement>(null);
  const [visible, setVisible] = useState(false);
  const health = healthOf(locale, session);
  const collectorLabel =
    collectorRunning === null
      ? t(locale, "settings.collector_unknown")
      : collectorRunning
        ? t(locale, "settings.collector_running")
        : t(locale, "settings.collector_paused");

  useEffect(() => {
    onEnter();
  }, [onEnter]);

  useLayoutEffect(() => {
    if (rootRef.current) {
      applySecretField(rootRef.current, secret, visible, locale);
    }
  }, [secret, visible, locale]);

  return (
    <div className="space-y-6" ref={rootRef}>
      <section className="space-y-3 rounded-xl border bg-card p-4">
        <div className="flex items-start justify-between gap-3">
          <div>
            <h2 className="text-base font-semibold">{t(locale, "settings.connection.title")}</h2>
            <p className="text-sm text-muted-foreground">{t(locale, "settings.connection.help")}</p>
          </div>
          <span className="text-sm">{health.title}</span>
        </div>
        <dl className="grid gap-2 text-sm sm:grid-cols-3">
          <div>
            <dt className="text-muted-foreground">{t(locale, "settings.session")}</dt>
            <dd>{health.title}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "settings.collector")}</dt>
            <dd>{collectorLabel}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "settings.transport")}</dt>
            <dd>{settings?.transport ?? ""}</dd>
          </div>
        </dl>
      </section>
      <section className="space-y-3 rounded-xl border bg-card p-4">
        <div className="flex items-start justify-between gap-3">
          <div>
            <h2 className="text-base font-semibold">{t(locale, "settings.controller_title")}</h2>
            <p className="text-sm text-muted-foreground">{t(locale, "settings.controller_help")}</p>
          </div>
          <span className="text-sm">
            {settings?.hasSecret ? t(locale, "settings.cred_yes") : t(locale, "settings.cred_no")}
          </span>
        </div>
        <label className={labelClass}>
          {t(locale, "settings.address")}
          <input
            id="controller-address"
            className={fieldClass}
            value={address}
            placeholder="127.0.0.1:9097"
            autoComplete="off"
            onChange={(event) => onAddress(event.target.value)}
          />
        </label>
        <label className={labelClass}>
          {t(locale, "settings.targets")}
          <input
            id="targets"
            className={fieldClass}
            value={targets}
            autoComplete="off"
            onChange={(event) => onTargets(event.target.value)}
          />
        </label>
        <p className="text-xs text-muted-foreground">
          {formatTemplate(t(locale, "settings.cred"), {
            status: settings?.hasSecret ? t(locale, "settings.cred_yes") : t(locale, "settings.cred_no"),
            mode: settings?.secretMode ?? "none"
          })}
        </p>
        <label className={labelClass}>
          {t(locale, "secret.label")}
          <span className="flex gap-2">
            <input
              id="controller-secret"
              className={fieldClass}
              type={visible ? "text" : "password"}
              autoComplete="off"
              spellCheck={false}
              value={secret}
              onChange={(event) => onSecret(event.target.value)}
            />
            <Button
              type="button"
              id="toggle-secret"
              variant="outline"
              aria-pressed={visible}
              aria-label={visible ? t(locale, "secret.hide") : t(locale, "secret.show")}
              onClick={() => setVisible((value) => !value)}
            >
              {visible ? t(locale, "secret.hide") : t(locale, "secret.show")}
            </Button>
          </span>
          <span className="text-xs text-muted-foreground">{t(locale, "secret.hint")}</span>
        </label>
        <p className="text-xs text-muted-foreground">{t(locale, "settings.port_note")}</p>
        <details className="text-sm">
          <summary>{t(locale, "settings.wizard")}</summary>
          <ol className="mt-2 list-decimal pl-5">
            <li>{t(locale, "settings.wizard.1")}</li>
            <li>{t(locale, "settings.wizard.2")}</li>
            <li>{t(locale, "settings.wizard.3")}</li>
            <li>{t(locale, "settings.wizard.4")}</li>
            <li>{t(locale, "settings.wizard.5")}</li>
          </ol>
        </details>
        <div className="flex flex-wrap gap-2">
          <Button type="button" onClick={onSave}>
            {t(locale, "settings.save")}
          </Button>
          <Button type="button" variant="outline" onClick={onTest}>
            {t(locale, "settings.test_single")}
          </Button>
          <Button type="button" variant="outline" onClick={onReconnect}>
            {t(locale, "settings.reconnect")}
          </Button>
          <Button type="button" variant="outline" onClick={onDisconnect}>
            {t(locale, "settings.disconnect")}
          </Button>
          {collectorRunning ? (
            <Button type="button" variant="outline" onClick={onPause}>
              {t(locale, "settings.pause")}
            </Button>
          ) : (
            <Button type="button" variant="outline" onClick={onResume}>
              {t(locale, "settings.resume")}
            </Button>
          )}
          {wizardComplete ? null : (
            <Button type="button" variant="outline" onClick={onCompleteWizard}>
              {t(locale, "settings.complete_wizard")}
            </Button>
          )}
        </div>
        <p className="text-xs text-muted-foreground">{t(locale, "settings.test_single_help")}</p>
        <p className="text-sm" data-state={probeState}>
          {probeMessage}
        </p>
      </section>
    </div>
  );
}
