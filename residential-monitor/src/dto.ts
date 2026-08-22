import type { UiDensity, UiFont, UiFontSize, UiTheme } from "./theme";

export const SCHEMA_VERSION = 1;

export type RouteId =
  | "overview"
  | "live"
  | "residential"
  | "host"
  | "rule"
  | "chain"
  | "process"
  | "reports"
  | "alerts"
  | "settings-data";

export interface RouteDescriptor {
  id: RouteId;
  titleZh: string;
  available: boolean;
  unavailableUntil: string | null;
}

export interface HealthView {
  session: string;
  storageOk: boolean;
  storageReason: string | null;
}

export type ObservationPhase =
  | "unconfigured"
  | "connecting"
  | "baselinePending"
  | "current"
  | "paused"
  | "disconnected"
  | "resyncRequired"
  | "decodeFailed";

export interface MetadataCoverage {
  connections: number;
  hostPresent: number;
  sniffHostOnly: number;
  destinationIpOnly: number;
  hostAbsent: number;
  processPresent: number;
  processPathOnly: number;
  processAbsent: number;
  chainsPresent: number;
  providerChainsOnly: number;
  chainsAbsent: number;
}

export const EMPTY_METADATA_COVERAGE: MetadataCoverage = {
  connections: 0,
  hostPresent: 0,
  sniffHostOnly: 0,
  destinationIpOnly: 0,
  hostAbsent: 0,
  processPresent: 0,
  processPathOnly: 0,
  processAbsent: 0,
  chainsPresent: 0,
  providerChainsOnly: 0,
  chainsAbsent: 0
};

export interface LiveOverview {
  schemaVersion: number;
  observationPhase: ObservationPhase;
  meterUpload: number | null;
  meterDownload: number | null;
  attributedUpload: number | null;
  attributedDownload: number | null;
  categoryUpload: Record<string, number>;
  categoryDownload: Record<string, number>;
  otherUpload: number | null;
  otherDownload: number | null;
  gapUpload: number | null;
  gapDownload: number | null;
  overUpload: number | null;
  overDownload: number | null;
  activeCount: number;
  lastSampleUtc: number | null;
  coverageKind: string | null;
  coverageReason: string | null;
  health: HealthView;
  metadataCoverage: MetadataCoverage;
}

export interface LiveConnectionView {
  identity: string;
  connectionId: string;
  epoch: number;
  upload: number;
  download: number;
  rateUpload: number | null;
  rateDownload: number | null;
  durationMs: number | null;
  primary: string | null;
  tags: string[];
  host: string | null;
  sourceIp: string | null;
  destinationIp: string | null;
  processName: string | null;
  processPath: string | null;
  network: string | null;
  inbound?: string | null;
  sourcePort?: string | null;
  destinationPort?: string | null;
  start?: string | null;
  rule: string | null;
  rulePayload: string | null;
  chains: string[];
}

export type MonitorStreamMessage =
  | {
      kind: "bootstrap";
      schemaVersion: number;
      subscriptionId: number;
      snapshot: LiveOverview;
      baseSeq: number;
      backendTime: number;
    }
  | {
      kind: "connectionDelta";
      schemaVersion: number;
      subscriptionId: number;
      seq: number;
      snapshot: LiveOverview;
      upserts: LiveConnectionView[];
      removes: string[];
      backendTime: number;
    }
  | {
      kind: "healthChanged";
      schemaVersion: number;
      subscriptionId: number;
      seq: number;
      health: HealthView;
      backendTime: number;
    }
  | {
      kind: "summaryChanged";
      schemaVersion: number;
      subscriptionId: number;
      seq: number;
      snapshot: LiveOverview;
      backendTime: number;
    }
  | {
      kind: "alertChanged";
      schemaVersion: number;
      subscriptionId: number;
      seq: number;
      summary: AlertSummary;
      backendTime: number;
    };

export interface AlertSummary {
  schemaVersion: number;
  activeCount: number;
  notEvaluableCount: number;
  outboxBacklog: number;
  lastEventUtc: number | null;
}

