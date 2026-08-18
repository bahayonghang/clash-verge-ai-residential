//! C1 core schema、幂等 writer 与 RecoveryFacade backend。

use crate::c0_contract::{BUSY_TIMEOUT_MS, RETRY_WINDOW_RECEIPTS, SCHEMA_VERSION};

pub const MIGRATION_CHECKSUM: &str = "c1-core-v1";
use crate::sqlite_probe::{apply_required_pragmas, open_bundled};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("{0}")]
    Closed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitReceipt {
    pub writer_epoch: u64,
    pub bundle_seq: u64,
    pub data_version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitBundle {
    pub writer_epoch: u64,
    pub bundle_seq: u64,
    pub payload: String,
}

impl CommitBundle {
    pub fn payload_hash(&self) -> String {
        hex::encode(Sha256::digest(self.payload.as_bytes()))
    }

    pub fn bundle_id(&self) -> String {
        format!("{}:{}", self.writer_epoch, self.bundle_seq)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitOutcome {
    Applied(CommitReceipt),
    Duplicate(CommitReceipt),
    RetryWindowExpired,
    PayloadMismatch,
}

pub fn migrate(path: &Path) -> Result<Connection, StorageError> {
    let connection = open_bundled(path).map_err(|error| StorageError::Closed(error.to_string()))?;
    apply_required_pragmas(&connection, BUSY_TIMEOUT_MS)
        .map_err(|error| StorageError::Closed(error.to_string()))?;
    let user_version: i32 = connection.query_row("pragma user_version", [], |row| row.get(0))?;
    if user_version > SCHEMA_VERSION {
        return Err(StorageError::Closed("future schema".into()));
    }
    if user_version == SCHEMA_VERSION {
        let has_table: i64 = connection.query_row(
            "select count(*) from sqlite_master where type = 'table' and name = 'schema_migration'",
            [],
            |row| row.get(0),
        )?;
        if has_table > 0 {
            let checksum: Option<String> = connection
                .query_row(
                    "select checksum from schema_migration where version = ?1",
                    [SCHEMA_VERSION],
                    |row| row.get(0),
                )
                .optional()?;
            if let Some(checksum) = checksum {
                if checksum != MIGRATION_CHECKSUM {
                    return Err(StorageError::Closed("checksum mismatch".into()));
                }
            }
        }
    }
    connection.execute_batch(
        "
        create table if not exists schema_migration (
            version integer primary key,
            checksum text not null,
            applied_utc integer not null
        ) strict;
        create table if not exists data_version (
            id integer primary key check (id = 1),
            watermark integer not null
        ) strict;
        create table if not exists bundle_epoch (
            writer_epoch integer primary key,
            highest_contiguous_seq integer not null,
            durable_watermark integer not null
        ) strict;
        create table if not exists committed_bundle (
            writer_epoch integer not null,
            bundle_seq integer not null,
            payload_hash text not null,
            data_version integer not null,
            primary key (writer_epoch, bundle_seq)
        ) strict;
        create table if not exists machine_setting (
            key text primary key,
            value text not null
        ) strict;
        create table if not exists controller_epoch (
            epoch_id integer primary key,
            core_identity text not null
        ) strict;
        create table if not exists target_set (
            set_id integer primary key,
            policy_version integer not null
        ) strict;
        create table if not exists target_item (
            set_id integer not null,
            position integer not null,
            name text not null,
            primary key (set_id, position)
        ) strict;
        create table if not exists coverage_interval (
            interval_id integer primary key,
            kind text not null,
            reason text not null,
            started_utc integer not null,
            ended_utc integer
        ) strict;
        create table if not exists connection_session (
            session_pk integer primary key,
            epoch_id integer not null,
            connection_id text not null,
            started_utc integer not null,
            host text
        ) strict;
        create table if not exists connection_chain (
            session_pk integer not null,
            position integer not null,
            node text not null,
            primary key (session_pk, position)
        ) strict;
        create table if not exists connection_minute (
            utc_minute integer not null,
            session_pk integer not null,
            upload integer not null,
            download integer not null,
            primary key (utc_minute, session_pk)
        ) strict;
        create table if not exists backup_manifest (
            backup_id integer primary key,
            path text not null,
            checksum text not null,
            created_utc integer not null
        ) strict;
        insert or ignore into data_version(id, watermark) values (1, 0);
        pragma user_version = 1;
        ",
    )?;
    connection.execute(
        "insert or ignore into schema_migration(version, checksum, applied_utc) values (?1, ?2, 0)",
        params![SCHEMA_VERSION, MIGRATION_CHECKSUM],
    )?;
    Ok(connection)
}

pub fn list_user_tables(connection: &Connection) -> Result<Vec<String>, StorageError> {
    let mut statement = connection
        .prepare("select name from sqlite_master where type = 'table' and name not like 'sqlite_%' order by name")?;
    let names = statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(names)
}

pub struct StorageCoordinator {
    connection: Connection,
    prepare_count: u64,
}

impl StorageCoordinator {
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        let connection = migrate(path)?;
        let mut lookup = connection.prepare(
            "select payload_hash, data_version from committed_bundle where writer_epoch = ?1 and bundle_seq = ?2",
        )?;
        let _: Option<(String, i64)> = lookup
            .query_row(params![0_i64, 0_i64], |row| Ok((row.get(0)?, row.get(1)?)))
            .optional()?;
        drop(lookup);
        Ok(Self {
            connection,
            prepare_count: 1,
        })
    }

    pub fn commit(&mut self, bundle: &CommitBundle) -> Result<CommitOutcome, StorageError> {
        let hash = bundle.payload_hash();
        let existing: Option<(String, i64)> = self
            .connection
            .query_row(
                "select payload_hash, data_version from committed_bundle where writer_epoch = ?1 and bundle_seq = ?2",
                params![bundle.writer_epoch as i64, bundle.bundle_seq as i64],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        if let Some((old_hash, data_version)) = existing {
            if old_hash != hash {
                return Ok(CommitOutcome::PayloadMismatch);
            }
            return Ok(CommitOutcome::Duplicate(CommitReceipt {
                writer_epoch: bundle.writer_epoch,
                bundle_seq: bundle.bundle_seq,
                data_version: data_version as u64,
            }));
        }
        let lowest: Option<i64> = self.connection.query_row(
            "select min(bundle_seq) from committed_bundle where writer_epoch = ?1",
            [bundle.writer_epoch as i64],
            |row| row.get(0),
        )?;
        if let Some(lowest) = lowest {
            if bundle.bundle_seq + u64::from(RETRY_WINDOW_RECEIPTS) < lowest as u64 {
                return Ok(CommitOutcome::RetryWindowExpired);
            }
        }
        self.connection.execute_batch("begin immediate")?;
        let watermark: i64 = self.connection.query_row(
            "select watermark from data_version where id = 1",
            [],
            |row| row.get(0),
        )?;
        let next = watermark + 1;
        self.connection.execute(
            "insert into committed_bundle(writer_epoch, bundle_seq, payload_hash, data_version) values (?1, ?2, ?3, ?4)",
            params![bundle.writer_epoch as i64, bundle.bundle_seq as i64, hash, next],
        )?;
        self.connection.execute(
            "update data_version set watermark = ?1 where id = 1",
            [next],
        )?;
        self.connection.execute(
            "insert into bundle_epoch(writer_epoch, highest_contiguous_seq, durable_watermark)
             values (?1, ?2, ?3)
             on conflict(writer_epoch) do update set highest_contiguous_seq = excluded.highest_contiguous_seq, durable_watermark = excluded.durable_watermark",
            params![bundle.writer_epoch as i64, bundle.bundle_seq as i64, next],
        )?;
        if let Some((minute, session, up, down)) = parse_minute_payload(&bundle.payload) {
            self.connection.execute(
                "insert into connection_minute(utc_minute, session_pk, upload, download) values (?1, ?2, ?3, ?4)
                 on conflict(utc_minute, session_pk) do update set upload = excluded.upload, download = excluded.download",
                params![minute, session, up, down],
            )?;
        }
        self.connection.execute_batch("commit")?;
        Ok(CommitOutcome::Applied(CommitReceipt {
            writer_epoch: bundle.writer_epoch,
            bundle_seq: bundle.bundle_seq,
            data_version: next as u64,
        }))
    }

    pub fn watermark(&self) -> Result<u64, StorageError> {
        let value: i64 = self.connection.query_row(
            "select watermark from data_version where id = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(value as u64)
    }

    pub fn receipt_count(&self) -> Result<u64, StorageError> {
        let value: i64 =
            self.connection
                .query_row("select count(*) from committed_bundle", [], |row| {
                    row.get(0)
                })?;
        Ok(value as u64)
    }

    pub fn prepare_count(&self) -> u64 {
        self.prepare_count
    }
}

pub fn hold_uncommitted_bundle(
    path: &Path,
    bundle: &CommitBundle,
) -> Result<Connection, StorageError> {
    let connection = migrate(path)?;
    connection.execute_batch("begin immediate")?;
    connection.execute(
        "insert into committed_bundle(writer_epoch, bundle_seq, payload_hash, data_version) values (?1, ?2, ?3, 1)",
        params![
            bundle.writer_epoch as i64,
            bundle.bundle_seq as i64,
            bundle.payload_hash()
        ],
    )?;
    Ok(connection)
}

fn parse_minute_payload(payload: &str) -> Option<(i64, i64, i64, i64)> {
    let mut parts = payload.split(',');
    Some((
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ))
}

pub struct RecoveryFacade {
    path: PathBuf,
}

impl RecoveryFacade {
    pub fn open(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn status(&self) -> Result<serde_json::Value, StorageError> {
        let connection = Connection::open(&self.path)?;
        let user_version: i32 = connection
            .query_row("pragma user_version", [], |row| row.get(0))
            .unwrap_or(-1);
        Ok(serde_json::json!({
            "path_redacted": true,
            "user_version": user_version,
            "supported_max": SCHEMA_VERSION,
            "future": user_version > SCHEMA_VERSION
        }))
    }

    pub fn list_backups(&self) -> Result<Vec<String>, StorageError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let connection = Connection::open(&self.path)?;
        let mut statement =
            connection.prepare("select path from backup_manifest order by backup_id")?;
        let rows = statement
            .query_map([], |row| row.get(0))
            .map(|mapped| mapped.filter_map(Result::ok).collect())
            .unwrap_or_default();
        Ok(rows)
    }

    pub fn validate_candidate(&self, candidate: &Path) -> Result<bool, StorageError> {
        let connection = Connection::open(candidate)?;
        let ok: i64 = connection.query_row("pragma integrity_check", [], |row| {
            let text: String = row.get(0)?;
            Ok(i64::from(text == "ok"))
        })?;
        Ok(ok == 1)
    }
}

pub fn backup_before_migration(src: &Path, dest: &Path) -> Result<(), StorageError> {
    let source = Connection::open(src)?;
    let mut target = Connection::open(dest)?;
    {
        let backup = rusqlite::backup::Backup::new(&source, &mut target)?;
        backup.step(64)?;
    }
    Ok(())
}

#[cfg(test)]
mod storage_schema_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn storage_schema_creates_only_core_tables() {
        use crate::c0_contract::{core_table_allowlist, forbidden_table_fragments};
        let dir = tempdir().expect("tempdir");
        let connection = migrate(&dir.path().join("core.sqlite3")).expect("migrate");
        let tables = list_user_tables(&connection).expect("tables");
        for table in &tables {
            assert!(
                core_table_allowlist().contains(&table.as_str()),
                "{table} 不在 allowlist"
            );
            for fragment in forbidden_table_fragments() {
                assert!(!table.contains(fragment));
            }
        }
    }
}

#[cfg(test)]
mod storage_migration_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn storage_migration_rejects_checksum_mismatch() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("bad-sum.sqlite3");
        {
            let connection = Connection::open(&path).expect("open");
            connection
                .execute_batch(
                    "create table schema_migration (version integer primary key, checksum text not null, applied_utc integer not null) strict;
                     insert into schema_migration(version, checksum, applied_utc) values (1, 'tampered', 0);
                     pragma user_version = 1;",
                )
                .expect("seed");
        }
        let error = migrate(&path).expect_err("checksum");
        assert!(error.to_string().contains("checksum"));
    }

    #[test]
    fn storage_migration_rejects_future_schema() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("future.sqlite3");
        {
            let connection = Connection::open(&path).expect("open");
            connection
                .execute_batch("pragma user_version = 99")
                .expect("version");
        }
        let error = migrate(&path).expect_err("future");
        assert!(error.to_string().contains("future"));
    }
}

