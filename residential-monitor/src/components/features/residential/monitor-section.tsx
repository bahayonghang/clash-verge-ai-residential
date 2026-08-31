import { ArrowDownToLine, ArrowUpFromLine, Link2 } from "lucide-react";
import type { BootstrapDto, LiveOverview, RouteId } from "../../../dto";
import {
  canShowHotspotValue,
  liveHotspotStatus,
  type LiveHotspotStatus
} from "../../../format/live-hotspot";
import { categoryRows } from "../../../format/overview";
import { formatBytes, formatRate, formatUtc } from "../../../format/units";
import { useLivePage } from "../../../hooks/use-live-page";
import { t, type UiLocale } from "../../../i18n";
import { liveEmptyKind } from "../../../ipc/live-empty";
import { defaultLiveQuery } from "../../../ipc/live-session";
import type { MonitorState } from "../../../ipc/reducer";
import { healthOf } from "../../../lib/health";
import { formatTemplate } from "../../../lib/utils";
import {
  DataTableTd,
  DataTableTh,
  dataTableClasses
} from "../../common/data-table";
import { StatCard } from "../../common/stat-card";
import { EmptyState } from "../live/empty-state";
import { HotspotCards } from "../live/hotspot-cards";
import { CaliberNote } from "./caliber-note";

export type ResidentialHealthState = "paused" | "disconnected" | "gap";

/** 暂停、未连接、缺口互斥。未配置 targets 由页级 TargetEmpty 单独成态。 */
export function residentialHealthState(status: LiveHotspotStatus): ResidentialHealthState | null {
  if (status === "paused" || status === "disconnected" || status === "gap") {
    return status;
  }
  return null;
}

function residentialFilter() {
  return { ...defaultLiveQuery().filter, residentialOnly: true, clauses: [] };
}

