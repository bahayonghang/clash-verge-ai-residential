//! 用户 Online Backup 与安全 restore。不复制热库文件。

use crate::c3::query::ReportError;
use crate::c3::schema::C3_SCHEMA_VERSION;
use crate::c3::space::SpaceBudget;
use crate::storage::{backup_pages, migrate, RecoveryFacade};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub schema_version: i32,
    pub user_version: i32,
    pub checksum: String,
    pub created_utc: i64,
    pub bytes: u64,
}

pub struct BackupRestoreService;

impl BackupRestoreService {
    pub fn create_backup(
        source: &Path,
        dest: &Path,
        space: &SpaceBudget,
        cancel: &Arc<AtomicBool>,
        created_utc: i64,
    ) -> Result<BackupManifest, ReportError> {
        if dest.exists() {
            return Err(ReportError::Failed("destination exists"));
        }
        let parent = dest.parent().unwrap_or_else(|| Path::new("."));
        let src_len = std::fs::metadata(source)
            .map(|item| item.len())
            .unwrap_or(0);
        space.check(parent, src_len.saturating_add(1024 * 1024))?;
        let partial = parent.join(format!(
            "{}.partial",
            dest.file_name()
                .and_then(|item| item.to_str())
                .unwrap_or("backup")
        ));
        let _ = std::fs::remove_file(&partial);
        if let Err(error) = backup_pages(source, &partial, cancel) {
            let _ = std::fs::remove_file(&partial);
            return Err(error);
        }
        let bytes = std::fs::metadata(&partial)
            .map(|item| item.len())
            .unwrap_or(0);
        let checksum = file_checksum(&partial)?;
        let user_version = read_user_version(&partial)?;
        if !integrity_ok(&partial) {
            let _ = std::fs::remove_file(&partial);
            return Err(ReportError::Failed("integrity"));
        }
        let manifest = BackupManifest {
            schema_version: C3_SCHEMA_VERSION,
            user_version,
            checksum: checksum.clone(),
            created_utc,
            bytes,
        };
        let manifest_path = dest.with_extension("manifest.json");
        std::fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).map_err(|_| ReportError::Failed("manifest"))?,
        )
        .map_err(|_| ReportError::Failed("manifest write"))?;
        std::fs::rename(&partial, dest).map_err(|_| ReportError::Failed("backup rename"))?;
        Ok(manifest)
    }

    pub fn restore(
        live: &Path,
        candidate: &Path,
        space: &SpaceBudget,
        cancel: &Arc<AtomicBool>,
    ) -> Result<(), ReportError> {
        if cancel.load(Ordering::SeqCst) {
            return Err(ReportError::Cancelled("restore"));
        }
        let parent = live.parent().unwrap_or_else(|| Path::new("."));
        let needed = std::fs::metadata(candidate)
            .map(|item| item.len().saturating_mul(3))
            .unwrap_or(0);
        space.check(parent, needed)?;
        if !Self::validate_candidate(candidate)? {
            return Err(ReportError::Failed("candidate invalid"));
        }
        let protect = parent.join("monitor.protect.sqlite3");
        if live.exists() {
            let cancel_backup = Arc::new(AtomicBool::new(false));
            backup_pages(live, &protect, &cancel_backup)?;
            if !integrity_ok(&protect) {
                let _ = std::fs::remove_file(&protect);
                return Err(ReportError::Failed("protect backup invalid"));
            }
        }
        let staged = parent.join("monitor.restore.partial");
        let _ = std::fs::remove_file(&staged);
        std::fs::copy(candidate, &staged).map_err(|_| ReportError::Failed("stage candidate"))?;
        let previous = parent.join("monitor.pre-restore.sqlite3");
        if live.exists() {
            let _ = std::fs::remove_file(&previous);
            std::fs::rename(live, &previous).map_err(|_| ReportError::Failed("park live"))?;
            rename_sidecar(live, &previous);
        }
        if let Err(error) = finish_swap(live, &staged) {
            let _ = std::fs::remove_file(live);
            if previous.exists() {
                let _ = std::fs::rename(&previous, live);
                rename_sidecar(&previous, live);
            }
            return Err(error);
        }
        Ok(())
    }

    pub fn validate_candidate(candidate: &Path) -> Result<bool, ReportError> {
        if !candidate.exists() {
            return Ok(false);
        }
        if !integrity_ok(candidate) {
            return Ok(false);
        }
        let user_version = read_user_version(candidate)?;
        if user_version > C3_SCHEMA_VERSION {
            return Ok(false);
        }
        let manifest = candidate.with_extension("manifest.json");
        if manifest.exists() {
            let raw = std::fs::read(&manifest).map_err(|_| ReportError::Failed("read manifest"))?;
            let parsed: BackupManifest =
                serde_json::from_slice(&raw).map_err(|_| ReportError::Failed("bad manifest"))?;
            let actual = file_checksum(candidate)?;
            if parsed.checksum != actual {
                return Ok(false);
            }
        }
        let facade = RecoveryFacade::open(candidate);
        facade
            .validate_candidate(candidate)
            .map_err(|_| ReportError::Failed("recovery validate"))
    }
}

