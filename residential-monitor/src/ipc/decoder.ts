import {
  SCHEMA_VERSION,
  type AlertSummary,
  type HealthView,
  type LiveOverview,
  type MonitorStreamMessage,
  type ObservationPhase
} from "../dto";

const OVERVIEW_FIELDS = [
  "schemaVersion",
  "observationPhase",
  "meterUpload",
  "meterDownload",
  "attributedUpload",
  "attributedDownload",
  "categoryUpload",
  "categoryDownload",
  "otherUpload",
  "otherDownload",
  "gapUpload",
  "gapDownload",
  "overUpload",
  "overDownload",
  "activeCount",
  "lastSampleUtc",
  "coverageKind",
  "coverageReason",
  "health"
] as const;

const OBSERVATION_PHASES = new Set<ObservationPhase>([
  "unconfigured",
  "connecting",
  "baselinePending",
  "current",
  "paused",
  "disconnected",
  "resyncRequired",
  "decodeFailed"
]);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function optionalNumber(value: unknown): number | null {
  if (value === null) {
    return null;
  }
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new Error("数字字段无效");
  }
  return value;
}

function requiredNumber(value: unknown, name: string): number {
  if (typeof value !== "number" || !Number.isSafeInteger(value) || value < 0) {
    throw new Error(`${name} 必须是数字`);
  }
  return value;
}

function numericRecord(value: unknown, name: string): Record<string, number> {
  if (!isRecord(value)) {
    throw new Error(`${name} 必须是对象`);
  }
  return Object.fromEntries(
    Object.entries(value).map(([key, item]) => [key, requiredNumber(item, `${name}.${key}`)])
  );
}

function requiredString(value: unknown, name: string): string {
  if (typeof value !== "string") {
    throw new Error(`${name} 必须是字符串`);
  }
  return value;
}

function decodeHealth(value: unknown): HealthView {
  if (!isRecord(value)) {
    throw new Error("health 缺失");
  }
  for (const field of ["session", "storageOk", "storageReason"] as const) {
    if (!Object.prototype.hasOwnProperty.call(value, field)) {
      throw new Error(`health 字段缺失: ${field}`);
    }
  }
  if (typeof value.storageOk !== "boolean") {
    throw new Error("health.storageOk 必须是布尔值");
  }
  return {
    session: requiredString(value.session, "health.session"),
    storageOk: value.storageOk,
    storageReason:
      value.storageReason === null
        ? null
        : requiredString(value.storageReason, "health.storageReason")
  };
}

export function decodeOverview(value: unknown): LiveOverview {
  if (!isRecord(value)) {
    throw new Error("概览必须是对象");
  }
  for (const field of OVERVIEW_FIELDS) {
    if (!Object.prototype.hasOwnProperty.call(value, field)) {
      throw new Error(`概览字段缺失: ${field}`);
    }
  }
  if (value.schemaVersion !== SCHEMA_VERSION) {
    throw new Error("不支持的 schemaVersion");
  }
  if (!OBSERVATION_PHASES.has(value.observationPhase as ObservationPhase)) {
    throw new Error("observationPhase 无效");
  }
  return {
    schemaVersion: SCHEMA_VERSION,
    observationPhase: value.observationPhase as ObservationPhase,
    meterUpload: optionalNumber(value.meterUpload),
    meterDownload: optionalNumber(value.meterDownload),
    attributedUpload: optionalNumber(value.attributedUpload),
    attributedDownload: optionalNumber(value.attributedDownload),
    categoryUpload: numericRecord(value.categoryUpload, "categoryUpload"),
    categoryDownload: numericRecord(value.categoryDownload, "categoryDownload"),
    otherUpload: optionalNumber(value.otherUpload),
    otherDownload: optionalNumber(value.otherDownload),
    gapUpload: optionalNumber(value.gapUpload),
    gapDownload: optionalNumber(value.gapDownload),
    overUpload: optionalNumber(value.overUpload),
    overDownload: optionalNumber(value.overDownload),
    activeCount: requiredNumber(value.activeCount, "activeCount"),
    lastSampleUtc: optionalNumber(value.lastSampleUtc),
    coverageKind: value.coverageKind === null
      ? null
      : requiredString(value.coverageKind, "coverageKind"),
    coverageReason: value.coverageReason === null
      ? null
      : requiredString(value.coverageReason, "coverageReason"),
    health: decodeHealth(value.health)
  };
}