export interface AlertRule {
  ruleId: string;
  version: number;
  enabled: boolean;
  kind: "health" | "rate" | "period-usage";
  selectorKind: "health-kind" | "primary-category" | "domain" | "process";
  selectorValue: string | null;
  direction: "upload" | "download" | "combined" | null;
  thresholdValue: number;
  recoveryThreshold: number | null;
  period: "rolling-1h" | "local-day" | "local-month" | null;
  timezone: string;
  cooldownSec: number;
  quietStartMin: number | null;
  quietEndMin: number | null;
  createdUtc: number;
  updatedUtc: number;
}

export interface AlertEvidence {
  ruleId: string;
  ruleVersion: number;
  dataVersion: number | null;
  evaluatedAtUtc: number;
  windowStartUtc: number | null;
  windowEndUtc: number | null;
  displayTimezone: string;
  selector: string;
  direction: "upload" | "download" | "combined" | null;
  observedValue: number | null;
  triggerThreshold: number;
  recoveryThreshold: number | null;
  coverageSummary: string;
  policyMetadata: string | null;
  reportQuery: ReportQuery | null;
  notEvaluableReason: string | null;
}

export interface AlertInstance {
  instanceId: string;
  ruleId: string;
  ruleVersion: number;
  selectorIdentity: string;
  status: "inactive" | "active" | "not-evaluable" | "resolved" | "superseded";
  startedUtc: number | null;
  resolvedUtc: number | null;
  lastEvalUtc: number;
  lastObserved: number | null;
  evidence: AlertEvidence;
}

export interface AlertCenterPage {
  schemaVersion: number;
  items: AlertInstance[];
  nextCursor: string | null;
}

export interface DiagnosticsSnapshot {
  schemaVersion: number;
  appVersion: string;
  sqliteUserVersion: number;
  supportedSchema: number;
  c4Checksum: string;
  journalMode: string;
  synchronous: string;
  controllerTransportStatus: string;
  coverageSummary: string;
  writerWatermark: number;
  writerReceipts: number;
  lastFrameUtc: number | null;
  reconnectHintZh: string;
  databaseOk: boolean;
  walCheckpointOk: boolean;
  backupRetentionNoteZh: string;
  alertActive: number;
  outboxBacklog: number;
  recentRedactedErrorClasses: string[];
  metadataCoverage: MetadataCoverage;
}

export interface NotifyCapability {
  available: boolean;
  reasonZh: string;
  canFocusApp: boolean;
  focusAssistUnknown: boolean;
}

export interface ControllerSettings {
  transport: string;
  address: string;
  credentialTarget: string;
  hasSecret: boolean;
  secretMode: string;
}

export interface RecoveryStatus {
  schemaVersion: number;
  appVersion: string;
  userVersion: number;
  supportedMax: number;
  future: boolean;
  restoreAvailable: boolean;
  restoreNoteZh: string;
  backups: string[];
}

export interface BootstrapDto {
  schemaVersion: number;
  branch: "normal-ready" | "recovery-only";
  routes: RouteDescriptor[];
  overview: LiveOverview;
  settings: ControllerSettings;
  wizardComplete: boolean;
  recovery: RecoveryStatus | null;
  launchMode: "interactive" | "background";
  uiLocale?: "zh" | "en";
  uiTheme?: UiTheme;
  uiFont?: UiFont;
  uiFontSize?: UiFontSize;
  uiDensity?: UiDensity;
  uiSidebarWidth?: number;
  liveTableLayout?: { widths: Record<string, number>; hidden: string[] };
  logDir?: string;
}

export interface OperationProgress {
  schemaVersion: number;
  operationId: string;
  kind: string;
  phase: string;
  current: number;
  total: number;
  unit: string;
  canCancel: boolean;
  status: string;
  redactedError: string | null;
}

export interface CloseState {
  requestId: string;
  identity: string;
  mark: "accepted" | "closed" | "unconfirmed";
}