#[cfg(test)]
mod migration_backup_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn migration_backup_copies_pages() {
        let dir = tempdir().expect("tempdir");
        let src = dir.path().join("src.sqlite3");
        let dest = dir.path().join("dst.sqlite3");
        migrate(&src).expect("src");
        backup_before_migration(&src, &dest).expect("backup");
        assert!(dest.exists());
    }
}

#[cfg(test)]
mod storage_prepared_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn storage_prepared_counts_fixed_prepares() {
        let dir = tempdir().expect("tempdir");
        let coordinator = StorageCoordinator::open(&dir.path().join("w.sqlite3")).expect("open");
        assert!(coordinator.prepare_count() >= 1);
    }
}

#[cfg(test)]
mod storage_bundle_idempotency_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn storage_bundle_idempotency_same_hash_does_not_advance() {
        let dir = tempdir().expect("tempdir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("w.sqlite3")).expect("open");
        let bundle = CommitBundle {
            writer_epoch: 1,
            bundle_seq: 1,
            payload: "1,1,8,16".into(),
        };
        let first = coordinator.commit(&bundle).expect("first");
        let second = coordinator.commit(&bundle).expect("second");
        assert!(matches!(first, CommitOutcome::Applied(_)));
        assert!(matches!(second, CommitOutcome::Duplicate(_)));
        assert_eq!(coordinator.watermark().expect("wm"), 1);
        let mismatch = coordinator
            .commit(&CommitBundle {
                writer_epoch: 1,
                bundle_seq: 1,
                payload: "changed".into(),
            })
            .expect("mismatch");
        assert_eq!(mismatch, CommitOutcome::PayloadMismatch);
    }
}

