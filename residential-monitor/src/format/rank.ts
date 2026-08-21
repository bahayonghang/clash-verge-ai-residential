import type { ReportFilters } from "../dto";

export const UNKNOWN_RANK_IDENTITY = "__unknown__";

export type DimensionKind = "host" | "process" | "rule" | "chain";

export type TopSort = "traffic" | "connections";

export const DIMENSION_KINDS: DimensionKind[] = ["host", "rule", "chain", "process"];

export const TOP_N_OPTIONS = [10, 20, 50, 100] as const;

export type TopNOption = (typeof TOP_N_OPTIONS)[number];

export function isUnknownIdentity(identity: string): boolean {
  return identity === UNKNOWN_RANK_IDENTITY;
}

export function rankDisplayLabel(identity: string, label: string, unknown: string): string {
  if (isUnknownIdentity(identity)) {
    return unknown;
  }
  return label.length > 0 ? label : unknown;
}

export function rankingTraffic(row: { upload: number; download: number }): number {
  return row.upload + row.download;
}

/** 占比分母必须是 totals，不得用可见行之和。 */
export function rankingShare(value: number, total: number): number {
  return total > 0 ? value / total : 0;
}

export function emptyReportFilters(): ReportFilters {
  return {
    category: null,
    host: null,
    process: null,
    rule: null,
    chain: null,
    network: null
  };
}

export function filtersForDrilldown(
  kind: DimensionKind,
  identity: string,
  base: ReportFilters = emptyReportFilters()
): ReportFilters {
  if (isUnknownIdentity(identity)) {
    return { ...base };
  }
  return { ...base, [kind]: identity };
}

export function drilldownTargets(kind: DimensionKind): [DimensionKind, ...DimensionKind[]] {
  switch (kind) {
    case "host":
      return ["rule", "chain", "process"];
    case "rule":
      return ["chain", "host"];
    case "chain":
      return ["rule", "host"];
    case "process":
      return ["host", "chain"];
  }
}

export function rankingSortValue(
  row: { upload: number; download: number; connectionCount: number; label: string },
  sort: TopSort
): number {
  return sort === "traffic" ? rankingTraffic(row) : row.connectionCount;
}
