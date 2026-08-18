//! C2 AppFacade：只经 C1 接口访问采集、存储、投影与恢复。

use crate::accounting::AccountingEngine;
use crate::c0_contract::{all_table_allowlist, forbidden_table_fragments};
use crate::c2::close::{CloseRegistry, CloseState, ControlResult};
use crate::c2::contract::SCHEMA_VERSION;
use crate::c2::desktop::{DesktopRuntime, FakeAutostart, InstanceClaim, LaunchMode, ShutdownPhase};
use crate::c2::hub::{health_from, LiveOverview, MonitorHub, MonitorStreamMessage};
use crate::c2::query::{query_connections, ConnectionPage, ConnectionQuery};
use crate::c2::settings::{
    validate_targets, ControllerSettings, SettingsError, SettingsWorkflow, WizardState,
};
use crate::c2::shell::{
    default_routes, recovery_status, validate_backup, BootBranch, FakeFileDialog, FileMode,
    FilePurpose, OperationProgress, OperationRegistry, RecoveryStatus, RouteDescriptor,
};
use crate::c3::backup::BackupRestoreService;
use crate::c3::export::{ExportPreview, ExportService, ExportSpec};
use crate::c3::query::{ReportError, ReportQuery, ReportResult, RAW_RETAIN_DAYS_DEFAULT};
use crate::c3::retention::{RetentionMode, RetentionPreview, RetentionService};
use crate::c3::snapshot::ReportSnapshotStore;
use crate::c3::space::SpaceBudget;
use crate::c4::engine::{AlertEngine, HealthSnapshot};
use crate::c4::notify::{FakeNotificationSink, NotificationSink, NotifyPayload};
use crate::c4::types::{
    validate_rule, AlertCenterPage, AlertRule, AlertSummary, ALERT_DTO_VERSION,
};
use crate::controller::{ControllerInput, SessionStatus};
use crate::credential::FakeCredentialStore;
use crate::session::ControllerSession;
use crate::storage::{AlertCommitSlice, CommitBundle, RecoveryFacade, StorageCoordinator};
use serde::Serialize;
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

impl AppErrorDto {
    pub fn from_settings(error: SettingsError) -> Self {
        Self {
            code: error.code().into(),
            message_zh: error.message_zh().into(),
            retryable: matches!(
                error,
                SettingsError::ProbeFailed | SettingsError::Unavailable
            ),
            action: "检查控制器或凭据后重试".into(),
            details_redacted: error.code().into(),
        }
    }

    pub fn from_status(status: SessionStatus) -> Self {
        let code = crate::c2::hub::session_status_name(status);
        Self {
            code: code.clone(),
            message_zh: status_message_zh(status).into(),
            retryable: !matches!(
                status,
                SessionStatus::NonLoopback | SessionStatus::ProtocolIncompatible
            ),
            action: status_action_zh(status).into(),
            details_redacted: code,
        }
    }
}

pub fn status_message_zh(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Connecting => "正在连接控制器。",
        SessionStatus::Connected => "已连接。",
        SessionStatus::AuthFailed => "TCP 鉴权失败。",
        SessionStatus::PipeAccessDenied => "管道访问被拒绝。",
        SessionStatus::PipeBusyTimeout => "管道忙超时。",
        SessionStatus::EndpointMissing => "控制器端点不存在。",
        SessionStatus::ProtocolIncompatible => "协议不兼容，请改用 TCP。",
        SessionStatus::PidMismatch => "管道进程身份不匹配。",
        SessionStatus::CoreRestarted => "核心已重启。",
        SessionStatus::Cancelled => "操作已取消。",
        SessionStatus::NonLoopback => "拒绝非回环地址。",
    }
}

pub fn status_action_zh(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::AuthFailed => "检查本机 secret 后重试",
        SessionStatus::PipeAccessDenied | SessionStatus::ProtocolIncompatible => {
            "启用 TCP External Controller"
        }
        SessionStatus::EndpointMissing => "检查控制器地址或重新发现",
        _ => "重试连接",
    }
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
    pub workflow: SettingsWorkflow<FakeCredentialStore>,
    pub closes: CloseRegistry,
    pub operations: OperationRegistry,
    pub dialog: FakeFileDialog,
    pub autostart: FakeAutostart,
    pub session: ControllerSession,
    pub data_dir: PathBuf,
    pub session_status: SessionStatus,
    pub snapshots: ReportSnapshotStore,
    pub space: SpaceBudget,
    pub raw_retain_days: i64,
    pub alerts: AlertEngine,
    pub notify: FakeNotificationSink,
    pub writer_epoch: u64,
    pub bundle_seq: u64,
    pub last_frame_utc: Option<i64>,
    pub last_period_eval_utc: i64,
}