export interface ReportFilters {
  category: string | null;
  host: string | null;
  process: string | null;
  rule: string | null;
  chain: string | null;
  network: string | null;
}

export interface ReportQuery {
  rangeStartUtc: number;
  rangeEndUtc: number;
  displayTimezone: string;
  granularity: "minute1" | "minute2" | "minute5" | "minute10" | "hour" | "day" | "month";
  filters: ReportFilters;
  grouping: "category" | "host" | "process" | "rule" | "chain" | "network";
  targetPolicy: "current" | "historical";
  comparison: { previousEqualWindow: boolean } | null;
  sort: { field: "upload" | "download" | "name" | "identity"; descending: boolean };
  page: { limit: number; after: string | null };
  topN: number;
  includeSessions: boolean;
}

export interface ReportResult {
  schemaVersion: number;
  dataVersion: number;
  reportSnapshotToken: string;
  queryEcho: ReportQuery;
  totals: {
    upload: number;
    download: number;
    connectionCount: number;
    activeDurationSec: number;
    previousUpload: number | null;
    previousDownload: number | null;
  };
  series: Array<{
    bucketUtc: number;
    upload: number;
    download: number;
    connectionCount: number;
    activeDurationSec: number;
  }>;
  rankings: Array<{
    /** identity `"__unknown__"` 表示维度值缺失；前端按未知渲染，不参与下钻。 */
    identity: string;
    label: string;
    upload: number;
    download: number;
    connectionCount: number;
    activeDurationSec: number;
  }>;
  coverage: {
    status: string;
    coveredSec: number;
    gapSec: number;
    slices: Array<{ kind: string; reason: string; startedUtc: number; endedUtc: number | null }>;
  };
  attributionQuality: {
    knownUpload: number;
    knownDownload: number;
    missingUpload: number;
    missingDownload: number;
    knownConnections: number;
    missingConnections: number;
    status: "complete" | "partial" | "unavailable";
  };
  drilldownCapability: {
    sessions: boolean;
    currentPolicy: boolean;
    crossDimension: boolean;
    exactTopN: boolean;
    noteZh: string;
  };
  policyMetadata: { targetPolicy: string; policyVersion: number | null; noteZh: string };
  dataTier: string;
  namedSql: string[];
  unit: string;
  generatedUtc: number;
}

export interface ResidentialShare {
  schemaVersion: number;
  residentialUpload: number | null;
  residentialDownload: number | null;
  attributedUpload: number | null;
  attributedDownload: number | null;
  coverageStatus: string;
  namedSql: string[];
  generatedUtc: number;
  targetCount: number;
  policyVersion: number | null;
}

export interface RetentionPreview {
  rawRetainDays: number;
  rawRows: number;
  hourlyRows: number;
  dailyDimRows: number;
  dailyCoreRows: number;
  autoDeleteEnabled: boolean;
  noteZh: string;
}

export interface AboutDto {
  schemaVersion: number;
  productName: string;
  binaryName: string;
  identifier: string;
  aumid: string;
  version: string;
  releasesUrl: string;
  signed: boolean;
  updaterPlugin: boolean;
  windowsService: boolean;
  signatureNoteZh: string;
}

export interface DeleteItem {
  id: string;
  kind: string;
  path: string;
  exists: boolean;
  noteZh: string;
}

export interface DeletePreview {
  schemaVersion: number;
  confirmPhrase: string;
  items: DeleteItem[];
  noteZh: string;
}

export interface DeleteReport {
  schemaVersion: number;
  allDeclaredOk: boolean;
  items: Array<{ id: string; ok: boolean; existed: boolean; messageZh: string }>;
  summaryZh: string;
}

export function decodeAbout(value: unknown): AboutDto {
  if (!isRecord(value) || value.schemaVersion !== 1 || typeof value.releasesUrl !== "string") {
    throw new Error("AboutDto 无效");
  }
  if (typeof value.signed !== "boolean" || value.signed) {
    throw new Error("未签名候选不得标记为 signed");
  }
  return value as unknown as AboutDto;
}