export function MonitorSection({
  locale,
  boot,
  stream,
  autoRefresh,
  overview,
  onRouteChange,
  onResubscribe
}: {
  locale: UiLocale;
  boot: BootstrapDto;
  stream: MonitorState;
  autoRefresh: boolean;
  overview: LiveOverview;
  onRouteChange: (route: RouteId) => void;
  onResubscribe: () => void;
}) {
  const unknown = t(locale, "common.unknown");
  const live = useLivePage({
    applied: residentialFilter(),
    sort: { sortField: "download", descending: true },
    cursor: null,
    refreshSignal: autoRefresh ? stream.lastSeq : null,
    locale
  });
  const address = boot.settings.address;
  const session = overview.health.session;
  const health = healthOf(locale, session);
  const statusInput = {
    page: live.page,
    address,
    session,
    observationPhase: overview.observationPhase,
    collectorRunning: live.collectorRunning,
    coverageKind: overview.coverageKind,
    coverageReason: overview.coverageReason,
    needResync: stream.needResync,
    frozen: stream.frozen
  };
  const hotspotStatus = liveHotspotStatus(statusInput);
  const healthState = residentialHealthState(hotspotStatus);
  const kind = liveEmptyKind({
    address,
    session,
    observationPhase: overview.observationPhase,
    collectorRunning: live.collectorRunning,
    coverageKind: overview.coverageKind,
    coverageReason: overview.coverageReason,
    rowCount: live.page?.rows.length ?? 0,
    needResync: stream.needResync,
    frozen: stream.frozen,
    errorZh: live.errorZh ?? stream.errorZh
  });
  const showHits = hotspotStatus === "ready" || hotspotStatus === "noMatch";
  const topDown = live.page?.summary.topDownload ?? null;
  const topUp = live.page?.summary.topUpload ?? null;
  const downRow = topDown ? live.page?.rows.find((row) => row.identity === topDown.identity) : undefined;
  const upRow = topUp ? live.page?.rows.find((row) => row.identity === topUp.identity) : undefined;
  const occupancy = categoryRows(overview.categoryUpload, overview.categoryDownload);

  return (
    <section className="space-y-4" aria-labelledby="residential-monitor-title">
      <div>
        <h2 id="residential-monitor-title" className="text-base font-semibold">
          {t(locale, "residential.monitor")}
        </h2>
        <div className="mt-1 flex flex-wrap gap-x-4 gap-y-1">
          <CaliberNote locale={locale} kind="filter" />
          <p className="text-xs text-muted-foreground/80">
            {formatTemplate(t(locale, "live.last_sample"), {
              time: formatUtc(overview.lastSampleUtc, t(locale, "common.no_sample"))
            })}
          </p>
        </div>
      </div>
      {healthState ? (
        <p className="text-sm text-muted-foreground" data-state={healthState} role="status">
          {t(locale, `residential.state.${healthState}`)}
        </p>
      ) : null}
      <EmptyState
        kind={kind}
        locale={locale}
        healthTitle={health.title}
        healthAction={health.action}
        onGoSettings={() => onRouteChange("settings-data")}
        onResubscribe={onResubscribe}
      />
      {live.errorZh ? (
        <p className="text-sm text-destructive" role="alert">
          {live.errorZh}
        </p>
      ) : null}
      <div className="grid gap-2.5 sm:grid-cols-3">
        <StatCard
          icon={<Link2 />}
          label={t(locale, "residential.monitor.hits")}
          value={showHits && live.page ? String(live.page.matchedCount) : unknown}
          colorToken={1}
          loading={live.loading && live.page === null}
        />
        <StatCard
          icon={<ArrowDownToLine />}
          label={t(locale, "residential.monitor.rate_down")}
          value={
            canShowHotspotValue(hotspotStatus)
              ? formatRate(downRow?.rateDownload ?? null, unknown)
              : unknown
          }
          colorToken={1}
          loading={live.loading && live.page === null}
        />
        <StatCard
          icon={<ArrowUpFromLine />}
          label={t(locale, "residential.monitor.rate_up")}
          value={
            canShowHotspotValue(hotspotStatus)
              ? formatRate(upRow?.rateUpload ?? null, unknown)
              : unknown
          }
          colorToken={2}
          loading={live.loading && live.page === null}
        />
      </div>
      <HotspotCards locale={locale} page={live.page} statusInput={statusInput} />
      <div>
        <div className="flex flex-wrap items-baseline justify-between gap-x-4">
          <h3 className="text-sm font-semibold">{t(locale, "residential.monitor.occupancy")}</h3>
          <CaliberNote locale={locale} kind="accounting" />
        </div>
        {occupancy.length === 0 ? (
          <p className="mt-2 text-sm text-muted-foreground">{t(locale, "common.none")}</p>
        ) : occupancy.length <= 3 ? (
          <ul className="mt-2 space-y-1.5">
            {occupancy.map((row) => (
              <li
                key={row.name}
                className="flex flex-wrap items-baseline gap-x-6 gap-y-0.5 text-sm"
              >
                <span className="font-medium">{row.name}</span>
                <span className="text-muted-foreground">
                  {t(locale, "overview.col.upload")}{" "}
                  <span className="tabular-nums text-foreground">
                    {formatBytes(row.upload, unknown)}
                  </span>
                </span>
                <span className="text-muted-foreground">
                  {t(locale, "overview.col.download")}{" "}
                  <span className="tabular-nums text-foreground">
                    {formatBytes(row.download, unknown)}
                  </span>
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <div className={`mt-2 ${dataTableClasses.wrapper}`}>
            <table className={dataTableClasses.table}>
              <thead>
                <tr className={dataTableClasses.headRow}>
                  <DataTableTh>{t(locale, "overview.col.name")}</DataTableTh>
                  <DataTableTh numeric>{t(locale, "overview.col.upload")}</DataTableTh>
                  <DataTableTh numeric>{t(locale, "overview.col.download")}</DataTableTh>
                </tr>
              </thead>
              <tbody>
                {occupancy.map((row) => (
                  <tr key={row.name} className={dataTableClasses.row}>
                    <DataTableTd>{row.name}</DataTableTd>
                    <DataTableTd numeric>{formatBytes(row.upload, unknown)}</DataTableTd>
                    <DataTableTd numeric>{formatBytes(row.download, unknown)}</DataTableTd>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </section>
  );
}
