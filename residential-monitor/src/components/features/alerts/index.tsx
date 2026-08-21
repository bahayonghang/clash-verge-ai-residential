import { useState } from "react";
import type { AlertRule, ReportQuery } from "../../../dto";
import { emptyAlertDraft, useAlerts } from "../../../hooks/use-alerts";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";
import { CenterList } from "./center-list";
import { DiagnosticsPanel } from "./diagnostics-panel";
import { EvidencePanel } from "./evidence-panel";
import { NotifyPanel } from "./notify-panel";
import { RuleEditor } from "./rule-editor";
import { RuleList } from "./rule-list";

export function AlertsPage({
  locale,
  onJumpReport
}: {
  locale: UiLocale;
  onJumpReport?: (query: ReportQuery) => void;
}) {
  const alerts = useAlerts(locale, true);
  const [draft, setDraft] = useState<AlertRule>(emptyAlertDraft);

  return (
    <div className="space-y-6">
      <p className="text-sm text-muted-foreground">{t(locale, "alerts.intro")}</p>
      <p className="text-sm">{alerts.statusZh}</p>
      {alerts.summary && alerts.summary.notEvaluableCount > 0 ? (
        <p role="status" className="text-sm">
          {t(locale, "alerts.not_evaluable_note")} {alerts.summary.notEvaluableCount}
        </p>
      ) : null}
      {alerts.errorZh ? (
        <p className="text-sm text-destructive" role="alert">
          {alerts.errorZh}
        </p>
      ) : null}
      <div className="flex flex-wrap gap-2">
        <Button type="button" onClick={() => void alerts.refresh()}>
          {t(locale, "alerts.refresh")}
        </Button>
      </div>
      <NotifyPanel locale={locale} notify={alerts.notify} onTest={() => void alerts.testNotify()} />
      <section className="space-y-3">
        <h2 className="text-sm font-semibold uppercase tracking-wider">{t(locale, "alerts.rules")}</h2>
        <RuleList
          locale={locale}
          rules={alerts.rules}
          selectedId={draft.ruleId}
          onSelect={setDraft}
        />
        <RuleEditor
          locale={locale}
          draft={draft}
          onChange={setDraft}
          onSave={() => void alerts.upsertRule(draft)}
        />
      </section>
      <section className="space-y-3">
        <h2 className="text-sm font-semibold uppercase tracking-wider">{t(locale, "alerts.history")}</h2>
        <CenterList
          locale={locale}
          items={alerts.page?.items ?? []}
          filter={alerts.statusFilter}
          selectedId={alerts.selected?.instanceId ?? null}
          hasMore={Boolean(alerts.page?.nextCursor)}
          onFilter={alerts.setStatusFilter}
          onSelect={alerts.setSelected}
          onMore={() => void alerts.loadMore()}
        />
        <EvidencePanel locale={locale} item={alerts.selected} onJump={onJumpReport} />
      </section>
      <DiagnosticsPanel
        locale={locale}
        snapshot={alerts.diagnostics}
        outboxCount={alerts.outboxCount}
        onExport={() => void alerts.exportDiagnostics()}
        onScan={() => void alerts.scanOutbox()}
      />
    </div>
  );
}