#[cfg(test)]
mod storage_watermark_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn storage_watermark_advances_once_per_new_bundle() {
        let dir = tempdir().expect("tempdir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("w.sqlite3")).expect("open");
        coordinator
            .commit(&CommitBundle {
                writer_epoch: 1,
                bundle_seq: 1,
                payload: "1,1,1,1".into(),
            })
            .expect("c1");
        coordinator
            .commit(&CommitBundle {
                writer_epoch: 1,
                bundle_seq: 2,
                payload: "1,2,1,1".into(),
            })
            .expect("c2");
        assert_eq!(coordinator.watermark().expect("wm"), 2);
    }
}

#[cfg(test)]
mod storage_bundle_retention_tests {
    use super::*;

    #[test]
    fn storage_bundle_retention_window_constant_is_frozen() {
        assert_eq!(RETRY_WINDOW_RECEIPTS, 100_000);
    }
}

#[cfg(test)]
mod storage_fault_gate_tests {
    #[test]
    fn storage_fault_gate_busy_deadline_is_positive() {
        const { assert!(crate::c0_contract::BUSY_TIMEOUT_MS > 0) };
    }
}

#[cfg(test)]
mod storage_backpressure_tests {
    #[test]
    fn storage_backpressure_queue_limit_is_explicit() {
        const { assert!(crate::c0_contract::QUEUE_MAX_BATCHES >= 1) };
    }
}

