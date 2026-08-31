import type { ReportFilters, ReportResult } from "../../../dto";
import { emptyReportFilters, RESIDENTIAL_ACCOUNTING_FILTER } from "../../../format/rank";

export type ResidentialDirection = "upload" | "download";

export type ResidentialAggregateState =
  | "loading"
  | "refreshing"
  | "error"
  | "uncovered"
  | "unsupported"
  | "empty"
  | "paused"
  | "ready"
  | "pending";

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

export function residentialAggregateState(
  result: ReportResult | null,
  loading: boolean,
  errorZh: string | null,
  autoRefresh: boolean
): ResidentialAggregateState {
  if (errorZh) {
    return "error";
  }
  if (loading) {
    return result ? "refreshing" : "loading";
  }
  if (!result) {
    return autoRefresh ? "pending" : "paused";
  }
  if (result.coverage.coveredSec === 0) {
    return "uncovered";
  }
  if (!result.drilldownCapability.exactTopN) {
    return "unsupported";
  }
  if (result.totals.upload === 0 && result.totals.download === 0) {
    return "empty";
  }
  return autoRefresh ? "ready" : "paused";
}
