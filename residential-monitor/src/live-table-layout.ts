export const DATA_COLUMNS = [
  "host",
  "download",
  "upload",
  "rateDownload",
  "rateUpload",
  "chain",
  "rule",
  "process",
  "duration",
  "source",
  "destination",
  "type"
] as const;

export type DataColumnId = (typeof DATA_COLUMNS)[number];

export const ACTION_COLUMN = "action";
export const ACTION_WIDTH = 76;
export const WIDTH_MAX = 640;

export const NUMERIC_COLUMNS: readonly DataColumnId[] = [
  "download",
  "upload",
  "rateDownload",
  "rateUpload",
  "duration"
];

const DEFAULT_WIDTH: Record<DataColumnId, number> = {
  host: 180,
  download: 88,
  upload: 88,
  rateDownload: 104,
  rateUpload: 104,
  chain: 280,
  rule: 220,
  process: 180,
  duration: 108,
  source: 160,
  destination: 160,
  type: 120
};

const MIN_WIDTH: Record<DataColumnId, number> = {
  host: 140,
  download: 72,
  upload: 72,
  rateDownload: 80,
  rateUpload: 80,
  chain: 160,
  rule: 160,
  process: 100,
  duration: 88,
  source: 120,
  destination: 120,
  type: 80
};

const LABEL_KEY: Record<DataColumnId, string> = {
  host: "live.col.host",
  download: "live.col.download",
  upload: "live.col.upload",
  rateDownload: "live.col.dl_speed",
  rateUpload: "live.col.ul_speed",
  chain: "live.col.chains",
  rule: "live.col.rule",
  process: "live.col.process",
  duration: "live.col.time",
  source: "live.col.source",
  destination: "live.col.destination",
  type: "live.col.type"
};

export interface LiveTableLayout {
  widths: Record<string, number>;
  hidden: string[];
}

export function isDataColumn(value: string): value is DataColumnId {
  return (DATA_COLUMNS as readonly string[]).includes(value);
}

export function defaultWidth(column: DataColumnId): number {
  return DEFAULT_WIDTH[column];
}

export function minWidth(column: DataColumnId): number {
  return MIN_WIDTH[column];
}

export function columnLabelKey(column: DataColumnId): string {
  return LABEL_KEY[column];
}

export function isNumericColumn(column: DataColumnId): boolean {
  return (NUMERIC_COLUMNS as readonly string[]).includes(column);
}

export function defaultLiveTableLayout(): LiveTableLayout {
  return sanitizeLiveTableLayout({ widths: {}, hidden: [] });
}

export function sanitizeLiveTableLayout(input: Partial<LiveTableLayout> | LiveTableLayout): LiveTableLayout {
  const widths: Record<string, number> = {};
  const inputWidths = input.widths && typeof input.widths === "object" ? input.widths : {};
  for (const column of DATA_COLUMNS) {
    const raw = inputWidths[column];
    const value = typeof raw === "number" && Number.isFinite(raw) ? raw : DEFAULT_WIDTH[column];
    widths[column] = Math.min(WIDTH_MAX, Math.max(MIN_WIDTH[column], Math.round(value)));
  }
  const hidden: string[] = [];
  const seen = new Set<string>();
  const inputHidden = Array.isArray(input.hidden) ? input.hidden : [];
  for (const column of inputHidden) {
    if (!isDataColumn(column) || seen.has(column)) {
      continue;
    }
    if (seen.size + 1 >= DATA_COLUMNS.length) {
      continue;
    }
    seen.add(column);
    hidden.push(column);
  }
  return { widths, hidden };
}

export function parseLiveTableLayout(value: unknown): LiveTableLayout {
  if (!value || typeof value !== "object") {
    return defaultLiveTableLayout();
  }
  const record = value as { widths?: unknown; hidden?: unknown };
  const widths =
    record.widths && typeof record.widths === "object" && !Array.isArray(record.widths)
      ? (record.widths as Record<string, number>)
      : {};
  const hidden = Array.isArray(record.hidden)
    ? record.hidden.filter((item): item is string => typeof item === "string")
    : [];
  return sanitizeLiveTableLayout({ widths, hidden });
}

export function visibleDataColumns(layout: LiveTableLayout): DataColumnId[] {
  const hidden = new Set(layout.hidden);
  return DATA_COLUMNS.filter((column) => !hidden.has(column));
}

export function columnWidth(layout: LiveTableLayout, column: DataColumnId): number {
  return layout.widths[column] ?? DEFAULT_WIDTH[column];
}

export function tablePixelWidth(layout: LiveTableLayout): number {
  return visibleDataColumns(layout).reduce((sum, column) => sum + columnWidth(layout, column), ACTION_WIDTH);
}

export function setColumnWidth(layout: LiveTableLayout, column: DataColumnId, width: number): LiveTableLayout {
  return sanitizeLiveTableLayout({
    widths: { ...layout.widths, [column]: width },
    hidden: layout.hidden
  });
}

export function setColumnHidden(layout: LiveTableLayout, column: DataColumnId, hidden: boolean): LiveTableLayout {
  if (!hidden) {
    return sanitizeLiveTableLayout({
      widths: layout.widths,
      hidden: layout.hidden.filter((item) => item !== column)
    });
  }
  return sanitizeLiveTableLayout({
    widths: layout.widths,
    hidden: [...layout.hidden, column]
  });
}
