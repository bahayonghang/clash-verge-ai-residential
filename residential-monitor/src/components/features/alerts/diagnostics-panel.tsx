import type { DiagnosticsSnapshot } from "../../../dto";
import { formatUtc } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { Button } from "../../ui/button";

export function DiagnosticsPanel({
  locale,
  snapshot,
  outboxCount,
  onExport,
  onScan
}: {
  locale: UiLocale;
  snapshot: DiagnosticsSnapshot | null;
  outboxCount: number | null;
  onExport: () => void;
  onScan: () => void;
}) {
  return (
    <section className="space-y-3">
      <h2 className="text-sm font-semibold uppercase tracking-wider">{t(locale, "alerts.diag.title")}</h2>
      {snapshot ? (
        <dl className="grid gap-2 text-sm sm:grid-cols-2">
          <div>
            <dt className="text-muted-foreground">{t(locale, "settings.about_label.version")}</dt>
            <dd className="font-mono">{snapshot.appVersion}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.diag.schema")}</dt>
            <dd>
              {snapshot.schemaVersion} / {snapshot.sqliteUserVersion} / {snapshot.supportedSchema}
            </dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.diag.c4")}</dt>
            <dd className="font-mono break-all">{snapshot.c4Checksum}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.diag.journal")}</dt>
            <dd>
              {snapshot.journalMode} / {snapshot.synchronous}
            </dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "settings.transport")}</dt>
            <dd>{snapshot.controllerTransportStatus}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.col.coverage")}</dt>
            <dd>{snapshot.coverageSummary}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.diag.watermark")}</dt>
            <dd className="tabular-nums">{snapshot.writerWatermark}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.diag.receipts")}</dt>
            <dd className="tabular-nums">{snapshot.writerReceipts}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.diag.last_frame")}</dt>
            <dd>{snapshot.lastFrameUtc == null ? t(locale, "common.no_sample") : formatUtc(snapshot.lastFrameUtc)}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.diag.reconnect")}</dt>
            <dd>{snapshot.reconnectHintZh}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.diag.database")}</dt>
            <dd>{snapshot.databaseOk ? t(locale, "alerts.diag.ok") : t(locale, "alerts.diag.bad")}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.diag.wal")}</dt>
            <dd>{snapshot.walCheckpointOk ? t(locale, "alerts.diag.ok") : t(locale, "alerts.diag.bad")}</dd>
          </div>
          <div className="sm:col-span-2">
            <dt className="text-muted-foreground">{t(locale, "alerts.diag.retention")}</dt>
            <dd>{snapshot.backupRetentionNoteZh}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.summary.active")}</dt>
            <dd className="tabular-nums">{snapshot.alertActive}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t(locale, "alerts.outbox_count")}</dt>
            <dd className="tabular-nums">{snapshot.outboxBacklog}</dd>
          </div>
          <div className="sm:col-span-2">
            <dt className="text-muted-foreground">{t(locale, "alerts.diag.errors")}</dt>
            <dd>
              {snapshot.recentRedactedErrorClasses.length > 0
                ? snapshot.recentRedactedErrorClasses.join(", ")
                : t(locale, "common.none")}
            </dd>
          </div>
        </dl>
      ) : (
        <p className="text-sm text-muted-foreground">{t(locale, "alerts.diag_idle")}</p>
      )}
      {outboxCount !== null ? (
        <p className="text-sm">
          {t(locale, "alerts.outbox_count")} {outboxCount}
        </p>
      ) : null}
      <div className="flex flex-wrap gap-2">
        <Button type="button" variant="outline" onClick={onExport}>
          {t(locale, "alerts.export_diag")}
        </Button>
        <Button type="button" variant="outline" onClick={onScan}>
          {t(locale, "alerts.scan_outbox")}
        </Button>
      </div>
    </section>
  );
}
