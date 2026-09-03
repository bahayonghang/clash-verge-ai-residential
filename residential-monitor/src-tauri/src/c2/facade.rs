//! C2 AppFacade：只经 C1 接口访问采集、存储、投影与恢复。

use crate::accounting::AccountingEngine;
use crate::app_log::{self, Level};
use crate::c0_contract::{all_table_allowlist, forbidden_table_fragments};
use crate::c2::close::{CloseRegistry, CloseState, ControlResult};
use crate::c2::contract::SCHEMA_VERSION;
use crate::c2::desktop::{DesktopRuntime, InstanceClaim, LaunchMode, ShutdownPhase};
use crate::c2::hub::{
    health_from, session_status_name, LiveOverview, MonitorHub, MonitorStreamMessage,
    ObservationPhase,
};
use crate::c2::query::{ConnectionPage, ConnectionQuery};
use crate::c2::settings::{
    validate_targets, ControllerSettings, SettingsError, SettingsWorkflow, WizardState,
};
use crate::c2::shell::{
    default_routes_for, recovery_status, validate_backup, BootBranch, OperationProgress,
    OperationRegistry, RecoveryStatus, RouteDescriptor,
};
use crate::c3::archive::{ReportArchivePage, ReportArchiveService};
use crate::c3::backup::BackupRestoreService;
use crate::c3::export::{ExportPreview, ExportService, ExportSpec, HtmlDocument};
use crate::c3::query::{ReportError, ReportQuery, ReportResult, RAW_RETAIN_DAYS_DEFAULT};
use crate::c3::retention::{RetentionMode, RetentionPreview, RetentionService};
use crate::c3::share::{query_residential_share, ResidentialShare};
use crate::c3::snapshot::ReportSnapshotStore;
use crate::c3::space::SpaceBudget;
use crate::c4::engine::{AlertEngine, HealthSnapshot};
use crate::c4::notify::{NotificationSink, NotifyPayload, WindowsNotificationSink};
use crate::c4::types::{
    validate_rule, AlertCenterPage, AlertRule, AlertSummary, ALERT_DTO_VERSION,
};
use crate::controller::{reject_non_loopback_ip, ControllerInput, SessionStatus};
use crate::credential::FakeCredentialStore;
use crate::dimension_rank_table_layout::{
    encode_setting as encode_dimension_rank_layout, parse_setting as parse_dimension_rank_layout,
    sanitize_layout as sanitize_dimension_rank_layout, DimensionRankTableLayout,
    LAYOUT_SETTING_KEY as DIMENSION_RANK_LAYOUT_KEY,
};
use crate::i18n::{t, UiLocale, SETTING_KEY};
use crate::live_table_layout::{
    encode_setting, parse_setting, sanitize_layout, LiveTableLayout, LAYOUT_SETTING_KEY,
};
use crate::session::ControllerSession;
use crate::storage::{
    AlertCommitSlice, CommitBundle, CommitOutcome, RecoveryFacade, StorageCoordinator, StorageError,
};
use crate::theme::{
    clamp_sidebar_width, parse_sidebar_width, UiDensity, UiFont, UiFontSize, UiTheme,
    DENSITY_SETTING_KEY, FONT_SETTING_KEY, FONT_SIZE_SETTING_KEY, SIDEBAR_WIDTH_DEFAULT,
    SIDEBAR_WIDTH_SETTING_KEY, THEME_SETTING_KEY,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppErrorDto {
    pub code: String,
    pub message_zh: String,
    pub retryable: bool,
    pub action: String,
    pub details_redacted: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub status: String,
    pub message_zh: String,
    pub action: String,
}

impl AppErrorDto {
    pub fn from_settings(error: SettingsError) -> Self {
        Self::from_settings_locale(error, UiLocale::Zh)
    }

    pub fn from_settings_locale(error: SettingsError, locale: UiLocale) -> Self {
        Self {
            code: error.code().into(),
            message_zh: error.message(locale).into(),
            retryable: matches!(
                error,
                SettingsError::ProbeFailed | SettingsError::Unavailable
            ),
            action: t(locale, "settings.check_controller").into(),
            details_redacted: error.code().into(),
        }
    }

    pub fn from_status(status: SessionStatus) -> Self {
        Self::from_status_locale(status, UiLocale::Zh)
    }

    pub fn from_status_locale(status: SessionStatus, locale: UiLocale) -> Self {
        let code = crate::c2::hub::session_status_name(status);
        Self {
            code: code.clone(),
            message_zh: status_message(status, locale).into(),
            retryable: !matches!(
                status,
                SessionStatus::NonLoopback | SessionStatus::ProtocolIncompatible
            ),
            action: status_action(status, locale).into(),
            details_redacted: code,
        }
    }
}

pub fn status_message(status: SessionStatus, locale: UiLocale) -> &'static str {
    let key = match status {
        SessionStatus::Connecting => "session.connecting",
        SessionStatus::Connected => "session.connected",
        SessionStatus::AuthFailed => "session.auth_failed",
        SessionStatus::PipeAccessDenied => "session.pipe_access_denied",
        SessionStatus::PipeBusyTimeout => "session.pipe_busy_timeout",
        SessionStatus::EndpointMissing => "session.endpoint_missing",
        SessionStatus::ProtocolIncompatible => "session.protocol_incompatible",
        SessionStatus::PidMismatch => "session.pid_mismatch",
        SessionStatus::CoreRestarted => "session.core_restarted",
        SessionStatus::Cancelled => "session.cancelled",
        SessionStatus::NonLoopback => "session.non_loopback",
    };
    t(locale, key)
}

pub fn status_message_zh(status: SessionStatus) -> &'static str {
    status_message(status, UiLocale::Zh)
}

pub fn status_action(status: SessionStatus, locale: UiLocale) -> &'static str {
    let key = match status {
        SessionStatus::AuthFailed => "action.check_secret",
        SessionStatus::PipeAccessDenied | SessionStatus::ProtocolIncompatible => {
            "action.enable_tcp"
        }
        SessionStatus::EndpointMissing => "action.check_address",
        _ => "action.retry_connect",
    };
    t(locale, key)
}

pub fn status_action_zh(status: SessionStatus) -> &'static str {
    status_action(status, UiLocale::Zh)
}

fn storage_error_class(error: &StorageError) -> &'static str {
    match error {
        StorageError::Sqlite(_) => "sqlite",
        StorageError::Closed(_) => "closed",
    }
}

