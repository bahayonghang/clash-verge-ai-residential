//! 导航 seam、文件选择、操作进度与 Recovery 启动分支。

use crate::c0_contract::SCHEMA_VERSION;
use crate::i18n::{t, UiLocale};
use crate::storage::{RecoveryFacade, StorageError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

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

pub trait FileDialogPort: Send + Sync {
    fn pick(&self, locale: UiLocale, purpose: FilePurpose, mode: FileMode) -> Option<PathBuf>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogSpec {
    pub title: String,
    pub file_name: String,
    pub filter_name: String,
    pub extensions: Vec<String>,
}

pub fn dialog_spec(locale: UiLocale, purpose: FilePurpose) -> DialogSpec {
    match purpose {
        FilePurpose::ReportExport => DialogSpec {
            title: t(locale, "dialog.report_export.title").into(),
            file_name: t(locale, "dialog.report_export.file").into(),
            filter_name: t(locale, "dialog.filter.report").into(),
            extensions: vec!["csv".into(), "json".into(), "html".into()],
        },
        FilePurpose::BackupCreate => DialogSpec {
            title: t(locale, "dialog.backup_create.title").into(),
            file_name: t(locale, "dialog.backup_create.file").into(),
            filter_name: t(locale, "dialog.filter.backup").into(),
            extensions: vec!["sqlite3".into()],
        },
        FilePurpose::BackupRestore => DialogSpec {
            title: t(locale, "dialog.backup_restore.title").into(),
            file_name: t(locale, "dialog.backup_restore.file").into(),
            filter_name: t(locale, "dialog.filter.backup").into(),
            extensions: vec!["sqlite3".into()],
        },
        FilePurpose::DiagnosticsExport => DialogSpec {
            title: t(locale, "dialog.diagnostics_export.title").into(),
            file_name: t(locale, "dialog.diagnostics_export.file").into(),
            filter_name: t(locale, "dialog.filter.json").into(),
            extensions: vec!["json".into()],
        },
    }
}

#[derive(Default)]
pub struct FakeFileDialog {
    pub next: std::sync::Mutex<Option<PathBuf>>,
}

impl FileDialogPort for FakeFileDialog {
    fn pick(&self, _locale: UiLocale, _purpose: FilePurpose, _mode: FileMode) -> Option<PathBuf> {
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

struct OperationEntry {
    progress: OperationProgress,
    cancel: Arc<AtomicBool>,
}

#[derive(Clone, Default)]
pub struct OperationRegistry {
    items: Arc<Mutex<HashMap<String, OperationEntry>>>,
    archive_tick_cancel: Arc<AtomicBool>,
}

impl OperationRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn entries(&self) -> MutexGuard<'_, HashMap<String, OperationEntry>> {
        // 只在短临界区修改进度；即使调用线程 panic，仍保留已有取消标志供其它 owner 中断。
        self.items
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    // 调用方为每次操作生成唯一 ID，并在实际命令结束后调用 finish。
    pub fn start_fixture(&self, operation_id: String, kind: String) -> OperationProgress {
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
        self.entries().insert(
            operation_id,
            OperationEntry {
                progress: progress.clone(),
                cancel: Arc::new(AtomicBool::new(false)),
            },
        );
        progress
    }

    pub fn cancel_flag(&self, operation_id: &str) -> Option<Arc<AtomicBool>> {
        self.entries()
            .get(operation_id)
            .map(|entry| Arc::clone(&entry.cancel))
    }

    pub fn archive_tick_cancel(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.archive_tick_cancel)
    }

    pub fn resolve_cancel(&self, operation_id: Option<&str>, _kind: &str) -> Arc<AtomicBool> {
        operation_id
            .and_then(|id| self.cancel_flag(id))
            .unwrap_or_else(|| Arc::new(AtomicBool::new(false)))
    }

    pub fn cancel(&self, operation_id: &str) -> Option<OperationProgress> {
        if let Some(entry) = self.entries().get_mut(operation_id) {
            entry.cancel.store(true, Ordering::SeqCst);
            entry.progress.status = "cancelled".into();
            entry.progress.can_cancel = false;
            entry.progress.phase = "cancelled".into();
            return Some(entry.progress.clone());
        }
        None
    }

    pub fn finish(&self, operation_id: &str) -> Option<OperationProgress> {
        let mut entry = self.entries().remove(operation_id)?;
        if entry.progress.status != "cancelled" {
            entry.progress.status = "completed".into();
            entry.progress.current = entry.progress.total;
            entry.progress.can_cancel = false;
            entry.progress.phase = "done".into();
        }
        Some(entry.progress)
    }

    pub fn get(&self, operation_id: &str) -> Option<OperationProgress> {
        self.entries()
            .get(operation_id)
            .map(|entry| entry.progress.clone())
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
            .pick(UiLocale::Zh, FilePurpose::ReportExport, FileMode::Save)
            .expect("path");
        assert!(path.ends_with("report.csv"));
    }

    #[test]
    fn dialog_spec_covers_all_purposes_in_both_locales() {
        for purpose in [
            FilePurpose::ReportExport,
            FilePurpose::BackupCreate,
            FilePurpose::BackupRestore,
            FilePurpose::DiagnosticsExport,
        ] {
            let zh = dialog_spec(UiLocale::Zh, purpose);
            let en = dialog_spec(UiLocale::En, purpose);
            for (name, spec) in [("zh", &zh), ("en", &en)] {
                assert!(!spec.title.is_empty(), "{name} title empty for {purpose:?}");
                assert!(
                    !spec.file_name.is_empty(),
                    "{name} file name empty for {purpose:?}"
                );
                assert!(!spec.filter_name.is_empty());
                assert!(!spec.extensions.is_empty());
            }
            // 英文译文必须可达，否则对话框标题会固定为中文。
            assert_ne!(zh.title, en.title, "{purpose:?} has no English title");
        }
    }

    #[test]
    fn export_purposes_filter_matches_file_name() {
        for purpose in [FilePurpose::ReportExport, FilePurpose::DiagnosticsExport] {
            let spec = dialog_spec(UiLocale::Zh, purpose);
            let extension = Path::new(&spec.file_name)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or_default();
            assert!(
                spec.extensions.iter().any(|item| item == extension),
                "{purpose:?}: default file name {extension:?} not in filter {:?}",
                spec.extensions
            );
        }
    }

    #[test]
    fn operation_progress_can_cancel_fixture() {
        let ops = OperationRegistry::new();
        ops.start_fixture("op-1".into(), "export".into());
        let flag = ops.cancel_flag("op-1").expect("flag");
        assert!(!flag.load(std::sync::atomic::Ordering::SeqCst));
        let cancelled = ops.cancel("op-1").expect("cancel");
        assert_eq!(cancelled.status, "cancelled");
        assert!(flag.load(std::sync::atomic::Ordering::SeqCst));
        assert!(!ops
            .archive_tick_cancel()
            .load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn operation_finish_releases_entry_and_preserves_borrowed_flag() {
        let ops = OperationRegistry::new();
        ops.start_fixture("report-1".into(), "report".into());
        let flag = ops.cancel_flag("report-1").expect("flag");
        assert_eq!(Arc::strong_count(&flag), 2);

        let completed = ops.finish("report-1").expect("finish");
        assert_eq!(completed.status, "completed");
        assert_eq!(completed.current, completed.total);
        assert!(!completed.can_cancel);
        assert!(ops.get("report-1").is_none());
        assert!(ops.cancel_flag("report-1").is_none());
        assert!(ops.finish("report-1").is_none());
        assert_eq!(Arc::strong_count(&flag), 1);
        assert!(!flag.load(Ordering::SeqCst));
    }

    #[test]
    fn operation_cancellation_never_leaks_to_other_queries() {
        let ops = OperationRegistry::new();
        ops.start_fixture("report-1".into(), "report".into());
        ops.start_fixture("report-2".into(), "report".into());
        let first = ops.resolve_cancel(Some("report-1"), "report");
        let second = ops.resolve_cancel(Some("report-2"), "report");
        let unspecified = ops.resolve_cancel(None, "report");
        let missing = ops.resolve_cancel(Some("missing"), "report");

        ops.cancel("report-1").expect("cancel first");
        assert!(first.load(Ordering::SeqCst));
        assert!(!second.load(Ordering::SeqCst));
        assert!(!unspecified.load(Ordering::SeqCst));
        assert!(!missing.load(Ordering::SeqCst));
        let cancelled = ops.finish("report-1").expect("finish cancelled");
        assert_eq!(cancelled.status, "cancelled");
        assert!(first.load(Ordering::SeqCst));
        assert_eq!(Arc::strong_count(&first), 1);

        assert!(ops.cancel("report-1").is_none());
        ops.cancel("report-2").expect("cancel second");
        assert!(second.load(Ordering::SeqCst));
        assert!(!unspecified.load(Ordering::SeqCst));
        assert!(!missing.load(Ordering::SeqCst));
        assert!(!ops.archive_tick_cancel().load(Ordering::SeqCst));
    }

    #[test]
    fn operation_repeated_lifecycle_keeps_only_in_flight_entries() {
        let ops = OperationRegistry::new();
        ops.start_fixture("long-report".into(), "report".into());
        let long_running = ops.cancel_flag("long-report").expect("long report");
        for sequence in 0..3_600 {
            let id = format!("report-{sequence}");
            ops.start_fixture(id.clone(), "report".into());
            let flag = ops.cancel_flag(&id).expect("flag");
            assert_eq!(ops.entries().len(), 2);
            if sequence % 2 == 0 {
                ops.cancel(&id).expect("cancel");
            }
            ops.finish(&id).expect("finish");
            assert_eq!(ops.entries().len(), 1);
            assert_eq!(Arc::strong_count(&flag), 1);
            assert!(!long_running.load(Ordering::SeqCst));
        }
        ops.finish("long-report").expect("finish long report");
        assert!(ops.entries().is_empty());
    }

    #[test]
    fn operation_clone_cancels_while_facade_owner_is_locked() {
        let ops = OperationRegistry::new();
        ops.start_fixture("retention".into(), "retention".into());
        let endpoint = ops.clone();
        let facade = Mutex::new(ops);
        let writer_owner = facade.lock().expect("writer owner");
        let running = writer_owner.resolve_cancel(Some("retention"), "retention");
        let (sender, receiver) = std::sync::mpsc::channel();
        let cancel_thread = std::thread::spawn(move || {
            sender
                .send(endpoint.cancel("retention"))
                .expect("cancel response");
        });

        let response = receiver
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("cancel must not wait for facade writer")
            .expect("registered operation");
        assert_eq!(response.status, "cancelled");
        assert!(running.load(Ordering::SeqCst));
        assert_eq!(
            writer_owner.get("retention").expect("progress").status,
            "cancelled"
        );
        writer_owner.finish("retention").expect("finish");
        assert!(writer_owner.entries().is_empty());
        cancel_thread.join().expect("cancel thread");
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
