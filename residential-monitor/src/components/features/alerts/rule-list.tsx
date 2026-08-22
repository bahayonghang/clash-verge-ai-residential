import type { AlertRule } from "../../../dto";
import { t, type UiLocale } from "../../../i18n";
import { cn } from "../../../lib/utils";

export function RuleList({
  locale,
  rules,
  selectedId,
  onSelect
}: {
  locale: UiLocale;
  rules: AlertRule[];
  selectedId: string | null;
  onSelect: (rule: AlertRule) => void;
}) {
  if (rules.length === 0) {
    return <p className="text-sm text-muted-foreground">{t(locale, "alerts.rules_empty")}</p>;
  }
  return (
    <ul className="space-y-1">
      {rules.map((rule) => (
        <li key={rule.ruleId}>
          <button
            type="button"
            className={cn(
              "w-full rounded-md px-2 py-1.5 text-left text-sm hover:bg-muted/40",
              selectedId === rule.ruleId && "bg-primary/10"
            )}
            onClick={() => onSelect(rule)}
          >
            <span className="font-medium">{rule.ruleId}</span>
            <span className="ml-2 text-muted-foreground">
              {t(locale, `alerts.kind.${rule.kind === "period-usage" ? "period" : rule.kind}`)} · v{rule.version}
            </span>
          </button>
        </li>
      ))}
    </ul>
  );
}
