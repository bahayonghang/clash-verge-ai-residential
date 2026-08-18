export function formatBytes(value: number | null): string {
  if (value === null) {
    return "未知";
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

export function formatRate(value: number | null): string {
  if (value === null) {
    return "未知";
  }
  return `${formatBytes(value)}/s`;
}

export function formatUtc(value: number | null): string {
  if (value === null) {
    return "无采样";
  }
  return new Date(value * 1000).toLocaleString();
}

export function unknownOr(value: string | null | undefined): string {
  return value && value.length > 0 ? value : "未知";
}
