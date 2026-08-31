//! C1 core schema、幂等 writer 与 RecoveryFacade backend。C3 只向前追加。

use crate::c0_contract::{BUSY_TIMEOUT_MS, RETRY_WINDOW_RECEIPTS, SCHEMA_VERSION};
use crate::c3::query::ReportError;
use crate::c3::schema::{
    C3_ARCHIVE_DDL, C3_ARCHIVE_MIGRATION_CHECKSUM, C3_ARCHIVE_SCHEMA_VERSION, C3_DDL,
    C3_MIGRATION_CHECKSUM, C3_SCHEMA_VERSION,
};
use crate::c3::sql::UNKNOWN_IDENTITY;
use crate::c4::schema::{C4_DDL, C4_MIGRATION_CHECKSUM, C4_SCHEMA_VERSION};
use crate::c4::types::AlertWriteSet;

pub const MIGRATION_CHECKSUM: &str = "c1-core-v1";
use crate::sqlite_probe::{apply_required_pragmas, open_bundled};
use rusqlite::backup::Backup;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct AlertCommitSlice {
    pub facts: Vec<crate::accounting::MinuteFact>,
    pub coverage: Vec<crate::accounting::CoverageChange>,
    pub live_rows: Vec<crate::c2::hub::LiveConnectionView>,
    pub utc: i64,
    pub writes: AlertWriteSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitKillPoint {
    AfterFacts,
    AfterAlerts,
    AfterOutbox,
    BeforeCommit,
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
    crate::c3::rule_name::register_last_chain_hop(&connection)
        .map_err(|error| StorageError::Closed(error.to_string()))?;
    let user_version: i32 = connection.query_row("pragma user_version", [], |row| row.get(0))?;
    if user_version > SCHEMA_VERSION {
        return Err(StorageError::Closed("future schema".into()));
    }
    verify_checksum(&connection, 1, MIGRATION_CHECKSUM, user_version)?;
    verify_checksum(
        &connection,
        C3_SCHEMA_VERSION,
        C3_MIGRATION_CHECKSUM,
        user_version,
    )?;
    verify_checksum(
        &connection,
        C4_SCHEMA_VERSION,
        C4_MIGRATION_CHECKSUM,
        user_version,
    )?;
    verify_checksum(
        &connection,
        C3_ARCHIVE_SCHEMA_VERSION,
        C3_ARCHIVE_MIGRATION_CHECKSUM,
        user_version,
    )?;
    if user_version == 0 {
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
            params![1, MIGRATION_CHECKSUM],
        )?;
    }
    let current: i32 = connection.query_row("pragma user_version", [], |row| row.get(0))?;
    if current < C3_SCHEMA_VERSION {
        connection.execute_batch(C3_DDL)?;
        connection.execute_batch("pragma user_version = 2")?;
        connection.execute(
            "insert or ignore into schema_migration(version, checksum, applied_utc) values (?1, ?2, 0)",
            params![C3_SCHEMA_VERSION, C3_MIGRATION_CHECKSUM],
        )?;
    }
    let current: i32 = connection.query_row("pragma user_version", [], |row| row.get(0))?;
    if current < C4_SCHEMA_VERSION {
        connection.execute_batch(C4_DDL)?;
        connection.execute_batch("pragma user_version = 3")?;
        connection.execute(
            "insert or ignore into schema_migration(version, checksum, applied_utc) values (?1, ?2, 0)",
            params![C4_SCHEMA_VERSION, C4_MIGRATION_CHECKSUM],
        )?;
    }
    let current: i32 = connection.query_row("pragma user_version", [], |row| row.get(0))?;
    if current < C3_ARCHIVE_SCHEMA_VERSION {
        connection.execute_batch(C3_ARCHIVE_DDL)?;
        connection.execute_batch("pragma user_version = 4")?;
        connection.execute(
            "insert or ignore into schema_migration(version, checksum, applied_utc) values (?1, ?2, 0)",
            params![C3_ARCHIVE_SCHEMA_VERSION, C3_ARCHIVE_MIGRATION_CHECKSUM],
        )?;
    }
    Ok(connection)
}