impl AppFacade {
    pub fn boot(data_dir: impl Into<PathBuf>, args: &[String], claim: InstanceClaim) -> Self {
        let data_dir = data_dir.into();
        let db_path = data_dir.join("monitor.sqlite3");
        let desktop = DesktopRuntime::start(args, claim);
        match StorageCoordinator::open(&db_path) {
            Ok(storage) => {
                let settings = storage
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
                Self {
                    branch: BootBranch::NormalReady,
                    desktop,
                    hub: MonitorHub::new(),
                    engine,
                    recovery: RecoveryFacade::open(&db_path),
                    storage: Some(storage),
                    settings,
                    wizard: WizardState::default(),
                    wizard_complete,
                    workflow: SettingsWorkflow::new(FakeCredentialStore::new(), true),
                    closes: CloseRegistry::new(),
                    operations: OperationRegistry::new(),
                    dialog: FakeFileDialog::default(),
                    autostart: FakeAutostart::new(),
                    session: ControllerSession::new(String::new()),
                    data_dir: data_dir.clone(),
                    session_status: SessionStatus::Connecting,
                    snapshots: ReportSnapshotStore::open(&data_dir),
                    space: SpaceBudget::unlimited(),
                    raw_retain_days: RAW_RETAIN_DAYS_DEFAULT,
                    alerts,
                    notify: FakeNotificationSink::default(),
                    writer_epoch: 1,
                    bundle_seq: 1,
                    last_frame_utc: None,
                    last_period_eval_utc: 0,
                }
            }
            Err(_) => Self {
                branch: BootBranch::RecoveryOnly,
                desktop,
                hub: MonitorHub::new(),
                engine: AccountingEngine::new(),
                storage: None,
                recovery: RecoveryFacade::open(&db_path),
                settings: ControllerSettings::default(),
                wizard: WizardState::default(),
                wizard_complete: false,
                workflow: SettingsWorkflow::new(FakeCredentialStore::new(), false),
                closes: CloseRegistry::new(),
                operations: OperationRegistry::new(),
                dialog: FakeFileDialog::default(),
                autostart: FakeAutostart::new(),
                session: ControllerSession::new(String::new()),
                data_dir: data_dir.clone(),
                session_status: SessionStatus::EndpointMissing,
                snapshots: ReportSnapshotStore::open(&data_dir),
                space: SpaceBudget::unlimited(),
                raw_retain_days: RAW_RETAIN_DAYS_DEFAULT,
                alerts: AlertEngine::new(),
                notify: FakeNotificationSink::default(),
                writer_epoch: 1,
                bundle_seq: 1,
                last_frame_utc: None,
                last_period_eval_utc: 0,
            },
        }
    }

    pub fn bootstrap(&self) -> Result<BootstrapDto, AppErrorDto> {
        let recovery = if self.branch == BootBranch::RecoveryOnly {
            Some(recovery_status(&self.recovery).map_err(|_| AppErrorDto {
                code: "recovery_status".into(),
                message_zh: "无法读取恢复诊断。".into(),
                retryable: true,
                action: "打开数据目录检查文件".into(),
                details_redacted: "recovery".into(),
            })?)
        } else {
            None
        };
        Ok(BootstrapDto {
            schema_version: SCHEMA_VERSION,
            branch: self.branch,
            routes: default_routes(),
            overview: self.hub.overview(),
            settings: self.settings.clone(),
            wizard_complete: self.wizard_complete,
            recovery,
            launch_mode: self.desktop.launch_mode,
        })
    }

    pub fn subscribe(&self) -> MonitorStreamMessage {
        self.hub.subscribe()
    }

    pub fn resync(&self, subscription_id: u64) -> MonitorStreamMessage {
        self.hub.resync(subscription_id)
    }

    pub fn apply_lifecycle(&mut self, input: ControllerInput) {
        let utc = chrono::Utc::now().timestamp();
        let batch = self.engine.apply(input, 0, utc);
        let health = health_from(
            self.session_status,
            self.storage
                .as_ref()
                .and_then(|item| item.health().ok())
                .as_ref(),
        );
        self.commit_eval(&batch, &[], utc, 0);
        let _ = self.hub.publish(&batch, Vec::new(), health, utc);
    }

