import type { LiveFilterClause } from "../ipc/live-session";

export const NUMERIC_FILTER_FIELDS = [
  "download",
  "upload",
  "rateDownload",
  "rateUpload",
  "duration"
] as const;

export const NUMERIC_MODES = ["gt", "gte", "lt", "lte", "eq"] as const;

export type NumericFilterField = (typeof NUMERIC_FILTER_FIELDS)[number];

const BYTE_UNITS = ["B", "KiB", "MiB", "GiB"] as const;
const RATE_UNITS = ["B", "KiB", "MiB"] as const;
const TIME_UNITS = ["s", "min", "h"] as const;

const FACTOR: Record<string, number> = {
  B: 1,
  KiB: 1024,
  MiB: 1024 * 1024,
  GiB: 1024 * 1024 * 1024,
  s: 1000,
  min: 60_000,
  h: 3_600_000
};

export function isNumericFilterField(field: string): field is NumericFilterField {
  return (NUMERIC_FILTER_FIELDS as readonly string[]).includes(field);
}

export function defaultFilterUnit(field: string): string {
  if (field === "duration") {
    return "min";
  }
  return "KiB";
}

export function unitsForField(field: string): readonly string[] {
  if (field === "duration") {
    return TIME_UNITS;
  }
  if (field === "rateDownload" || field === "rateUpload") {
    return RATE_UNITS;
  }
  return BYTE_UNITS;
}

export function unitLabelKey(field: string, unit: string): string {
  if (field === "rateDownload" || field === "rateUpload") {
    return `live.filter.unit.${unit}_s`;
  }
  return `live.filter.unit.${unit}`;
}

export function toQueryMagnitude(raw: string, unit: string): string | null {
  const trimmed = raw.trim();
  if (trimmed.length === 0) {
    return null;
  }
  const amount = Number(trimmed);
  const factor = FACTOR[unit];
  if (!Number.isFinite(amount) || amount < 0 || factor == null) {
    return null;
  }
  const product = amount * factor;
  if (!Number.isFinite(product) || product >= 2 ** 64) {
    return null;
  }
  return String(Math.round(product));
}

export function clauseForField(field: string): LiveFilterClause {
  if (isNumericFilterField(field)) {
    return {
      field,
      mode: "gte",
      value: "",
      unit: defaultFilterUnit(field)
    };
  }
  return { field, mode: "contains", value: "" };
}

export function toQueryClause(clause: LiveFilterClause): LiveFilterClause {
  if (!isNumericFilterField(clause.field)) {
    return { field: clause.field, mode: clause.mode, value: clause.value };
  }
  const allowed = unitsForField(clause.field);
  const unit = clause.unit && (allowed as readonly string[]).includes(clause.unit)
    ? clause.unit
    : defaultFilterUnit(clause.field);
  return {
    field: clause.field,
    mode: clause.mode,
    value: toQueryMagnitude(clause.value, unit) ?? ""
  };
}