fn slice_fingerprint(slice: &AlertCommitSlice, monotonic_ms: u64) -> Result<String, ()> {
    let encoded = serde_json::to_vec(slice).map_err(|_| ())?;
    let mut digest = Sha256::new();
    digest.update(monotonic_ms.to_le_bytes());
    digest.update(encoded);
    Ok(hex::encode(digest.finalize()))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapDto {
    pub schema_version: u32,
    pub branch: BootBranch,
    pub routes: Vec<RouteDescriptor>,
    pub overview: LiveOverview,
    pub settings: ControllerSettings,
    pub wizard_complete: bool,
    pub recovery: Option<RecoveryStatus>,
    pub launch_mode: LaunchMode,
    pub ui_locale: UiLocale,
    pub ui_theme: UiTheme,
    pub ui_font: UiFont,
    pub ui_font_size: UiFontSize,
    pub ui_density: UiDensity,
    pub ui_sidebar_width: i32,
    pub live_table_layout: LiveTableLayout,
    pub dimension_rank_table_layout: DimensionRankTableLayout,
    pub log_dir: String,
}

pub struct AppFacade {
    pub branch: BootBranch,
    pub desktop: DesktopRuntime,
    pub hub: MonitorHub,
    pub engine: AccountingEngine,
    pub storage: Option<StorageCoordinator>,
    pub recovery: RecoveryFacade,
    pub settings: ControllerSettings,
    pub wizard: WizardState,
    pub wizard_complete: bool,
    pub workflow: SettingsWorkflow,
    pub closes: CloseRegistry,
    pub operations: OperationRegistry,
    pub session: ControllerSession,
    pub data_dir: PathBuf,
    pub session_status: SessionStatus,
    pub snapshots: ReportSnapshotStore,
    pub space: SpaceBudget,
    pub raw_retain_days: i64,
    pub alerts: AlertEngine,
    pub notify: Box<dyn NotificationSink + Send>,
    pub writer_epoch: u64,
    pub bundle_seq: u64,
    controller_epoch_ready: bool,
    pub last_frame_utc: Option<i64>,
    pub metadata_coverage: crate::controller::MetadataCoverage,
    pub last_period_eval_utc: i64,
    pub ui_locale: UiLocale,
    pub ui_theme: UiTheme,
    pub ui_font: UiFont,
    pub ui_font_size: UiFontSize,
    pub ui_density: UiDensity,
    pub ui_sidebar_width: i32,
    pub live_table_layout: LiveTableLayout,
    pub dimension_rank_table_layout: DimensionRankTableLayout,
    last_logged_session: Option<SessionStatus>,
}

/// 存储派生状态：打开主库后读取的全部启动值。`boot()` 与
/// `reboot_storage()` 共用，保证还原 / 删除 / VACUUM 后的存储侧状态与
/// 冷启动一致（`writer_epoch`、targets、告警规则与实例、设置）。
struct StorageState {
    storage: StorageCoordinator,
    writer_epoch: u64,
    settings: ControllerSettings,
    wizard_complete: bool,
    ui_locale: UiLocale,
    ui_theme: UiTheme,
    ui_font: UiFont,
    ui_font_size: UiFontSize,
    ui_density: UiDensity,
    ui_sidebar_width: i32,
    live_table_layout: LiveTableLayout,
    dimension_rank_table_layout: DimensionRankTableLayout,
    engine: AccountingEngine,
    alerts: AlertEngine,
}

impl StorageState {
    fn open(db_path: &Path) -> Result<Self, StorageError> {
        let mut storage = StorageCoordinator::open(db_path)?;
        let writer_epoch = match storage.reserve_writer_epoch() {
            Ok(value) => value,
            Err(error) => {
                app_log::emit(
                    Level::Error,
                    "writer_epoch_reserve",
                    serde_json::json!({ "class": storage_error_class(&error) }),
                );
                return Err(error);
            }
        };
        let settings: ControllerSettings = storage
            .get_setting("controller")
            .ok()
            .flatten()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default();
        let wizard_complete = storage
            .get_setting("wizard_complete")
            .ok()
            .flatten()
            .as_deref()
            == Some("1");
        let ui_locale = UiLocale::parse(storage.get_setting(SETTING_KEY).ok().flatten().as_deref());
        let ui_theme = UiTheme::parse(
            storage
                .get_setting(THEME_SETTING_KEY)
                .ok()
                .flatten()
                .as_deref(),
        );
        let ui_font = UiFont::parse(
            storage
                .get_setting(FONT_SETTING_KEY)
                .ok()
                .flatten()
                .as_deref(),
        );
        let ui_font_size = UiFontSize::parse(
            storage
                .get_setting(FONT_SIZE_SETTING_KEY)
                .ok()
                .flatten()
                .as_deref(),
        );
        let ui_density = UiDensity::parse(
            storage
                .get_setting(DENSITY_SETTING_KEY)
                .ok()
                .flatten()
                .as_deref(),
        );
        let ui_sidebar_width = parse_sidebar_width(
            storage
                .get_setting(SIDEBAR_WIDTH_SETTING_KEY)
                .ok()
                .flatten()
                .as_deref(),
        );
        let live_table_layout = parse_setting(
            storage
                .get_setting(LAYOUT_SETTING_KEY)
                .ok()
                .flatten()
                .as_deref(),
        );
        let dimension_rank_table_layout = parse_dimension_rank_layout(
            storage
                .get_setting(DIMENSION_RANK_LAYOUT_KEY)
                .ok()
                .flatten()
                .as_deref(),
        );
        let mut engine = AccountingEngine::new();
        if let Ok((_, targets)) = storage.load_targets() {
            if !targets.is_empty() {
                engine.set_targets(targets);
            }
        }
        let mut alerts = AlertEngine::new();
        if let Ok(rules) = crate::c4::store::load_rules(storage.connection()) {
            let _ = alerts.load_rules(rules);
        }
        if let Ok(instances) = crate::c4::store::load_instances(storage.connection()) {
            for instance in instances {
                alerts.restore_instance(instance);
            }
        }
        Ok(Self {
            storage,
            writer_epoch,
            settings,
            wizard_complete,
            ui_locale,
            ui_theme,
            ui_font,
            ui_font_size,
            ui_density,
            ui_sidebar_width,
            live_table_layout,
            dimension_rank_table_layout,
            engine,
            alerts,
        })
    }
}

impl AppFacade {
    pub fn boot(data_dir: impl Into<PathBuf>, args: &[String], claim: InstanceClaim) -> Self {
        let data_dir = data_dir.into();
        let db_path = data_dir.join("monitor.sqlite3");
        let desktop = DesktopRuntime::start(args, claim);
        match StorageState::open(&db_path) {
            Ok(state) => {
                app_log::emit(
                    Level::Info,
                    "storage_open",
                    serde_json::json!({ "class": "ok" }),
                );
                let hub = MonitorHub::new();
                if state.settings.address.trim().is_empty() {
                    hub.set_observation_phase(ObservationPhase::Unconfigured);
                }
                Self {
                    branch: BootBranch::NormalReady,
                    desktop,
                    hub,
                    engine: state.engine,
                    recovery: RecoveryFacade::open(&db_path),
                    storage: Some(state.storage),
                    settings: state.settings,
                    wizard: WizardState::default(),
                    wizard_complete: state.wizard_complete,
                    workflow: SettingsWorkflow::new(FakeCredentialStore::new(), true),
                    closes: CloseRegistry::new(),
                    operations: OperationRegistry::new(),
                    session: ControllerSession::new(String::new()),
                    data_dir: data_dir.clone(),
                    session_status: SessionStatus::Connecting,
                    snapshots: ReportSnapshotStore::open(&data_dir),
                    space: SpaceBudget::unlimited(),
                    raw_retain_days: RAW_RETAIN_DAYS_DEFAULT,
                    alerts: state.alerts,
                    notify: Box::new(WindowsNotificationSink::new()),
                    writer_epoch: state.writer_epoch,
                    bundle_seq: 1,
                    controller_epoch_ready: false,
                    last_frame_utc: None,
                    metadata_coverage: crate::controller::MetadataCoverage::default(),
                    last_period_eval_utc: 0,
                    ui_locale: state.ui_locale,
                    ui_theme: state.ui_theme,
                    ui_font: state.ui_font,
                    ui_font_size: state.ui_font_size,
                    ui_density: state.ui_density,
                    ui_sidebar_width: state.ui_sidebar_width,
                    live_table_layout: state.live_table_layout,
                    dimension_rank_table_layout: state.dimension_rank_table_layout,
                    last_logged_session: None,
                }
            }
            Err(error) => {
                app_log::emit(
                    Level::Error,
                    "storage_open",
                    serde_json::json!({ "class": storage_error_class(&error) }),
                );
                Self::recovery_only(desktop, data_dir, db_path)
            }
        }
    }

    fn recovery_only(desktop: DesktopRuntime, data_dir: PathBuf, db_path: PathBuf) -> Self {
        let hub = MonitorHub::new();
        hub.set_observation_phase(ObservationPhase::Unconfigured);
        Self {
            branch: BootBranch::RecoveryOnly,
            desktop,
            hub,
            engine: AccountingEngine::new(),
            storage: None,
            recovery: RecoveryFacade::open(&db_path),
            settings: ControllerSettings::default(),
            wizard: WizardState::default(),
            wizard_complete: false,
            workflow: SettingsWorkflow::new(FakeCredentialStore::new(), false),
            closes: CloseRegistry::new(),
            operations: OperationRegistry::new(),
            session: ControllerSession::new(String::new()),
            data_dir: data_dir.clone(),
            session_status: SessionStatus::EndpointMissing,
            snapshots: ReportSnapshotStore::open(&data_dir),
            space: SpaceBudget::unlimited(),
            raw_retain_days: RAW_RETAIN_DAYS_DEFAULT,
            alerts: AlertEngine::new(),
            notify: Box::new(WindowsNotificationSink::new()),
            writer_epoch: 0,
            bundle_seq: 1,
            controller_epoch_ready: false,
            last_frame_utc: None,
            metadata_coverage: crate::controller::MetadataCoverage::default(),
            last_period_eval_utc: 0,
            ui_locale: UiLocale::Zh,
            ui_theme: UiTheme::Mocha,
            ui_font: UiFont::system(),
            ui_font_size: UiFontSize::Md,
            ui_density: UiDensity::Comfortable,
            ui_sidebar_width: SIDEBAR_WIDTH_DEFAULT,
            live_table_layout: LiveTableLayout::default(),
            dimension_rank_table_layout: DimensionRankTableLayout::default(),
            last_logged_session: None,
        }
    }

    /// 还原 / 删除本地数据 / VACUUM 关闭旧连接后重跑存储侧启动。
    /// 只在 `StorageState::open` 全部成功后才写 `self`；失败时 `self.storage`
    /// 保持入口处已置的 `None`，由调用方决定进入 `RecoveryOnly`。
    /// 不触碰 `workflow`（会话 secret 与 Windows 凭据适配）与 `session_status`
    /// （采集会话不重连，下一轮 tick 自然更新）。
    fn reboot_storage(&mut self) -> Result<(), StorageError> {
        let db_path = self.data_dir.join("monitor.sqlite3");
        let state = StorageState::open(&db_path)?;
        self.storage = Some(state.storage);
        self.writer_epoch = state.writer_epoch;
        self.bundle_seq = 1;
        self.settings = state.settings;
        self.wizard_complete = state.wizard_complete;
        self.ui_locale = state.ui_locale;
        self.ui_theme = state.ui_theme;
        self.ui_font = state.ui_font;
        self.ui_font_size = state.ui_font_size;
        self.ui_density = state.ui_density;
        self.ui_sidebar_width = state.ui_sidebar_width;
        self.live_table_layout = state.live_table_layout;
        self.dimension_rank_table_layout = state.dimension_rank_table_layout;
        self.engine = state.engine;
        self.alerts = state.alerts;
        if self.settings.address.trim().is_empty() {
            self.hub
                .set_observation_phase(ObservationPhase::Unconfigured);
        }
        self.branch = BootBranch::NormalReady;
        Ok(())
    }

    pub fn bootstrap(&self) -> Result<BootstrapDto, AppErrorDto> {
        let recovery = if self.branch == BootBranch::RecoveryOnly {
            Some(recovery_status(&self.recovery).map_err(|_| {
                self.err(
                    "recovery_status",
                    "error.recovery_status",
                    "action.open_data_dir",
                    true,
                )
            })?)
        } else {
            None
        };
        Ok(BootstrapDto {
            schema_version: SCHEMA_VERSION,
            branch: self.branch,
            routes: default_routes_for(self.ui_locale),
            overview: self.hub.overview(),
            settings: self.settings.clone(),
            wizard_complete: self.wizard_complete,
            recovery,
            launch_mode: self.desktop.launch_mode,
            ui_locale: self.ui_locale,
            ui_theme: self.ui_theme,
            ui_font: self.ui_font.clone(),
            ui_font_size: self.ui_font_size,
            ui_density: self.ui_density,
            ui_sidebar_width: self.ui_sidebar_width,
            live_table_layout: self.live_table_layout.clone(),
            dimension_rank_table_layout: self.dimension_rank_table_layout.clone(),
            log_dir: app_log::dir().to_string_lossy().into_owned(),
        })
    }

    pub fn err(
        &self,
        code: &str,
        message_key: &str,
        action_key: &str,
        retryable: bool,
    ) -> AppErrorDto {
        localized_error(self.ui_locale, code, message_key, action_key, retryable)
    }

    fn log_session_change(&mut self, to: SessionStatus) {
        if self.last_logged_session == Some(to) {
            return;
        }
        let from = self
            .last_logged_session
            .map(session_status_name)
            .unwrap_or_else(|| "none".into());
        let to_name = session_status_name(to);
        let level = if matches!(to, SessionStatus::Connected) {
            Level::Info
        } else {
            Level::Warn
        };
        app_log::emit(
            level,
            "session",
            serde_json::json!({ "from": from, "to": to_name }),
        );
        self.last_logged_session = Some(to);
    }

    pub fn pause_collector(&mut self) -> Option<MonitorStreamMessage> {
        let input = self.desktop.set_collector_running(false);
        app_log::emit(Level::Info, "collector_pause", serde_json::json!({}));
        self.apply_lifecycle(input)
    }

    pub fn open_log_dir(&self) -> Result<String, AppErrorDto> {
        match app_log::open_in_explorer() {
            Ok(path) => Ok(path.to_string_lossy().into_owned()),
            Err(_) => Err(self.err(
                "open_log_dir",
                "error.open_log_dir",
                "action.open_log_dir",
                true,
            )),
        }
    }

    pub fn save_ui_locale(&mut self, raw: &str) -> Result<UiLocale, AppErrorDto> {
        let locale = UiLocale::parse(Some(raw));
        if let Some(storage) = &self.storage {
            storage
                .put_setting(SETTING_KEY, locale.as_str())
                .map_err(|_| self.err("storage", "error.locale", "action.check_disk", true))?;
        }
        self.ui_locale = locale;
        Ok(locale)
    }

    pub fn save_ui_theme(&mut self, raw: &str) -> Result<UiTheme, AppErrorDto> {
        let theme = UiTheme::parse(Some(raw));
        if let Some(storage) = &self.storage {
            storage
                .put_setting(THEME_SETTING_KEY, theme.as_str())
                .map_err(|_| self.err("storage", "error.theme", "action.check_disk", true))?;
        }
        self.ui_theme = theme;
        Ok(theme)
    }

    pub fn save_ui_font(&mut self, raw: &str) -> Result<UiFont, AppErrorDto> {
        let font = UiFont::parse(Some(raw));
        if let Some(storage) = &self.storage {
            storage
                .put_setting(FONT_SETTING_KEY, font.as_str())
                .map_err(|_| self.err("storage", "error.theme", "action.check_disk", true))?;
        }
        self.ui_font = font.clone();
        Ok(font)
    }

    pub fn save_ui_font_size(&mut self, raw: &str) -> Result<UiFontSize, AppErrorDto> {
        let size = UiFontSize::parse(Some(raw));
        if let Some(storage) = &self.storage {
            storage
                .put_setting(FONT_SIZE_SETTING_KEY, size.as_str())
                .map_err(|_| self.err("storage", "error.theme", "action.check_disk", true))?;
        }
        self.ui_font_size = size;
        Ok(size)
    }

    pub fn save_ui_density(&mut self, raw: &str) -> Result<UiDensity, AppErrorDto> {
        let density = UiDensity::parse(Some(raw));
        if let Some(storage) = &self.storage {
            storage
                .put_setting(DENSITY_SETTING_KEY, density.as_str())
                .map_err(|_| self.err("storage", "error.theme", "action.check_disk", true))?;
        }
        self.ui_density = density;
        Ok(density)
    }

    pub fn save_ui_sidebar_width(&mut self, width: i32) -> Result<i32, AppErrorDto> {
        let width = clamp_sidebar_width(width);
        if let Some(storage) = &self.storage {
            storage
                .put_setting(SIDEBAR_WIDTH_SETTING_KEY, &width.to_string())
                .map_err(|_| self.err("storage", "error.theme", "action.check_disk", true))?;
        }
        self.ui_sidebar_width = width;
        Ok(width)
    }

    pub fn save_live_table_layout(
        &mut self,
        layout: LiveTableLayout,
    ) -> Result<LiveTableLayout, AppErrorDto> {
        let layout = sanitize_layout(layout);
        if let Some(storage) = &self.storage {
            let encoded = encode_setting(&layout)
                .ok_or_else(|| self.err("encode", "error.encode", "action.check_disk", true))?;
            storage
                .put_setting(LAYOUT_SETTING_KEY, &encoded)
                .map_err(|_| self.err("storage", "error.layout", "action.check_disk", true))?;
        }
        self.live_table_layout = layout.clone();
        Ok(layout)
    }

    pub fn save_dimension_rank_table_layout(
        &mut self,
        layout: DimensionRankTableLayout,
    ) -> Result<DimensionRankTableLayout, AppErrorDto> {
        let layout = sanitize_dimension_rank_layout(layout);
        if let Some(storage) = &self.storage {
            let encoded = encode_dimension_rank_layout(&layout)
                .ok_or_else(|| self.err("encode", "error.encode", "action.check_disk", true))?;
            storage
                .put_setting(DIMENSION_RANK_LAYOUT_KEY, &encoded)
                .map_err(|_| self.err("storage", "error.layout", "action.check_disk", true))?;
        }
        self.dimension_rank_table_layout = layout.clone();
        Ok(layout)
    }

    pub fn subscribe(&self) -> MonitorStreamMessage {
        self.hub.subscribe()
    }

    pub fn resync(&self, subscription_id: u64) -> MonitorStreamMessage {
        self.hub.resync(subscription_id)
    }

    /// Handle an owner-window transition initiated by the tray or a second
    /// instance. Recovery is deliberately tied to the hidden -> visible
    /// transition so an explicit disconnect remains cancelled while the window
    /// stays open. A persisted, valid loopback endpoint is required; the UI's
    /// display fallback is never used as an implicit controller address.
    pub fn open_main_window(&mut self) -> Option<MonitorStreamMessage> {
        if !self.desktop.open_window() || self.branch != BootBranch::NormalReady {
            return None;
        }
        if !self.has_valid_controller_address() {
            return None;
        }
        if matches!(self.session_status, SessionStatus::Cancelled) {
            self.reconnect_now()
        } else if !self.desktop.collector_running {
            self.resume_collector()
        } else {
            None
        }
    }

    fn has_valid_controller_address(&self) -> bool {
        let Ok(address) = parse_socket(&self.settings.address) else {
            return false;
        };
        reject_non_loopback_ip(address.ip()).is_ok()
    }

    pub fn apply_lifecycle(&mut self, input: ControllerInput) -> Option<MonitorStreamMessage> {
        let utc = chrono::Utc::now().timestamp();
        if matches!(
            &input,
            ControllerInput::Restarted { .. } | ControllerInput::Disconnected { .. }
        ) {
            self.controller_epoch_ready = false;
        }
        let keep_rows = retain_live_rows(&input);
        let batch = self.engine.apply(input, 0, utc);
        let live = if keep_rows {
            self.hub.rows()
        } else {
            Vec::new()
        };
        let commit_error = self.commit_eval(&batch, &live, utc, 0);
        let mut health = health_from(
            self.session_status,
            self.storage
                .as_ref()
                .and_then(|item| item.health().ok())
                .as_ref(),
        );
        if let Some(reason) = commit_error {
            health.storage_ok = false;
            health.storage_reason = Some(reason.into());
        }
        self.hub
            .publish_lifecycle(&batch, keep_rows, health, utc)
            .ok()
            .flatten()
    }

    pub fn ingest_snapshot(
        &mut self,
        input: ControllerInput,
        utc: i64,
        mono: u64,
    ) -> Option<MonitorStreamMessage> {
        if let ControllerInput::Snapshot {
            upload_total,
            download_total,
            connections,
            ..
        } = input
        {
            self.metadata_coverage =
                crate::controller::MetadataCoverage::from_connections(&connections);
            let needs_generation = !self.controller_epoch_ready
                || self.session_status != SessionStatus::Connected
                || self.engine.snapshot_requires_new_generation(
                    &connections,
                    upload_total,
                    download_total,
                );
            if needs_generation {
                let epoch = match self
                    .storage
                    .as_mut()
                    .map(|storage| storage.reserve_controller_epoch("collector-http"))
                {
                    Some(Ok(epoch)) => epoch,
                    Some(Err(error)) => {
                        let class = storage_error_class(&error);
                        app_log::emit(
                            Level::Error,
                            "controller_epoch_reserve",
                            serde_json::json!({ "class": class }),
                        );
                        let mut health = health_from(
                            self.session_status,
                            self.storage
                                .as_ref()
                                .and_then(|item| item.health().ok())
                                .as_ref(),
                        );
                        health.storage_ok = false;
                        health.storage_reason = Some(format!("controller_epoch_{class}"));
                        return Some(self.hub.publish_health(health, utc));
                    }
                    None => return None,
                };
                self.engine.reset_epoch(epoch);
                self.controller_epoch_ready = true;
            }
            let (batch, live) = self.engine.apply_snapshot_and_project(
                connections,
                upload_total,
                download_total,
                mono,
                utc,
            );
            let removed: Vec<String> = {
                let current: std::collections::HashSet<_> =
                    live.iter().map(|row| row.identity.clone()).collect();
                self.hub
                    .rows()
                    .into_iter()
                    .map(|row| row.identity)
                    .filter(|id| !current.contains(id))
                    .collect()
            };
            for identity in removed {
                let _ = self.closes.on_remove(&identity);
            }
            self.session_status = SessionStatus::Connected;
            self.log_session_change(SessionStatus::Connected);
            let commit_error = self.commit_eval(&batch, &live, utc, utc as u64);
            let mut health = health_from(
                SessionStatus::Connected,
                self.storage
                    .as_ref()
                    .and_then(|item| item.health().ok())
                    .as_ref(),
            );
            if let Some(reason) = commit_error {
                health.storage_ok = false;
                health.storage_reason = Some(reason.into());
            }
            self.hub.publish(&batch, live, health, utc).ok().flatten()
        } else {
            self.apply_lifecycle(input)
        }
    }

    fn commit_eval(
        &mut self,
        batch: &crate::accounting::AccountingBatch,
        live: &[crate::c2::hub::LiveConnectionView],
        utc: i64,
        mono: u64,
    ) -> Option<&'static str> {
        self.storage.as_ref()?;
        let data_version = self
            .storage
            .as_ref()
            .and_then(|item| item.watermark().ok())
            .unwrap_or(0);
        let storage_health = self.storage.as_ref().and_then(|item| item.health().ok());
        let path = self
            .storage
            .as_ref()
            .map(|item| item.path().to_path_buf())
            .unwrap_or_default();
        let rules = self
            .storage
            .as_ref()
            .and_then(|item| crate::c4::store::load_rules(item.connection()).ok())
            .unwrap_or_default();
        let health = HealthSnapshot {
            session: Some(self.session_status),
            storage: storage_health,
            coverage_kinds: batch
                .coverage
                .iter()
                .map(|item| item.kind.to_string())
                .collect(),
            migration_failed: false,
            backup_failed: false,
        };
        let mut usages = Vec::new();
        if utc.saturating_sub(self.last_period_eval_utc) >= 60 {
            let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            usages = crate::c4::period::evaluate_period_rules(
                &path,
                &mut self.snapshots,
                &rules,
                utc,
                self.raw_retain_days,
                &cancel,
            );
            self.last_period_eval_utc = utc;
        }
        let bundle_id = format!("{}:{}", self.writer_epoch, self.bundle_seq);
        let writes = self.alerts.evaluate_frame(crate::c4::engine::FrameInput {
            batch,
            live,
            health: &health,
            usages: &usages,
            now_utc: utc,
            now_mono: mono,
            data_version,
            bundle_id: &bundle_id,
        });
        let bundle = CommitBundle {
            writer_epoch: self.writer_epoch,
            bundle_seq: self.bundle_seq,
            payload: String::new(),
        };
        let slice = AlertCommitSlice {
            facts: batch.facts.clone(),
            coverage: batch.coverage.clone(),
            live_rows: live.to_vec(),
            utc,
            writes,
        };
        let payload = match slice_fingerprint(&slice, mono) {
            Ok(payload) => payload,
            Err(()) => {
                app_log::emit(
                    Level::Error,
                    "commit_fingerprint",
                    serde_json::json!({ "class": "serialize" }),
                );
                return Some("commit_serialize");
            }
        };
        let bundle = CommitBundle { payload, ..bundle };
        let outcome = self
            .storage
            .as_mut()
            .map(|storage| storage.commit_alert_bundle(&bundle, &slice));
        match outcome {
            Some(Ok(
                crate::storage::CommitOutcome::Applied(_)
                | crate::storage::CommitOutcome::Duplicate(_),
            )) => {
                self.bundle_seq = self.bundle_seq.saturating_add(1);
                self.last_frame_utc = Some(utc);
                let token = format!("lease-{}", self.bundle_seq);
                if let Some(storage) = self.storage.as_mut() {
                    let _ = crate::c4::outbox::scan_once(
                        storage,
                        self.notify.as_mut(),
                        utc,
                        &token,
                        self.ui_locale,
                    );
                }
                None
            }
            Some(Ok(crate::storage::CommitOutcome::PayloadMismatch)) => {
                app_log::emit(
                    Level::Error,
                    "commit_bundle",
                    serde_json::json!({ "class": "payload_mismatch" }),
                );
                Some("payload_mismatch")
            }
            Some(Ok(crate::storage::CommitOutcome::RetryWindowExpired)) => {
                app_log::emit(
                    Level::Error,
                    "commit_bundle",
                    serde_json::json!({ "class": "retry_window_expired" }),
                );
                Some("retry_window_expired")
            }
            Some(Err(error)) => {
                let class = storage_error_class(&error);
                app_log::emit(
                    Level::Error,
                    "commit_bundle",
                    serde_json::json!({ "class": class }),
                );
                Some(class)
            }
            None => None,
        }
    }

    pub fn query(&self, query: &ConnectionQuery) -> ConnectionPage {
        let (rows, overview) = self.hub.query_snapshot();
        crate::c2::query::query_connections_with_targets_at(
            &rows,
            query,
            self.engine.targets(),
            overview.last_sample_utc,
        )
    }

    pub fn accept_close(&mut self, identity: String, request_id: String) -> CloseState {
        self.closes.accept(identity, request_id)
    }

    pub fn mark_close_accepted_from_control(
        &mut self,
        identity: String,
        request_id: String,
        result: ControlResult,
    ) -> CloseState {
        match result {
            ControlResult::Accepted => self.closes.accept(identity, request_id),
        }
    }

    pub fn persist_settings(&mut self) -> Result<(), AppErrorDto> {
        let storage = self.storage.as_ref().ok_or_else(|| {
            self.err(
                "recovery_only",
                "error.recovery_only",
                "action.fix_db",
                false,
            )
        })?;
        let encoded = serde_json::to_string(&self.settings)
            .map_err(|_| self.err("encode", "error.encode", "action.retry", false))?;
        storage
            .put_setting("controller", &encoded)
            .map_err(|_| self.err("storage", "error.storage", "action.check_disk", true))?;
        storage
            .put_setting(
                "wizard_complete",
                if self.wizard_complete { "1" } else { "0" },
            )
            .map_err(|_| self.err("storage", "error.wizard", "action.check_disk", true))?;
        Ok(())
    }

    pub fn save_controller(
        &mut self,
        address: String,
        secret: Option<String>,
        session_only: bool,
    ) -> Result<ControllerSettings, AppErrorDto> {
        let probe_ok = true;
        let next = self
            .workflow
            .save_secret(
                &self.settings,
                &address,
                secret.as_deref(),
                session_only,
                probe_ok,
            )
            .map_err(|error| AppErrorDto::from_settings_locale(error, self.ui_locale))?;
        self.settings = next.clone();
        self.session.endpoint = address;
        self.persist_settings()?;
        Ok(next)
    }

    /// 只给设置页密码框回填。不得写入日志、Channel、SQLite 或错误详情。
    pub fn reveal_secret(&self) -> Result<Option<String>, AppErrorDto> {
        if !self.settings.has_secret {
            return Ok(None);
        }
        let secret = self
            .workflow
            .resolve(&self.settings.credential_target, &self.settings.secret_mode)
            .map_err(|error| AppErrorDto::from_settings_locale(error, self.ui_locale))?;
        Ok(Some(
            String::from_utf8_lossy(secret.as_header_bytes()).into_owned(),
        ))
    }

    pub fn apply_probe_ok(&mut self, inputs: Vec<ControllerInput>) -> Vec<MonitorStreamMessage> {
        let utc = chrono::Utc::now().timestamp();
        let mut messages = Vec::new();
        for input in inputs {
            let message = if matches!(input, ControllerInput::Snapshot { .. }) {
                self.ingest_snapshot(input, utc, utc as u64)
            } else {
                self.apply_lifecycle(input)
            };
            if let Some(message) = message {
                messages.push(message);
            }
        }
        self.session_status = SessionStatus::Connected;
        self.log_session_change(SessionStatus::Connected);
        messages
    }

    pub fn apply_probe_err(&mut self, status: SessionStatus) -> Option<MonitorStreamMessage> {
        self.session_status = status;
        self.log_session_change(status);
        self.apply_lifecycle(ControllerInput::Disconnected { reason: status })
    }

    pub fn disconnect_now(&mut self) -> Option<MonitorStreamMessage> {
        self.session_status = SessionStatus::Cancelled;
        self.log_session_change(SessionStatus::Cancelled);
        self.apply_lifecycle(ControllerInput::Disconnected {
            reason: SessionStatus::Cancelled,
        })
    }

    /// 离开 Cancelled，让已有采集循环的下一拍可以取帧。
    pub fn reconnect_now(&mut self) -> Option<MonitorStreamMessage> {
        self.session_status = SessionStatus::Connecting;
        self.log_session_change(SessionStatus::Connecting);
        let _ = self.desktop.set_collector_running(true);
        app_log::emit(Level::Info, "reconnect", serde_json::json!({}));
        let input = self.desktop.reconnect();
        self.apply_lifecycle(input)
    }

    pub fn resume_collector(&mut self) -> Option<MonitorStreamMessage> {
        if matches!(self.session_status, SessionStatus::Cancelled) {
            self.session_status = SessionStatus::Connecting;
            self.log_session_change(SessionStatus::Connecting);
        }
        let input = self.desktop.set_collector_running(true);
        app_log::emit(Level::Info, "collector_resume", serde_json::json!({}));
        self.apply_lifecycle(input)
    }

    pub fn probe_result(status: SessionStatus) -> ProbeResult {
        Self::probe_result_locale(status, UiLocale::Zh)
    }

    pub fn probe_result_locale(status: SessionStatus, locale: UiLocale) -> ProbeResult {
        ProbeResult {
            status: crate::c2::hub::session_status_name(status),
            message_zh: status_message(status, locale).into(),
            action: status_action(status, locale).into(),
        }
    }

    pub fn save_targets(&mut self, targets: Vec<String>) -> Result<u32, AppErrorDto> {
        validate_targets(&targets)
            .map_err(|error| AppErrorDto::from_settings_locale(error, self.ui_locale))?;
        let storage = self.storage.as_ref().ok_or_else(|| {
            self.err(
                "recovery_only",
                "error.recovery_only_targets",
                "action.fix_db",
                false,
            )
        })?;
        let version = storage
            .save_targets(&targets)
            .map_err(|_| self.err("storage", "error.targets", "action.check_disk", true))?;
        self.engine.set_targets(targets);
        Ok(version)
    }

    pub fn complete_wizard(&mut self) -> Result<(), AppErrorDto> {
        self.wizard_complete = true;
        self.persist_settings()
    }

    pub fn recovery(&self) -> Result<RecoveryStatus, AppErrorDto> {
        recovery_status(&self.recovery).map_err(|_| {
            self.err(
                "recovery_status",
                "error.recovery_status",
                "action.open_data_dir_short",
                true,
            )
        })
    }

    pub fn validate_candidate(&self, path: &Path) -> Result<bool, AppErrorDto> {
        validate_backup(&self.recovery, path).map_err(|_| {
            self.err(
                "validate_backup",
                "error.invalid_backup",
                "action.other_backup",
                false,
            )
        })
    }

    pub fn start_operation(&mut self, id: String, kind: String) -> OperationProgress {
        self.operations.start_fixture(id, kind)
    }

    pub fn shutdown(&mut self) -> Vec<ShutdownPhase> {
        self.workflow.clear_session();
        let _ = self.apply_lifecycle(ControllerInput::Shutdown);
        let phases = self.desktop.begin_shutdown();
        for phase in &phases {
            let name = match phase {
                ShutdownPhase::Idle => "idle",
                ShutdownPhase::StopIntake => "stop-intake",
                ShutdownPhase::FlushWriter => "flush-writer",
                ShutdownPhase::CloseCoverage => "close-coverage",
                ShutdownPhase::Checkpoint => "checkpoint",
                ShutdownPhase::RemoveTray => "remove-tray",
                ShutdownPhase::Exit => "exit",
            };
            app_log::emit(
                Level::Info,
                "shutdown",
                serde_json::json!({ "phase": name }),
            );
        }
        phases
    }

    pub fn residential_share(
        &self,
        range_start_utc: i64,
        range_end_utc: i64,
        display_timezone: String,
    ) -> Result<ResidentialShare, AppErrorDto> {
        let path = self
            .storage
            .as_ref()
            .ok_or_else(recovery_only)?
            .path()
            .to_path_buf();
        let now = chrono::Utc::now().timestamp();
        query_residential_share(
            &path,
            range_start_utc,
            range_end_utc,
            &display_timezone,
            now,
        )
        .map_err(map_report)
    }

    pub fn run_report(
        &mut self,
        query: ReportQuery,
        persist_manual: bool,
    ) -> Result<ReportResult, AppErrorDto> {
        let path = self
            .storage
            .as_ref()
            .ok_or_else(recovery_only)?
            .path()
            .to_path_buf();
        let now = chrono::Utc::now().timestamp();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let result = crate::c3::ReportService::run(
            &path,
            &mut self.snapshots,
            query,
            now,
            self.raw_retain_days,
            &cancel,
            None,
        )
        .map_err(map_report)?;
        if persist_manual {
            let storage = self.storage.as_ref().ok_or_else(recovery_only)?;
            ReportArchiveService::persist_manual(storage.connection(), result.clone(), now)
                .map_err(map_report)?;
        }
        Ok(result)
    }

    pub fn get_report(&mut self, token: &str) -> Result<ReportResult, AppErrorDto> {
        let now = chrono::Utc::now().timestamp();
        self.snapshots.get(token, now).cloned().map_err(map_report)
    }

    pub fn release_report(&mut self, token: &str) -> bool {
        self.snapshots.release(token)
    }

    pub fn list_report_archives(
        &self,
        kind: Option<String>,
        after: Option<String>,
        limit: Option<u32>,
    ) -> Result<ReportArchivePage, AppErrorDto> {
        let storage = self.storage.as_ref().ok_or_else(recovery_only)?;
        ReportArchiveService::list(
            storage.connection(),
            kind.as_deref(),
            after.as_deref(),
            limit,
        )
        .map_err(map_report)
    }

    pub fn get_report_archive(&mut self, archive_id: &str) -> Result<ReportResult, AppErrorDto> {
        let now = chrono::Utc::now().timestamp();
        let frozen = {
            let storage = self.storage.as_ref().ok_or_else(recovery_only)?;
            ReportArchiveService::load_frozen(storage.connection(), archive_id)
                .map_err(map_report)?
        };
        let query = frozen.query_echo.clone();
        self.snapshots
            .insert(&query, frozen, now, false)
            .map_err(map_report)
    }

    pub fn preview_export(
        &mut self,
        token: &str,
        spec: &ExportSpec,
    ) -> Result<ExportPreview, AppErrorDto> {
        let result = self.get_report(token)?;
        ExportService::preview(&result, spec).map_err(map_report)
    }

    pub fn render_report_html(
        &mut self,
        token: &str,
        spec: &ExportSpec,
    ) -> Result<HtmlDocument, AppErrorDto> {
        let result = self.get_report(token)?;
        let mut spec = spec.clone();
        spec.ui_locale = self.ui_locale;
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        ExportService::render_html(&result, &spec, &cancel)
            .map(|html| HtmlDocument { html })
            .map_err(map_report)
    }

    pub fn get_latest_residential_manual(&mut self) -> Result<Option<ReportResult>, AppErrorDto> {
        let frozen = {
            let storage = self.storage.as_ref().ok_or_else(recovery_only)?;
            ReportArchiveService::load_latest_residential_manual(storage.connection())
                .map_err(map_report)?
        };
        let Some(frozen) = frozen else {
            return Ok(None);
        };
        let now = chrono::Utc::now().timestamp();
        let query = frozen.query_echo.clone();
        self.snapshots
            .insert(&query, frozen, now, false)
            .map(Some)
            .map_err(map_report)
    }

    pub fn export_report(
        &mut self,
        token: &str,
        spec: &ExportSpec,
        dest: &Path,
    ) -> Result<String, AppErrorDto> {
        let result = self.get_report(token)?;
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut spec = spec.clone();
        spec.ui_locale = self.ui_locale;
        ExportService::export_to_path(&result, &spec, dest, &self.space, &cancel)
            .map(|path| path.to_string_lossy().into_owned())
            .map_err(map_report)
    }

    pub fn retention_preview(&self) -> Result<RetentionPreview, AppErrorDto> {
        let storage = self.storage.as_ref().ok_or_else(recovery_only)?;
        RetentionService::preview(storage, self.raw_retain_days).map_err(map_report)
    }

    pub fn run_retention(&mut self, delete: bool) -> Result<RetentionPreview, AppErrorDto> {
        let storage = self.storage.as_mut().ok_or_else(recovery_only)?;
        let now = chrono::Utc::now().timestamp();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mode = if delete {
            RetentionMode::DeleteEnabled
        } else {
            RetentionMode::MaterializeOnly
        };
        let preview = RetentionService::run(
            storage,
            now,
            self.raw_retain_days,
            mode,
            &self.space,
            &cancel,
        );
        app_log::emit(
            if preview.is_ok() {
                Level::Info
            } else {
                Level::Error
            },
            "retention",
            serde_json::json!({ "ok": preview.is_ok() }),
        );
        let preview = preview.map_err(map_report)?;
        if let Some(storage) = self.storage.as_ref() {
            let _ = crate::c4::store::retain_alerts(storage.connection(), now);
        }
        Ok(preview)
    }

    pub fn list_alert_rules(&self) -> Result<Vec<AlertRule>, AppErrorDto> {
        let storage = self.storage.as_ref().ok_or_else(recovery_only)?;
        crate::c4::store::load_rules(storage.connection())
            .map_err(|_| self.err("storage", "error.alert_rules", "action.check_disk", true))
    }

    pub fn upsert_alert_rule(&mut self, rule: AlertRule) -> Result<AlertRule, AppErrorDto> {
        validate_rule(&rule).map_err(|error| AppErrorDto {
            code: error.code().into(),
            message_zh: error.message_zh().into(),
            retryable: false,
            action: error.action_zh().into(),
            details_redacted: error.code().into(),
        })?;
        let now = chrono::Utc::now().timestamp();
        let writes = self
            .alerts
            .upsert_rule(rule.clone(), now)
            .map_err(|error| AppErrorDto {
                code: error.code().into(),
                message_zh: error.message_zh().into(),
                retryable: false,
                action: error.action_zh().into(),
                details_redacted: error.code().into(),
            })?;
        let locale = self.ui_locale;
        let storage = self.storage.as_mut().ok_or_else(recovery_only)?;
        crate::c4::store::upsert_rule(storage.connection(), &rule).map_err(|_| {
            localized_error(
                locale,
                "storage",
                "error.alert_write",
                "action.check_disk",
                true,
            )
        })?;
        if !writes.instances.is_empty() || !writes.events.is_empty() || !writes.outbox.is_empty() {
            let slice = AlertCommitSlice {
                utc: now,
                writes,
                ..AlertCommitSlice::default()
            };
            let payload = slice_fingerprint(&slice, 0).map_err(|()| {
                localized_error(
                    locale,
                    "commit_serialize",
                    "error.alert_write",
                    "action.check_disk",
                    true,
                )
            })?;
            let bundle = CommitBundle {
                writer_epoch: self.writer_epoch,
                bundle_seq: self.bundle_seq,
                payload,
            };
            match storage.commit_alert_bundle(&bundle, &slice) {
                Ok(CommitOutcome::Applied(_) | CommitOutcome::Duplicate(_)) => {
                    self.bundle_seq = self.bundle_seq.saturating_add(1);
                }
                Ok(CommitOutcome::PayloadMismatch) => {
                    app_log::emit(
                        Level::Error,
                        "alert_rule_commit",
                        serde_json::json!({ "class": "payload_mismatch" }),
                    );
                    return Err(localized_error(
                        locale,
                        "payload_mismatch",
                        "error.alert_write",
                        "action.check_disk",
                        false,
                    ));
                }
                Ok(CommitOutcome::RetryWindowExpired) => {
                    app_log::emit(
                        Level::Error,
                        "alert_rule_commit",
                        serde_json::json!({ "class": "retry_window_expired" }),
                    );
                    return Err(localized_error(
                        locale,
                        "retry_window_expired",
                        "error.alert_write",
                        "action.check_disk",
                        false,
                    ));
                }
                Err(error) => {
                    let class = storage_error_class(&error);
                    app_log::emit(
                        Level::Error,
                        "alert_rule_commit",
                        serde_json::json!({ "class": class }),
                    );
                    return Err(localized_error(
                        locale,
                        "storage",
                        "error.alert_write",
                        "action.check_disk",
                        true,
                    ));
                }
            }
        }
        app_log::emit(Level::Info, "alert_rule", serde_json::json!({ "ok": true }));
        Ok(rule)
    }

    pub fn list_alert_center(
        &self,
        status: Option<String>,
        after: Option<String>,
    ) -> Result<AlertCenterPage, AppErrorDto> {
        let storage = self.storage.as_ref().ok_or_else(recovery_only)?;
        let (after_utc, after_id) = after
            .as_deref()
            .and_then(|item| item.split_once('|'))
            .and_then(|(utc, id)| utc.parse().ok().map(|value| (value, id)))
            .map(|(utc, id)| (Some(utc), Some(id)))
            .unwrap_or((None, None));
        let items = crate::c4::store::list_instances(
            storage.connection(),
            status.as_deref(),
            after_utc,
            after_id,
            50,
        )
        .map_err(|_| AppErrorDto {
            code: "storage".into(),
            message_zh: t(self.ui_locale, "error.alert_center").into(),
            retryable: true,
            action: "刷新后重试".into(),
            details_redacted: "alert".into(),
        })?;
        let next_cursor = items
            .last()
            .map(|item| format!("{}|{}", item.last_eval_utc, item.instance_id));
        Ok(AlertCenterPage {
            schema_version: ALERT_DTO_VERSION,
            items,
            next_cursor,
        })
    }

    pub fn alert_summary(&self) -> Result<AlertSummary, AppErrorDto> {
        let storage = self.storage.as_ref().ok_or_else(recovery_only)?;
        let active = crate::c4::store::count_status(storage.connection(), "active").unwrap_or(0);
        let not_eval =
            crate::c4::store::count_status(storage.connection(), "not_evaluable").unwrap_or(0);
        Ok(AlertSummary {
            schema_version: ALERT_DTO_VERSION,
            active_count: active,
            not_evaluable_count: not_eval,
            outbox_backlog: crate::c4::outbox::backlog(storage).unwrap_or(0),
            last_event_utc: crate::c4::store::last_event_utc(storage.connection())
                .ok()
                .flatten(),
        })
    }

    pub fn attach_notification_handle(&mut self, app: tauri::AppHandle) {
        self.notify.attach(app);
    }

    pub fn test_notification(
        &mut self,
    ) -> Result<crate::c4::notify::NotifyCapability, AppErrorDto> {
        let cap = self.notify.capability();
        if !cap.available {
            app_log::emit(
                Level::Warn,
                "notify_unavailable",
                serde_json::json!({ "class": "unavailable" }),
            );
        }
        let payload = NotifyPayload {
            title_zh: t(self.ui_locale, "notify.test_title").into(),
            body_zh: t(self.ui_locale, "notify.test_body").into(),
            event_id: "test-notify".into(),
            instance_id: None,
            test_only: true,
        };
        match self.notify.send(&payload) {
            Ok(()) => Ok(cap),
            Err(error) => {
                app_log::emit(
                    Level::Warn,
                    "notify_unavailable",
                    serde_json::json!({ "class": error.class() }),
                );
                Ok(self.notify.capability())
            }
        }
    }

    pub fn get_diagnostics(&self) -> Result<crate::c4::diagnose::DiagnosticsSnapshot, AppErrorDto> {
        let storage = self.storage.as_ref().ok_or_else(recovery_only)?;
        let coverage = self
            .hub
            .overview()
            .coverage_kind
            .unwrap_or_else(|| "unknown".into());
        crate::c4::diagnose::collect(
            storage,
            self.session_status,
            self.last_frame_utc,
            &coverage,
            self.metadata_coverage.clone(),
        )
        .map_err(|_| {
            self.err(
                "diagnostics",
                "error.diagnostics",
                "action.retry_later",
                true,
            )
        })
    }

    pub fn export_diagnostics(&self, dest: &std::path::Path) -> Result<String, AppErrorDto> {
        let snap = self.get_diagnostics()?;
        crate::c4::diagnose::export_atomic(&snap, dest).map_err(|_| {
            self.err(
                "diagnostics_export",
                "error.diagnostics_export",
                "action.change_path",
                true,
            )
        })
    }

    pub fn scan_outbox(&mut self) -> Result<u32, AppErrorDto> {
        let now = chrono::Utc::now().timestamp();
        let token = format!("scan-{}", self.bundle_seq);
        let locale = self.ui_locale;
        let storage = self.storage.as_mut().ok_or_else(recovery_only)?;
        crate::c4::outbox::scan_once(storage, self.notify.as_mut(), now, &token, locale).map_err(
            |_| localized_error(locale, "outbox", "error.outbox", "action.retry_later", true),
        )
    }

    pub fn create_backup(&self, dest: &Path) -> Result<String, AppErrorDto> {
        if self.storage.is_none() {
            return Err(recovery_only());
        }
        let live = self.data_dir.join("monitor.sqlite3");
        let now = chrono::Utc::now().timestamp();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let result = BackupRestoreService::create_backup(&live, dest, &self.space, &cancel, now);
        app_log::emit(
            if result.is_ok() {
                Level::Info
            } else {
                Level::Error
            },
            "backup",
            serde_json::json!({ "ok": result.is_ok() }),
        );
        result.map(|manifest| manifest.checksum).map_err(map_report)
    }

    pub fn restore_backup(&mut self, candidate: &Path) -> Result<(), AppErrorDto> {
        self.storage = None;
        self.branch = BootBranch::RecoveryOnly;
        let live = self.data_dir.join("monitor.sqlite3");
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let restored = BackupRestoreService::restore(&live, candidate, &self.space, &cancel);
        app_log::emit(
            if restored.is_ok() {
                Level::Info
            } else {
                Level::Error
            },
            "restore",
            serde_json::json!({ "ok": restored.is_ok() }),
        );
        restored.map_err(map_report)?;
        match self.reboot_storage() {
            Ok(()) => Ok(()),
            Err(_) => {
                self.branch = BootBranch::RecoveryOnly;
                self.storage = None;
                Err(self.err(
                    "restore_reopen",
                    "error.restore_reopen",
                    "action.check_backup",
                    true,
                ))
            }
        }
    }

    pub fn about(&self) -> crate::c5::AboutDto {
        let _ = self;
        crate::c5::about()
    }

    pub fn preview_delete_local_data(&self) -> crate::c5::DeletePreview {
        crate::c5::preview_delete(&self.data_dir, &app_log::dir())
    }

    pub fn confirm_delete_local_data(
        &mut self,
        phrase: &str,
    ) -> Result<crate::c5::DeleteReport, AppErrorDto> {
        let _ = self.desktop.set_collector_running(false);
        self.storage = None;
        let target = self.settings.credential_target.clone();
        let data_dir = self.data_dir.clone();
        let log_dir = app_log::dir();
        let report = crate::c5::confirm_delete(&data_dir, &log_dir, phrase, || {
            self.workflow
                .delete_stored_target(&target)
                .map_err(|error| error.message_zh().to_string())
        })
        .map_err(|message| AppErrorDto {
            code: "delete_not_confirmed".into(),
            message_zh: message,
            retryable: false,
            action: "输入完整确认短语".into(),
            details_redacted: "delete".into(),
        })?;
        if report.all_declared_ok {
            if self.reboot_storage().is_err() {
                self.branch = BootBranch::RecoveryOnly;
            }
        } else {
            self.branch = BootBranch::RecoveryOnly;
        }
        let failed = report.items.iter().filter(|item| !item.ok).count() as u32;
        app_log::emit(
            if report.all_declared_ok {
                Level::Info
            } else {
                Level::Error
            },
            "delete",
            serde_json::json!({ "ok": report.all_declared_ok, "failed": failed }),
        );
        Ok(report)
    }

    pub fn run_user_vacuum(&mut self) -> Result<(), AppErrorDto> {
        let live = self.data_dir.join("monitor.sqlite3");
        self.storage = None;
        let result = crate::c5::run_user_vacuum(&live, &self.space);
        app_log::emit(
            if result.is_ok() {
                Level::Info
            } else {
                Level::Error
            },
            "vacuum",
            serde_json::json!({ "ok": result.is_ok() }),
        );
        if self.reboot_storage().is_err() {
            self.branch = BootBranch::RecoveryOnly;
        }
        result.map_err(map_report)
    }
}

