import { invoke } from "@tauri-apps/api/core";
import { decodeAutostartState } from "../dto";
import { t, type UiLocale } from "../i18n";
import { isTauriRuntime } from "../ipc/live-session";
import { invokeErrorZh } from "../lib/utils";

export interface AutostartRequestState {
  enabled: boolean;
  loaded: boolean;
  loading: boolean;
  saving: boolean;
  errorZh: string | null;
}

export const INITIAL_AUTOSTART_STATE: AutostartRequestState = {
  enabled: false,
  loaded: false,
  loading: false,
  saving: false,
  errorZh: null
};

export interface AutostartBackend {
  available: () => boolean;
  read: () => Promise<unknown>;
  write: (enabled: boolean) => Promise<unknown>;
}

export const TAURI_AUTOSTART_BACKEND: AutostartBackend = {
  available: isTauriRuntime,
  read: () => invoke<unknown>("get_autostart_state"),
  write: (enabled) => invoke<unknown>("set_autostart_enabled", { enabled })
};

export class AutostartRequestController {
  private requestSeq = 0;
  private state: AutostartRequestState = { ...INITIAL_AUTOSTART_STATE };

  constructor(
    private locale: UiLocale,
    private readonly backend: AutostartBackend,
    private readonly publish: (state: AutostartRequestState) => void
  ) {}

  setLocale(locale: UiLocale): void {
    this.locale = locale;
  }

  snapshot(): AutostartRequestState {
    return { ...this.state };
  }

  async load(): Promise<void> {
    if (this.state.saving) {
      return;
    }
    const token = ++this.requestSeq;
    this.commit({ ...this.state, loading: true, errorZh: null });
    const fallback = t(this.locale, "settings.autostart.load_fail");
    if (!this.backend.available()) {
      if (token === this.requestSeq) {
        this.commit({
          ...this.state,
          loaded: false,
          loading: false,
          errorZh: t(this.locale, "settings.autostart.preview_unavailable")
        });
      }
      return;
    }
    try {
      const result = decodeAutostartState(await this.backend.read());
      if (token !== this.requestSeq) {
        return;
      }
      this.commit({
        enabled: result.enabled,
        loaded: true,
        loading: false,
        saving: false,
        errorZh: null
      });
    } catch (caught: unknown) {
      if (token !== this.requestSeq) {
        return;
      }
      this.commit({
        ...this.state,
        loading: false,
        errorZh: invokeErrorZh(caught, fallback)
      });
    }
  }

  async setEnabled(enabled: boolean): Promise<void> {
    if (!this.state.loaded || this.state.loading || this.state.saving) {
      return;
    }
    const token = ++this.requestSeq;
    this.commit({ ...this.state, saving: true, errorZh: null });
    const fallback = t(this.locale, "settings.autostart.save_fail");
    if (!this.backend.available()) {
      if (token === this.requestSeq) {
        this.commit({
          ...this.state,
          saving: false,
          errorZh: t(this.locale, "settings.autostart.preview_unavailable")
        });
      }
      return;
    }
    try {
      const result = decodeAutostartState(await this.backend.write(enabled));
      if (token !== this.requestSeq) {
        return;
      }
      this.commit({
        enabled: result.enabled,
        loaded: true,
        loading: false,
        saving: false,
        errorZh: null
      });
    } catch (caught: unknown) {
      if (token !== this.requestSeq) {
        return;
      }
      this.commit({
        ...this.state,
        saving: false,
        errorZh: invokeErrorZh(caught, fallback)
      });
    }
  }

  private commit(state: AutostartRequestState): void {
    this.state = state;
    this.publish({ ...state });
  }
}
