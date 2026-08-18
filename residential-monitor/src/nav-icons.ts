import type { RouteId } from "./dto";
import markApp from "./assets/icons/mark-app.jpg";
import iconOverview from "./assets/icons/overview.jpg";
import iconLive from "./assets/icons/live.jpg";
import iconReports from "./assets/icons/reports.jpg";
import iconAlerts from "./assets/icons/alerts.jpg";
import iconSettings from "./assets/icons/settings.jpg";

export const BRAND_MARK = markApp;

export const ROUTE_ICONS: Record<RouteId, string> = {
  overview: iconOverview,
  live: iconLive,
  reports: iconReports,
  alerts: iconAlerts,
  "settings-data": iconSettings
};