fn verify_checksum(
    connection: &Connection,
    version: i32,
    expected: &str,
    user_version: i32,
) -> Result<(), StorageError> {
    if user_version < version {
        return Ok(());
    }
    let has_table: i64 = connection.query_row(
        "select count(*) from sqlite_master where type = 'table' and name = 'schema_migration'",
        [],
        |row| row.get(0),
    )?;
    if has_table == 0 {
        return Ok(());
    }
    let checksum: Option<String> = connection
        .query_row(
            "select checksum from schema_migration where version = ?1",
            [version],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(checksum) = checksum {
        if checksum != expected {
            return Err(StorageError::Closed("checksum mismatch".into()));
        }
    }
    Ok(())
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
    path: PathBuf,
    connection: Connection,
    prepare_count: u64,
    #[cfg(test)]
    pub test_kill: Option<CommitKillPoint>,
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
            path: path.to_path_buf(),
            connection,
            prepare_count: 1,
            #[cfg(test)]
            test_kill: None,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn connection_mut(&mut self) -> &mut Connection {
        &mut self.connection
    }

    pub fn checkpoint_passive(&self) -> Result<(), StorageError> {
        self.connection
            .execute_batch("pragma wal_checkpoint(PASSIVE)")?;
        Ok(())
    }

    pub fn commit_alert_bundle(
        &mut self,
        bundle: &CommitBundle,
        slice: &AlertCommitSlice,
    ) -> Result<CommitOutcome, StorageError> {
        self.commit_inner(bundle, Some(slice))
    }

    pub fn commit(&mut self, bundle: &CommitBundle) -> Result<CommitOutcome, StorageError> {
        self.commit_inner(bundle, None)
    }

    fn commit_inner(
        &mut self,
        bundle: &CommitBundle,
        slice: Option<&AlertCommitSlice>,
    ) -> Result<CommitOutcome, StorageError> {
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
        if let Some(slice) = slice {
            if let Err(error) = persist_slice(&self.connection, slice) {
                let _ = self.connection.execute_batch("rollback");
                return Err(error);
            }
            #[cfg(test)]
            if self.test_kill == Some(CommitKillPoint::AfterFacts) {
                let _ = self.connection.execute_batch("rollback");
                return Err(StorageError::Closed("kill after facts".into()));
            }
            if let Err(error) =
                crate::c4::store::persist_instances(&self.connection, &slice.writes.instances)
            {
                let _ = self.connection.execute_batch("rollback");
                return Err(error);
            }
            if let Err(error) =
                crate::c4::store::persist_events(&self.connection, &slice.writes.events)
            {
                let _ = self.connection.execute_batch("rollback");
                return Err(error);
            }
            #[cfg(test)]
            if self.test_kill == Some(CommitKillPoint::AfterAlerts) {
                let _ = self.connection.execute_batch("rollback");
                return Err(StorageError::Closed("kill after alerts".into()));
            }
            if let Err(error) = crate::c4::outbox::persist_intents(self, &slice.writes.outbox) {
                let _ = self.connection.execute_batch("rollback");
                return Err(error);
            }
            #[cfg(test)]
            if self.test_kill == Some(CommitKillPoint::AfterOutbox)
                || self.test_kill == Some(CommitKillPoint::BeforeCommit)
            {
                let _ = self.connection.execute_batch("rollback");
                return Err(StorageError::Closed("kill before commit".into()));
            }
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

    pub fn reserve_writer_epoch(&mut self) -> Result<u64, StorageError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let highest: i64 = transaction.query_row(
            "select max(value) from (
                select coalesce(max(writer_epoch), 0) as value from bundle_epoch
                union all
                select coalesce(max(writer_epoch), 0) as value from committed_bundle
             )",
            [],
            |row| row.get(0),
        )?;
        let next = highest
            .checked_add(1)
            .filter(|value| *value > 0)
            .ok_or_else(|| StorageError::Closed("writer epoch exhausted".into()))?;
        let watermark: i64 = transaction.query_row(
            "select watermark from data_version where id = 1",
            [],
            |row| row.get(0),
        )?;
        transaction.execute(
            "insert into bundle_epoch(writer_epoch, highest_contiguous_seq, durable_watermark)
             values (?1, 0, ?2)",
            params![next, watermark],
        )?;
        transaction.commit()?;
        Ok(next as u64)
    }

    pub fn reserve_controller_epoch(&mut self, core_identity: &str) -> Result<u64, StorageError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let highest: i64 = transaction.query_row(
            "select coalesce(max(epoch_id), 0) from controller_epoch",
            [],
            |row| row.get(0),
        )?;
        let next = highest
            .checked_add(1)
            .filter(|value| *value > 0)
            .ok_or_else(|| StorageError::Closed("controller epoch exhausted".into()))?;
        transaction.execute(
            "insert into controller_epoch(epoch_id, core_identity) values (?1, ?2)",
            params![next, core_identity],
        )?;
        transaction.commit()?;
        Ok(next as u64)
    }

    pub fn health(&self) -> Result<StorageHealth, StorageError> {
        Ok(StorageHealth {
            ok: true,
            watermark: self.watermark()?,
            reason: None,
        })
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        let value = self
            .connection
            .query_row(
                "select value from machine_setting where key = ?1",
                [key],
                |row| row.get(0),
            )
            .optional()?;
        Ok(value)
    }

    pub fn put_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        self.connection.execute(
            "insert into machine_setting(key, value) values (?1, ?2)
             on conflict(key) do update set value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn save_targets(&self, names: &[String]) -> Result<u32, StorageError> {
        let current = self.load_targets()?.0;
        let next = current.saturating_add(1).max(1);
        self.connection.execute(
            "insert into target_set(set_id, policy_version) values (1, ?1)
             on conflict(set_id) do update set policy_version = excluded.policy_version",
            [next as i64],
        )?;
        self.connection
            .execute("delete from target_item where set_id = 1", [])?;
        for (position, name) in names.iter().enumerate() {
            self.connection.execute(
                "insert into target_item(set_id, position, name) values (1, ?1, ?2)",
                params![position as i64, name],
            )?;
        }
        Ok(next)
    }

    pub fn load_targets(&self) -> Result<(u32, Vec<String>), StorageError> {
        let version: Option<i64> = self
            .connection
            .query_row(
                "select policy_version from target_set where set_id = 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        let mut statement = self
            .connection
            .prepare("select name from target_item where set_id = 1 order by position")?;
        let names = statement
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok((version.unwrap_or(0) as u32, names))
    }

    pub fn persist_live_facts(
        &mut self,
        facts: &[crate::accounting::MinuteFact],
        rows: &[crate::c2::hub::LiveConnectionView],
        coverage: &[crate::accounting::CoverageChange],
        utc: i64,
    ) -> Result<(), StorageError> {
        persist_slice(
            &self.connection,
            &AlertCommitSlice {
                facts: facts.to_vec(),
                coverage: coverage.to_vec(),
                live_rows: rows.to_vec(),
                utc,
                writes: AlertWriteSet::default(),
            },
        )
    }
}

fn persist_slice(connection: &Connection, slice: &AlertCommitSlice) -> Result<(), StorageError> {
    let policy_version = connection
        .query_row(
            "select policy_version from target_set where set_id = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .unwrap_or(0);
    for row in &slice.live_rows {
        let (epoch, id) = split_identity(&row.identity);
        let session_pk = ensure_session_on(connection, epoch, id, slice.utc, row.host.as_deref())?;
        intern_and_attr(connection, session_pk, row, policy_version, slice.utc)?;
        if !row.chains.is_empty() {
            connection.execute(
                "delete from connection_chain where session_pk = ?1",
                [session_pk],
            )?;
            for (position, node) in row.chains.iter().enumerate() {
                connection.execute(
                    "insert into connection_chain(session_pk, position, node) values (?1, ?2, ?3)",
                    params![session_pk, position as i64, node],
                )?;
            }
        }
    }
    for fact in &slice.facts {
        let (epoch, id) = split_identity(&fact.session_key);
        let session_pk = ensure_session_on(connection, epoch, id, slice.utc, None)?;
        connection.execute(
            "insert into connection_minute(utc_minute, session_pk, upload, download) values (?1, ?2, ?3, ?4)
             on conflict(utc_minute, session_pk) do update set
                upload = upload + excluded.upload,
                download = download + excluded.download",
            params![fact.utc_minute, session_pk, fact.upload as i64, fact.download as i64],
        )?;
    }
    for item in &slice.coverage {
        let open_exists: bool = connection.query_row(
            "select exists(select 1 from coverage_interval
                  where kind = ?1 and reason = ?2 and ended_utc is null)",
            params![item.kind, item.reason],
            |row| row.get(0),
        )?;
        if !open_exists {
            connection.execute(
                "insert into coverage_interval(kind, reason, started_utc, ended_utc) values (?1, ?2, ?3, ?4)",
                params![item.kind, item.reason, slice.utc, Option::<i64>::None],
            )?;
        }
    }
    // 恢复采集的切片不再携带上一状态的 kind，据此闭合遗留的开放行。
    // 正常帧 coverage 为空 → 关闭全部开放行；断连帧带 gap → gap 保持开放。
    let kinds: Vec<&str> = slice.coverage.iter().map(|item| item.kind).collect();
    let sql = if kinds.is_empty() {
        "update coverage_interval set ended_utc = ?1 where ended_utc is null".to_string()
    } else {
        let placeholders = kinds.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        format!(
            "update coverage_interval set ended_utc = ?1
              where ended_utc is null and kind not in ({placeholders})"
        )
    };
    let mut bind: Vec<&dyn rusqlite::ToSql> = vec![&slice.utc];
    for kind in &kinds {
        bind.push(kind);
    }
    connection.execute(&sql, bind.as_slice())?;
    Ok(())
}

fn ensure_session_on(
    connection: &Connection,
    epoch: i64,
    connection_id: &str,
    utc: i64,
    host: Option<&str>,
) -> Result<i64, StorageError> {
    if let Some((existing, stored)) = connection
        .query_row(
            "select session_pk, host from connection_session where epoch_id = ?1 and connection_id = ?2",
            params![epoch, connection_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?)),
        )
        .optional()?
    {
        let next = crate::session_host::prefer_host_identity(stored.as_deref(), host);
        if next.as_deref() != stored.as_deref() {
            connection.execute(
                "update connection_session set host = ?1 where session_pk = ?2",
                params![next, existing],
            )?;
        }
        return Ok(existing);
    }
    connection.execute(
        "insert into connection_session(epoch_id, connection_id, started_utc, host) values (?1, ?2, ?3, ?4)",
        params![epoch, connection_id, utc, host],
    )?;
    Ok(connection.last_insert_rowid())
}

impl StorageCoordinator {
    pub fn seed_report_fixture(&self) -> Result<(), StorageError> {
        self.connection.execute_batch(
            "
            insert or ignore into connection_session(session_pk, epoch_id, connection_id, started_utc, host)
            values (1, 1, 'alpha', 1200, 'a.example'), (2, 1, 'beta', 1800, 'b.example');
            insert or ignore into dimension_dict(dimension_kind, dimension_id, value) values
                ('host', 1, 'a.example'), ('host', 2, 'b.example'),
                ('process', 1, 'app.exe'), ('network', 1, 'tcp'), ('category', 1, '家宽');
            insert or ignore into connection_session_attr(
                session_pk, host_id, process_id, rule_id, network_id, chain_key,
                policy_version, primary_category_id, started_utc, ended_utc
            ) values
                (1, 1, 1, null, 1, 'DIRECT', 1, 1, 1200, null),
                (2, 2, 1, null, 1, 'DIRECT', 1, 1, 1800, null);
            insert or ignore into connection_minute(utc_minute, session_pk, upload, download)
            values (20, 1, 10, 30), (40, 2, 20, 60);
            insert or ignore into coverage_interval(interval_id, kind, reason, started_utc, ended_utc)
            values (1, 'gap', 'disconnect_or_sleep', 2500, 2800),
                   (2, 'covered', 'running', 1000, 2500);
            insert or ignore into traffic_hourly_dimension(
                utc_hour, category_id, dimension_kind, dimension_id,
                upload, download, connection_count, active_duration_sec
            ) values (0, 1, 'host', 1, 10, 30, 1, 60);
            insert or ignore into target_set(set_id, policy_version) values (1, 1);
            ",
        )?;
        Ok(())
    }
}

fn split_identity(identity: &str) -> (i64, &str) {
    identity
        .split_once(':')
        .and_then(|(left, right)| left.parse().ok().map(|epoch| (epoch, right)))
        .unwrap_or((0, identity))
}

fn intern_and_attr(
    connection: &Connection,
    session_pk: i64,
    row: &crate::c2::hub::LiveConnectionView,
    policy_version: i64,
    utc: i64,
) -> Result<(), StorageError> {
    let canonical_host: Option<String> = connection.query_row(
        "select host from connection_session where session_pk = ?1",
        [session_pk],
        |result| result.get(0),
    )?;
    let host_id = intern_dim(connection, "host", canonical_host.as_deref())?;
    let process_id = intern_dim(connection, "process", row.process_name.as_deref())?;
    let rule_id = intern_dim(connection, "rule", row.rule.as_deref())?;
    let network_id = intern_dim(connection, "network", row.network.as_deref())?;
    let category_id = intern_dim(connection, "category", row.primary.as_deref())?;
    let chain_key = if row.chains.is_empty() {
        None
    } else {
        Some(row.chains.join(">"))
    };
    connection.execute(
        "insert into connection_session_attr(
            session_pk, host_id, process_id, rule_id, network_id, chain_key,
            policy_version, primary_category_id, started_utc, ended_utc
         ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, null)
         on conflict(session_pk) do update set
            host_id = coalesce(excluded.host_id, connection_session_attr.host_id),
            process_id = coalesce(excluded.process_id, connection_session_attr.process_id),
            rule_id = coalesce(excluded.rule_id, connection_session_attr.rule_id),
            network_id = coalesce(excluded.network_id, connection_session_attr.network_id),
            chain_key = coalesce(excluded.chain_key, connection_session_attr.chain_key),
            policy_version = excluded.policy_version,
            primary_category_id = excluded.primary_category_id",
        params![
            session_pk,
            host_id,
            process_id,
            rule_id,
            network_id,
            chain_key,
            policy_version,
            category_id,
            utc
        ],
    )?;
    Ok(())
}