fn retain_live_rows(input: &ControllerInput) -> bool {
    !matches!(
        input,
        ControllerInput::Restarted { .. }
            | ControllerInput::Shutdown
            | ControllerInput::Disconnected {
                reason: SessionStatus::Cancelled | SessionStatus::CoreRestarted
            }
    )
}

fn localized_error(
    locale: UiLocale,
    code: &str,
    message_key: &str,
    action_key: &str,
    retryable: bool,
) -> AppErrorDto {
    AppErrorDto {
        code: code.into(),
        message_zh: t(locale, message_key).into(),
        retryable,
        action: t(locale, action_key).into(),
        details_redacted: code.into(),
    }
}

fn recovery_only() -> AppErrorDto {
    recovery_only_locale(UiLocale::Zh)
}

fn recovery_only_locale(locale: UiLocale) -> AppErrorDto {
    AppErrorDto {
        code: "recovery_only".into(),
        message_zh: t(locale, "error.recovery_only_report").into(),
        retryable: false,
        action: t(locale, "action.restore_db").into(),
        details_redacted: "recovery".into(),
    }
}

fn map_report(error: ReportError) -> AppErrorDto {
    map_report_locale(error, UiLocale::Zh)
}

fn map_report_locale(error: ReportError, locale: UiLocale) -> AppErrorDto {
    AppErrorDto {
        code: error.code().into(),
        message_zh: error.message(locale).into(),
        retryable: matches!(
            error,
            ReportError::StorageBusy(_) | ReportError::DeadlineExceeded(_)
        ),
        action: error.action(locale).into(),
        details_redacted: error.code().into(),
    }
}

