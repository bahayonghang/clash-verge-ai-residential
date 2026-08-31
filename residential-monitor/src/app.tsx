import { lazy, Suspense, useEffect, useState, type ReactNode } from "react";
import { UnavailablePage } from "./components/features/recovery/unavailable";
import { Header } from "./components/layout/header";
import { Shell } from "./components/layout/shell";
import { Card, CardDescription, CardHeader, CardTitle } from "./components/ui/card";
import { Skeleton } from "./components/ui/skeleton";
import type { BootstrapDto, LiveOverview, ReportQuery, RouteId } from "./dto";
import { useBootstrap } from "./hooks/use-bootstrap";
import { useMonitorStream } from "./hooks/use-monitor-stream";
import { usePreferences } from "./hooks/use-preferences";
import { useSidebarResize } from "./hooks/use-sidebar-resize";
import { t, type UiLocale } from "./i18n";
import type { MonitorState } from "./ipc/reducer";
import { healthOf } from "./lib/health";
import {
  defaultTimeRange,
  rollTimeRange,
  startRollingTimeRange,
  timeRangeFromPreset,
  type TimeRange
} from "./lib/time-range";

const OverviewPage = lazy(() =>
  import("./components/features/overview").then((module) => ({ default: module.OverviewPage }))
);
const LivePage = lazy(() =>
  import("./components/features/live").then((module) => ({ default: module.LivePage }))
);
const ResidentialPage = lazy(() =>
  import("./components/features/residential").then((module) => ({ default: module.ResidentialPage }))
);
const DimensionPage = lazy(() =>
  import("./components/features/dimension/dimension-page").then((module) => ({
    default: module.DimensionPage
  }))
);
const ReportsPage = lazy(() =>
  import("./components/features/reports").then((module) => ({ default: module.ReportsPage }))
);
const AlertsPage = lazy(() =>
  import("./components/features/alerts").then((module) => ({ default: module.AlertsPage }))
);
const SettingsPage = lazy(() =>
  import("./components/features/settings").then((module) => ({ default: module.SettingsPage }))
);
const RecoveryPage = lazy(() =>
  import("./components/features/recovery").then((module) => ({ default: module.RecoveryPage }))
);

function PageFallback() {
  return (
    <div className="flex h-full items-center justify-center p-6">
      <Skeleton className="h-8 w-48" />
    </div>
  );
}

export function App() {
  const { boot, error: bootError } = useBootstrap();
  const preferences = usePreferences(boot);
  const [resyncTick, setResyncTick] = useState(0);
  const stream = useMonitorStream(boot, preferences.prefs.locale, resyncTick);
  const [route, setRoute] = useState<RouteId>("overview");
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [timeRange, setTimeRange] = useState<TimeRange>(() => defaultTimeRange());
  const [reportJump, setReportJump] = useState<ReportQuery | null>(null);
  const { displayWidth, ...resize } = useSidebarResize(
    preferences.prefs.sidebarWidth,
    preferences.commitSidebarWidth
  );

  useEffect(() => {
    if (boot?.branch === "recovery-only") {
      setRoute("settings-data");
    }
  }, [boot]);

  useEffect(() => {
    const skip = document.getElementById("skip-link");
    if (skip) {
      skip.textContent = t(preferences.prefs.locale, "a11y.skip");
    }
  }, [preferences.prefs.locale]);

  useEffect(() => {
    if (!autoRefresh) {
      return;
    }
    return startRollingTimeRange(() => {
      setTimeRange((current) => rollTimeRange(current));
    });
  }, [autoRefresh]);

  if (!boot) {
    return (
      <div className="flex h-full items-center justify-center p-6">
        {bootError ? (
          <p className="text-sm text-destructive" role="alert">
            {bootError}
          </p>
        ) : (
          <Skeleton className="h-8 w-48" />
        )}
      </div>
    );
  }

  const locale = preferences.prefs.locale;
  const recovery = boot.branch === "recovery-only";
  const session = stream.snapshot?.health.session ?? boot.overview.health.session;
  const health = healthOf(locale, session);
  const errorZh = preferences.errorZh ?? stream.errorZh ?? bootError;

  return (
    <Shell
      locale={locale}
      route={route}
      recovery={recovery}
      healthSession={session}
      healthLabel={health.title}
      width={displayWidth}
      onRouteChange={setRoute}
      resize={resize}
      errorZh={errorZh}
      header={
        <Header
          locale={locale}
          theme={preferences.prefs.theme}
          font={preferences.prefs.font}
          fontSize={preferences.prefs.fontSize}
          density={preferences.prefs.density}
          fonts={preferences.fonts}
          fontsError={preferences.fontsError}
          healthSession={session}
          healthLabel={health.title}
          healthAction={health.action}
          autoRefresh={autoRefresh}
          timeRange={timeRange}
          onLocaleChange={(next) => void preferences.setLocale(next)}
          onThemeChange={(next) => void preferences.setTheme(next)}
          onFontChange={(next) => void preferences.setFont(next)}
          onFontSizeChange={(next) => void preferences.setFontSize(next)}
          onDensityChange={(next) => void preferences.setDensity(next)}
          onAutoRefreshToggle={() => setAutoRefresh((current) => !current)}
          onTimeRangeChange={(preset) => setTimeRange(timeRangeFromPreset(preset))}
        />
      }
    >
      {recovery ? (
        <Suspense fallback={<PageFallback />}>
          <RecoveryPage locale={locale} boot={boot} />
        </Suspense>
      ) : (
        <Workspace
          route={route}
          locale={locale}
          boot={boot}
          stream={stream}
          autoRefresh={autoRefresh}
          timeRange={timeRange}
          overview={stream.snapshot ?? boot.overview}
          preferences={preferences}
          reportJump={reportJump}
          onRouteChange={setRoute}
          onResubscribe={() => setResyncTick((current) => current + 1)}
          onJumpReport={(query) => {
            setReportJump(query);
            setRoute("reports");
          }}
        />
      )}
    </Shell>
  );
}