export function decodeDeletePreview(value: unknown): DeletePreview {
  if (!isRecord(value) || value.schemaVersion !== 1 || !Array.isArray(value.items)) {
    throw new Error("DeletePreview 无效");
  }
  return value as unknown as DeletePreview;
}

export function decodeDeleteReport(value: unknown): DeleteReport {
  if (!isRecord(value) || value.schemaVersion !== 1 || typeof value.allDeclaredOk !== "boolean") {
    throw new Error("DeleteReport 无效");
  }
  return value as unknown as DeleteReport;
}

export function decodeAlertCenter(value: unknown): AlertCenterPage {
  if (!isRecord(value) || value.schemaVersion !== 1 || !Array.isArray(value.items)) {
    throw new Error("AlertCenterPage 无效");
  }
  return value as unknown as AlertCenterPage;
}

export function decodeDiagnostics(value: unknown): DiagnosticsSnapshot {
  if (!isRecord(value) || value.schemaVersion !== 1 || typeof value.c4Checksum !== "string") {
    throw new Error("DiagnosticsSnapshot 无效");
  }
  return value as unknown as DiagnosticsSnapshot;
}

const REPORT_GRANULARITIES = [
  "minute1",
  "minute2",
  "minute5",
  "minute10",
  "hour",
  "day",
  "month"
] as const;

const REPORT_GROUPINGS = ["category", "host", "process", "rule", "chain", "network"] as const;
const REPORT_SORT_FIELDS = ["upload", "download", "name", "identity"] as const;

function own(record: Record<string, unknown>, field: string, owner: string): unknown {
  if (!Object.prototype.hasOwnProperty.call(record, field)) {
    throw new Error(`${owner} 缺失 ${field}`);
  }
  return record[field];
}

function reportInteger(value: unknown, label: string, nonNegative = false): number {
  if (
    typeof value !== "number" ||
    !Number.isSafeInteger(value) ||
    (nonNegative && value < 0)
  ) {
    throw new Error(`ReportResult ${label} 无效`);
  }
  return value;
}

function reportString(value: unknown, label: string): string {
  if (typeof value !== "string") {
    throw new Error(`ReportResult ${label} 无效`);
  }
  return value;
}

function reportBoolean(value: unknown, label: string): boolean {
  if (typeof value !== "boolean") {
    throw new Error(`ReportResult ${label} 无效`);
  }
  return value;
}

function reportNullableString(value: unknown, label: string): string | null {
  return value === null ? null : reportString(value, label);
}

function reportNullableInteger(value: unknown, label: string, nonNegative = false): number | null {
  return value === null ? null : reportInteger(value, label, nonNegative);
}

function decodeReportFilters(value: unknown): ReportFilters {
  if (!isRecord(value)) {
    throw new Error("ReportResult queryEcho.filters 无效");
  }
  return {
    category: reportNullableString(own(value, "category", "filters"), "filters.category"),
    host: reportNullableString(own(value, "host", "filters"), "filters.host"),
    process: reportNullableString(own(value, "process", "filters"), "filters.process"),
    rule: reportNullableString(own(value, "rule", "filters"), "filters.rule"),
    chain: reportNullableString(own(value, "chain", "filters"), "filters.chain"),
    network: reportNullableString(own(value, "network", "filters"), "filters.network")
  };
}

