import type { ReportFilters } from "../dto";
import { t, type UiLocale } from "../i18n";

export const UNKNOWN_RANK_IDENTITY = "__unknown__";

/** 核算口径过滤哨兵：命中任一重点目标。不是字典值。 */
export const RESIDENTIAL_ACCOUNTING_FILTER = "__residential__";

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

export function missingDimensionLabel(
  locale: UiLocale,
  kind: DimensionKind
): string {
  return t(locale, `dimension.missing.${kind}`);
}

export function looksLikeIp(value: string): boolean {
  const trimmed = value.trim().replace(/^\[/, "").replace(/\]$/, "");
  if (/^(\d{1,3}\.){3}\d{1,3}$/.test(trimmed)) {
    return trimmed.split(".").every((part) => {
      const n = Number(part);
      return Number.isInteger(n) && n >= 0 && n <= 255;
    });
  }
  return trimmed.includes(":") && /^[0-9a-fA-F:]+$/.test(trimmed);
}

export function formatRankLabel(
  identity: string,
  label: string,
  unknown: string,
  missingLabel = unknown
): string {
  const text = isUnknownIdentity(identity)
    ? missingLabel
    : rankDisplayLabel(identity, label, unknown);
  if (!isUnknownIdentity(identity) && looksLikeIp(identity)) {
    return `${text}  IP`;
  }
  return text;
}

export function ellipsizeLabel(label: string, maxChars: number): string {
  if (maxChars < 2 || label.length <= maxChars) {
    return label;
  }
  return `…${label.slice(-(maxChars - 1))}`;
}

export function rankAxisWidth(labels: string[]): number {
  const longest = labels.reduce((max, label) => Math.max(max, label.length), 0);
  return Math.min(220, Math.max(96, longest * 7 + 12));
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
    if (kind === "host") {
      return { ...base, host: identity };
    }
    if (kind === "process") {
      return { ...base, process: identity };
    }
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
