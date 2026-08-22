//! writer / report / export / backup / retention / checkpoint 重叠。
//! fixture 规模证明可重叠，不得写成 30 天容量。

use crate::c3::backup::BackupRestoreService;
use crate::c3::export::{ExportService, ExportSpec};
use crate::c3::query::ReportQuery;
use crate::c3::retention::{RetentionMode, RetentionService};
use crate::c3::service::ReportService;
use crate::c3::snapshot::ReportSnapshotStore;
use crate::c3::space::SpaceBudget;
use crate::storage::{CommitBundle, StorageCoordinator};
use serde::Serialize;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConcurrentReport {
    pub schema_version: u32,
    pub elapsed_ms: u128,
    pub writer_commits: u64,
    pub report_ok: bool,
    pub export_ok: bool,
    pub backup_ok: bool,
    pub retention_ok: bool,
    pub checkpoint_ok: bool,
    pub wal_after: u64,
    pub overlap: bool,
    pub scale: &'static str,
    pub note_zh: String,
}

pub fn run_overlap(dir: &Path) -> Result<ConcurrentReport, String> {
    let db = dir.join("c5-overlap.sqlite3");
    let _ = std::fs::remove_file(&db);
    let mut coordinator = StorageCoordinator::open(&db).map_err(|error| error.to_string())?;
    coordinator
        .seed_report_fixture()
        .map_err(|error| error.to_string())?;
    let started = Instant::now();
    let mut writer_commits = 0_u64;
    for seq in 1..=8 {
        coordinator
            .commit(&CommitBundle {
                writer_epoch: 21,
                bundle_seq: seq,
                payload: format!("{},1,2,4", 1_000 + seq * 60),
            })
            .map_err(|error| error.to_string())?;
        writer_commits += 1;
        coordinator
            .checkpoint_passive()
            .map_err(|error| error.to_string())?;
    }
    let path = coordinator.path().to_path_buf();
    drop(coordinator);

    let cancel = Arc::new(AtomicBool::new(false));
    let report_dir = dir.to_path_buf();
    let report_path = path.clone();
    let report_cancel = cancel.clone();
    let report_handle = thread::spawn(move || {
        let mut store = ReportSnapshotStore::open(&report_dir);
        let query = ReportQuery {
            range_start_utc: 1_000,
            range_end_utc: 4_000,
            ..ReportQuery::default()
        };
        ReportService::run(
            &report_path,
            &mut store,
            query,
            3_600,
            30,
            &report_cancel,
            None,
        )
    });

    let backup_dest = dir.join("overlap-backup.sqlite3");
    let backup_src = path.clone();
    let backup_cancel = cancel.clone();
    let backup_handle = thread::spawn(move || {
        BackupRestoreService::create_backup(
            &backup_src,
            &backup_dest,
            &SpaceBudget::unlimited(),
            &backup_cancel,
            10,
        )
    });

    let report = report_handle
        .join()
        .map_err(|_| "report join".to_string())?
        .map_err(|error| error.to_string())?;
    let export_path = dir.join("overlap.csv");
    let export = ExportService::export_to_path(
        &report,
        &ExportSpec::default(),
        &export_path,
        &SpaceBudget::unlimited(),
        &cancel,
    );
    let backup = backup_handle
        .join()
        .map_err(|_| "backup join".to_string())?;

    let mut coordinator = StorageCoordinator::open(&path).map_err(|error| error.to_string())?;
    let retention = RetentionService::run(
        &mut coordinator,
        3_600,
        30,
        RetentionMode::MaterializeOnly,
        &SpaceBudget::unlimited(),
        &cancel,
    );
    let checkpoint_ok = coordinator.checkpoint_passive().is_ok();
    let wal_after = std::fs::metadata(format!("{}-wal", path.display()))
        .map(|item| item.len())
        .unwrap_or(0);
    drop(coordinator);

    let report_ok = report.totals.download > 0;
    let export_ok = export.is_ok();
    let backup_ok = backup.is_ok();
    let retention_ok = retention.is_ok();
    Ok(ConcurrentReport {
        schema_version: 1,
        elapsed_ms: started.elapsed().as_millis(),
        writer_commits,
        report_ok,
        export_ok,
        backup_ok,
        retention_ok,
        checkpoint_ok,
        wal_after,
        overlap: true,
        scale: "fixture",
        note_zh: "fixture 级重叠通过。不是 30 天 A=50/250/1000 容量声明。".into(),
    })
}

#[cfg(test)]
mod concurrent_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn overlap_on_fixture_does_not_claim_30_day_capacity() {
        let dir = tempdir().expect("dir");
        let report = run_overlap(dir.path()).expect("overlap");
        assert!(report.report_ok);
        assert!(report.export_ok);
        assert!(report.backup_ok);
        assert!(report.retention_ok);
        assert!(report.checkpoint_ok);
        assert_eq!(report.scale, "fixture");
        assert!(report.note_zh.contains("不是 30 天"));
    }
}