function decodeReportQuery(value: unknown): ReportQuery {
  if (!isRecord(value)) {
    throw new Error("ReportResult queryEcho 无效");
  }
  const granularity = reportString(own(value, "granularity", "queryEcho"), "queryEcho.granularity");
  const grouping = reportString(own(value, "grouping", "queryEcho"), "queryEcho.grouping");
  const targetPolicy = reportString(own(value, "targetPolicy", "queryEcho"), "queryEcho.targetPolicy");
  const sort = own(value, "sort", "queryEcho");
  const page = own(value, "page", "queryEcho");
  const comparison = own(value, "comparison", "queryEcho");
  if (!(REPORT_GRANULARITIES as readonly string[]).includes(granularity)) {
    throw new Error("ReportResult queryEcho.granularity 无效");
  }
  if (!(REPORT_GROUPINGS as readonly string[]).includes(grouping)) {
    throw new Error("ReportResult queryEcho.grouping 无效");
  }
  if (targetPolicy !== "current" && targetPolicy !== "historical") {
    throw new Error("ReportResult queryEcho.targetPolicy 无效");
  }
  if (!isRecord(sort) || !isRecord(page)) {
    throw new Error("ReportResult queryEcho 分页或排序无效");
  }
  const sortField = reportString(own(sort, "field", "sort"), "sort.field");
  if (!(REPORT_SORT_FIELDS as readonly string[]).includes(sortField)) {
    throw new Error("ReportResult queryEcho.sort.field 无效");
  }
  let decodedComparison: ReportQuery["comparison"] = null;
  if (comparison !== null) {
    if (!isRecord(comparison)) {
      throw new Error("ReportResult queryEcho.comparison 无效");
    }
    decodedComparison = {
      previousEqualWindow: reportBoolean(
        own(comparison, "previousEqualWindow", "comparison"),
        "comparison.previousEqualWindow"
      )
    };
  }
  return {
    rangeStartUtc: reportInteger(own(value, "rangeStartUtc", "queryEcho"), "queryEcho.rangeStartUtc"),
    rangeEndUtc: reportInteger(own(value, "rangeEndUtc", "queryEcho"), "queryEcho.rangeEndUtc"),
    displayTimezone: reportString(own(value, "displayTimezone", "queryEcho"), "queryEcho.displayTimezone"),
    granularity: granularity as ReportQuery["granularity"],
    filters: decodeReportFilters(own(value, "filters", "queryEcho")),
    grouping: grouping as ReportQuery["grouping"],
    targetPolicy: targetPolicy as ReportQuery["targetPolicy"],
    comparison: decodedComparison,
    sort: {
      field: sortField as ReportQuery["sort"]["field"],
      descending: reportBoolean(own(sort, "descending", "sort"), "sort.descending")
    },
    page: {
      limit: reportInteger(own(page, "limit", "page"), "page.limit", true),
      after: reportNullableString(own(page, "after", "page"), "page.after")
    },
    topN: reportInteger(own(value, "topN", "queryEcho"), "queryEcho.topN", true),
    includeSessions: reportBoolean(own(value, "includeSessions", "queryEcho"), "queryEcho.includeSessions")
  };
}

function optionalU64(value: unknown, label: string): number | null {
  if (value === null) {
    return null;
  }
  if (typeof value === "number" && Number.isSafeInteger(value) && value >= 0) {
    return value;
  }
  throw new Error(`ResidentialShare ${label} 无效`);
}

export function decodeResidentialShare(value: unknown): ResidentialShare {
  if (!isRecord(value) || value.schemaVersion !== 1) {
    throw new Error("ResidentialShare 无效");
  }
  if (typeof value.coverageStatus !== "string" || !Array.isArray(value.namedSql)) {
    throw new Error("ResidentialShare 字段缺失");
  }
  if (typeof value.generatedUtc !== "number" || !Number.isSafeInteger(value.generatedUtc)) {
    throw new Error("ResidentialShare generatedUtc 无效");
  }
  if (typeof value.targetCount !== "number" || !Number.isSafeInteger(value.targetCount) || value.targetCount < 0) {
    throw new Error("ResidentialShare targetCount 无效");
  }
  let policyVersion: number | null = null;
  if (value.policyVersion !== null && value.policyVersion !== undefined) {
    if (typeof value.policyVersion !== "number" || !Number.isSafeInteger(value.policyVersion)) {
      throw new Error("ResidentialShare policyVersion 无效");
    }
    policyVersion = value.policyVersion;
  }
  return {
    schemaVersion: 1,
    residentialUpload: optionalU64(value.residentialUpload, "residentialUpload"),
    residentialDownload: optionalU64(value.residentialDownload, "residentialDownload"),
    attributedUpload: optionalU64(value.attributedUpload, "attributedUpload"),
    attributedDownload: optionalU64(value.attributedDownload, "attributedDownload"),
    coverageStatus: value.coverageStatus,
    namedSql: value.namedSql.map((item) => String(item)),
    generatedUtc: value.generatedUtc,
    targetCount: value.targetCount,
    policyVersion
  };
}

