export const SCHEMA_VERSION = 1;

export type RouteId = "overview" | "live" | "reports" | "alerts" | "settings-data";

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

export interface LiveOverview {
  schemaVersion: number;
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
  granularity: "hour" | "day" | "month";
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

export interface RetentionPreview {
  rawRetainDays: number;
  rawRows: number;
  hourlyRows: number;
  dailyDimRows: number;
  dailyCoreRows: number;
  autoDeleteEnabled: boolean;
  noteZh: string;
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

export function decodeReportResult(value: unknown): ReportResult {
  if (!isRecord(value) || value.schemaVersion !== 1) {
    throw new Error("ReportResult 无效");
  }
  if (typeof value.reportSnapshotToken !== "string" || !isRecord(value.totals) || !isRecord(value.coverage)) {
    throw new Error("ReportResult 字段缺失");
  }
  return value as unknown as ReportResult;
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
