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
        <h2 id="residential-monitor-title" className="text-sm font-semibold">
          {t(locale, "residential.monitor")}
        </h2>
        <CaliberNote locale={locale} kind="filter" />
        <p className="text-xs text-muted-foreground">
          {formatTemplate(t(locale, "live.last_sample"), {
            time: formatUtc(overview.lastSampleUtc, t(locale, "common.no_sample"))
          })}
        </p>
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
          color="#3B82F6"
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
          color="#3B82F6"
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
          color="#A855F7"
          loading={live.loading && live.page === null}
        />
      </div>
      <HotspotCards locale={locale} page={live.page} statusInput={statusInput} />
      <div>
        <h3 className="text-sm font-semibold">{t(locale, "residential.monitor.occupancy")}</h3>
        <CaliberNote locale={locale} kind="accounting" />
        <div className="mt-2 overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border/60 text-left text-muted-foreground">
                <th className="py-2 font-medium">{t(locale, "overview.col.name")}</th>
                <th className="py-2 font-medium">{t(locale, "overview.col.upload")}</th>
                <th className="py-2 font-medium">{t(locale, "overview.col.download")}</th>
              </tr>
            </thead>
            <tbody>
              {occupancy.length === 0 ? (
                <tr>
                  <td className="py-3 text-muted-foreground" colSpan={3}>
                    {t(locale, "common.none")}
                  </td>
                </tr>
              ) : (
                occupancy.map((row) => (
                  <tr key={row.name} className="border-b border-border/40 last:border-0">
                    <td className="py-2">{row.name}</td>
                    <td className="py-2 tabular-nums">{formatBytes(row.upload, unknown)}</td>
                    <td className="py-2 tabular-nums">{formatBytes(row.download, unknown)}</td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </section>
  );
}