export function decodeMonitorMessage(value: unknown): MonitorStreamMessage {
  if (!isRecord(value)) {
    throw new Error("Channel 消息必须是对象");
  }
  if (value.schemaVersion !== SCHEMA_VERSION) {
    throw new Error("不支持的 schemaVersion");
  }
  const kind = requiredString(value.kind, "kind");
  const subscriptionId = requiredNumber(value.subscriptionId, "subscriptionId");
  if (kind === "bootstrap") {
    return {
      kind,
      schemaVersion: SCHEMA_VERSION,
      subscriptionId,
      snapshot: decodeOverview(value.snapshot),
      baseSeq: requiredNumber(value.baseSeq, "baseSeq"),
      backendTime: requiredNumber(value.backendTime, "backendTime")
    };
  }
  if (kind === "connectionDelta") {
    if (!Array.isArray(value.upserts) || !Array.isArray(value.removes)) {
      throw new Error("connectionDelta 字段不完整");
    }
    return {
      kind,
      schemaVersion: SCHEMA_VERSION,
      subscriptionId,
      seq: requiredNumber(value.seq, "seq"),
      snapshot: decodeOverview(value.snapshot),
      upserts: value.upserts.map((item) => {
        if (!isRecord(item)) {
          throw new Error("connectionDelta upsert 无效");
        }
        return {
          identity: requiredString(item.identity, "identity"),
          connectionId: requiredString(item.connectionId, "connectionId"),
          epoch: requiredNumber(item.epoch, "epoch"),
          upload: requiredNumber(item.upload, "upload"),
          download: requiredNumber(item.download, "download"),
          rateUpload: optionalNumber(item.rateUpload),
          rateDownload: optionalNumber(item.rateDownload),
          durationMs: optionalNumber(item.durationMs),
          primary: item.primary == null ? null : requiredString(item.primary, "primary"),
          tags: Array.isArray(item.tags) ? item.tags.map((tag) => String(tag)) : [],
          host: item.host == null ? null : String(item.host),
          sourceIp: item.sourceIp == null ? null : String(item.sourceIp),
          destinationIp: item.destinationIp == null ? null : String(item.destinationIp),
          processName: item.processName == null ? null : String(item.processName),
          processPath: item.processPath == null ? null : String(item.processPath),
          network: item.network == null ? null : String(item.network),
          inbound: item.inbound == null ? null : String(item.inbound),
          sourcePort: item.sourcePort == null ? null : String(item.sourcePort),
          destinationPort: item.destinationPort == null ? null : String(item.destinationPort),
          start: item.start == null ? null : String(item.start),
          rule: item.rule == null ? null : String(item.rule),
          rulePayload: item.rulePayload == null ? null : String(item.rulePayload),
          chains: Array.isArray(item.chains) ? item.chains.map((node) => String(node)) : []
        };
      }),
      removes: value.removes.map((item) => requiredString(item, "remove")),
      backendTime: requiredNumber(value.backendTime, "backendTime")
    };
  }
  if (kind === "healthChanged") {
    return {
      kind,
      schemaVersion: SCHEMA_VERSION,
      subscriptionId,
      seq: requiredNumber(value.seq, "seq"),
      health: decodeHealth(value.health),
      backendTime: requiredNumber(value.backendTime, "backendTime")
    };
  }
  if (kind === "summaryChanged") {
    return {
      kind,
      schemaVersion: SCHEMA_VERSION,
      subscriptionId,
      seq: requiredNumber(value.seq, "seq"),
      snapshot: decodeOverview(value.snapshot),
      backendTime: requiredNumber(value.backendTime, "backendTime")
    };
  }
  if (kind === "alertChanged") {
    if (!isRecord(value.summary)) {
      throw new Error("alertChanged.summary 缺失");
    }
    const summary = value.summary as unknown as AlertSummary;
    return {
      kind,
      schemaVersion: SCHEMA_VERSION,
      subscriptionId,
      seq: requiredNumber(value.seq, "seq"),
      summary,
      backendTime: requiredNumber(value.backendTime, "backendTime")
    };
  }
  throw new Error("未知 Channel 消息变体");
}