pub fn parse_socket(address: &str) -> Result<SocketAddr, AppErrorDto> {
    parse_socket_locale(address, UiLocale::Zh)
}

pub fn parse_socket_locale(address: &str, locale: UiLocale) -> Result<SocketAddr, AppErrorDto> {
    SocketAddr::from_str(address).map_err(|_| AppErrorDto {
        code: "invalid_address".into(),
        message_zh: t(locale, "error.invalid_address").into(),
        retryable: false,
        action: t(locale, "action.change_loopback").into(),
        details_redacted: "address".into(),
    })
}

pub fn assert_no_forbidden_tables(names: &[String]) {
    let allow = all_table_allowlist();
    for name in names {
        assert!(
            allow.contains(&name.as_str()),
            "{name} 不在 C1+C3 allowlist"
        );
        for fragment in forbidden_table_fragments() {
            assert!(!name.contains(fragment), "{name} 不得包含 {fragment}");
        }
    }
}

#[cfg(test)]
mod c2_facade_contract_tests {
    use super::*;
    use crate::c2::contract::c2_consumes_c1_modules;
    use crate::live::LiveProjection;
    use crate::storage::{list_user_tables, migrate};
    use tempfile::tempdir;

    #[test]
    fn test_notification_reason_has_no_internal_type_names() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        let capability = facade.test_notification().expect("capability");
        assert!(
            !capability.reason_zh.contains("Fake"),
            "{}",
            capability.reason_zh
        );
    }

    #[test]
    fn c2_only_names_frozen_c1_owners() {
        assert_eq!(
            c2_consumes_c1_modules(),
            [
                "ControllerSession",
                "AccountingEngine",
                "StorageCoordinator",
                "LiveProjection",
                "RecoveryFacade"
            ]
        );
        let _ = LiveProjection::new();
    }

    fn snapshot(id: &str) -> ControllerInput {
        ControllerInput::Snapshot {
            received_monotonic_ms: 1,
            received_utc: 1,
            upload_total: 10,
            download_total: 20,
            connections: vec![crate::controller::ConnectionFact {
                id: id.into(),
                upload: 1,
                download: 2,
                chains: vec!["DIRECT".into()],
                provider_chains: Vec::new(),
                meta: crate::controller::ConnectionMeta {
                    host: Some("a.test".into()),
                    ..crate::controller::ConnectionMeta::default()
                },
            }],
        }
    }

    #[test]
    fn reboot_reserves_new_receipt_and_controller_generation_before_first_frame() {
        let dir = tempdir().expect("dir");
        let mut first = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        let first_writer = first.writer_epoch;
        first.ingest_snapshot(snapshot("same-id"), 1, 1);
        let first_watermark = first
            .storage
            .as_ref()
            .expect("storage")
            .watermark()
            .expect("watermark");
        assert_eq!(first_watermark, 1);
        drop(first);

        let mut second = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert!(second.writer_epoch > first_writer);
        second.ingest_snapshot(snapshot("same-id"), 2, 2);
        let storage = second.storage.as_ref().expect("storage");
        assert_eq!(storage.watermark().expect("watermark"), 2);
        let epochs: i64 = storage
            .connection()
            .query_row(
                "select count(distinct epoch_id) from connection_session where connection_id='same-id'",
                [],
                |row| row.get(0),
            )
            .expect("epochs");
        assert_eq!(epochs, 2);
    }

    #[test]
    fn receipt_payload_mismatch_is_visible_in_live_health() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        let conflicting = CommitBundle {
            writer_epoch: facade.writer_epoch,
            bundle_seq: facade.bundle_seq,
            payload: "different-frame".into(),
        };
        facade
            .storage
            .as_mut()
            .expect("storage")
            .commit_alert_bundle(&conflicting, &AlertCommitSlice::default())
            .expect("seed conflicting receipt");

        facade.ingest_snapshot(snapshot("same-id"), 1, 1);

        let health = facade.hub.overview().health;
        assert!(!health.storage_ok);
        assert_eq!(health.storage_reason.as_deref(), Some("payload_mismatch"));
        assert_eq!(facade.bundle_seq, 1);
    }

    #[test]
    fn transient_decode_failure_keeps_rows_and_last_sample_until_reconnect() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.ingest_snapshot(snapshot("same-id"), 100, 100);
        assert_eq!(facade.hub.row_count(), 1);
        assert_eq!(facade.hub.overview().last_sample_utc, Some(100));

        facade.apply_probe_err(SessionStatus::ProtocolIncompatible);

        assert_eq!(facade.hub.row_count(), 1);
        assert_eq!(facade.hub.overview().last_sample_utc, Some(100));
        assert_eq!(
            facade.hub.overview().observation_phase,
            ObservationPhase::DecodeFailed
        );
        let before: i64 = facade
            .storage
            .as_ref()
            .expect("storage")
            .connection()
            .query_row("select count(*) from controller_epoch", [], |row| {
                row.get(0)
            })
            .expect("controller epochs");
        assert_eq!(before, 1);

        facade.ingest_snapshot(snapshot("same-id"), 101, 101);
        let after: i64 = facade
            .storage
            .as_ref()
            .expect("storage")
            .connection()
            .query_row("select count(*) from controller_epoch", [], |row| {
                row.get(0)
            })
            .expect("controller epochs");
        assert_eq!(after, 2);
        assert_eq!(facade.hub.row_count(), 1);
        assert_eq!(facade.hub.overview().last_sample_utc, Some(101));
    }

    #[test]
    fn controller_epoch_reservation_failure_updates_live_health() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade
            .storage
            .as_ref()
            .expect("storage")
            .connection()
            .execute(
                "insert into controller_epoch(epoch_id, core_identity) values (?1, 'max')",
                [i64::MAX],
            )
            .expect("seed max controller epoch");

        let message = facade.ingest_snapshot(snapshot("same-id"), 1, 1);

        assert!(matches!(
            message,
            Some(MonitorStreamMessage::HealthChanged { .. })
        ));
        let health = facade.hub.overview().health;
        assert!(!health.storage_ok);
        assert_eq!(
            health.storage_reason.as_deref(),
            Some("controller_epoch_closed")
        );
        assert!(!facade.controller_epoch_ready);
    }

    #[test]
    fn explicit_restart_reserves_durable_generation_on_next_frame() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.ingest_snapshot(snapshot("reused"), 1, 1);
        facade.apply_lifecycle(ControllerInput::Restarted {
            old_identity: "old".into(),
            new_identity: "new".into(),
        });
        assert_eq!(
            facade.hub.overview().observation_phase,
            ObservationPhase::ResyncRequired
        );
        facade.ingest_snapshot(snapshot("reused"), 2, 2);
        let storage = facade.storage.as_ref().expect("storage");
        let generations: i64 = storage
            .connection()
            .query_row("select count(*) from controller_epoch", [], |row| {
                row.get(0)
            })
            .expect("generations");
        let sessions: i64 = storage
            .connection()
            .query_row(
                "select count(*) from connection_session where connection_id='reused'",
                [],
                |row| row.get(0),
            )
            .expect("sessions");
        assert_eq!(generations, 2);
        assert_eq!(sessions, 2);
    }

    #[test]
    fn ui_locale_persists_and_falls_back_to_zh() {
        let dir = tempdir().expect("dir");
        let mut first = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(first.ui_locale, UiLocale::Zh);
        assert_eq!(first.save_ui_locale("en").expect("save"), UiLocale::En);
        drop(first);
        let second = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(second.ui_locale, UiLocale::En);
        assert_eq!(second.bootstrap().expect("boot").ui_locale, UiLocale::En);
        drop(second);
        let mut third = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(third.save_ui_locale("nope").expect("bad"), UiLocale::Zh);
        drop(third);
        let fourth = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(fourth.ui_locale, UiLocale::Zh);
    }

    #[test]
    fn ui_theme_persists_and_falls_back_to_mocha() {
        let dir = tempdir().expect("dir");
        let mut first = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(first.ui_theme, UiTheme::Mocha);
        assert_eq!(first.save_ui_theme("latte").expect("save"), UiTheme::Latte);
        drop(first);
        let second = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(second.ui_theme, UiTheme::Latte);
        assert_eq!(second.bootstrap().expect("boot").ui_theme, UiTheme::Latte);
        drop(second);
        let mut third = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(third.save_ui_theme("nope").expect("bad"), UiTheme::Mocha);
        drop(third);
        let fourth = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(fourth.ui_theme, UiTheme::Mocha);
    }

    #[test]
    fn ui_font_size_and_density_persist_and_fall_back() {
        let dir = tempdir().expect("dir");
        let mut first = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(first.ui_font, UiFont::system());
        assert_eq!(first.ui_font_size, UiFontSize::Md);
        assert_eq!(first.ui_density, UiDensity::Comfortable);
        assert_eq!(first.save_ui_font("yahei").expect("font").as_str(), "yahei");
        assert_eq!(first.save_ui_font_size("sm").expect("size"), UiFontSize::Sm);
        assert_eq!(
            first.save_ui_density("compact").expect("density"),
            UiDensity::Compact
        );
        drop(first);
        let second = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        let boot = second.bootstrap().expect("boot");
        assert_eq!(second.ui_font.as_str(), "yahei");
        assert_eq!(boot.ui_font.as_str(), "yahei");
        assert_eq!(boot.ui_font_size, UiFontSize::Sm);
        assert_eq!(boot.ui_density, UiDensity::Compact);
        drop(second);
        let mut third = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(
            third.save_ui_font("nope;").expect("bad font").as_str(),
            "system"
        );
        assert_eq!(
            third
                .save_ui_font("Microsoft YaHei")
                .expect("family")
                .as_str(),
            "Microsoft YaHei"
        );
        assert_eq!(
            third.save_ui_font_size("20").expect("bad size"),
            UiFontSize::Md
        );
        assert_eq!(
            third.save_ui_density("tight").expect("bad density"),
            UiDensity::Comfortable
        );
        drop(third);
        let fourth = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(fourth.ui_font.as_str(), "Microsoft YaHei");
        assert_eq!(fourth.ui_font_size, UiFontSize::Md);
        assert_eq!(fourth.ui_density, UiDensity::Comfortable);
    }

    #[test]
    fn ui_sidebar_width_persists_and_falls_back() {
        let dir = tempdir().expect("dir");
        let mut first = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(first.ui_sidebar_width, 220);
        assert_eq!(first.save_ui_sidebar_width(280).expect("save"), 280);
        drop(first);
        let second = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(second.ui_sidebar_width, 280);
        assert_eq!(second.bootstrap().expect("boot").ui_sidebar_width, 280);
        drop(second);
        let mut third = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(third.save_ui_sidebar_width(159).expect("low"), 160);
        assert_eq!(third.save_ui_sidebar_width(400).expect("high"), 352);
        drop(third);
        let fourth = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(fourth.ui_sidebar_width, 352);
        fourth
            .storage
            .as_ref()
            .expect("storage")
            .put_setting(crate::theme::SIDEBAR_WIDTH_SETTING_KEY, "nope")
            .expect("put");
        drop(fourth);
        let fifth = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(fifth.ui_sidebar_width, 220);
    }

    #[test]
    fn ui_sidebar_width_without_storage_stays_in_memory() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("monitor.sqlite3");
        {
            let connection = rusqlite::Connection::open(&path).expect("open");
            connection
                .execute_batch("pragma user_version = 99")
                .expect("ver");
        }
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(facade.branch, BootBranch::RecoveryOnly);
        assert!(facade.storage.is_none());
        assert_eq!(facade.save_ui_sidebar_width(280).expect("save"), 280);
        assert_eq!(facade.ui_sidebar_width, 280);
        drop(facade);
        let second = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(second.branch, BootBranch::RecoveryOnly);
        assert_eq!(second.ui_sidebar_width, 220);
    }

    #[test]
    fn live_table_layout_persists_and_sanitizes() {
        let dir = tempdir().expect("dir");
        let mut first = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert!(first.live_table_layout.hidden.is_empty());
        let mut layout = LiveTableLayout::default();
        layout.widths.insert("host".into(), 220);
        layout.hidden = vec!["process".into(), "action".into()];
        let saved = first.save_live_table_layout(layout).expect("save");
        assert_eq!(saved.widths.get("host"), Some(&220));
        assert_eq!(saved.hidden, vec!["process".to_string()]);
        drop(first);
        let second = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(second.live_table_layout.widths.get("host"), Some(&220));
        assert_eq!(second.live_table_layout.hidden, vec!["process".to_string()]);
        assert_eq!(
            second.bootstrap().expect("boot").live_table_layout.hidden,
            vec!["process".to_string()]
        );
    }

    #[test]
    fn dimension_rank_table_layout_persists_without_touching_live() {
        let dir = tempdir().expect("dir");
        let mut first = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        let live_before = first.live_table_layout.clone();
        let mut layout = DimensionRankTableLayout::default();
        layout.widths.insert("name".into(), 320);
        layout.widths.insert("rank".into(), 12);
        let saved = first
            .save_dimension_rank_table_layout(layout)
            .expect("save");
        assert_eq!(saved.widths.get("name"), Some(&320));
        assert!(!saved.widths.contains_key("rank"));
        assert_eq!(first.live_table_layout, live_before);
        drop(first);
        let second = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(
            second.dimension_rank_table_layout.widths.get("name"),
            Some(&320)
        );
        assert_eq!(second.live_table_layout, live_before);
        assert_eq!(
            second
                .bootstrap()
                .expect("boot")
                .dimension_rank_table_layout
                .widths
                .get("name"),
            Some(&320)
        );
    }

    #[test]
    fn dimension_rank_layout_without_storage_stays_in_memory() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("monitor.sqlite3");
        {
            let connection = rusqlite::Connection::open(&path).expect("open");
            connection
                .execute_batch("pragma user_version = 99")
                .expect("ver");
        }
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(facade.branch, BootBranch::RecoveryOnly);
        assert!(facade.storage.is_none());
        let mut layout = DimensionRankTableLayout::default();
        layout.widths.insert("name".into(), 300);
        let saved = facade
            .save_dimension_rank_table_layout(layout)
            .expect("save");
        assert_eq!(saved.widths.get("name"), Some(&300));
        assert_eq!(
            facade.dimension_rank_table_layout.widths.get("name"),
            Some(&300)
        );
        drop(facade);
        let second = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(second.branch, BootBranch::RecoveryOnly);
        assert_eq!(
            second.dimension_rank_table_layout.widths.get("name"),
            Some(&280)
        );
    }

    #[test]
    fn normal_boot_does_not_create_report_tables() {
        let dir = tempdir().expect("dir");
        let facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(facade.branch, BootBranch::NormalReady);
        let connection = migrate(&dir.path().join("monitor.sqlite3")).expect("reopen");
        let tables = list_user_tables(&connection).expect("tables");
        assert_no_forbidden_tables(&tables);
        assert!(tables.iter().any(|item| item == "traffic_hourly_dimension"));
        assert!(tables.iter().any(|item| item == "alert_rule"));
        assert!(tables.iter().any(|item| item == "notification_outbox"));
    }

    #[test]
    fn disconnect_now_leaves_cancelled_health() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.session_status = SessionStatus::Connected;
        facade.disconnect_now();
        assert_eq!(facade.session_status, SessionStatus::Cancelled);
        assert_eq!(facade.hub.overview().health.session, "cancelled");
    }

    #[test]
    fn storage_open_failure_logs_class_without_secret() {
        let _lock = crate::app_log::exclusive_test();
        let dir = tempdir().expect("dir");
        let logs = dir.path().join("logs");
        crate::app_log::init_at(logs.clone(), crate::app_log::DEFAULT_MAX_BYTES);
        std::fs::create_dir_all(dir.path().join("monitor.sqlite3")).expect("dir-as-db");
        let facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(facade.branch, BootBranch::RecoveryOnly);
        let text = std::fs::read_to_string(logs.join(crate::app_log::FILE_NAME)).expect("log");
        assert!(text.contains("storage_open"));
        assert!(text.contains("\"class\":\"sqlite\"") || text.contains("\"class\":\"closed\""));
        assert!(!crate::redact::scan_text_for_secrets(&text));
        crate::app_log::reset_for_test();
    }

    #[test]
    fn session_change_is_logged_once_per_code() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.apply_probe_err(SessionStatus::AuthFailed);
        assert_eq!(facade.last_logged_session, Some(SessionStatus::AuthFailed));
        facade.apply_probe_err(SessionStatus::AuthFailed);
        assert_eq!(facade.last_logged_session, Some(SessionStatus::AuthFailed));
        facade.apply_probe_err(SessionStatus::EndpointMissing);
        assert_eq!(
            facade.last_logged_session,
            Some(SessionStatus::EndpointMissing)
        );
    }

    #[test]
    fn persistent_secret_can_be_revealed_for_settings() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        let saved = facade
            .save_controller("127.0.0.1:9097".into(), Some("echo-secret".into()), false)
            .expect("save");
        assert_eq!(saved.secret_mode, "persistent");
        assert!(saved.has_secret);
        let revealed = facade.reveal_secret().expect("reveal");
        assert_eq!(revealed.as_deref(), Some("echo-secret"));
    }

    #[test]
    fn reconnect_now_leaves_connecting_health() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.session_status = SessionStatus::Connected;
        facade.disconnect_now();
        facade.reconnect_now();
        assert_eq!(facade.session_status, SessionStatus::Connecting);
        assert_eq!(facade.hub.overview().health.session, "connecting");
        assert!(facade.desktop.collector_running);
    }

    #[test]
    fn reopening_hidden_window_reconnects_cancelled_owner() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.settings.address = "127.0.0.1:9097".into();
        facade.disconnect_now();
        assert_eq!(facade.session_status, SessionStatus::Cancelled);
        assert_eq!(
            facade.desktop.close_window(),
            crate::c2::desktop::CloseWindowResult::HiddenFirstExplain
        );

        facade.open_main_window();

        assert_eq!(facade.session_status, SessionStatus::Connecting);
        assert!(facade.desktop.collector_running);
    }

    #[test]
    fn opening_window_without_persisted_address_does_not_reconnect() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.disconnect_now();
        let _ = facade.desktop.close_window();

        facade.open_main_window();

        assert_eq!(facade.session_status, SessionStatus::Cancelled);
    }

    #[test]
    fn opening_window_with_non_loopback_address_does_not_reconnect() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.settings.address = "192.0.2.10:9097".into();
        facade.disconnect_now();
        let _ = facade.desktop.close_window();

        assert!(facade.open_main_window().is_none());

        assert_eq!(facade.session_status, SessionStatus::Cancelled);
    }

    #[test]
    fn visible_window_keeps_manual_disconnect_cancelled() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.settings.address = "127.0.0.1:9097".into();
        facade.disconnect_now();

        facade.open_main_window();

        assert_eq!(facade.session_status, SessionStatus::Cancelled);
    }

    #[test]
    fn future_schema_enters_recovery_without_writer() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("monitor.sqlite3");
        {
            let connection = rusqlite::Connection::open(&path).expect("open");
            connection
                .execute_batch("pragma user_version = 99")
                .expect("ver");
        }
        let facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(facade.branch, BootBranch::RecoveryOnly);
        assert!(facade.storage.is_none());
        let status = facade.recovery().expect("status");
        assert!(status.future);
        assert!(status.restore_available);
        assert!(facade.storage.is_none());
    }

    fn recovery_only_boot() -> (tempfile::TempDir, AppFacade) {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("monitor.sqlite3");
        {
            let connection = rusqlite::Connection::open(&path).expect("open");
            connection
                .execute_batch("pragma user_version = 99")
                .expect("ver");
        }
        let facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        assert_eq!(facade.branch, BootBranch::RecoveryOnly);
        assert!(facade.storage.is_none());
        (dir, facade)
    }

    fn sqlite_table_count(path: &Path, table: &str) -> i64 {
        let sql = match table {
            "target_item" => "select count(*) from target_item",
            "alert_rule" => "select count(*) from alert_rule",
            other => panic!("unexpected table {other}"),
        };
        let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY;
        let connection = rusqlite::Connection::open_with_flags(path, flags).expect("ro");
        let exists: i64 = connection
            .query_row(
                "select count(*) from sqlite_master where type = 'table' and name = ?1",
                [table],
                |row| row.get(0),
            )
            .expect("master");
        if exists == 0 {
            0
        } else {
            connection
                .query_row(sql, [], |row| row.get(0))
                .expect("count")
        }
    }

    #[test]
    fn recovery_only_write_entry_points_return_recovery_only() {
        let (dir, mut facade) = recovery_only_boot();
        let db_path = dir.path().join("monitor.sqlite3");

        let report = facade
            .run_report(ReportQuery::default(), false)
            .expect_err("report");
        assert_eq!(report.code, "recovery_only");

        let targets = facade
            .save_targets(vec!["家宽".into()])
            .expect_err("targets");
        assert_eq!(targets.code, "recovery_only");
        assert_eq!(sqlite_table_count(&db_path, "target_item"), 0);

        let rule = facade
            .upsert_alert_rule(threshold_rule(100))
            .expect_err("rule");
        assert_eq!(rule.code, "recovery_only");
        assert_eq!(sqlite_table_count(&db_path, "alert_rule"), 0);

        let dest = dir.path().join("recovery-backup.sqlite3");
        let backup = facade.create_backup(&dest).expect_err("backup");
        assert_eq!(backup.code, "recovery_only");
        assert!(!dest.exists());
    }

    // ---- 08-22-storage-reboot-after-recovery：还原 / 删除 / VACUUM 后重跑存储侧启动 ----

    fn threshold_rule(threshold: i64) -> crate::c4::types::AlertRule {
        crate::c4::types::AlertRule {
            rule_id: "rate-cat".into(),
            version: 1,
            enabled: true,
            kind: crate::c4::types::AlertKind::Rate,
            selector_kind: crate::c4::types::SelectorKind::PrimaryCategory,
            selector_value: Some("家宽".into()),
            direction: Some(crate::c4::types::AlertDirection::Download),
            threshold_value: threshold,
            recovery_threshold: Some(40),
            period: None,
            timezone: "UTC".into(),
            cooldown_sec: 300,
            quiet_start_min: None,
            quiet_end_min: None,
            created_utc: 0,
            updated_utc: 0,
        }
    }

    fn db_max_writer_epoch(facade: &AppFacade) -> i64 {
        facade
            .storage
            .as_ref()
            .expect("storage")
            .connection()
            .query_row(
                "select coalesce(max(writer_epoch), 0) from committed_bundle",
                [],
                |row| row.get(0),
            )
            .expect("max epoch")
    }

    #[test]
    fn restore_backup_reboots_writer_epoch_across_backup_history() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        let epoch_before = facade.writer_epoch;
        facade.ingest_snapshot(snapshot("pre-backup"), 1, 1);
        facade.ingest_snapshot(snapshot("pre-backup"), 2, 2);
        assert!(db_max_writer_epoch(&facade) > 0);

        let backup_dir = tempdir().expect("backup dir");
        let backup = backup_dir.path().join("backup.sqlite3");
        facade.create_backup(&backup).expect("backup");

        // 备份之后继续提交，使还原库内的 epoch 历史落后于当前。
        facade.ingest_snapshot(snapshot("post-backup"), 3, 3);
        facade.restore_backup(&backup).expect("restore");

        assert!(facade.writer_epoch > epoch_before);
        assert!(facade.writer_epoch as i64 > db_max_writer_epoch(&facade));
        assert_eq!(facade.branch, BootBranch::NormalReady);
        assert!(facade.storage.is_some());
    }

    #[test]
    fn first_commit_after_restore_is_not_rejected_by_receipt() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.ingest_snapshot(snapshot("pre"), 1, 1);
        let backup_dir = tempdir().expect("backup dir");
        let backup = backup_dir.path().join("backup.sqlite3");
        facade.create_backup(&backup).expect("backup");
        facade.ingest_snapshot(snapshot("post"), 2, 2);
        facade.restore_backup(&backup).expect("restore");

        facade.ingest_snapshot(snapshot("after-restore"), 5, 5);
        let health = &facade.hub.overview().health;
        assert!(health.storage_ok, "health: {:?}", health.storage_reason);
        assert_ne!(health.storage_reason.as_deref(), Some("payload_mismatch"));
        assert_ne!(
            health.storage_reason.as_deref(),
            Some("retry_window_expired")
        );
    }

    #[test]
    fn restore_backup_recovers_alert_rules_from_backup_point() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.upsert_alert_rule(threshold_rule(100)).expect("R1");
        let backup_dir = tempdir().expect("backup dir");
        let backup = backup_dir.path().join("backup.sqlite3");
        facade.create_backup(&backup).expect("backup");

        let mut changed = threshold_rule(200);
        changed.version = 2;
        facade.upsert_alert_rule(changed).expect("R2");
        let rate = |facade: &AppFacade| {
            facade
                .list_alert_rules()
                .expect("rules")
                .into_iter()
                .find(|rule| rule.rule_id == "rate-cat")
                .expect("rate-cat rule")
        };
        assert_eq!(rate(&facade).threshold_value, 200);

        facade.restore_backup(&backup).expect("restore");
        let restored = rate(&facade);
        assert_eq!(restored.threshold_value, 100);
        assert_eq!(restored.version, 1);
    }

    #[test]
    fn user_vacuum_reboots_writer_epoch() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.ingest_snapshot(snapshot("pre"), 1, 1);
        let before = facade.writer_epoch;
        facade.run_user_vacuum().expect("vacuum");
        assert!(facade.writer_epoch > before);
        assert_eq!(facade.branch, BootBranch::NormalReady);
        assert!(facade.storage.is_some());
    }

    #[test]
    fn confirm_delete_reboots_storage_to_first_epoch() {
        // confirm_delete 会删除真实日志目录；用环境变量指到临时目录。
        let log_dir = tempdir().expect("log dir");
        std::env::set_var("RESIDENTIAL_MONITOR_LOG_DIR", log_dir.path());
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.ingest_snapshot(snapshot("pre"), 1, 1);
        assert!(facade.writer_epoch >= 1);

        let report = facade
            .confirm_delete_local_data(crate::identity::DELETE_CONFIRM_PHRASE)
            .expect("delete");
        assert!(report.all_declared_ok);

        assert_eq!(facade.branch, BootBranch::NormalReady);
        assert!(facade.storage.is_some());
        assert_eq!(facade.settings, ControllerSettings::default());
        // 库只含迁移种子规则（health-*），无用户规则。
        let rule_ids: Vec<String> = facade
            .list_alert_rules()
            .expect("rules")
            .into_iter()
            .map(|rule| rule.rule_id)
            .collect();
        assert!(rule_ids.iter().all(|id| id.starts_with("health-")));
        assert_eq!(facade.writer_epoch, 1);
        std::env::remove_var("RESIDENTIAL_MONITOR_LOG_DIR");
    }

    #[test]
    fn failed_reopen_after_restore_stays_recovery_only() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.ingest_snapshot(snapshot("pre"), 1, 1);
        let bogus = dir.path().join("not-a-backup.sqlite3");
        std::fs::write(&bogus, b"not a sqlite db").expect("bogus");

        let result = facade.restore_backup(&bogus);
        assert!(result.is_err());
        assert_eq!(facade.branch, BootBranch::RecoveryOnly);
        assert!(facade.storage.is_none());
    }

    #[test]
    fn restore_backup_keeps_data_dir_and_launch_mode() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.ingest_snapshot(snapshot("pre"), 1, 1);
        let backup_dir = tempdir().expect("backup dir");
        let backup = backup_dir.path().join("backup.sqlite3");
        facade.create_backup(&backup).expect("backup");
        facade.ingest_snapshot(snapshot("post"), 2, 2);

        let data_dir_before = facade.data_dir.clone();
        let launch_before = facade.desktop.launch_mode;
        facade.restore_backup(&backup).expect("restore");
        assert_eq!(facade.data_dir, data_dir_before);
        assert_eq!(facade.desktop.launch_mode, launch_before);
    }

    #[test]
    fn restore_backup_keeps_workflow_and_session_secret() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        let persistent_before = facade.workflow.persistent_available();
        let settings = facade
            .workflow
            .save_secret(
                &facade.settings.clone(),
                "127.0.0.1:9090",
                Some("session-secret"),
                true,
                false,
            )
            .expect("save secret");
        facade.settings = settings;
        facade.ingest_snapshot(snapshot("pre"), 1, 1);
        let backup_dir = tempdir().expect("backup dir");
        let backup = backup_dir.path().join("backup.sqlite3");
        facade.create_backup(&backup).expect("backup");
        facade.ingest_snapshot(snapshot("post"), 2, 2);

        facade.restore_backup(&backup).expect("restore");
        assert_eq!(facade.workflow.persistent_available(), persistent_before);
        let secret = facade
            .workflow
            .resolve(&facade.settings.credential_target, "session")
            .expect("resolve session secret");
        assert_eq!(secret.as_header_bytes(), b"session-secret");
    }
}

