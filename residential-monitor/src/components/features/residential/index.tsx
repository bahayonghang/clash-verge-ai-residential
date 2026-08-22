import type { BootstrapDto, LiveOverview, RouteId } from "../../../dto";
import { useResidentialShare } from "../../../hooks/use-residential-share";
import { t, type UiLocale } from "../../../i18n";
import type { MonitorState } from "../../../ipc/reducer";
import type { TimeRange } from "../../../lib/time-range";
import { AggregateSection } from "./aggregate-section";
import { MonitorSection } from "./monitor-section";
import { ReportSection } from "./report-section";
import { TargetEmpty } from "./target-empty";

export function ResidentialPage({
  locale,
  boot,
  stream,
  autoRefresh,
  timeRange,
  overview,
  onRouteChange,
  onResubscribe
}: {
  locale: UiLocale;
  boot: BootstrapDto;
  stream: MonitorState;
  autoRefresh: boolean;
  timeRange: TimeRange;
  overview: LiveOverview;
  onRouteChange: (route: RouteId) => void;
  onResubscribe: () => void;
}) {
  const share = useResidentialShare(timeRange);
  const targetsEmpty = share.share !== null && share.share.targetCount === 0;

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-lg font-semibold">{t(locale, "route.residential")}</h1>
        <p className="text-xs text-muted-foreground">{t(locale, "residential.caliber.diff")}</p>
      </div>
      <MonitorSection
        locale={locale}
        boot={boot}
        stream={stream}
        autoRefresh={autoRefresh}
        overview={overview}
        onRouteChange={onRouteChange}
        onResubscribe={onResubscribe}
      />
      {targetsEmpty ? (
        <TargetEmpty locale={locale} onGoSettings={() => onRouteChange("settings-data")} />
      ) : (
        <>
          <AggregateSection
            locale={locale}
            timeRange={timeRange}
            share={share.share}
            shareLoading={share.loading}
            shareError={share.errorZh}
          />
          <ReportSection locale={locale} timeRange={timeRange} />
        </>
      )}
    </div>
  );
}