export function decodeReportResult(value: unknown): ReportResult {
  if (!isRecord(value) || value.schemaVersion !== 1) {
    throw new Error("ReportResult 无效");
  }
  const totals = own(value, "totals", "ReportResult");
  const coverage = own(value, "coverage", "ReportResult");
  const quality = own(value, "attributionQuality", "ReportResult");
  const drilldown = own(value, "drilldownCapability", "ReportResult");
  const policy = own(value, "policyMetadata", "ReportResult");
  const series = own(value, "series", "ReportResult");
  const rankings = own(value, "rankings", "ReportResult");
  const namedSql = own(value, "namedSql", "ReportResult");
  if (
    !isRecord(totals) ||
    !isRecord(coverage) ||
    !isRecord(quality) ||
    !isRecord(drilldown) ||
    !isRecord(policy) ||
    !Array.isArray(series) ||
    !Array.isArray(rankings) ||
    !Array.isArray(namedSql)
  ) {
    throw new Error("ReportResult 字段缺失");
  }
  const qualityStatus = reportString(own(quality, "status", "attributionQuality"), "attributionQuality.status");
  if (!["complete", "partial", "unavailable"].includes(qualityStatus)) {
    throw new Error("ReportResult attributionQuality.status 无效");
  }
  const decoded: ReportResult = {
    schemaVersion: 1,
    dataVersion: reportInteger(own(value, "dataVersion", "ReportResult"), "dataVersion", true),
    reportSnapshotToken: reportString(
      own(value, "reportSnapshotToken", "ReportResult"),
      "reportSnapshotToken"
    ),
    queryEcho: decodeReportQuery(own(value, "queryEcho", "ReportResult")),
    totals: {
      upload: reportInteger(own(totals, "upload", "totals"), "totals.upload", true),
      download: reportInteger(own(totals, "download", "totals"), "totals.download", true),
      connectionCount: reportInteger(
        own(totals, "connectionCount", "totals"),
        "totals.connectionCount",
        true
      ),
      activeDurationSec: reportInteger(
        own(totals, "activeDurationSec", "totals"),
        "totals.activeDurationSec",
        true
      ),
      previousUpload: reportNullableInteger(
        own(totals, "previousUpload", "totals"),
        "totals.previousUpload",
        true
      ),
      previousDownload: reportNullableInteger(
        own(totals, "previousDownload", "totals"),
        "totals.previousDownload",
        true
      )
    },
    series: series.map((item, index) => {
      if (!isRecord(item)) throw new Error(`ReportResult series[${index}] 无效`);
      return {
        bucketUtc: reportInteger(own(item, "bucketUtc", "series"), `series[${index}].bucketUtc`),
        upload: reportInteger(own(item, "upload", "series"), `series[${index}].upload`, true),
        download: reportInteger(own(item, "download", "series"), `series[${index}].download`, true),
        connectionCount: reportInteger(
          own(item, "connectionCount", "series"),
          `series[${index}].connectionCount`,
          true
        ),
        activeDurationSec: reportInteger(
          own(item, "activeDurationSec", "series"),
          `series[${index}].activeDurationSec`,
          true
        )
      };
    }),
    rankings: rankings.map((item, index) => {
      if (!isRecord(item)) throw new Error(`ReportResult rankings[${index}] 无效`);
      return {
        identity: reportString(own(item, "identity", "rankings"), `rankings[${index}].identity`),
        label: reportString(own(item, "label", "rankings"), `rankings[${index}].label`),
        upload: reportInteger(own(item, "upload", "rankings"), `rankings[${index}].upload`, true),
        download: reportInteger(own(item, "download", "rankings"), `rankings[${index}].download`, true),
        connectionCount: reportInteger(
          own(item, "connectionCount", "rankings"),
          `rankings[${index}].connectionCount`,
          true
        ),
        activeDurationSec: reportInteger(
          own(item, "activeDurationSec", "rankings"),
          `rankings[${index}].activeDurationSec`,
          true
        )
      };
    }),
    coverage: {
      status: reportString(own(coverage, "status", "coverage"), "coverage.status"),
      coveredSec: reportInteger(own(coverage, "coveredSec", "coverage"), "coverage.coveredSec", true),
      gapSec: reportInteger(own(coverage, "gapSec", "coverage"), "coverage.gapSec", true),
      slices: (() => {
        const slices = own(coverage, "slices", "coverage");
        if (!Array.isArray(slices)) throw new Error("ReportResult coverage.slices 无效");
        return slices.map((item, index) => {
          if (!isRecord(item)) throw new Error(`ReportResult coverage.slices[${index}] 无效`);
          return {
            kind: reportString(own(item, "kind", "coverage slice"), `coverage.slices[${index}].kind`),
            reason: reportString(own(item, "reason", "coverage slice"), `coverage.slices[${index}].reason`),
            startedUtc: reportInteger(
              own(item, "startedUtc", "coverage slice"),
              `coverage.slices[${index}].startedUtc`
            ),
            endedUtc: reportNullableInteger(
              own(item, "endedUtc", "coverage slice"),
              `coverage.slices[${index}].endedUtc`
            )
          };
        });
      })()
    },
    attributionQuality: {
      knownUpload: reportInteger(own(quality, "knownUpload", "attributionQuality"), "quality.knownUpload", true),
      knownDownload: reportInteger(own(quality, "knownDownload", "attributionQuality"), "quality.knownDownload", true),
      missingUpload: reportInteger(own(quality, "missingUpload", "attributionQuality"), "quality.missingUpload", true),
      missingDownload: reportInteger(own(quality, "missingDownload", "attributionQuality"), "quality.missingDownload", true),
      knownConnections: reportInteger(
        own(quality, "knownConnections", "attributionQuality"),
        "quality.knownConnections",
        true
      ),
      missingConnections: reportInteger(
        own(quality, "missingConnections", "attributionQuality"),
        "quality.missingConnections",
        true
      ),
      status: qualityStatus as ReportResult["attributionQuality"]["status"]
    },
    drilldownCapability: {
      sessions: reportBoolean(own(drilldown, "sessions", "drilldownCapability"), "drilldown.sessions"),
      currentPolicy: reportBoolean(
        own(drilldown, "currentPolicy", "drilldownCapability"),
        "drilldown.currentPolicy"
      ),
      crossDimension: reportBoolean(
        own(drilldown, "crossDimension", "drilldownCapability"),
        "drilldown.crossDimension"
      ),
      exactTopN: reportBoolean(own(drilldown, "exactTopN", "drilldownCapability"), "drilldown.exactTopN"),
      noteZh: reportString(own(drilldown, "noteZh", "drilldownCapability"), "drilldown.noteZh")
    },
    policyMetadata: {
      targetPolicy: reportString(own(policy, "targetPolicy", "policyMetadata"), "policy.targetPolicy"),
      policyVersion: reportNullableInteger(
        own(policy, "policyVersion", "policyMetadata"),
        "policy.policyVersion",
        true
      ),
      noteZh: reportString(own(policy, "noteZh", "policyMetadata"), "policy.noteZh")
    },
    dataTier: reportString(own(value, "dataTier", "ReportResult"), "dataTier"),
    namedSql: namedSql.map((item, index) => reportString(item, `namedSql[${index}]`)),
    unit: reportString(own(value, "unit", "ReportResult"), "unit"),
    generatedUtc: reportInteger(own(value, "generatedUtc", "ReportResult"), "generatedUtc")
  };
  if (
    decoded.attributionQuality.knownUpload + decoded.attributionQuality.missingUpload !==
      decoded.totals.upload ||
    decoded.attributionQuality.knownDownload + decoded.attributionQuality.missingDownload !==
      decoded.totals.download ||
    decoded.attributionQuality.knownConnections + decoded.attributionQuality.missingConnections !==
      decoded.totals.connectionCount
  ) {
    throw new Error("ReportResult attributionQuality 不守恒");
  }
  const hasKnown =
    decoded.attributionQuality.knownUpload > 0 ||
    decoded.attributionQuality.knownDownload > 0 ||
    decoded.attributionQuality.knownConnections > 0;
  const hasMissing =
    decoded.attributionQuality.missingUpload > 0 ||
    decoded.attributionQuality.missingDownload > 0 ||
    decoded.attributionQuality.missingConnections > 0;
  const expectedStatus = hasMissing ? (hasKnown ? "partial" : "unavailable") : "complete";
  if (decoded.attributionQuality.status !== expectedStatus) {
    throw new Error("ReportResult attributionQuality.status 与计数不一致");
  }
  return decoded;
}

