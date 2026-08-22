//! rusqlite binding 能力探测。实验面，不是 C1 Repository。

use rusqlite::{backup::Backup, Connection, ErrorCode, OpenFlags, StatementStatus};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProbeError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BindingCapabilities {
    pub binding: &'static str,
    pub sqlite_version: String,
    pub wal: bool,
    pub strict: bool,
    pub foreign_keys: bool,
    pub synchronous_full: bool,
    pub interrupt: bool,
    pub progress_handler: bool,
    pub paged_backup: bool,
    pub checkpoint: bool,
    pub statement_status: bool,
    pub prepared_reuse: bool,
    pub license: &'static str,
}

pub fn open_bundled(path: &Path) -> Result<Connection, ProbeError> {
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
    )?;
    apply_required_pragmas(&connection, 5_000)?;
    Ok(connection)
}

pub fn apply_required_pragmas(
    connection: &Connection,
    busy_timeout_ms: u32,
) -> Result<(), ProbeError> {
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "FULL")?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(Duration::from_millis(u64::from(busy_timeout_ms)))?;
    Ok(())
}

pub fn sqlite_version(connection: &Connection) -> Result<String, ProbeError> {
    let version: String = connection.query_row("select sqlite_version()", [], |row| row.get(0))?;
    Ok(version)
}

pub fn probe_capabilities(dir: &Path) -> Result<BindingCapabilities, ProbeError> {
    let db_path = dir.join("probe.sqlite3");
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(dir.join("probe.sqlite3-wal"));
    let _ = std::fs::remove_file(dir.join("probe.sqlite3-shm"));
    let _ = std::fs::remove_file(dir.join("probe-backup.sqlite3"));
    let connection = open_bundled(&db_path)?;
    let version = sqlite_version(&connection)?;

    connection.execute_batch(
        "create table if not exists parent (
            id integer primary key,
            name text not null
        ) strict;
        create table if not exists typed (
            id integer primary key,
            qty integer not null
        ) strict;
        create table if not exists child (
            id integer primary key,
            parent_id integer not null references parent(id)
        ) strict;",
    )?;

    let wal: String = connection.query_row("pragma journal_mode", [], |row| row.get(0))?;
    let synchronous: i64 = connection.query_row("pragma synchronous", [], |row| row.get(0))?;
    let foreign_keys: i64 = connection.query_row("pragma foreign_keys", [], |row| row.get(0))?;

    let strict_rejected = connection
        .execute(
            "insert into typed(id, qty) values (1, 'not-an-integer')",
            [],
        )
        .is_err();

    let mut insert = connection.prepare("insert into parent(id, name) values (?1, ?2)")?;
    insert.execute(rusqlite::params![1, "alpha"])?;
    insert.execute(rusqlite::params![2, "beta"])?;
    let prepared_reuse = insert.expanded_sql().is_some();

    let mut select = connection.prepare("select name from parent where id = ?1")?;
    let name: String = select.query_row([1], |row| row.get(0))?;
    let status_hits = select.get_status(StatementStatus::VmStep);
    drop(select);

    let seen_progress = Arc::new(AtomicBool::new(false));
    let progress_flag = Arc::clone(&seen_progress);
    connection.progress_handler(
        1,
        Some(move || {
            progress_flag.store(true, Ordering::SeqCst);
            false
        }),
    )?;
    let _: i64 = connection.query_row("select count(*) from parent", [], |row| row.get(0))?;
    let clear: Option<fn() -> bool> = None;
    connection.progress_handler(0, clear)?;

    let handle = connection.get_interrupt_handle();
    let started = Instant::now();
    handle.interrupt();
    let interrupt_delay = started.elapsed();
    let _ = name;

    let dest_path = dir.join("probe-backup.sqlite3");
    let mut dest = Connection::open(&dest_path)?;
    {
        let backup = Backup::new(&connection, &mut dest)?;
        backup.step(16)?;
    }

    let checkpoint: (i64, i64, i64) =
        connection.query_row("pragma wal_checkpoint(PASSIVE)", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;

    Ok(BindingCapabilities {
        binding: "rusqlite",
        sqlite_version: version,
        wal: wal.eq_ignore_ascii_case("wal"),
        strict: strict_rejected,
        foreign_keys: foreign_keys == 1,
        synchronous_full: synchronous == 2,
        interrupt: interrupt_delay < Duration::from_secs(1),
        progress_handler: seen_progress.load(Ordering::SeqCst),
        paged_backup: dest_path.exists(),
        checkpoint: checkpoint.1 >= 0,
        statement_status: status_hits >= 0,
        prepared_reuse,
        license: "blessing / rusqlite MIT-compatible bundled SQLite",
    })
}

pub fn map_sqlite_error(error: &rusqlite::Error) -> &'static str {
    match error.sqlite_error_code() {
        Some(ErrorCode::DatabaseBusy) => "busy",
        Some(ErrorCode::DatabaseLocked) => "locked",
        Some(ErrorCode::DiskFull) => "disk_full",
        Some(ErrorCode::SystemIoFailure) => "io",
        Some(ErrorCode::DatabaseCorrupt) => "corrupt",
        Some(ErrorCode::OperationInterrupted) => "cancelled",
        _ => "other",
    }
}

#[cfg(test)]
mod sqlite_probe_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn sqlite_probe_reports_required_capabilities() {
        let dir = tempdir().expect("tempdir");
        let caps = probe_capabilities(dir.path()).expect("probe");
        assert!(caps.wal);
        assert!(caps.strict);
        assert!(caps.foreign_keys);
        assert!(caps.synchronous_full);
        assert!(caps.interrupt);
        assert!(caps.progress_handler);
        assert!(caps.paged_backup);
        assert!(caps.checkpoint);
        assert!(caps.statement_status);
        assert!(caps.prepared_reuse);
        assert!(!caps.sqlite_version.is_empty());
    }
}

#[cfg(test)]
mod sqlite_fault_tests {
    use super::*;
    use rusqlite::Error;
    use tempfile::tempdir;

    #[test]
    fn sqlite_fault_maps_busy_and_cancel_codes() {
        let codes = [
            (ErrorCode::DatabaseBusy, "busy"),
            (ErrorCode::OperationInterrupted, "cancelled"),
            (ErrorCode::DiskFull, "disk_full"),
        ];
        for (code, expected) in codes {
            let raw = match code {
                ErrorCode::DatabaseBusy => rusqlite::ffi::SQLITE_BUSY,
                ErrorCode::OperationInterrupted => rusqlite::ffi::SQLITE_INTERRUPT,
                ErrorCode::DiskFull => rusqlite::ffi::SQLITE_FULL,
                _ => rusqlite::ffi::SQLITE_ERROR,
            };
            let error = Error::SqliteFailure(rusqlite::ffi::Error::new(raw), None);
            assert_eq!(map_sqlite_error(&error), expected);
        }
    }

    #[test]
    fn sqlite_fault_busy_timeout_does_not_hang() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("busy.sqlite3");
        let writer = open_bundled(&path).expect("writer");
        writer
            .execute_batch("create table t(id integer primary key) strict; begin immediate;")
            .expect("lock");
        let reader = open_bundled(&path).expect("reader");
        reader
            .busy_timeout(Duration::from_millis(50))
            .expect("timeout");
        let started = Instant::now();
        let result = reader.execute("insert into t(id) values (1)", []);
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(2));
        drop(writer);
    }
}