fn finish_swap(live: &Path, staged: &Path) -> Result<(), ReportError> {
    std::fs::rename(staged, live).map_err(|_| ReportError::Failed("swap"))?;
    migrate(live).map_err(|_| ReportError::Failed("forward migrate"))?;
    if !integrity_ok(live) {
        return Err(ReportError::Failed("smoke integrity"));
    }
    let connection = Connection::open(live).map_err(|_| ReportError::Failed("smoke open"))?;
    let _: i64 = connection
        .query_row("select count(*) from data_version", [], |row| row.get(0))
        .map_err(|_| ReportError::Failed("smoke query"))?;
    Ok(())
}

fn rename_sidecar(from: &Path, to: &Path) {
    for suffix in ["-wal", "-shm"] {
        let src = sidecar(from, suffix);
        let dest = sidecar(to, suffix);
        if src.exists() {
            let _ = std::fs::rename(src, dest);
        }
    }
}

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

fn integrity_ok(path: &Path) -> bool {
    Connection::open(path)
        .and_then(|connection| {
            connection.query_row("pragma integrity_check", [], |row| {
                let text: String = row.get(0)?;
                Ok(text == "ok")
            })
        })
        .unwrap_or(false)
}

fn read_user_version(path: &Path) -> Result<i32, ReportError> {
    let connection = Connection::open(path).map_err(|_| ReportError::Failed("open version"))?;
    connection
        .query_row("pragma user_version", [], |row| row.get(0))
        .map_err(|_| ReportError::Failed("user_version"))
}

fn file_checksum(path: &Path) -> Result<String, ReportError> {
    let bytes = std::fs::read(path).map_err(|_| ReportError::Failed("read checksum"))?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

#[cfg(test)]
mod backup_restore_tests {
    use super::*;
    use crate::c3::space::SpaceBudget;
    use crate::storage::StorageCoordinator;
    use tempfile::tempdir;

    #[test]
    fn backup_then_restore_keeps_current_on_bad_candidate() {
        let dir = tempdir().expect("dir");
        let live = dir.path().join("monitor.sqlite3");
        let coordinator = StorageCoordinator::open(&live).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        drop(coordinator);
        let dest = dir.path().join("ok.sqlite3");
        let manifest = BackupRestoreService::create_backup(
            &live,
            &dest,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
            100,
        )
        .expect("backup");
        assert!(!manifest.checksum.is_empty());
        let bad = dir.path().join("bad.sqlite3");
        std::fs::write(&bad, b"not-a-database").expect("bad");
        let error = BackupRestoreService::restore(
            &live,
            &bad,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
        )
        .expect_err("bad");
        assert_eq!(error.code(), "storage_failure");
        assert!(live.exists());
        let coordinator = StorageCoordinator::open(&live).expect("reopen");
        let _ = coordinator.watermark().expect("wm");
    }

    #[test]
    fn low_space_and_cancel_fail_closed() {
        let dir = tempdir().expect("dir");
        let live = dir.path().join("monitor.sqlite3");
        StorageCoordinator::open(&live).expect("open");
        let error = BackupRestoreService::create_backup(
            &live,
            &dir.path().join("x.sqlite3"),
            &SpaceBudget::exhausted(),
            &Arc::new(AtomicBool::new(false)),
            1,
        )
        .expect_err("space");
        assert_eq!(error.code(), "insufficient_space");
        let cancel = Arc::new(AtomicBool::new(true));
        let error = BackupRestoreService::create_backup(
            &live,
            &dir.path().join("y.sqlite3"),
            &SpaceBudget::unlimited(),
            &cancel,
            1,
        )
        .expect_err("cancel");
        assert_eq!(error.code(), "cancelled");
        assert!(!dir.path().join("y.sqlite3").exists());
    }

    #[test]
    fn restore_from_valid_backup_and_rejects_future_schema() {
        let dir = tempdir().expect("dir");
        let live = dir.path().join("monitor.sqlite3");
        StorageCoordinator::open(&live).expect("open");
        let dest = dir.path().join("ok.sqlite3");
        BackupRestoreService::create_backup(
            &live,
            &dest,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
            2,
        )
        .expect("backup");
        std::fs::remove_file(&live).expect("rm");
        BackupRestoreService::restore(
            &live,
            &dest,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
        )
        .expect("restore");
        assert!(live.exists());
        let future = dir.path().join("future.sqlite3");
        {
            let connection = Connection::open(&future).expect("f");
            connection
                .execute_batch("pragma user_version = 99")
                .expect("ver");
        }
        assert!(!BackupRestoreService::validate_candidate(&future).expect("val"));
    }
}