    pub fn ingest_snapshot(&mut self, input: ControllerInput, utc: i64, mono: u64) {
        if let ControllerInput::Snapshot {
            ref connections, ..
        } = input
        {
            let live = self.engine.project_live(connections);
            let batch = self.engine.apply(input, mono, utc);
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
            let health = health_from(
                SessionStatus::Connected,
                self.storage
                    .as_ref()
                    .and_then(|item| item.health().ok())
                    .as_ref(),
            );
            self.session_status = SessionStatus::Connected;
            self.commit_eval(&batch, &live, utc, utc as u64);
            let _ = self.hub.publish(&batch, live, health, utc);
        } else {
            self.apply_lifecycle(input);
        }
    }

    fn commit_eval(
        &mut self,
        batch: &crate::accounting::AccountingBatch,
        live: &[crate::c2::hub::LiveConnectionView],
        utc: i64,
        mono: u64,
    ) {
        if self.storage.is_none() {
            return;
        }
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
        let outcome = self
            .storage
            .as_mut()
            .and_then(|storage| storage.commit_alert_bundle(&bundle, &slice).ok());
        if matches!(
            outcome,
            Some(
                crate::storage::CommitOutcome::Applied(_)
                    | crate::storage::CommitOutcome::Duplicate(_)
            )
        ) {
            self.bundle_seq = self.bundle_seq.saturating_add(1);
            self.last_frame_utc = Some(utc);
            let token = format!("lease-{}", self.bundle_seq);
            if let Some(storage) = self.storage.as_mut() {
                let _ = crate::c4::outbox::scan_once(storage, &mut self.notify, utc, &token);
            }
        }
    }

    pub fn query(&self, query: &ConnectionQuery) -> ConnectionPage {
        let rows = self.hub.rows();
        query_connections(&rows, query)
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
        let storage = self.storage.as_ref().ok_or_else(|| AppErrorDto {
            code: "recovery_only".into(),
            message_zh: "当前处于恢复模式，不能保存设置。".into(),
            retryable: false,
            action: "先修复数据库".into(),
            details_redacted: "recovery".into(),
        })?;
        let encoded = serde_json::to_string(&self.settings).map_err(|_| AppErrorDto {
            code: "encode".into(),
            message_zh: "设置编码失败。".into(),
            retryable: false,
            action: "重试".into(),
            details_redacted: "encode".into(),
        })?;
        storage
            .put_setting("controller", &encoded)
            .map_err(|_| AppErrorDto {
                code: "storage".into(),
                message_zh: "设置写入失败。".into(),
                retryable: true,
                action: "检查磁盘后重试".into(),
                details_redacted: "storage".into(),
            })?;
        storage
            .put_setting(
                "wizard_complete",
                if self.wizard_complete { "1" } else { "0" },
            )
            .map_err(|_| AppErrorDto {
                code: "storage".into(),
                message_zh: "向导状态写入失败。".into(),
                retryable: true,
                action: "检查磁盘后重试".into(),
                details_redacted: "storage".into(),
            })?;
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
            .map_err(AppErrorDto::from_settings)?;
        self.settings = next.clone();
        self.session.endpoint = address;
        self.persist_settings()?;
        Ok(next)
    }

    pub fn save_targets(&mut self, targets: Vec<String>) -> Result<u32, AppErrorDto> {
        validate_targets(&targets).map_err(AppErrorDto::from_settings)?;
        let storage = self.storage.as_ref().ok_or_else(|| AppErrorDto {
            code: "recovery_only".into(),
            message_zh: "恢复模式不能保存目标。".into(),
            retryable: false,
            action: "先修复数据库".into(),
            details_redacted: "recovery".into(),
        })?;
        let version = storage.save_targets(&targets).map_err(|_| AppErrorDto {
            code: "storage".into(),
            message_zh: "目标写入失败。".into(),
            retryable: true,
            action: "检查磁盘后重试".into(),
            details_redacted: "storage".into(),
        })?;
        self.engine.set_targets(targets);
        Ok(version)
    }

    pub fn complete_wizard(&mut self) -> Result<(), AppErrorDto> {
        self.wizard_complete = true;
        self.persist_settings()
    }

    pub fn recovery(&self) -> Result<RecoveryStatus, AppErrorDto> {
        recovery_status(&self.recovery).map_err(|_| AppErrorDto {
            code: "recovery_status".into(),
            message_zh: "无法读取恢复诊断。".into(),
            retryable: true,
            action: "打开数据目录".into(),
            details_redacted: "recovery".into(),
        })
    }

