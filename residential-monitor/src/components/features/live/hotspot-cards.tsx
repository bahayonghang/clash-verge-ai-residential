import { ArrowDownToLine, ArrowUpFromLine } from "lucide-react";
import type { ConnectionHotspot, LiveConnectionPage } from "../../../ipc/live-session";
import {
  canShowHotspotSnapshotFacts,
  canShowHotspotValue,
  hotspotDisplayDetail,
  hotspotDisplayLabel,
  liveHotspotStatus,
  type LiveHotspotStatus,
  type LiveHotspotStatusInput
} from "../../../format/live-hotspot";
import { formatBytes, formatUtc } from "../../../format/units";
import { t, type UiLocale } from "../../../i18n";
import { Card, CardContent, CardHeader, CardTitle } from "../../ui/card";

export const HOTSPOT_STATUSES: readonly LiveHotspotStatus[] = [
  "ready",
  "noMatch",
  "paused",
  "gap",
  "unconfigured",
  "disconnected",
  "unknown"
];

function HotspotCard({
  locale,
  direction,
  hotspot,
  page,
  status
}: {
  locale: UiLocale;
  direction: "download" | "upload";
  hotspot: ConnectionHotspot | null;
  page: LiveConnectionPage | null;
  status: LiveHotspotStatus;
}) {
  const unknown = t(locale, "common.unknown");
  const usable = canShowHotspotValue(status) && hotspot !== null;
  const showFacts = canShowHotspotSnapshotFacts(status);
  const Icon = direction === "download" ? ArrowDownToLine : ArrowUpFromLine;

  return (
    <Card data-state={status} className="gap-3">
      <CardHeader className="flex flex-row items-start justify-between gap-2">
        <CardTitle className="flex items-center gap-2 text-sm font-semibold">
          <Icon className="size-4 text-muted-foreground" aria-hidden="true" />
          {t(locale, `live.hotspot.${direction}`)}
        </CardTitle>
        <span className="text-xs text-muted-foreground">{t(locale, `live.hotspot.direction.${direction}`)}</span>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        {usable && hotspot ? (
          <>
            <p className="truncate font-medium">{hotspotDisplayLabel(hotspot, unknown)}</p>
            <p className="truncate text-xs text-muted-foreground">{hotspotDisplayDetail(hotspot, unknown)}</p>
            <p className="text-lg font-semibold tabular-nums">{formatBytes(hotspot.value, unknown)}</p>
          </>
        ) : null}
        {showFacts ? (
          <dl className="grid grid-cols-3 gap-2 text-xs">
            <div>
              <dt className="text-muted-foreground">{t(locale, "live.hotspot.matched")}</dt>
              <dd className="tabular-nums">{page === null ? unknown : String(page.matchedCount)}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">{t(locale, "live.hotspot.sample")}</dt>
              <dd>{formatUtc(page?.sampleUtc ?? null, t(locale, "common.no_sample"))}</dd>
            </div>
            <div>
              <dt className="text-muted-foreground">{t(locale, "live.hotspot.state")}</dt>
              <dd>{t(locale, `live.hotspot.status.${status}`)}</dd>
            </div>
          </dl>
        ) : null}
      </CardContent>
    </Card>
  );
}

export function HotspotCards({
  locale,
  page,
  statusInput
}: {
  locale: UiLocale;
  page: LiveConnectionPage | null;
  statusInput: LiveHotspotStatusInput;
}) {
  const status = liveHotspotStatus(statusInput);
  return (
    <section className="flex min-w-0 flex-col gap-3" aria-labelledby="live-hotspot-title">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <h2 id="live-hotspot-title" className="text-sm font-semibold">
            {t(locale, "live.hotspot.title")}
          </h2>
          <p className="text-xs text-muted-foreground">{t(locale, "live.hotspot.scope")}</p>
        </div>
        <p className="text-xs text-muted-foreground" data-state={status} role="status" aria-live="polite">
          {t(locale, `live.hotspot.status.${status}`)}
        </p>
      </div>
      <div className="grid gap-3 md:grid-cols-2">
        <HotspotCard
          locale={locale}
          direction="download"
          hotspot={page?.summary.topDownload ?? null}
          page={page}
          status={status}
        />
        <HotspotCard
          locale={locale}
          direction="upload"
          hotspot={page?.summary.topUpload ?? null}
          page={page}
          status={status}
        />
      </div>
    </section>
  );
}
