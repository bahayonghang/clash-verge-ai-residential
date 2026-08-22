import type { RouteId } from "./dto";

export type BusinessRouteId = Exclude<RouteId, "settings-data">;

export const BUSINESS_NAV_TINTS: Record<BusinessRouteId, string> = {
  overview: "var(--nav-overview)",
  live: "var(--nav-live)",
  residential: "var(--nav-residential)",
  host: "var(--nav-host)",
  rule: "var(--nav-rule)",
  chain: "var(--nav-chain)",
  process: "var(--nav-process)",
  reports: "var(--nav-reports)",
  alerts: "var(--nav-alerts)"
};

export function businessNavTint(id: RouteId): string | null {
  if (id === "settings-data") {
    return null;
  }
  return BUSINESS_NAV_TINTS[id];
}
