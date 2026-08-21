import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  decodeAbout,
  decodeDeletePreview,
  decodeDeleteReport,
  type AboutDto,
  type BootstrapDto,
  type ControllerSettings,
  type DeletePreview,
  type DeleteReport,
  type OperationProgress,
  type RetentionPreview
} from "../dto";
import { t, type UiLocale } from "../i18n";
import { fetchTraySummary, isTauriRuntime } from "../ipc/live-session";
import { invokeErrorZh } from "../lib/utils";

export interface ProbeView {
  messageZh: string;
  state: string;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
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

function decodeProgress(value: unknown): OperationProgress {
  if (!isRecord(value) || typeof value.operationId !== "string") {
    throw new Error("OperationProgress 无效");
  }
  return value as unknown as OperationProgress;
}

function decodeRetention(value: unknown): RetentionPreview {
  if (!isRecord(value) || typeof value.noteZh !== "string") {
    throw new Error("RetentionPreview 无效");
  }
  return value as unknown as RetentionPreview;
}

export function useSettings(locale: UiLocale, boot: BootstrapDto | null): {
  address: string;
  targets: string;
  secret: string;
  settings: ControllerSettings | null;
  collectorRunning: boolean | null;
  probe: ProbeView;
  about: AboutDto | null;
  aboutLoading: boolean;
  aboutError: string;
  deletePreview: DeletePreview | null;
  deleteReport: DeleteReport | null;
  retention: RetentionPreview | null;
  dataDir: string;
  progress: OperationProgress | null;
  errorZh: string | null;
  setAddress: (value: string) => void;
  setTargets: (value: string) => void;
  setSecret: (value: string) => void;
  loadSecret: () => Promise<void>;
  saveConnection: () => Promise<void>;
  testConnection: () => Promise<void>;
  disconnect: () => Promise<void>;
  reconnect: () => Promise<void>;
  pauseCollector: () => Promise<void>;
  resumeCollector: () => Promise<void>;
  refreshCollector: () => Promise<void>;
  loadAbout: (force: boolean) => Promise<void>;
  openReleases: () => Promise<string | null>;
  previewDelete: () => Promise<void>;
  confirmDelete: (phrase: string) => Promise<void>;
  previewRetention: () => Promise<void>;
  runRetention: () => Promise<void>;
  loadDataDir: () => Promise<void>;
  openLogDir: () => Promise<void>;
  createBackup: () => Promise<void>;
  restoreBackup: () => Promise<void>;
  validateBackup: () => Promise<boolean | null>;
  vacuum: () => Promise<void>;
  completeWizard: () => Promise<void>;
  cancelOperation: () => Promise<void>;
} {
  const seq = useRef(0);
  const [address, setAddress] = useState(boot?.settings.address ?? "");
  const [targets, setTargets] = useState("家宽");
  const [secret, setSecret] = useState("");
  const secretLoaded = useRef(false);
  const [settings, setSettings] = useState<ControllerSettings | null>(boot?.settings ?? null);
  const [collectorRunning, setCollectorRunning] = useState<boolean | null>(null);
  const [probe, setProbe] = useState<ProbeView>({ messageZh: "", state: "no_data" });
  const [about, setAbout] = useState<AboutDto | null>(null);
  const [aboutLoading, setAboutLoading] = useState(false);
  const [aboutError, setAboutError] = useState("");
  const aboutLoaded = useRef(false);
  const aboutLoadingRef = useRef(false);
  const [deletePreview, setDeletePreview] = useState<DeletePreview | null>(null);
  const [deleteReport, setDeleteReport] = useState<DeleteReport | null>(null);
  const [retention, setRetention] = useState<RetentionPreview | null>(null);
  const [dataDir, setDataDir] = useState("");
  const [progress, setProgress] = useState<OperationProgress | null>(null);
  const [errorZh, setErrorZh] = useState<string | null>(null);

  useEffect(() => {
    if (!boot) {
      return;
    }
    setAddress(boot.settings.address);
    setSettings(boot.settings);
  }, [boot]);

  const loadSecret = useCallback(async (): Promise<void> => {
    if (secretLoaded.current) {
      return;
    }
    secretLoaded.current = true;
    if (!isTauriRuntime() || !boot?.settings.hasSecret) {
      return;
    }
    const token = ++seq.current;
    try {
      const value = await invoke<string | null>("get_controller_secret");
      if (token !== seq.current) {
        return;
      }
      setSecret(value ?? "");
    } catch {
      if (token !== seq.current) {
        return;
      }
    }
  }, [boot?.settings.hasSecret]);

  const saveConnection = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    const fallback = t(locale, "settings.save_fail");
    if (!isTauriRuntime()) {
      setErrorZh(fallback);
      return;
    }
    try {
      const saved = decodeSettings(
        await invoke<unknown>("save_settings", {
          address,
          secret: secret.length > 0 ? secret : null,
          sessionOnly: false
        })
      );
      await invoke("save_targets", {
        targets: targets
          .split(",")
          .map((item) => item.trim())
          .filter(Boolean)
      });
      if (token !== seq.current) {
        return;
      }
      setSettings(saved);
      setErrorZh(null);
      setProbe({ messageZh: t(locale, "settings.saved"), state: "connected" });
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, fallback));
      setProbe({ messageZh: fallback, state: "storage_failure" });
    }
  }, [address, locale, secret, targets]);

  const testConnection = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    const fallback = t(locale, "settings.connect_fail");
    setProbe({ messageZh: t(locale, "settings.probing"), state: "connecting" });
    if (!isTauriRuntime()) {
      setProbe({ messageZh: fallback, state: "storage_failure" });
      return;
    }
    try {
      const result = await invoke<{ messageZh: string; status: string; action: string }>(
        "test_controller",
        {
          address,
          secret: secret.length > 0 ? secret : null
        }
      );
      if (token !== seq.current) {
        return;
      }
      const extra = result.status === "endpoint_missing" ? t(locale, "settings.verge_port") : "";
      setProbe({
        messageZh: `${result.messageZh}${result.action}${extra}`,
        state: result.status
      });
      setErrorZh(null);
      const next = decodeSettings((await invoke<unknown>("get_settings")) as unknown);
      if (token === seq.current) {
        setSettings(next);
      }
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      const message = invokeErrorZh(caught, fallback);
      setProbe({ messageZh: message, state: "storage_failure" });
      setErrorZh(message);
    }
  }, [address, locale, secret]);

  const disconnect = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    const fallback = t(locale, "settings.disconnect_fail");
    if (!isTauriRuntime()) {
      setErrorZh(fallback);
      return;
    }
    try {
      const result = await invoke<{ messageZh: string; status: string; action: string }>(
        "disconnect_controller"
      );
      if (token !== seq.current) {
        return;
      }
      setProbe({ messageZh: `${result.messageZh}${result.action}`, state: result.status });
      setErrorZh(null);
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, fallback));
      setProbe({ messageZh: fallback, state: "storage_failure" });
    }
  }, [locale]);

  const reconnect = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    setProbe({ messageZh: t(locale, "settings.reconnecting"), state: "connecting" });
    const fallback = t(locale, "settings.connect_fail");
    if (!isTauriRuntime()) {
      setProbe({ messageZh: fallback, state: "storage_failure" });
      return;
    }
    try {
      await invoke("reconnect_now");
      if (token !== seq.current) {
        return;
      }
      setProbe({ messageZh: t(locale, "settings.reconnected"), state: "connected" });
      setErrorZh(null);
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      const message = invokeErrorZh(caught, fallback);
      setProbe({ messageZh: message, state: "storage_failure" });
      setErrorZh(message);
    }
  }, [locale]);

  const refreshCollector = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    if (!isTauriRuntime()) {
      return;
    }
    try {
      const tray = await fetchTraySummary();
      if (token !== seq.current) {
        return;
      }
      setCollectorRunning(tray.collectorRunning);
    } catch {
      if (token !== seq.current) {
        return;
      }
      setCollectorRunning(null);
    }
  }, []);

  const pauseCollector = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    if (!isTauriRuntime()) {
      return;
    }
    try {
      await invoke("pause_collector");
      if (token !== seq.current) {
        return;
      }
      setErrorZh(null);
      await refreshCollector();
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, t(locale, "settings.connect_fail")));
    }
  }, [locale, refreshCollector]);

  const resumeCollector = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    if (!isTauriRuntime()) {
      return;
    }
    try {
      await invoke("resume_collector");
      if (token !== seq.current) {
        return;
      }
      setErrorZh(null);
      await refreshCollector();
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, t(locale, "settings.connect_fail")));
    }
  }, [locale, refreshCollector]);

  const loadAbout = useCallback(
    async (force: boolean): Promise<void> => {
      if (aboutLoadingRef.current) {
        return;
      }
      if (!force && aboutLoaded.current) {
        return;
      }
      aboutLoadingRef.current = true;
      setAboutLoading(true);
      if (force) {
        setAbout(null);
        setAboutError("");
        aboutLoaded.current = false;
      }
      const token = ++seq.current;
      const fallback = t(locale, "settings.about_fail");
      try {
        if (!isTauriRuntime()) {
          throw new Error(fallback);
        }
        const next = decodeAbout(await invoke<unknown>("get_about"));
        if (token !== seq.current) {
          return;
        }
        setAbout(next);
        setAboutError("");
        aboutLoaded.current = true;
      } catch (caught: unknown) {
        if (token !== seq.current) {
          return;
        }
        setAbout(null);
        setAboutError(invokeErrorZh(caught, fallback));
        aboutLoaded.current = true;
      } finally {
        aboutLoadingRef.current = false;
        if (token === seq.current) {
          setAboutLoading(false);
        }
      }
    },
    [locale]
  );

  const openReleases = useCallback(async (): Promise<string | null> => {
    const token = ++seq.current;
    if (!isTauriRuntime()) {
      return about?.releasesUrl ?? null;
    }
    try {
      const url = await invoke<string>("open_releases");
      if (token !== seq.current) {
        return null;
      }
      return url;
    } catch {
      if (token !== seq.current) {
        return null;
      }
      return about?.releasesUrl ?? null;
    }
  }, [about?.releasesUrl]);

  const previewDelete = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    const fallback = t(locale, "settings.delete.preview_fail");
    if (!isTauriRuntime()) {
      setErrorZh(fallback);
      return;
    }
    try {
      const next = decodeDeletePreview(await invoke<unknown>("preview_delete_local_data"));
      if (token !== seq.current) {
        return;
      }
      setDeletePreview(next);
      setErrorZh(null);
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, fallback));
    }
  }, [locale]);

  const confirmDelete = useCallback(
    async (phrase: string): Promise<void> => {
      const token = ++seq.current;
      const fallback = t(locale, "settings.delete.confirm_fail");
      if (!isTauriRuntime()) {
        setErrorZh(fallback);
        return;
      }
      try {
        const next = decodeDeleteReport(
          await invoke<unknown>("confirm_delete_local_data", { phrase })
        );
        if (token !== seq.current) {
          return;
        }
        setDeleteReport(next);
        setErrorZh(null);
      } catch (caught: unknown) {
        if (token !== seq.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, fallback));
      }
    },
    [locale]
  );

  const withOperation = useCallback(
    async (
      kind: string,
      fallback: string,
      task: () => Promise<"done" | "cancelled">
    ): Promise<void> => {
      const token = ++seq.current;
      const operationId = `op-${Date.now()}`;
      try {
        if (isTauriRuntime()) {
          const started = decodeProgress(
            await invoke<unknown>("start_operation", { operationId, kind })
          );
          if (token === seq.current) {
            setProgress(started);
          }
        }
        const result = await task();
        if (token !== seq.current) {
          return;
        }
        if (result === "cancelled") {
          setProgress(null);
          return;
        }
        setProgress((current) =>
          current && current.operationId === operationId
            ? { ...current, status: "done", canCancel: false, current: current.total }
            : current
        );
        setErrorZh(null);
      } catch (caught: unknown) {
        if (token !== seq.current) {
          return;
        }
        const message = invokeErrorZh(caught, fallback);
        setErrorZh(message);
        setProgress((current) =>
          current && current.operationId === operationId
            ? { ...current, status: "error", canCancel: false, redactedError: message }
            : current
        );
      }
    },
    []
  );

  const previewRetention = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    const fallback = t(locale, "settings.retention_preview");
    if (!isTauriRuntime()) {
      return;
    }
    try {
      const next = decodeRetention(await invoke<unknown>("retention_preview"));
      if (token !== seq.current) {
        return;
      }
      setRetention(next);
      setErrorZh(null);
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, fallback));
    }
  }, [locale]);

  const runRetention = useCallback(async (): Promise<void> => {
    await withOperation("retention", t(locale, "settings.retention_preview"), async () => {
      const next = decodeRetention(await invoke<unknown>("run_retention", { delete: false }));
      setRetention(next);
      return "done";
    });
  }, [locale, withOperation]);

  const loadDataDir = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    if (!isTauriRuntime()) {
      return;
    }
    try {
      const dir = await invoke<string>("data_directory");
      if (token !== seq.current) {
        return;
      }
      setDataDir(dir);
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, t(locale, "settings.log_dir_unknown")));
    }
  }, [locale]);

  const openLogDir = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    const fallback = t(locale, "settings.open_log_dir_fail");
    if (!isTauriRuntime()) {
      setErrorZh(fallback);
      return;
    }
    try {
      await invoke("open_log_dir");
      if (token !== seq.current) {
        return;
      }
      setErrorZh(null);
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, fallback));
    }
  }, [locale]);

  const createBackup = useCallback(async (): Promise<void> => {
    await withOperation("backup", t(locale, "settings.backup_fail"), async () => {
      const picked = await invoke<string | null>("pick_file", {
        purpose: "backup-create",
        mode: "save"
      });
      if (!picked) {
        return "cancelled";
      }
      await invoke("create_backup", { path: picked });
      return "done";
    });
  }, [withOperation, locale]);

  const restoreBackup = useCallback(async (): Promise<void> => {
    await withOperation("restore", t(locale, "settings.restore_fail"), async () => {
      const picked = await invoke<string | null>("pick_file", {
        purpose: "backup-restore",
        mode: "open"
      });
      if (!picked) {
        return "cancelled";
      }
      await invoke("restore_backup", { path: picked });
      return "done";
    });
  }, [locale, withOperation]);

  const validateBackup = useCallback(async (): Promise<boolean | null> => {
    const token = ++seq.current;
    const fallback = t(locale, "settings.validate_fail");
    if (!isTauriRuntime()) {
      setErrorZh(fallback);
      return null;
    }
    try {
      const picked = await invoke<string | null>("pick_file", {
        purpose: "backup-restore",
        mode: "open"
      });
      if (!picked) {
        return null;
      }
      const ok = await invoke<boolean>("validate_backup", { path: picked });
      if (token !== seq.current) {
        return null;
      }
      setErrorZh(ok ? null : fallback);
      return ok;
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return null;
      }
      setErrorZh(invokeErrorZh(caught, fallback));
      return false;
    }
  }, [locale]);

  const vacuum = useCallback(async (): Promise<void> => {
    await withOperation("vacuum", t(locale, "settings.vacuum_fail"), async () => {
      await invoke("run_user_vacuum");
      return "done";
    });
  }, [locale, withOperation]);

  const completeWizard = useCallback(async (): Promise<void> => {
    const token = ++seq.current;
    if (!isTauriRuntime()) {
      return;
    }
    try {
      await invoke("complete_wizard");
      if (token !== seq.current) {
        return;
      }
      setErrorZh(null);
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, t(locale, "settings.save_fail")));
    }
  }, [locale]);

  const cancelOperation = useCallback(async (): Promise<void> => {
    const id = progress?.operationId;
    if (!id || !progress.canCancel || !isTauriRuntime()) {
      return;
    }
    const token = ++seq.current;
    try {
      const next = await invoke<unknown>("cancel_operation", { operationId: id });
      if (token !== seq.current) {
        return;
      }
      if (next) {
        setProgress(decodeProgress(next));
      }
    } catch (caught: unknown) {
      if (token !== seq.current) {
        return;
      }
      setErrorZh(invokeErrorZh(caught, t(locale, "settings.progress.cancel")));
    }
  }, [locale, progress]);

  return {
    address,
    targets,
    secret,
    settings,
    collectorRunning,
    probe,
    about,
    aboutLoading,
    aboutError,
    deletePreview,
    deleteReport,
    retention,
    dataDir,
    progress,
    errorZh,
    setAddress,
    setTargets,
    setSecret,
    loadSecret,
    saveConnection,
    testConnection,
    disconnect,
    reconnect,
    pauseCollector,
    resumeCollector,
    refreshCollector,
    loadAbout,
    openReleases,
    previewDelete,
    confirmDelete,
    previewRetention,
    runRetention,
    loadDataDir,
    openLogDir,
    createBackup,
    restoreBackup,
    validateBackup,
    vacuum,
    completeWizard,
    cancelOperation
  };
}