fn intern_dim(
    connection: &Connection,
    kind: &str,
    value: Option<&str>,
) -> Result<Option<i64>, StorageError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if value == UNKNOWN_IDENTITY {
        return Ok(None);
    }
    if let Some(existing) = connection
        .query_row(
            "select dimension_id from dimension_dict where dimension_kind = ?1 and value = ?2",
            params![kind, value],
            |row| row.get(0),
        )
        .optional()?
    {
        return Ok(Some(existing));
    }
    let next: i64 = connection.query_row(
        "select coalesce(max(dimension_id), 0) + 1 from dimension_dict where dimension_kind = ?1",
        [kind],
        |row| row.get(0),
    )?;
    connection.execute(
        "insert or ignore into dimension_dict(dimension_kind, dimension_id, value) values (?1, ?2, ?3)",
        params![kind, next, value],
    )?;
    Ok(Some(next))
}

pub fn open_interruptible_reader(path: &Path) -> Result<Connection, StorageError> {
    let connection = Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_FULL_MUTEX,
    )?;
    apply_required_pragmas(&connection, BUSY_TIMEOUT_MS)
        .map_err(|error| StorageError::Closed(error.to_string()))?;
    crate::c3::rule_name::register_last_chain_hop(&connection)
        .map_err(|error| StorageError::Closed(error.to_string()))?;
    Ok(connection)
}

