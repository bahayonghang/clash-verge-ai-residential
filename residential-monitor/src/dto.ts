export type ShellPhase = "c0-skeleton";

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
  if (value.phase !== "c0-skeleton") {
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
