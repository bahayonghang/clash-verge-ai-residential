//! 用户主动 VACUUM。低空间 fail closed。不自动执行。

use crate::c3::query::ReportError;
use crate::c3::space::SpaceBudget;
use crate::sqlite_probe::open_bundled;
use rusqlite::Connection;
use std::path::Path;

pub fn run_user_vacuum(path: &Path, space: &SpaceBudget) -> Result<(), ReportError> {
    if !path.exists() {
        return Err(ReportError::Failed("database missing"));
    }
    let bytes = std::fs::metadata(path).map(|item| item.len()).unwrap_or(0);
    let needed = bytes.saturating_mul(2).saturating_add(1024 * 1024);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    space.check(parent, needed)?;
    let connection = open_bundled(path).map_err(|_| ReportError::Failed("open vacuum"))?;
    connection
        .execute_batch("vacuum")
        .map_err(|_| ReportError::Failed("vacuum"))?;
    drop(connection);
    if !integrity_ok(path) {
        return Err(ReportError::Failed("vacuum integrity"));
    }
    Ok(())
}

fn integrity_ok(path: &Path) -> bool {
    let Ok(connection) = Connection::open(path) else {
        return false;
    };
    connection
        .query_row("pragma integrity_check", [], |row| row.get::<_, String>(0))
        .ok()
        .as_deref()
        == Some("ok")
}

#[cfg(test)]
mod vacuum_tests {
    use super::*;
    use crate::storage::StorageCoordinator;
    use tempfile::tempdir;

    #[test]
    fn exhausted_space_does_not_start_vacuum() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("monitor.sqlite3");
        StorageCoordinator::open(&path).expect("open");
        let before = std::fs::read(&path).expect("read");
        let error = run_user_vacuum(&path, &SpaceBudget::exhausted()).expect_err("space");
        assert_eq!(error.code(), "insufficient_space");
        let after = std::fs::read(&path).expect("after");
        assert_eq!(before, after);
    }

    #[test]
    fn vacuum_keeps_current_database() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("monitor.sqlite3");
        StorageCoordinator::open(&path).expect("open");
        run_user_vacuum(&path, &SpaceBudget::unlimited()).expect("vacuum");
        assert!(path.exists());
        assert!(integrity_ok(&path));
        StorageCoordinator::open(&path).expect("reopen");
    }
}