export type ReportArchiveKind = "hour" | "day" | "manual";

export interface ReportArchiveSummary {
  archiveId: string;
  kind: ReportArchiveKind;
  rangeStartUtc: number;
  rangeEndUtc: number;
  displayTimezone: string;
  grouping: string;
  status: "ok" | "failed" | string;
  generatedUtc: number;
  dataVersion: number | null;
  coverageStatus: string | null;
  totalsUpload: number | null;
  totalsDownload: number | null;
  connectionCount: number | null;
  errorCode: string | null;
  noteZh: string | null;
}

export interface ReportArchivePage {
  schemaVersion: number;
  items: ReportArchiveSummary[];
  next: string | null;
}

function decodeReportArchiveSummary(value: unknown): ReportArchiveSummary {
  if (
    !isRecord(value) ||
    typeof value.archiveId !== "string" ||
    (value.kind !== "hour" && value.kind !== "day" && value.kind !== "manual") ||
    typeof value.status !== "string" ||
    typeof value.rangeStartUtc !== "number" ||
    typeof value.rangeEndUtc !== "number"
  ) {
    throw new Error("ReportArchivePage 无效");
  }
  return value as unknown as ReportArchiveSummary;
}

export function decodeReportArchivePage(value: unknown): ReportArchivePage {
  if (
    !isRecord(value) ||
    value.schemaVersion !== 1 ||
    !Array.isArray(value.items) ||
    (value.next !== null && typeof value.next !== "string")
  ) {
    throw new Error("ReportArchivePage 无效");
  }
  return {
    schemaVersion: 1,
    items: value.items.map(decodeReportArchiveSummary),
    next: typeof value.next === "string" ? value.next : null
  };
}

export type ShellPhase = "c0-skeleton" | "c2-shell";

export interface ShellStatus {
  schemaVersion: 1;
  kind: "shellStatus";
  identifier: string;
  phase: ShellPhase;
  messageZh: string;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function decodeShellStatus(value: unknown): ShellStatus {
  if (!isRecord(value)) {
    throw new Error("ShellStatus 必须是对象");
  }
  if (value.schemaVersion !== 1) {
    throw new Error("不支持的 schemaVersion");
  }
  if (value.kind !== "shellStatus") {
    throw new Error("kind 必须是 shellStatus");
  }
  if (typeof value.identifier !== "string" || value.identifier.length === 0) {
    throw new Error("identifier 缺失");
  }
  if (value.phase !== "c0-skeleton" && value.phase !== "c2-shell") {
    throw new Error("phase 不受支持");
  }
  if (typeof value.messageZh !== "string" || value.messageZh.length === 0) {
    throw new Error("messageZh 缺失");
  }
  return {
    schemaVersion: 1,
    kind: "shellStatus",
    identifier: value.identifier,
    phase: value.phase,
    messageZh: value.messageZh
  };
}
