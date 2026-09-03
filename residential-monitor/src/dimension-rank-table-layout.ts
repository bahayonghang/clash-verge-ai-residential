export const DATA_COLUMNS = [
  "name",
  "upload",
  "download",
  "connections",
  "share",
  "attribution"
] as const;

export type RankDataColumnId = (typeof DATA_COLUMNS)[number];

export const RANK_COL_WIDTH = 48;
export const DRILL_COL_WIDTH = 88;
export const WIDTH_MIN = 48;
export const WIDTH_MAX = 640;

export const NUMERIC_COLUMNS: readonly RankDataColumnId[] = [
  "upload",
  "download",
  "connections",
  "share"
];

const DEFAULT_WIDTH: Record<RankDataColumnId, number> = {
  name: 280,
  upload: 88,
  download: 88,
  connections: 72,
  share: 64,
  attribution: 160
};

export interface DimensionRankTableLayout {
  widths: Record<string, number>;
}

export function isNumericRankColumn(column: RankDataColumnId): boolean {
  return (NUMERIC_COLUMNS as readonly string[]).includes(column);
}

export function defaultDimensionRankTableLayout(): DimensionRankTableLayout {
  return sanitizeDimensionRankTableLayout({ widths: {} });
}

export function sanitizeDimensionRankTableLayout(
  input: Partial<DimensionRankTableLayout> | DimensionRankTableLayout
): DimensionRankTableLayout {
  const widths: Record<string, number> = {};
  const inputWidths = input.widths && typeof input.widths === "object" ? input.widths : {};
  for (const column of DATA_COLUMNS) {
    const raw = inputWidths[column];
    const value = typeof raw === "number" && Number.isFinite(raw) ? raw : DEFAULT_WIDTH[column];
    widths[column] = Math.min(WIDTH_MAX, Math.max(WIDTH_MIN, Math.round(value)));
  }
  return { widths };
}

export function parseDimensionRankTableLayout(value: unknown): DimensionRankTableLayout {
  if (!value || typeof value !== "object") {
    return defaultDimensionRankTableLayout();
  }
  const record = value as { widths?: unknown };
  const widths =
    record.widths && typeof record.widths === "object" && !Array.isArray(record.widths)
      ? (record.widths as Record<string, number>)
      : {};
  return sanitizeDimensionRankTableLayout({ widths });
}

export function visibleRankDataColumns(showAttribution: boolean): RankDataColumnId[] {
  return DATA_COLUMNS.filter((column) => showAttribution || column !== "attribution");
}

export function columnWidth(layout: DimensionRankTableLayout, column: RankDataColumnId): number {
  return layout.widths[column] ?? DEFAULT_WIDTH[column];
}

export function rankTablePixelWidth(
  layout: DimensionRankTableLayout,
  options: { attribution: boolean; drill: boolean }
): number {
  const data = visibleRankDataColumns(options.attribution).reduce(
    (sum, column) => sum + columnWidth(layout, column),
    RANK_COL_WIDTH
  );
  return options.drill ? data + DRILL_COL_WIDTH : data;
}

export function setRankColumnWidth(
  layout: DimensionRankTableLayout,
  column: RankDataColumnId,
  width: number
): DimensionRankTableLayout {
  return sanitizeDimensionRankTableLayout({
    widths: { ...layout.widths, [column]: width }
  });
}
