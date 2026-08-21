import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  SCHEMA_VERSION,
  type BootstrapDto,
  type ControllerSettings,
  type RecoveryStatus,
  type RouteDescriptor
} from "../dto";
import { parseUiLocale, t } from "../i18n";
import { decodeOverview } from "../ipc/decoder";
import { isTauriRuntime } from "../ipc/live-session";
import { isRouteId } from "../nav-icons";
import { parseUiSidebarWidth, SHELL_WIDTH_DEFAULT } from "../shell-width";
import { parseUiDensity, parseUiFont, parseUiFontSize, parseUiTheme } from "../theme";
import { invokeErrorZh } from "../lib/utils";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function decodeRoute(value: unknown): RouteDescriptor | null {
  if (!isRecord(value) || typeof value.id !== "string" || !isRouteId(value.id)) {
    return null;
  }
  return {
    id: value.id,
    titleZh: typeof value.titleZh === "string" ? value.titleZh : value.id,
    available: value.available !== false,
    unavailableUntil: value.unavailableUntil == null ? null : String(value.unavailableUntil)
  };
}

function decodeSettings(value: unknown): ControllerSettings {
  if (!isRecord(value)) {
    throw new Error("settings 缺失");
  }
  return {
    transport: typeof value.transport === "string" ? value.transport : "",
    address: typeof value.address === "string" ? value.address : "",
    credentialTarget: typeof value.credentialTarget === "string" ? value.credentialTarget : "",
    hasSecret: value.hasSecret === true,
    secretMode: typeof value.secretMode === "string" ? value.secretMode : "none"
  };
}

function decodeRecovery(value: unknown): RecoveryStatus | null {
  if (value == null) {
    return null;
  }
  if (!isRecord(value)) {
    throw new Error("recovery 无效");
  }
  return {
    schemaVersion: typeof value.schemaVersion === "number" ? value.schemaVersion : SCHEMA_VERSION,
    appVersion: typeof value.appVersion === "string" ? value.appVersion : "",
    userVersion: typeof value.userVersion === "number" ? value.userVersion : 0,
    supportedMax: typeof value.supportedMax === "number" ? value.supportedMax : 0,
    future: value.future === true,
    restoreAvailable: value.restoreAvailable === true,
    restoreNoteZh: typeof value.restoreNoteZh === "string" ? value.restoreNoteZh : "",
    backups: Array.isArray(value.backups) ? value.backups.map((item) => String(item)) : []
  };
}

export function previewBootstrap(): BootstrapDto {
  return {
    schemaVersion: SCHEMA_VERSION,
    branch: "normal-ready",
    routes: [
      { id: "overview", titleZh: "概览", available: true, unavailableUntil: null },
      { id: "live", titleZh: "实时连接", available: true, unavailableUntil: null },
      { id: "residential", titleZh: "家宽", available: true, unavailableUntil: null },
      { id: "host", titleZh: "主机", available: true, unavailableUntil: null },
      { id: "rule", titleZh: "规则", available: true, unavailableUntil: null },
      { id: "chain", titleZh: "链路", available: true, unavailableUntil: null },
      { id: "process", titleZh: "进程", available: true, unavailableUntil: null },
      { id: "reports", titleZh: "分析报告", available: true, unavailableUntil: null },
      { id: "alerts", titleZh: "告警", available: true, unavailableUntil: null },
      { id: "settings-data", titleZh: "设置 / 数据管理", available: true, unavailableUntil: null }
    ],
    overview: {
      schemaVersion: SCHEMA_VERSION,
      observationPhase: "unconfigured",
      meterUpload: null,
      meterDownload: null,
      attributedUpload: null,
      attributedDownload: null,
      categoryUpload: {},
      categoryDownload: {},
      otherUpload: null,
      otherDownload: null,
      gapUpload: null,
      gapDownload: null,
      overUpload: null,
      overDownload: null,
      activeCount: 0,
      lastSampleUtc: null,
      coverageKind: null,
      coverageReason: null,
      health: { session: "no_data", storageOk: true, storageReason: null }
    },
    settings: {
      transport: "tcp",
      address: "",
      credentialTarget: "io.github.bahayonghang.residential-monitor/controller",
      hasSecret: false,
      secretMode: "none"
    },
    wizardComplete: false,
    recovery: null,
    launchMode: "interactive",
    uiLocale: "zh",
    uiTheme: "mocha",
    uiFont: "system",
    uiFontSize: "md",
    uiDensity: "comfortable",
    uiSidebarWidth: SHELL_WIDTH_DEFAULT,
    logDir: ""
  };
}

export function decodeBootstrap(value: unknown): BootstrapDto {
  if (!isRecord(value)) {
    throw new Error("引导数据必须是对象");
  }
  if (value.schemaVersion !== SCHEMA_VERSION) {
    throw new Error("不支持的 schemaVersion");
  }
  if (value.branch !== "normal-ready" && value.branch !== "recovery-only") {
    throw new Error("未知启动分支");
  }
  if (!Array.isArray(value.routes)) {
    throw new Error("routes 缺失");
  }
  const routes: RouteDescriptor[] = [];
  for (const item of value.routes) {
    const route = decodeRoute(item);
    if (!route) {
      throw new Error("routes 含未知或无效项");
    }
    routes.push(route);
  }
  return {
    schemaVersion: SCHEMA_VERSION,
    branch: value.branch,
    routes,
    overview: decodeOverview(value.overview),
    settings: decodeSettings(value.settings),
    wizardComplete: value.wizardComplete === true,
    recovery: decodeRecovery(value.recovery),
    launchMode: value.launchMode === "background" ? "background" : "interactive",
    uiLocale: parseUiLocale(value.uiLocale),
    uiTheme: parseUiTheme(value.uiTheme),
    uiFont: parseUiFont(value.uiFont),
    uiFontSize: parseUiFontSize(value.uiFontSize),
    uiDensity: parseUiDensity(value.uiDensity),
    uiSidebarWidth: parseUiSidebarWidth(value.uiSidebarWidth),
    logDir: typeof value.logDir === "string" ? value.logDir : ""
  };
}

export function useBootstrap(): { boot: BootstrapDto | null; error: string | null } {
  const [boot, setBoot] = useState<BootstrapDto | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const fallback = t("zh", "boot.fail");
    if (!isTauriRuntime()) {
      setBoot(previewBootstrap());
      return;
    }
    void invoke<unknown>("get_bootstrap")
      .then((raw) => {
        if (cancelled) {
          return;
        }
        setBoot(decodeBootstrap(raw));
        setError(null);
      })
      .catch((caught: unknown) => {
        if (cancelled) {
          return;
        }
        setBoot(null);
        setError(invokeErrorZh(caught, fallback));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return { boot, error };
}