    pub fn validate_candidate(&self, path: &Path) -> Result<bool, AppErrorDto> {
        validate_backup(&self.recovery, path).map_err(|_| AppErrorDto {
            code: "validate_backup".into(),
            message_zh: "候选备份无效。".into(),
            retryable: false,
            action: "选择其他备份".into(),
            details_redacted: "backup".into(),
        })
    }

    pub fn pick_file(&self, purpose: FilePurpose, mode: FileMode) -> Option<PathBuf> {
        use crate::c2::shell::FileDialogPort;
        self.dialog.pick(purpose, mode)
    }

    pub fn start_operation(&mut self, id: String, kind: String) -> OperationProgress {
        self.operations.start_fixture(id, kind)
    }

    pub fn shutdown(&mut self) -> Vec<ShutdownPhase> {
        self.workflow.clear_session();
        self.apply_lifecycle(ControllerInput::Shutdown);
        self.desktop.begin_shutdown()
    }

    pub fn run_report(&mut self, query: ReportQuery) -> Result<ReportResult, AppErrorDto> {
        let path = self
            .storage
            .as_ref()
            .ok_or_else(recovery_only)?
            .path()
            .to_path_buf();
        let now = chrono::Utc::now().timestamp();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        crate::c3::ReportService::run(
            &path,
            &mut self.snapshots,
            query,
            now,
            self.raw_retain_days,
            &cancel,
            None,
        )
        .map_err(map_report)
    }

    pub fn get_report(&self, token: &str) -> Result<ReportResult, AppErrorDto> {
        let now = chrono::Utc::now().timestamp();
        self.snapshots.get(token, now).cloned().map_err(map_report)
    }

    pub fn release_report(&mut self, token: &str) -> bool {
        self.snapshots.release(token)
    }

    pub fn preview_export(
        &self,
        token: &str,
        spec: &ExportSpec,
    ) -> Result<ExportPreview, AppErrorDto> {
        let result = self.get_report(token)?;
        ExportService::preview(&result, spec).map_err(map_report)
    }

