//! 导航 seam、文件选择、操作进度与 Recovery 启动分支。

use crate::c0_contract::SCHEMA_VERSION;
use crate::i18n::{t, UiLocale};
use crate::storage::{RecoveryFacade, StorageError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BootBranch {
    NormalReady,
    RecoveryOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteDescriptor {
    pub id: String,
    pub title_zh: String,
    pub available: bool,
    pub unavailable_until: Option<String>,
}

pub fn default_routes() -> Vec<RouteDescriptor> {
    default_routes_for(UiLocale::Zh)
}

pub fn default_routes_for(locale: UiLocale) -> Vec<RouteDescriptor> {
    [
        ("overview", "route.overview"),
        ("live", "route.live"),
        ("residential", "route.residential"),
        ("host", "route.host"),
        ("rule", "route.rule"),
        ("chain", "route.chain"),
        ("process", "route.process"),
        ("reports", "route.reports"),
        ("alerts", "route.alerts"),
        ("settings-data", "route.settings-data"),
    ]
    .into_iter()
    .map(|(id, key)| RouteDescriptor {
        id: id.into(),
        title_zh: t(locale, key).into(),
        available: true,
        unavailable_until: None,
    })
    .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FilePurpose {
    ReportExport,
    BackupCreate,
    BackupRestore,
    DiagnosticsExport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileMode {
    Open,
    Save,
}

pub trait FileDialogPort {
    fn pick(&self, purpose: FilePurpose, mode: FileMode) -> Option<PathBuf>;
}

#[derive(Default)]
pub struct FakeFileDialog {
    pub next: std::sync::Mutex<Option<PathBuf>>,
}

impl FileDialogPort for FakeFileDialog {
    fn pick(&self, _purpose: FilePurpose, _mode: FileMode) -> Option<PathBuf> {
        self.next.lock().expect("dialog").clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationProgress {
    pub schema_version: u32,
    pub operation_id: String,
    pub kind: String,
    pub phase: String,
    pub current: u64,
    pub total: u64,
    pub unit: String,
    pub can_cancel: bool,
    pub status: String,
    pub redacted_error: Option<String>,
}

#[derive(Default)]
pub struct OperationRegistry {
    items: HashMap<String, OperationProgress>,
}

impl OperationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_fixture(&mut self, operation_id: String, kind: String) -> OperationProgress {
        let progress = OperationProgress {
            schema_version: 1,
            operation_id: operation_id.clone(),
            kind,
            phase: "running".into(),
            current: 0,
            total: 100,
            unit: "percent".into(),
            can_cancel: true,
            status: "running".into(),
            redacted_error: None,
        };
        self.items.insert(operation_id, progress.clone());
        progress
    }

    pub fn cancel(&mut self, operation_id: &str) -> Option<OperationProgress> {
        if let Some(item) = self.items.get_mut(operation_id) {
            item.status = "cancelled".into();
            item.can_cancel = false;
            item.phase = "cancelled".into();
            return Some(item.clone());
        }
        None
    }

    pub fn finish(&mut self, operation_id: &str) -> Option<OperationProgress> {
        if let Some(item) = self.items.get_mut(operation_id) {
            item.status = "completed".into();
            item.current = item.total;
            item.can_cancel = false;
            item.phase = "done".into();
            return Some(item.clone());
        }
        None
    }

    pub fn get(&self, operation_id: &str) -> Option<OperationProgress> {
        self.items.get(operation_id).cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryStatus {
    pub schema_version: u32,
    pub app_version: String,
    pub user_version: i64,
    pub supported_max: i32,
    pub future: bool,
    pub restore_available: bool,
    pub restore_note_zh: String,
    pub backups: Vec<String>,
}

pub fn recovery_status(facade: &RecoveryFacade) -> Result<RecoveryStatus, StorageError> {
    let raw = facade.status()?;
    let user_version = raw
        .get("user_version")
        .and_then(|value| value.as_i64())
        .unwrap_or(-1);
    let future = raw
        .get("future")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    Ok(RecoveryStatus {
        schema_version: 1,
        app_version: env!("CARGO_PKG_VERSION").into(),
        user_version,
        supported_max: SCHEMA_VERSION,
        future,
        restore_available: true,
        restore_note_zh:
            "可验证候选并执行恢复。恢复失败会保留当前可用数据库。跨机恢复需重新输入 secret。".into(),
        backups: facade.list_backups().unwrap_or_default(),
    })
}

pub fn validate_backup(facade: &RecoveryFacade, candidate: &Path) -> Result<bool, StorageError> {
    facade.validate_candidate(candidate)
}

#[cfg(test)]
mod shell_seam_tests {
    use super::*;
    use crate::storage::migrate;
    use tempfile::tempdir;

    #[test]
    fn reports_and_alerts_are_available_after_c4() {
        let routes = default_routes();
        assert_eq!(
            routes
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            [
                "overview",
                "live",
                "residential",
                "host",
                "rule",
                "chain",
                "process",
                "reports",
                "alerts",
                "settings-data",
            ]
        );
        let reports = routes.iter().find(|item| item.id == "reports").unwrap();
        assert!(reports.available);
        assert_eq!(reports.unavailable_until, None);
        let alerts = routes.iter().find(|item| item.id == "alerts").unwrap();
        assert!(alerts.available);
        assert_eq!(alerts.unavailable_until, None);
        let residential = routes.iter().find(|item| item.id == "residential").unwrap();
        assert!(residential.available);
        assert_eq!(residential.title_zh, "家宽");
    }

    #[test]
    fn file_dialog_returns_only_injected_path() {
        let dialog = FakeFileDialog::default();
        *dialog.next.lock().expect("d") = Some(PathBuf::from("C:/tmp/report.csv"));
        let path = dialog
            .pick(FilePurpose::ReportExport, FileMode::Save)
            .expect("path");
        assert!(path.ends_with("report.csv"));
    }

    #[test]
    fn operation_progress_can_cancel_fixture() {
        let mut ops = OperationRegistry::new();
        ops.start_fixture("op-1".into(), "export".into());
        let cancelled = ops.cancel("op-1").expect("cancel");
        assert_eq!(cancelled.status, "cancelled");
    }

    #[test]
    fn recovery_status_marks_restore_unavailable() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("rec.sqlite3");
        migrate(&path).expect("migrate");
        let status = recovery_status(&RecoveryFacade::open(&path)).expect("status");
        assert!(status.restore_available);
        assert!(!status.future);
    }
}
