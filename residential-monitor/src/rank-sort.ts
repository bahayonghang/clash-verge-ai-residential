import type { ReportQuery } from "./dto";

export type RankSortField = ReportQuery["sort"]["field"];
export type RankSortSpec = ReportQuery["sort"];

export const RANK_SORT_FIELDS = ["upload", "download", "name", "identity"] as const;

export const DEFAULT_RANK_SORT: RankSortSpec = { field: "download", descending: true };

export function isRankSortField(value: string): value is RankSortField {
  return (RANK_SORT_FIELDS as readonly string[]).includes(value);
}

export function nextRankSort(column: RankSortField, current: RankSortSpec): RankSortSpec {
  if (current.field === column) {
    return { field: column, descending: !current.descending };
  }
  return { field: column, descending: column !== "name" && column !== "identity" };
}

export function rankSortAria(
  column: RankSortField,
  current: RankSortSpec
): "none" | "ascending" | "descending" {
  if (current.field !== column) {
    return "none";
  }
  return current.descending ? "descending" : "ascending";
}