function Workspace({
  route,
  locale,
  boot,
  stream,
  autoRefresh,
  timeRange,
  overview,
  preferences,
  reportJump,
  onRouteChange,
  onResubscribe,
  onJumpReport
}: {
  route: RouteId;
  locale: UiLocale;
  boot: BootstrapDto;
  stream: MonitorState;
  autoRefresh: boolean;
  timeRange: TimeRange;
  overview: LiveOverview;
  preferences: ReturnType<typeof usePreferences>;
  reportJump: ReportQuery | null;
  onRouteChange: (route: RouteId) => void;
  onResubscribe: () => void;
  onJumpReport: (query: ReportQuery) => void;
}) {
  const [liveMounted, setLiveMounted] = useState(route === "live");
  if (route === "live" && !liveMounted) {
    setLiveMounted(true);
  }
  const livePage = liveMounted ? (
    <div className={route === "live" ? "contents" : "hidden"}>
      <Suspense fallback={route === "live" ? <PageFallback /> : null}>
        <LivePage
          locale={locale}
          boot={boot}
          stream={stream}
          autoRefresh={autoRefresh}
          active={route === "live"}
          onRouteChange={onRouteChange}
          onResubscribe={onResubscribe}
        />
      </Suspense>
    </div>
  ) : null;
  const descriptor = boot.routes.find((item) => item.id === route);
  if (descriptor && !descriptor.available) {
    return (
      <>
        {livePage}
        <UnavailablePage
          locale={locale}
          name={t(locale, `route.${route}`)}
          until={descriptor.unavailableUntil ?? ""}
        />
      </>
    );
  }

  let rest: ReactNode = null;
  switch (route) {
    case "overview":
      rest = (
        <OverviewPage
          locale={locale}
          timeRange={timeRange}
          overview={overview}
          onNavigate={onRouteChange}
        />
      );
      break;
    case "live":
      break;
    case "residential":
      rest = (
        <ResidentialPage
          locale={locale}
          boot={boot}
          stream={stream}
          autoRefresh={autoRefresh}
          timeRange={timeRange}
          overview={overview}
          onRouteChange={onRouteChange}
          onResubscribe={onResubscribe}
        />
      );
      break;
    case "host":
    case "rule":
    case "chain":
    case "process":
      rest = (
        <DimensionPage
          key={route}
          locale={locale}
          kind={route}
          timeRange={timeRange}
          overview={overview}
        />
      );
      break;
    case "reports":
      rest = <ReportsPage locale={locale} jumpQuery={reportJump} />;
      break;
    case "alerts":
      rest = <AlertsPage locale={locale} onJumpReport={onJumpReport} />;
      break;
    case "settings-data":
      rest = (
        <SettingsPage locale={locale} boot={boot} overview={overview} preferences={preferences} />
      );
      break;
    default:
      rest = unknownRoute(locale, route);
  }
  return (
    <>
      {livePage}
      <Suspense fallback={<PageFallback />}>{rest}</Suspense>
    </>
  );
}

function unknownRoute(locale: UiLocale, _route: never): ReactNode {
  void _route;
  return (
    <Card className="max-w-xl" role="alert">
      <CardHeader>
        <CardTitle>{t(locale, "page.unknown")}</CardTitle>
        <CardDescription>{t(locale, "page.unknown_body")}</CardDescription>
      </CardHeader>
    </Card>
  );
}