    pub fn export_report(
        &self,
        token: &str,
        spec: &ExportSpec,
        dest: &Path,
    ) -> Result<String, AppErrorDto> {
        let result = self.get_report(token)?;
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        ExportService::export_to_path(&result, spec, dest, &self.space, &cancel)
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
        )
        .map_err(map_report)?;
        if let Some(storage) = self.storage.as_ref() {
            let _ = crate::c4::store::retain_alerts(storage.connection(), now);
        }
        Ok(preview)
    }

    pub fn list_alert_rules(&self) -> Result<Vec<AlertRule>, AppErrorDto> {
        let storage = self.storage.as_ref().ok_or_else(recovery_only)?;
        crate::c4::store::load_rules(storage.connection()).map_err(|_| AppErrorDto {
            code: "storage".into(),
            message_zh: "无法读取告警规则。".into(),
            retryable: true,
            action: "检查磁盘后重试".into(),
            details_redacted: "alert".into(),
        })
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
        let storage = self.storage.as_mut().ok_or_else(recovery_only)?;
        crate::c4::store::upsert_rule(storage.connection(), &rule).map_err(|_| AppErrorDto {
            code: "storage".into(),
            message_zh: "规则写入失败。".into(),
            retryable: true,
            action: "检查磁盘后重试".into(),
            details_redacted: "alert".into(),
        })?;
        if !writes.instances.is_empty() || !writes.events.is_empty() {
            let bundle = CommitBundle {
                writer_epoch: self.writer_epoch,
                bundle_seq: self.bundle_seq,
                payload: String::new(),
            };
            let slice = AlertCommitSlice {
                utc: now,
                writes,
                ..AlertCommitSlice::default()
            };
            if storage.commit_alert_bundle(&bundle, &slice).is_ok() {
                self.bundle_seq = self.bundle_seq.saturating_add(1);
            }
        }
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
            message_zh: "无法读取告警中心。".into(),
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

    pub fn test_notification(
        &mut self,
    ) -> Result<crate::c4::notify::NotifyCapability, AppErrorDto> {
        let cap = self.notify.capability();
        let payload = NotifyPayload {
            title_zh: "测试通知".into(),
            body_zh: "这是测试通知，不会写入告警历史。".into(),
            event_id: "test-notify".into(),
            instance_id: None,
            test_only: true,
        };
        match self.notify.send(&payload) {
            Ok(()) => Ok(cap),
            Err(_) => Ok(self.notify.capability()),
        }
    }

    pub fn get_diagnostics(&self) -> Result<crate::c4::diagnose::DiagnosticsSnapshot, AppErrorDto> {
        let storage = self.storage.as_ref().ok_or_else(recovery_only)?;
        let coverage = self
            .hub
            .overview()
            .coverage_kind
            .unwrap_or_else(|| "unknown".into());
        crate::c4::diagnose::collect(storage, self.session_status, self.last_frame_utc, &coverage)
            .map_err(|_| AppErrorDto {
                code: "diagnostics".into(),
                message_zh: "诊断生成失败。采集未中断。".into(),
                retryable: true,
                action: "稍后重试".into(),
                details_redacted: "diagnostics".into(),
            })
    }

    pub fn export_diagnostics(&self, dest: &std::path::Path) -> Result<String, AppErrorDto> {
        let snap = self.get_diagnostics()?;
        crate::c4::diagnose::export_atomic(&snap, dest).map_err(|_| AppErrorDto {
            code: "diagnostics_export".into(),
            message_zh: "诊断导出失败。采集与告警未回滚。".into(),
            retryable: true,
            action: "更换路径后重试".into(),
            details_redacted: "diagnostics".into(),
        })
    }

    pub fn scan_outbox(&mut self) -> Result<u32, AppErrorDto> {
        let now = chrono::Utc::now().timestamp();
        let token = format!("scan-{}", self.bundle_seq);
        let storage = self.storage.as_mut().ok_or_else(recovery_only)?;
        crate::c4::outbox::scan_once(storage, &mut self.notify, now, &token).map_err(|_| {
            AppErrorDto {
                code: "outbox".into(),
                message_zh: "通知扫描失败。".into(),
                retryable: true,
                action: "稍后重试".into(),
                details_redacted: "outbox".into(),
            }
        })
    }

    pub fn create_backup(&self, dest: &Path) -> Result<String, AppErrorDto> {
        let live = self.data_dir.join("monitor.sqlite3");
        let now = chrono::Utc::now().timestamp();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        BackupRestoreService::create_backup(&live, dest, &self.space, &cancel, now)
            .map(|manifest| manifest.checksum)
            .map_err(map_report)
    }

    pub fn restore_backup(&mut self, candidate: &Path) -> Result<(), AppErrorDto> {
        self.storage = None;
        self.branch = BootBranch::RecoveryOnly;
        let live = self.data_dir.join("monitor.sqlite3");
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        BackupRestoreService::restore(&live, candidate, &self.space, &cancel)
            .map_err(map_report)?;
        match StorageCoordinator::open(&live) {
            Ok(storage) => {
                self.storage = Some(storage);
                self.branch = BootBranch::NormalReady;
                Ok(())
            }
            Err(_) => Err(AppErrorDto {
                code: "restore_reopen".into(),
                message_zh: "恢复后无法打开数据库，仍停留在 Recovery Shell。".into(),
                retryable: true,
                action: "检查备份后重试".into(),
                details_redacted: "restore".into(),
            }),
        }
    }
}

fn recovery_only() -> AppErrorDto {
    AppErrorDto {
        code: "recovery_only".into(),
        message_zh: "恢复模式不能运行普通报告。".into(),
        retryable: false,
        action: "先恢复数据库".into(),
        details_redacted: "recovery".into(),
    }
}

fn map_report(error: ReportError) -> AppErrorDto {
    AppErrorDto {
        code: error.code().into(),
        message_zh: error.message_zh().into(),
        retryable: matches!(
            error,
            ReportError::StorageBusy(_) | ReportError::DeadlineExceeded(_)
        ),
        action: error.action_zh().into(),
        details_redacted: error.code().into(),
    }
}

pub fn parse_socket(address: &str) -> Result<SocketAddr, AppErrorDto> {
    SocketAddr::from_str(address).map_err(|_| AppErrorDto {
        code: "invalid_address".into(),
        message_zh: "控制器地址无效。".into(),
        retryable: false,
        action: "改用 127.0.0.1:端口".into(),
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
                    },
                })
                .collect(),
        }
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
}

#[cfg(test)]
mod c2_peak_ui_tests {
    use super::*;
    use crate::c2::hub::LiveConnectionView;
    use crate::c2::query::ConnectionFilter;
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
        let _ = query_connections(&hub.rows(), &ConnectionQuery::default());
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