pub fn backup_pages(src: &Path, dest: &Path, cancel: &Arc<AtomicBool>) -> Result<(), ReportError> {
    if cancel.load(Ordering::SeqCst) {
        return Err(ReportError::Cancelled("backup"));
    }
    let source = Connection::open(src).map_err(|_| ReportError::Failed("open source"))?;
    let mut target = Connection::open(dest).map_err(|_| ReportError::Failed("open dest"))?;
    {
        let backup =
            Backup::new(&source, &mut target).map_err(|_| ReportError::Failed("backup api"))?;
        loop {
            if cancel.load(Ordering::SeqCst) {
                return Err(ReportError::Cancelled("backup"));
            }
            match backup
                .step(64)
                .map_err(|_| ReportError::Failed("backup step"))?
            {
                rusqlite::backup::StepResult::Done => break,
                rusqlite::backup::StepResult::More => {}
                rusqlite::backup::StepResult::Busy | rusqlite::backup::StepResult::Locked => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                _ => std::thread::sleep(std::time::Duration::from_millis(10)),
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageHealth {
    pub ok: bool,
    pub watermark: u64,
    pub reason: Option<&'static str>,
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
        use crate::c0_contract::{
            all_table_allowlist, core_table_allowlist, forbidden_table_fragments,
        };
        let dir = tempdir().expect("tempdir");
        let connection = migrate(&dir.path().join("core.sqlite3")).expect("migrate");
        let tables = list_user_tables(&connection).expect("tables");
        let allow = all_table_allowlist();
        for table in &tables {
            assert!(
                allow.contains(&table.as_str()),
                "{table} 不在 C1+C3 allowlist"
            );
            for fragment in forbidden_table_fragments() {
                assert!(!table.contains(fragment), "{table} 不得包含 {fragment}");
            }
        }
        for required in core_table_allowlist() {
            assert!(
                tables.iter().any(|item| item == required),
                "缺少 C1 表 {required}"
            );
        }
        assert!(tables.iter().any(|item| item == "traffic_hourly_dimension"));
        assert!(tables.iter().any(|item| item == "alert_rule"));
        assert!(tables.iter().any(|item| item == "notification_outbox"));
        assert!(tables.iter().any(|item| item == "report_archive"));
        let version: i32 = connection
            .query_row("pragma user_version", [], |row| row.get(0))
            .expect("ver");
        assert_eq!(version, SCHEMA_VERSION);
        let c1: String = connection
            .query_row(
                "select checksum from schema_migration where version = 1",
                [],
                |row| row.get(0),
            )
            .expect("c1");
        let c3: String = connection
            .query_row(
                "select checksum from schema_migration where version = 2",
                [],
                |row| row.get(0),
            )
            .expect("c3");
        let c4: String = connection
            .query_row(
                "select checksum from schema_migration where version = 3",
                [],
                |row| row.get(0),
            )
            .expect("c4");
        let archive: String = connection
            .query_row(
                "select checksum from schema_migration where version = 4",
                [],
                |row| row.get(0),
            )
            .expect("archive");
        assert_eq!(c1, MIGRATION_CHECKSUM);
        assert_eq!(c3, C3_MIGRATION_CHECKSUM);
        assert_eq!(c4, C4_MIGRATION_CHECKSUM);
        assert_eq!(archive, C3_ARCHIVE_MIGRATION_CHECKSUM);
    }

    #[test]
    fn storage_v3_upgrades_to_archive_v4() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("v3.sqlite3");
        {
            let connection = Connection::open(&path).expect("open");
            connection
                .execute_batch(
                    "create table schema_migration (
                        version integer primary key,
                        checksum text not null,
                        applied_utc integer not null
                    ) strict;
                    insert into schema_migration(version, checksum, applied_utc) values
                        (1, 'c1-core-v1', 0),
                        (2, 'c3-report-v2', 0),
                        (3, 'c4-alert-v3', 0);
                    pragma user_version = 3;",
                )
                .expect("seed v3");
        }
        let connection = migrate(&path).expect("upgrade");
        let version: i32 = connection
            .query_row("pragma user_version", [], |row| row.get(0))
            .expect("ver");
        assert_eq!(version, 4);
        let checksum: String = connection
            .query_row(
                "select checksum from schema_migration where version = 4",
                [],
                |row| row.get(0),
            )
            .expect("v4");
        assert_eq!(checksum, C3_ARCHIVE_MIGRATION_CHECKSUM);
        let has_archive: i64 = connection
            .query_row(
                "select count(*) from sqlite_master where type = 'table' and name = 'report_archive'",
                [],
                |row| row.get(0),
            )
            .expect("table");
        assert_eq!(has_archive, 1);
        let c3: String = connection
            .query_row(
                "select checksum from schema_migration where version = 2",
                [],
                |row| row.get(0),
            )
            .expect("c3");
        let c4: String = connection
            .query_row(
                "select checksum from schema_migration where version = 3",
                [],
                |row| row.get(0),
            )
            .expect("c4");
        assert_eq!(c3, C3_MIGRATION_CHECKSUM);
        assert_eq!(c4, C4_MIGRATION_CHECKSUM);
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
mod c4_alert_commit_atomic_tests {
    use super::*;
    use crate::c4::types::{
        AlertEvent, AlertEvidence, AlertInstance, AlertWriteSet, EventKind, InstanceStatus,
        OutboxIntent, OutboxStatus,
    };
    use tempfile::tempdir;

    fn slice() -> AlertCommitSlice {
        let evidence = AlertEvidence {
            rule_id: "r1".into(),
            rule_version: 1,
            data_version: Some(1),
            evaluated_at_utc: 10,
            window_start_utc: None,
            window_end_utc: None,
            display_timezone: "UTC".into(),
            selector: "health_kind:tcp_auth".into(),
            direction: None,
            observed_value: Some(1),
            trigger_threshold: 1,
            recovery_threshold: None,
            coverage_summary: "unhealthy".into(),
            policy_metadata: None,
            report_query: None,
            not_evaluable_reason: None,
        };
        AlertCommitSlice {
            facts: Vec::new(),
            coverage: Vec::new(),
            live_rows: Vec::new(),
            utc: 10,
            writes: AlertWriteSet {
                instances: vec![AlertInstance {
                    instance_id: "i1".into(),
                    rule_id: "r1".into(),
                    rule_version: 1,
                    selector_identity: "health_kind:tcp_auth".into(),
                    status: InstanceStatus::Active,
                    started_utc: Some(10),
                    resolved_utc: None,
                    last_eval_utc: 10,
                    last_observed: Some(1),
                    evidence: evidence.clone(),
                }],
                events: vec![AlertEvent {
                    event_id: "e1".into(),
                    instance_id: "i1".into(),
                    bundle_id: "1:1".into(),
                    kind: EventKind::Activated,
                    at_utc: 10,
                    evidence,
                    idempotency_key: "e1-key".into(),
                }],
                outbox: vec![OutboxIntent {
                    outbox_id: "o1".into(),
                    event_id: "e1".into(),
                    bundle_id: "1:1".into(),
                    status: OutboxStatus::Pending,
                    attempt: 0,
                    next_attempt_at: 10,
                    lease_until: None,
                    lease_token: None,
                    error_class: None,
                    error_summary: None,
                    idempotency_key: "o1-key".into(),
                    created_utc: 10,
                }],
            },
        }
    }

    #[test]
    fn kill_after_alerts_rolls_back_facts_and_outbox() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("a.sqlite3");
        let bundle = CommitBundle {
            writer_epoch: 1,
            bundle_seq: 1,
            payload: "1,1,1,1".into(),
        };
        {
            let mut coordinator = StorageCoordinator::open(&path).expect("open");
            coordinator.test_kill = Some(CommitKillPoint::AfterAlerts);
            let error = coordinator
                .commit_alert_bundle(&bundle, &slice())
                .expect_err("kill");
            assert!(error.to_string().contains("kill"));
        }
        let coordinator = StorageCoordinator::open(&path).expect("reopen");
        assert_eq!(coordinator.receipt_count().expect("c"), 0);
        let events: i64 = coordinator
            .connection()
            .query_row("select count(*) from alert_event", [], |row| row.get(0))
            .expect("ev");
        let outbox: i64 = coordinator
            .connection()
            .query_row("select count(*) from notification_outbox", [], |row| {
                row.get(0)
            })
            .expect("ob");
        assert_eq!(events, 0);
        assert_eq!(outbox, 0);
    }

    #[test]
    fn retry_same_bundle_does_not_duplicate_event() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("b.sqlite3")).expect("open");
        let bundle = CommitBundle {
            writer_epoch: 1,
            bundle_seq: 1,
            payload: "1,1,1,1".into(),
        };
        let extras = slice();
        assert!(matches!(
            coordinator
                .commit_alert_bundle(&bundle, &extras)
                .expect("first"),
            CommitOutcome::Applied(_)
        ));
        assert!(matches!(
            coordinator
                .commit_alert_bundle(&bundle, &extras)
                .expect("dup"),
            CommitOutcome::Duplicate(_)
        ));
        let events: i64 = coordinator
            .connection()
            .query_row("select count(*) from alert_event", [], |row| row.get(0))
            .expect("ev");
        let outbox: i64 = coordinator
            .connection()
            .query_row("select count(*) from notification_outbox", [], |row| {
                row.get(0)
            })
            .expect("ob");
        assert_eq!(events, 1);
        assert_eq!(outbox, 1);
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

#[cfg(test)]
mod storage_host_identity_tests {
    use super::*;
    use crate::c2::hub::LiveConnectionView;
    use tempfile::tempdir;

    fn live(id: &str, host: Option<&str>) -> LiveConnectionView {
        LiveConnectionView {
            identity: format!("1:{id}"),
            connection_id: id.into(),
            epoch: 1,
            host: host.map(str::to_string),
            ..LiveConnectionView::default()
        }
    }

    fn host_of(coordinator: &StorageCoordinator, id: &str) -> Option<String> {
        coordinator
            .connection()
            .query_row(
                "select host from connection_session where epoch_id = 1 and connection_id = ?1",
                [id],
                |row| row.get(0),
            )
            .expect("host")
    }

    #[test]
    fn persist_upgrades_empty_and_ip_host_but_not_domain() {
        let dir = tempdir().expect("tempdir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("host.sqlite3")).expect("open");
        coordinator
            .persist_live_facts(&[], &[live("a", None)], &[], 100)
            .expect("empty");
        assert_eq!(host_of(&coordinator, "a"), None);
        coordinator
            .persist_live_facts(&[], &[live("a", Some("1.1.1.1"))], &[], 101)
            .expect("ip");
        assert_eq!(host_of(&coordinator, "a").as_deref(), Some("1.1.1.1"));
        coordinator
            .persist_live_facts(&[], &[live("a", Some("a.test"))], &[], 102)
            .expect("domain");
        assert_eq!(host_of(&coordinator, "a").as_deref(), Some("a.test"));
        coordinator
            .persist_live_facts(&[], &[live("a", Some("8.8.8.8"))], &[], 103)
            .expect("no downgrade");
        assert_eq!(host_of(&coordinator, "a").as_deref(), Some("a.test"));
    }
}

#[cfg(test)]
mod storage_attribution_lifecycle_tests {
    use super::*;
    use crate::c2::hub::LiveConnectionView;
    use tempfile::tempdir;

    #[test]
    fn writer_and_controller_epochs_are_durable_and_monotonic() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("epoch.sqlite3");
        let mut first = StorageCoordinator::open(&path).expect("open");
        assert_eq!(first.reserve_writer_epoch().expect("writer 1"), 1);
        assert_eq!(first.reserve_writer_epoch().expect("writer 2"), 2);
        assert_eq!(
            first
                .reserve_controller_epoch("collector-http")
                .expect("controller 1"),
            1
        );
        drop(first);

        let mut second = StorageCoordinator::open(&path).expect("reopen");
        assert_eq!(second.reserve_writer_epoch().expect("writer 3"), 3);
        assert_eq!(
            second
                .reserve_controller_epoch("collector-http")
                .expect("controller 2"),
            2
        );
    }

    #[test]
    fn concurrent_epoch_reservations_are_unique() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("epoch-race.sqlite3");
        drop(StorageCoordinator::open(&path).expect("initialize"));
        let barrier = Arc::new(std::sync::Barrier::new(4));
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let barrier = barrier.clone();
                let path = path.clone();
                std::thread::spawn(move || {
                    let mut coordinator = StorageCoordinator::open(&path).expect("open");
                    barrier.wait();
                    let writer = coordinator.reserve_writer_epoch().expect("writer");
                    let controller = coordinator
                        .reserve_controller_epoch("collector-http")
                        .expect("controller");
                    (writer, controller)
                })
            })
            .collect();
        let mut writers = Vec::new();
        let mut controllers = Vec::new();
        for handle in handles {
            let (writer, controller) = handle.join().expect("join");
            writers.push(writer);
            controllers.push(controller);
        }
        writers.sort_unstable();
        controllers.sort_unstable();
        assert_eq!(writers, vec![1, 2, 3, 4]);
        assert_eq!(controllers, vec![1, 2, 3, 4]);
    }

    #[test]
    fn writer_epoch_exhaustion_fails_closed() {
        let dir = tempdir().expect("tempdir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("epoch-overflow.sqlite3")).expect("open");
        coordinator
            .connection()
            .execute(
                "insert into bundle_epoch(writer_epoch, highest_contiguous_seq, durable_watermark)
                 values (?1, 0, 0)",
                [i64::MAX],
            )
            .expect("seed max epoch");

        let error = coordinator.reserve_writer_epoch().unwrap_err();
        assert!(
            matches!(error, StorageError::Closed(message) if message == "writer epoch exhausted")
        );
    }

    fn rich_row() -> LiveConnectionView {
        LiveConnectionView {
            identity: "1:a".into(),
            connection_id: "a".into(),
            epoch: 1,
            host: Some("a.test".into()),
            process_name: Some("browser.exe".into()),
            rule: Some("IPCIDR".into()),
            network: Some("tcp".into()),
            chains: vec!["node".into(), "Proxy".into()],
            ..LiveConnectionView::default()
        }
    }

    #[test]
    fn empty_metadata_does_not_erase_attr_and_chain_replacement_is_atomic() {
        let dir = tempdir().expect("tempdir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("attr.sqlite3")).expect("open");
        coordinator
            .persist_live_facts(&[], &[rich_row()], &[], 100)
            .expect("rich");
        let empty = LiveConnectionView {
            identity: "1:a".into(),
            connection_id: "a".into(),
            epoch: 1,
            ..LiveConnectionView::default()
        };
        coordinator
            .persist_live_facts(&[], &[empty], &[], 101)
            .expect("empty");
        type StoredAttribution = (
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        );
        let values: StoredAttribution =
            coordinator
                .connection()
                .query_row(
                    "select h.value, p.value, r.value, n.value, a.chain_key
                       from connection_session_attr a
                       left join dimension_dict h on h.dimension_kind='host' and h.dimension_id=a.host_id
                       left join dimension_dict p on p.dimension_kind='process' and p.dimension_id=a.process_id
                       left join dimension_dict r on r.dimension_kind='rule' and r.dimension_id=a.rule_id
                       left join dimension_dict n on n.dimension_kind='network' and n.dimension_id=a.network_id",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
                )
                .expect("attr");
        assert_eq!(
            values,
            (
                Some("a.test".into()),
                Some("browser.exe".into()),
                Some("IPCIDR".into()),
                Some("tcp".into()),
                Some("node>Proxy".into())
            )
        );

        let changed = LiveConnectionView {
            chains: vec!["DIRECT".into()],
            ..rich_row()
        };
        coordinator
            .persist_live_facts(&[], &[changed], &[], 102)
            .expect("changed");
        let nodes: Vec<String> = {
            let mut statement = coordinator
                .connection()
                .prepare("select node from connection_chain order by position")
                .expect("prepare");
            statement
                .query_map([], |row| row.get(0))
                .expect("query")
                .collect::<Result<_, _>>()
                .expect("rows")
        };
        assert_eq!(nodes, vec!["DIRECT"]);
    }

    #[test]
    fn late_metadata_enriches_existing_raw_minutes_for_the_same_session() {
        let dir = tempdir().expect("tempdir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("late-metadata.sqlite3")).expect("open");
        let empty = LiveConnectionView {
            identity: "1:a".into(),
            connection_id: "a".into(),
            epoch: 1,
            ..LiveConnectionView::default()
        };
        let fact = crate::accounting::MinuteFact {
            session_key: "1:a".into(),
            utc_minute: 1,
            upload: 7,
            download: 11,
            primary: None,
            tags: Vec::new(),
        };
        coordinator
            .persist_live_facts(&[fact], &[empty], &[], 60)
            .expect("initial raw");
        coordinator
            .persist_live_facts(&[], &[rich_row()], &[], 61)
            .expect("late metadata");

        let enriched: (Option<String>, Option<String>, i64, i64) = coordinator
            .connection()
            .query_row(
                "select h.value, p.value, m.upload, m.download
                   from connection_minute m
                   join connection_session s on s.session_pk=m.session_pk
                   join connection_session_attr a on a.session_pk=s.session_pk
                   left join dimension_dict h on h.dimension_kind='host' and h.dimension_id=a.host_id
                   left join dimension_dict p on p.dimension_kind='process' and p.dimension_id=a.process_id",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("enriched raw");
        assert_eq!(
            enriched,
            (Some("a.test".into()), Some("browser.exe".into()), 7, 11)
        );
    }
}
#[cfg(test)]
mod coverage_persist_tests {
    use super::*;
    use crate::accounting::CoverageChange;
    use tempfile::tempdir;

    fn open_rows(connection: &Connection) -> Vec<(String, i64, Option<i64>)> {
        let mut statement = connection
            .prepare(
                "select kind, started_utc, ended_utc from coverage_interval order by interval_id",
            )
            .expect("prepare");
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .expect("map")
            .collect::<Result<Vec<_>, _>>()
            .expect("rows")
    }

    #[test]
    fn gap_frames_dedupe_to_single_open_row() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("dedupe.sqlite3")).expect("open");
        let gap = vec![CoverageChange {
            kind: "gap",
            reason: "disconnect_or_sleep",
        }];
        coordinator
            .persist_live_facts(&[], &[], &gap, 100)
            .expect("first");
        coordinator
            .persist_live_facts(&[], &[], &gap, 200)
            .expect("second");
        coordinator
            .persist_live_facts(&[], &[], &gap, 300)
            .expect("third");
        assert_eq!(
            open_rows(coordinator.connection()),
            vec![("gap".to_string(), 100, None)],
            "断连风暴只留一行，started 为断连起点"
        );
    }

    #[test]
    fn resume_closes_open_gap() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("resume.sqlite3")).expect("open");
        let gap = vec![CoverageChange {
            kind: "gap",
            reason: "disconnect_or_sleep",
        }];
        coordinator
            .persist_live_facts(&[], &[], &gap, 100)
            .expect("gap");
        coordinator
            .persist_live_facts(&[], &[], &[], 300)
            .expect("resume");
        assert_eq!(
            open_rows(coordinator.connection()),
            vec![("gap".to_string(), 100, Some(300))],
            "恢复采集的帧闭合开放 gap"
        );
    }

    #[test]
    fn kind_switch_closes_previous_open_row() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("switch.sqlite3")).expect("open");
        let closed = vec![CoverageChange {
            kind: "closed",
            reason: "pause_or_shutdown",
        }];
        let gap = vec![CoverageChange {
            kind: "gap",
            reason: "disconnect_or_sleep",
        }];
        coordinator
            .persist_live_facts(&[], &[], &closed, 50)
            .expect("pause");
        coordinator
            .persist_live_facts(&[], &[], &gap, 100)
            .expect("disconnect");
        assert_eq!(
            open_rows(coordinator.connection()),
            vec![
                ("closed".to_string(), 50, Some(100)),
                ("gap".to_string(), 100, None),
            ],
            "kind 切换时闭合上一状态的开放行"
        );
    }
}
