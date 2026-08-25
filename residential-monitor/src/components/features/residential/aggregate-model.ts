import type { ReportFilters, ReportResult } from "../../../dto";
import { emptyReportFilters, RESIDENTIAL_ACCOUNTING_FILTER } from "../../../format/rank";

export type ResidentialDirection = "upload" | "download";

export function residentialReportFilters(): ReportFilters {
  return {
    ...emptyReportFilters(),
    category: RESIDENTIAL_ACCOUNTING_FILTER
  };
}

export function matchesResidentialRankQuery(
  result: ReportResult | null,
  direction: ResidentialDirection,
  topN: number
): result is ReportResult {
  if (!result) {
    return false;
  }
  const query = result.queryEcho;
  return (
    query.grouping === "host" &&
    query.filters.category === RESIDENTIAL_ACCOUNTING_FILTER &&
    query.sort.field === direction &&
    query.sort.descending &&
    query.topN === topN
  );
}

export function directionTraffic(
  row: { upload: number; download: number },
  direction: ResidentialDirection
): number {
  return row[direction];
}

export function shouldShowResidentialRankLoading(
  requestLoading: boolean,
  errorZh: string | null,
  hasRetainedResult: boolean,
  hasMatchingResult: boolean
): boolean {
  return (
    requestLoading ||
    (errorZh === null && hasRetainedResult && !hasMatchingResult)
  );
}

export function newestFirstSeries(
  series: ReportResult["series"]
): ReportResult["series"] {
  return [...series].sort((left, right) => right.bucketUtc - left.bucketUtc);
}