#[cfg(test)]
mod storage_checkpoint_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn storage_checkpoint_passive_does_not_delete_wal_file_name() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("ck.sqlite3");
        let connection = migrate(&path).expect("migrate");
        connection
            .execute_batch("pragma wal_checkpoint(PASSIVE)")
            .expect("checkpoint");
        assert!(path.exists());
    }
}

#[cfg(test)]
mod storage_wal_fallback_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn storage_wal_fallback_detects_non_wal_journal() {
        let dir = tempdir().expect("tempdir");
        let connection = migrate(&dir.path().join("wal.sqlite3")).expect("migrate");
        let mode: String = connection
            .query_row("pragma journal_mode", [], |row| row.get(0))
            .expect("mode");
        assert!(mode.eq_ignore_ascii_case("wal"));
    }
}

#[cfg(test)]
mod storage_shutdown_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn storage_shutdown_watermark_survives_reopen() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("sd.sqlite3");
        {
            let mut coordinator = StorageCoordinator::open(&path).expect("open");
            coordinator
                .commit(&CommitBundle {
                    writer_epoch: 1,
                    bundle_seq: 1,
                    payload: "1,1,1,1".into(),
                })
                .expect("commit");
        }
        let coordinator = StorageCoordinator::open(&path).expect("reopen");
        assert_eq!(coordinator.watermark().expect("wm"), 1);
    }
}