#[cfg(test)]
mod c2_close_control_tests {
    use super::*;
    use crate::c2::close::CloseMark;
    use crate::controller::{ConnectionFact, ConnectionMeta};
    use tempfile::tempdir;

    fn snap(ids: &[&str]) -> ControllerInput {
        ControllerInput::Snapshot {
            received_monotonic_ms: 1,
            received_utc: 1,
            upload_total: 0,
            download_total: 0,
            connections: ids
                .iter()
                .map(|id| ConnectionFact {
                    id: (*id).into(),
                    upload: 1,
                    download: 1,
                    chains: Vec::new(),
                    provider_chains: Vec::new(),
                    meta: ConnectionMeta {
                        host: Some(format!("{id}.test")),
                        source_ip: None,
                        destination_ip: None,
                        process_name: None,
                        process_path: None,
                        network: Some("tcp".into()),
                        rule: None,
                        rule_payload: None,
                        ..crate::controller::ConnectionMeta::default()
                    },
                })
                .collect(),
        }
    }

    #[test]
    fn pause_keeps_existing_projection_rows() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.ingest_snapshot(snap(&["keep"]), 10, 10);
        assert_eq!(facade.hub.row_count(), 1);
        facade.apply_lifecycle(ControllerInput::Paused);
        assert_eq!(facade.hub.row_count(), 1);
        assert_eq!(facade.hub.rows()[0].connection_id, "keep");
        facade.apply_lifecycle(ControllerInput::Resumed);
        assert_eq!(facade.hub.row_count(), 1);
        facade.apply_lifecycle(ControllerInput::SleepGap {
            started_utc: 1,
            ended_utc: 2,
        });
        assert_eq!(facade.hub.row_count(), 1);
        facade.apply_lifecycle(ControllerInput::Disconnected {
            reason: SessionStatus::Cancelled,
        });
        assert_eq!(facade.hub.row_count(), 0);
    }

    #[test]
    fn missing_id_204_is_accepted_until_remove() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.ingest_snapshot(snap(&["keep"]), 10, 10);
        let state = facade.mark_close_accepted_from_control(
            "0:missing".into(),
            "req-204".into(),
            ControlResult::Accepted,
        );
        assert_eq!(state.mark, CloseMark::Accepted);
        facade.ingest_snapshot(snap(&["keep"]), 11, 20);
        assert_eq!(
            facade.closes.mark_of("0:missing"),
            Some(CloseMark::Accepted)
        );
    }

    #[test]
    fn query_returns_rows_and_sample_time_from_one_hub_snapshot() {
        let dir = tempdir().expect("dir");
        let mut facade = AppFacade::boot(dir.path(), &["app".into()], InstanceClaim::Owner);
        facade.ingest_snapshot(snap(&["keep"]), 123, 123);

        let page = facade.query(&ConnectionQuery::default());

        assert_eq!(page.sample_utc, Some(123));
        assert_eq!(page.matched_count, 1);
        assert_eq!(page.rows[0].connection_id, "keep");
        assert_eq!(
            page.summary
                .top_download
                .as_ref()
                .map(|item| item.identity.as_str()),
            Some(page.rows[0].identity.as_str())
        );
    }
}

