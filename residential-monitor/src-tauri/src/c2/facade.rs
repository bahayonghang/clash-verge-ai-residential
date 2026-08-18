//! C2 AppFacade：只经 C1 接口访问采集、存储、投影与恢复。

use crate::accounting::AccountingEngine;
use crate::c0_contract::{core_table_allowlist, forbidden_table_fragments};
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
use crate::controller::{ControllerInput, SessionStatus};
use crate::credential::FakeCredentialStore;
use crate::session::ControllerSession;
use crate::storage::{RecoveryFacade, StorageCoordinator};
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
                    data_dir,
                    session_status: SessionStatus::Connecting,
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
                data_dir,
                session_status: SessionStatus::EndpointMissing,
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
            let _ = self.hub.publish(&batch, live, health, utc);
        } else {
            self.apply_lifecycle(input);
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
    for name in names {
        assert!(
            core_table_allowlist().contains(&name.as_str()),
            "{name} 不在 C1 allowlist"
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
        assert!(!status.restore_available);
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