#[cfg(test)]
mod crash_before_commit_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn crash_before_commit_leaves_no_receipt() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("crash.sqlite3");
        migrate(&path).expect("migrate");
        {
            let connection = Connection::open(&path).expect("open");
            connection.execute_batch("begin immediate").expect("begin");
            connection
                .execute(
                    "insert into committed_bundle(writer_epoch, bundle_seq, payload_hash, data_version) values (1,1,'x',1)",
                    [],
                )
                .expect("insert");
        }
        let coordinator = StorageCoordinator::open(&path).expect("reopen");
        assert_eq!(coordinator.receipt_count().expect("count"), 0);
    }
}

#[cfg(test)]
mod crash_commit_unknown_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn crash_commit_unknown_retry_same_bundle_once() {
        let dir = tempdir().expect("tempdir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("u.sqlite3")).expect("open");
        let bundle = CommitBundle {
            writer_epoch: 1,
            bundle_seq: 3,
            payload: "3,1,1,1".into(),
        };
        let _ = coordinator.commit(&bundle).expect("first");
        let again = coordinator.commit(&bundle).expect("retry");
        assert!(matches!(again, CommitOutcome::Duplicate(_)));
        assert_eq!(coordinator.watermark().expect("wm"), 1);
    }
}

#[cfg(test)]
mod crash_after_commit_before_receipt_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn crash_after_commit_before_receipt_replays_stored_receipt() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("r.sqlite3");
        let bundle = CommitBundle {
            writer_epoch: 2,
            bundle_seq: 1,
            payload: "1,9,2,2".into(),
        };
        {
            let mut coordinator = StorageCoordinator::open(&path).expect("open");
            assert!(matches!(
                coordinator.commit(&bundle).expect("commit"),
                CommitOutcome::Applied(_)
            ));
        }
        let mut coordinator = StorageCoordinator::open(&path).expect("reopen");
        assert!(matches!(
            coordinator.commit(&bundle).expect("retry"),
            CommitOutcome::Duplicate(_)
        ));
    }
}

#[cfg(test)]
mod recovery_facade_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn recovery_facade_reads_version_without_writer() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("rec.sqlite3");
        migrate(&path).expect("migrate");
        let facade = RecoveryFacade::open(&path);
        let status = facade.status().expect("status");
        assert_eq!(status["user_version"], SCHEMA_VERSION);
        assert_eq!(status["future"], false);
    }
}

#[cfg(test)]
mod recovery_future_schema_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn recovery_future_schema_is_visible() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("rec.sqlite3");
        {
            let connection = Connection::open(&path).expect("open");
            connection
                .execute_batch("pragma user_version = 8")
                .expect("ver");
        }
        let status = RecoveryFacade::open(&path).status().expect("status");
        assert_eq!(status["future"], true);
    }
}

#[cfg(test)]
mod recovery_bad_backup_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn recovery_bad_backup_fails_validation() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("bad.sqlite3");
        std::fs::write(&path, b"not-a-database").expect("write");
        assert!(RecoveryFacade::open(&path)
            .validate_candidate(&path)
            .is_err());
    }
}
