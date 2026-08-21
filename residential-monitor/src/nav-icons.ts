import type { RouteId } from "./dto";
import markApp from "./assets/icons/mark-app.jpg";
import iconOverview from "./assets/icons/overview.jpg";
import iconLive from "./assets/icons/live.jpg";
import iconResidential from "./assets/icons/residential.svg";
import iconHost from "./assets/icons/host.svg";
import iconRule from "./assets/icons/rule.svg";
import iconChain from "./assets/icons/chain.svg";
import iconProcess from "./assets/icons/process.svg";
import iconReports from "./assets/icons/reports.jpg";
import iconAlerts from "./assets/icons/alerts.jpg";
import iconSettings from "./assets/icons/settings.jpg";

export const BRAND_MARK = markApp;

export const ROUTE_ORDER: RouteId[] = [
  "overview",
  "live",
  "residential",
  "host",
  "rule",
  "chain",
  "process",
  "reports",
  "alerts",
  "settings-data"
];

export const BUSINESS_ROUTES: RouteId[] = ROUTE_ORDER.filter((id) => id !== "settings-data");

export const ROUTE_ICONS: Record<RouteId, string> = {
  overview: iconOverview,
  live: iconLive,
  residential: iconResidential,
  host: iconHost,
  rule: iconRule,
  chain: iconChain,
  process: iconProcess,
  reports: iconReports,
  alerts: iconAlerts,
  "settings-data": iconSettings
};

export function isRouteId(value: string): value is RouteId {
  return Object.hasOwn(ROUTE_ICONS, value);
}
