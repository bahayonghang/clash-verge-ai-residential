import type { AlertRule } from "../../../dto";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";
import { fieldClass, labelClass } from "../form-styles";

export function RuleEditor({
  locale,
  draft,
  onChange,
  onSave
}: {
  locale: UiLocale;
  draft: AlertRule;
  onChange: (rule: AlertRule) => void;
  onSave: () => void;
}) {
  return (
    <form
      className="grid gap-3 sm:grid-cols-2"
      onSubmit={(event) => {
        event.preventDefault();
        onSave();
      }}
    >
      <p className="sm:col-span-2 text-sm text-muted-foreground">{t(locale, "alerts.rules_help")}</p>
      <label className={labelClass}>
        {t(locale, "alerts.rule_id")}
        <input
          className={fieldClass}
          value={draft.ruleId}
          onChange={(event) => onChange({ ...draft, ruleId: event.target.value })}
        />
      </label>
      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          checked={draft.enabled}
          onChange={(event) => onChange({ ...draft, enabled: event.target.checked })}
        />
        {t(locale, "alerts.enabled")}
      </label>
      <label className={labelClass}>
        {t(locale, "alerts.kind")}
        <select
          className={fieldClass}
          value={draft.kind}
          onChange={(event) => onChange({ ...draft, kind: event.target.value as AlertRule["kind"] })}
        >
          <option value="rate">{t(locale, "alerts.kind.rate")}</option>
          <option value="period-usage">{t(locale, "alerts.kind.period")}</option>
          <option value="health">{t(locale, "alerts.kind.health")}</option>
        </select>
      </label>
      <label className={labelClass}>
        {t(locale, "alerts.selector")}
        <select
          className={fieldClass}
          value={draft.selectorKind}
          onChange={(event) =>
            onChange({ ...draft, selectorKind: event.target.value as AlertRule["selectorKind"] })
          }
        >
          <option value="primary-category">{t(locale, "alerts.selector.category")}</option>
          <option value="domain">{t(locale, "alerts.selector.domain")}</option>
          <option value="process">{t(locale, "alerts.selector.process")}</option>
          <option value="health-kind">{t(locale, "alerts.selector.health")}</option>
        </select>
      </label>
      <label className={labelClass}>
        {t(locale, "alerts.selector_value")}
        <input
          className={fieldClass}
          value={draft.selectorValue ?? ""}
          onChange={(event) => onChange({ ...draft, selectorValue: event.target.value || null })}
        />
      </label>
      <label className={labelClass}>
        {t(locale, "alerts.direction")}
        <select
          className={fieldClass}
          value={draft.direction ?? "download"}
          onChange={(event) =>
            onChange({ ...draft, direction: event.target.value as AlertRule["direction"] })
          }
        >
          <option value="download">{t(locale, "alerts.dir.down")}</option>
          <option value="upload">{t(locale, "alerts.dir.up")}</option>
          <option value="combined">{t(locale, "alerts.dir.combined")}</option>
        </select>
      </label>
      <label className={labelClass}>
        {t(locale, "alerts.threshold")}
        <input
          className={fieldClass}
          type="number"
          value={draft.thresholdValue}
          onChange={(event) => onChange({ ...draft, thresholdValue: Number(event.target.value) })}
        />
      </label>
      <label className={labelClass}>
        {t(locale, "alerts.recovery")}
        <input
          className={fieldClass}
          type="number"
          value={draft.recoveryThreshold ?? ""}
          onChange={(event) =>
            onChange({
              ...draft,
              recoveryThreshold: event.target.value === "" ? null : Number(event.target.value)
            })
          }
        />
      </label>
      <label className={labelClass}>
        {t(locale, "alerts.period")}
        <select
          className={fieldClass}
          value={draft.period ?? ""}
          onChange={(event) =>
            onChange({
              ...draft,
              period: event.target.value === "" ? null : (event.target.value as AlertRule["period"])
            })
          }
        >
          <option value="">{t(locale, "alerts.period.none")}</option>
          <option value="rolling-1h">{t(locale, "alerts.period.1h")}</option>
          <option value="local-day">{t(locale, "alerts.period.day")}</option>
          <option value="local-month">{t(locale, "alerts.period.month")}</option>
        </select>
      </label>
      <label className={labelClass}>
        {t(locale, "alerts.timezone")}
        <input
          className={fieldClass}
          value={draft.timezone}
          onChange={(event) => onChange({ ...draft, timezone: event.target.value })}
        />
      </label>
      <label className={labelClass}>
        {t(locale, "alerts.cooldown")}
        <input
          className={fieldClass}
          type="number"
          value={draft.cooldownSec}
          onChange={(event) => onChange({ ...draft, cooldownSec: Number(event.target.value) })}
        />
      </label>
      <label className={labelClass}>
        {t(locale, "alerts.quiet_start")}
        <input
          className={fieldClass}
          type="number"
          value={draft.quietStartMin ?? ""}
          onChange={(event) =>
            onChange({
              ...draft,
              quietStartMin: event.target.value === "" ? null : Number(event.target.value)
            })
          }
        />
      </label>
      <label className={labelClass}>
        {t(locale, "alerts.quiet_end")}
        <input
          className={fieldClass}
          type="number"
          value={draft.quietEndMin ?? ""}
          onChange={(event) =>
            onChange({
              ...draft,
              quietEndMin: event.target.value === "" ? null : Number(event.target.value)
            })
          }
        />
      </label>
      <div className="sm:col-span-2">
        <Button type="submit">{t(locale, "alerts.save")}</Button>
      </div>
    </form>
  );
}
