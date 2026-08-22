import type { DataColumnId } from "./live-table-layout";

export interface LiveSortState {
  sortField: string;
  descending: boolean;
}

export function nextLiveSort(column: DataColumnId, current: LiveSortState): LiveSortState {
  if (current.sortField !== column) {
    return { sortField: column, descending: true };
  }
  if (current.descending) {
    return { sortField: column, descending: false };
  }
  return { sortField: "identity", descending: false };
}

export function sortAria(column: DataColumnId, current: LiveSortState): "none" | "ascending" | "descending" {
  if (current.sortField !== column) {
    return "none";
  }
  return current.descending ? "descending" : "ascending";
}

export function sortMarker(column: DataColumnId, current: LiveSortState): string {
  const aria = sortAria(column, current);
  if (aria === "descending") {
    return " ▼";
  }
  if (aria === "ascending") {
    return " ▲";
  }
  return "";
}
