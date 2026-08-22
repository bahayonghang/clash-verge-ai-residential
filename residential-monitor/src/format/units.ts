export function formatBytes(value: number | null, unknown = "未知"): string {
  if (value === null) {
    return unknown;
  }
  const units = ["B", "KiB", "MiB", "GiB"];
  let amount = value;
  let unit = 0;
  while (amount >= 1024 && unit < units.length - 1) {
    amount /= 1024;
    unit += 1;
  }
  return `${amount.toFixed(unit === 0 ? 0 : 1)} ${units[unit]}`;
}

export function formatRate(value: number | null, unknown = "未知"): string {
  if (value === null) {
    return unknown;
  }
  return `${formatBytes(value, unknown)}/s`;
}

export function formatUtc(value: number | null, empty = "无采样"): string {
  if (value === null) {
    return empty;
  }
  return new Date(value * 1000).toLocaleString();
}

export function unknownOr(value: string | null | undefined, fallback = "未知"): string {
  return value && value.length > 0 ? value : fallback;
}
