export interface CategoryRow {
  name: string;
  upload: number | null;
  download: number | null;
}

export function categoryRows(
  upload: Record<string, number>,
  download: Record<string, number>
): CategoryRow[] {
  const names = new Set([...Object.keys(upload), ...Object.keys(download)]);
  return [...names]
    .sort((left, right) => left.localeCompare(right, "zh"))
    .map((name) => ({
      name,
      upload: Object.hasOwn(upload, name) ? upload[name] : null,
      download: Object.hasOwn(download, name) ? download[name] : null
    }));
}