#[cfg(test)]
mod c2_peak_ui_tests {
    use super::*;
    use crate::c2::hub::LiveConnectionView;
    use crate::c2::query::{query_connections, ConnectionFilter};
    use std::time::Instant;

    fn row(index: u32, upload: u64) -> LiveConnectionView {
        LiveConnectionView {
            identity: format!("0:c{index:05}"),
            connection_id: format!("c{index:05}"),
            epoch: 0,
            upload,
            download: upload,
            rate_upload: None,
            rate_download: None,
            duration_ms: None,
            primary: None,
            tags: Vec::new(),
            host: Some(format!("h{index:05}.test")),
            source_ip: None,
            destination_ip: None,
            process_name: Some("app.exe".into()),
            process_path: None,
            network: Some("tcp".into()),
            rule: None,
            rule_payload: None,
            chains: Vec::new(),
            ..LiveConnectionView::default()
        }
    }

    #[test]
    fn peak_10k_1800_frames_stay_bounded_and_fast_to_filter() {
        let hub = MonitorHub::new();
        let _ = hub.subscribe();
        let mut last_count = 0;
        let mut latencies = Vec::new();
        let frames = 30_u64;
        for frame in 0..frames {
            let rows: Vec<_> = (0..10_000)
                .map(|index| row(index, frame + u64::from(index)))
                .collect();
            let batch = crate::accounting::AccountingEngine::new().apply(
                ControllerInput::Paused,
                frame,
                frame as i64,
            );
            let started = Instant::now();
            hub.publish(
                &batch,
                rows,
                health_from(SessionStatus::Connected, None),
                frame as i64,
            )
            .expect("publish");
            let page = query_connections(
                &hub.rows(),
                &ConnectionQuery {
                    filter: ConnectionFilter {
                        host: Some("h00001.test".into()),
                        ..ConnectionFilter::default()
                    },
                    limit: 200,
                    ..ConnectionQuery::default()
                },
            );
            latencies.push(started.elapsed().as_secs_f64() * 1000.0);
            last_count = hub.row_count();
            assert_eq!(page.rows.len(), 1);
        }
        assert_eq!(last_count, 10_000);
        latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95 = latencies[(latencies.len() * 95) / 100];
        assert!(
            p95 < 150.0,
            "筛选 p95={p95}ms，必须小于 150ms；10k 短时峰值不是 30 天容量"
        );
    }
}

pub fn run_peak_ui_replay(frames: u32, active: u32) -> PeakUiReport {
    let hub = MonitorHub::new();
    let _ = hub.subscribe();
    let mut latencies = Vec::new();
    for frame in 0..frames {
        let rows: Vec<_> = (0..active)
            .map(|index| crate::c2::hub::LiveConnectionView {
                identity: format!("0:c{index:05}"),
                connection_id: format!("c{index:05}"),
                epoch: 0,
                upload: u64::from(frame + index),
                download: u64::from(frame),
                rate_upload: None,
                rate_download: None,
                duration_ms: None,
                primary: None,
                tags: Vec::new(),
                host: Some(format!("h{index:05}.test")),
                source_ip: None,
                destination_ip: None,
                process_name: Some("app.exe".into()),
                process_path: None,
                network: Some("tcp".into()),
                rule: None,
                rule_payload: None,
                chains: Vec::new(),
                ..crate::c2::hub::LiveConnectionView::default()
            })
            .collect();
        let batch = AccountingEngine::new().apply(
            ControllerInput::Paused,
            u64::from(frame),
            i64::from(frame),
        );
        let started = std::time::Instant::now();
        let _ = hub.publish(
            &batch,
            rows,
            health_from(SessionStatus::Connected, None),
            i64::from(frame),
        );
        let _ = crate::c2::query::query_connections(&hub.rows(), &ConnectionQuery::default());
        latencies.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let index = |pct: usize| latencies[(latencies.len().saturating_sub(1) * pct) / 100];
    PeakUiReport {
        frames,
        active,
        row_count: hub.row_count() as u32,
        p50_ms: index(50),
        p95_ms: index(95),
        p99_ms: index(99),
        max_ms: *latencies.last().unwrap_or(&0.0),
        not_thirty_day_capacity: true,
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeakUiReport {
    pub frames: u32,
    pub active: u32,
    pub row_count: u32,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
    pub not_thirty_day_capacity: bool,
}
