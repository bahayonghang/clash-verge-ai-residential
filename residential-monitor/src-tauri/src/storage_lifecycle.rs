//! writer / controller 退役证明与有界回收。只由现有 coordinator 持有事务。

use super::*;
use crate::c0_contract::{RETRY_WINDOW_HOURS, RETRY_WINDOW_RECEIPTS};

impl StorageError {
    pub fn maintenance_report_error(&self, cancelled: bool) -> ReportError {
        if cancelled {
            return ReportError::Cancelled("辅助账本维护已取消");
        }
        match self {
            StorageError::Sqlite(error) => match error.sqlite_error_code() {
                Some(rusqlite::ErrorCode::OperationInterrupted) => {
                    ReportError::DeadlineExceeded("辅助账本维护让出采集时间")
                }
                Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked) => {
                    ReportError::StorageBusy("辅助账本维护等待空闲")
                }
                Some(rusqlite::ErrorCode::DiskFull) => {
                    ReportError::InsufficientSpace("辅助账本维护空间不足")
                }
                _ => ReportError::Failed("辅助账本维护失败，未完成事务已回滚"),
            },
            _ => ReportError::Failed("辅助账本维护失败，未完成事务已回滚"),
        }
    }
}

pub(super) fn bundle_expired(
    connection: &Connection,
    bundle: &CommitBundle,
) -> Result<bool, StorageError> {
    let expired: bool = connection.query_row(
        "select exists(select 1 from bundle_epoch where writer_epoch=?1
            and (expired_through_seq >= ?2 or retired_utc is not null))",
        params![bundle.writer_epoch as i64, bundle.bundle_seq as i64],
        |row| row.get(0),
    )?;
    Ok(expired)
}

pub(super) fn advance_contiguous(connection: &Connection, epoch: u64) -> Result<(), StorageError> {
    let old: i64 = connection.query_row(
        "select highest_contiguous_seq from bundle_epoch where writer_epoch=?1",
        [epoch as i64],
        |row| row.get(0),
    )?;
    let mut next = old;
    while next < i64::MAX
        && connection.query_row(
            "select exists(select 1 from committed_bundle
        where writer_epoch=?1 and bundle_seq=?2)",
            params![epoch as i64, next + 1],
            |row| row.get::<_, bool>(0),
        )?
    {
        next += 1;
    }
    if next != old {
        connection.execute(
            "update bundle_epoch set highest_contiguous_seq=?1 where writer_epoch=?2",
            params![next, epoch as i64],
        )?;
    }
    Ok(())
}

impl StorageCoordinator {
    /// 只查新进入活跃集合的 id；历史身份由已有唯一索引承担，核算器不保留无限 retired 集合。
    pub fn contains_session_id(&self, epoch: u64, ids: &[&str]) -> Result<bool, StorageError> {
        if ids.is_empty() {
            return Ok(false);
        }
        let mut statement = self.connection.prepare_cached("select exists(select 1 from connection_session where epoch_id=?1 and connection_id=?2)")?;
        for id in ids {
            if statement.query_row(params![epoch as i64, id], |row| row.get::<_, bool>(0))? {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// 同一操作的取消标记与 250ms 预算覆盖辅助扫描、边界和删除；每个事务独立回滚。
    pub fn cleanup_ledger_with_cancel(
        &mut self,
        now: i64,
        cutoff: i64,
        limit: usize,
        protected: &[String],
        cancel: &Arc<AtomicBool>,
    ) -> Result<(), StorageError> {
        if cancel.load(Ordering::SeqCst) {
            return Err(StorageError::Closed("cancelled".into()));
        }
        if !self.connection.is_autocommit() {
            return Err(StorageError::Closed(
                "maintenance requires idle writer".into(),
            ));
        }
        let started = std::time::Instant::now();
        let flag = Arc::clone(cancel);
        self.connection
            .busy_timeout(std::time::Duration::from_millis(200))?;
        self.connection.progress_handler(
            1,
            Some(move || {
                flag.load(Ordering::SeqCst)
                    || started.elapsed() >= std::time::Duration::from_millis(250)
            }),
        )?;
        let result = (|| {
            // 在硬中断前提交已处理前缀，给收据与 COMMIT 留出预算；取消仍由 handler 回滚。
            self.cleanup_retired_sessions_until(
                cutoff,
                limit,
                protected,
                Some(started + std::time::Duration::from_millis(100)),
            )?;
            if started.elapsed() < std::time::Duration::from_millis(150) {
                self.prune_receipts_until(
                    now,
                    limit,
                    &[],
                    Some(started + std::time::Duration::from_millis(200)),
                )?;
            }
            Ok(())
        })();
        self.connection.progress_handler(0, None::<fn() -> bool>)?;
        // 持续取消也会中断 Transaction::drop 的 ROLLBACK；移除 handler 后完成回滚，
        // 不把未提交的辅助删除留给下一次 writer 操作。
        let rollback = if result.is_err() && !self.connection.is_autocommit() {
            self.connection.execute_batch("rollback")
        } else {
            Ok(())
        };
        self.connection
            .busy_timeout(std::time::Duration::from_millis(u64::from(BUSY_TIMEOUT_MS)))?;
        rollback?;
        result
    }

    /// 仅桌面唯一 owner 在启动 / reopen 时调用：旧进程的内存队列已不可能继续提交。
    /// 不由通用 open 或 CLI 调用，不根据 epoch 数值大小猜测存活状态。
    pub fn retire_abandoned_generations(
        &mut self,
        current_writer: u64,
        now_utc: i64,
    ) -> Result<(), StorageError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            "update bundle_epoch set retired_utc=?1
            where writer_epoch != ?2 and retired_utc is null",
            params![now_utc, current_writer as i64],
        )?;
        transaction.execute(
            "update controller_epoch set retired_utc=?1 where retired_utc is null",
            [now_utc],
        )?;
        transaction.commit()?;
        Ok(())
    }

    /// generation 更换与退役同一事务；调用方须先解决所有 pending bundle。
    pub fn replace_controller_epoch(
        &mut self,
        previous: Option<u64>,
        core_identity: &str,
        now_utc: i64,
    ) -> Result<u64, StorageError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let highest: i64 = transaction.query_row(
            "select coalesce(max(epoch_id),0) from controller_epoch",
            [],
            |r| r.get(0),
        )?;
        let next = highest
            .checked_add(1)
            .filter(|value| *value > 0)
            .ok_or_else(|| StorageError::Closed("controller epoch exhausted".into()))?;
        transaction.execute(
            "insert into controller_epoch(epoch_id,core_identity) values (?1,?2)",
            params![next, core_identity],
        )?;
        if let Some(previous) = previous {
            transaction.execute("update controller_epoch set retired_utc=?1 where epoch_id=?2 and retired_utc is null",
                params![now_utc,previous as i64])?;
        }
        transaction.commit()?;
        Ok(next as u64)
    }

    /// 已关闭或有 owner 退役证明，且全部 raw 退出后，才回收辅助行；不伪造 legacy ended_utc。
    pub fn cleanup_retired_sessions(
        &mut self,
        cutoff_utc: i64,
        limit: usize,
        protected_identities: &[String],
    ) -> Result<usize, StorageError> {
        self.cleanup_retired_sessions_until(cutoff_utc, limit, protected_identities, None)
    }

    fn cleanup_retired_sessions_until(
        &mut self,
        cutoff_utc: i64,
        limit: usize,
        protected_identities: &[String],
        yield_at: Option<std::time::Instant>,
    ) -> Result<usize, StorageError> {
        let limit = limit.min(1_000);
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let building: bool =
            transaction.query_row("select exists(select 1 from retention_build)", [], |row| {
                row.get(0)
            })?;
        if building {
            return Ok(0);
        }
        let mut cursor = cleanup_cursor(&transaction, "session_cleanup_cursor")?;
        let mut scanned = 0;
        let mut deleted = 0;
        'pages: while scanned < limit {
            // 单页读取也有界，避免先获取 1000 行耗尽整段预算后只能全量回滚。
            let page_limit = (limit - scanned).min(128);
            let candidates: Vec<(i64, String, bool)> = {
                let mut statement = transaction.prepare_cached(
                "select s.session_pk, s.epoch_id || ':' || s.connection_id,
                  (s.started_utc < ?1
                  and (a.ended_utc <= ?1 or exists(select 1 from controller_epoch e
                    where e.epoch_id=s.epoch_id and e.retired_utc is not null))
                  and not exists(select 1 from connection_minute m where m.session_pk=s.session_pk)) is true
                from connection_session s left join connection_session_attr a using(session_pk)
                where s.session_pk > ?2 order by s.session_pk limit ?3",
            )?;
                let rows = statement
                    .query_map(params![cutoff_utc, cursor, page_limit as i64], |row| {
                        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                rows
            };
            let page_finished_scan = candidates.len() < page_limit;
            for (pk, identity, eligible) in candidates {
                cursor = pk;
                scanned += 1;
                if eligible && !protected_identities.contains(&identity) {
                    transaction
                        .prepare_cached("delete from connection_chain where session_pk=?1")?
                        .execute([pk])?;
                    transaction
                        .prepare_cached("delete from connection_session_attr where session_pk=?1")?
                        .execute([pk])?;
                    deleted += transaction
                        .prepare_cached("delete from connection_session where session_pk=?1")?
                        .execute([pk])?;
                }
                if yield_at.is_some_and(|deadline| std::time::Instant::now() >= deadline) {
                    break 'pages;
                }
            }
            if page_finished_scan {
                cursor = 0;
                break;
            }
        }
        save_cleanup_cursor(&transaction, "session_cleanup_cursor", cursor)?;
        if deleted > 0 {
            // 已扫描的字典页可能因本次 attr 删除而失去引用；与删除一起提交失效标记。
            transaction
                .prepare_cached(
                    "insert into retention_watermark(layer,watermark_utc,delete_watermark_utc)
                 values('dictionary_cleanup_cursor',-1,0)
                 on conflict(layer) do update set watermark_utc=-1",
                )?
                .execute([])?;
        }
        transaction.commit()?;
        Ok(deleted)
    }

    /// 保留最近 24h 与最新 100000 条的并集。边界 / 摘要与 DELETE 同事务，空 epoch 仍拒绝过期重放。
    pub fn prune_receipts(
        &mut self,
        now_utc: i64,
        limit: usize,
        protected_bundles: &[(u64, u64)],
    ) -> Result<usize, StorageError> {
        self.prune_receipts_until(now_utc, limit, protected_bundles, None)
    }

    fn prune_receipts_until(
        &mut self,
        now_utc: i64,
        limit: usize,
        protected_bundles: &[(u64, u64)],
        yield_at: Option<std::time::Instant>,
    ) -> Result<usize, StorageError> {
        let limit = limit.min(1_000);
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let newest_floor: Option<i64> = transaction
            .prepare_cached(
                "select data_version from committed_bundle
            order by data_version desc limit 1 offset ?1",
            )?
            .query_row([i64::from(RETRY_WINDOW_RECEIPTS) - 1], |row| row.get(0))
            .optional()?;
        let Some(newest_floor) = newest_floor else {
            return Ok(0);
        };
        let cutoff = now_utc.saturating_sub(i64::from(RETRY_WINDOW_HOURS) * 3600);
        let mut cursor = cleanup_cursor(&transaction, "receipt_cleanup_cursor")?;
        let mut scanned = 0;
        let mut deleted = 0;
        'pages: while scanned < limit {
            let page_limit = (limit - scanned).min(128);
            let candidates: Vec<(i64, i64, String, i64, bool)> = {
                let mut statement = transaction.prepare_cached(
                "select b.writer_epoch,b.bundle_seq,b.payload_hash,b.data_version,
                  (b.bundle_seq <= e.highest_contiguous_seq
                  and (b.committed_utc < ?2 or (b.committed_utc is null and e.retired_utc < ?2))) is true
                from committed_bundle b join bundle_epoch e using(writer_epoch)
                where b.data_version < ?1 and b.data_version > ?3
                order by b.data_version limit ?4",
            )?;
                let rows = statement
                    .query_map(
                        params![newest_floor, cutoff, cursor, page_limit as i64],
                        |row| {
                            Ok((
                                row.get(0)?,
                                row.get(1)?,
                                row.get(2)?,
                                row.get(3)?,
                                row.get(4)?,
                            ))
                        },
                    )?
                    .collect::<Result<Vec<_>, _>>()?;
                rows
            };
            let page_finished_scan = candidates.len() < page_limit;
            for (epoch, seq, hash, version, eligible) in candidates {
                cursor = version;
                scanned += 1;
                // pending 引用阻止越过该序号，即使它本身尚无收据。
                let protected = protected_bundles
                    .iter()
                    .any(|(e, s)| *e == epoch as u64 && *s <= seq as u64);
                if eligible && !protected {
                    let (expired,digest): (i64,String) = transaction.prepare_cached(
                        "select expired_through_seq,expired_payload_digest from bundle_epoch where writer_epoch=?1")?
                        .query_row([epoch], |row| Ok((row.get(0)?,row.get(1)?)))?;
                    if expired.checked_add(1) == Some(seq) {
                        let digest = hex::encode(Sha256::digest(
                            format!("{digest}:{seq}:{hash}").as_bytes(),
                        ));
                        transaction.prepare_cached("update bundle_epoch set expired_through_seq=?1,expired_payload_digest=?2 where writer_epoch=?3")?
                            .execute(params![seq,digest,epoch])?;
                        deleted += transaction.prepare_cached("delete from committed_bundle where writer_epoch=?1 and bundle_seq=?2")?
                            .execute(params![epoch,seq])?;
                    }
                }
                if yield_at.is_some_and(|deadline| std::time::Instant::now() >= deadline) {
                    break 'pages;
                }
            }
            if page_finished_scan {
                cursor = 0;
                break;
            }
        }
        save_cleanup_cursor(&transaction, "receipt_cleanup_cursor", cursor)?;
        transaction.commit()?;
        Ok(deleted)
    }
}

fn cleanup_cursor(connection: &Connection, key: &str) -> Result<i64, StorageError> {
    Ok(connection
        .query_row(
            "select cast(value as integer) from machine_setting where key=?1",
            [key],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or(0))
}

fn save_cleanup_cursor(
    connection: &Connection,
    key: &str,
    cursor: i64,
) -> Result<(), StorageError> {
    connection.execute(
        "insert into machine_setting(key,value) values(?1,?2)
        on conflict(key) do update set value=excluded.value where value is not excluded.value",
        params![key, cursor.to_string()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn bundle(epoch: u64, seq: u64) -> CommitBundle {
        CommitBundle {
            writer_epoch: epoch,
            bundle_seq: seq,
            payload: format!("opaque-{epoch}-{seq}"),
        }
    }

    fn newest_window(db: &StorageCoordinator) {
        db.connection().execute_batch("begin;
            with recursive seq(n) as (values(1) union all select n+1 from seq where n<100000)
            insert into committed_bundle(writer_epoch,bundle_seq,payload_hash,data_version,committed_utc)
                select 99,n,'new',n+100,1000000 from seq;
            insert into bundle_epoch(writer_epoch,highest_contiguous_seq,durable_watermark) values(99,100000,100100);
            commit;").unwrap();
    }

    #[test]
    fn receipt_union_protects_recent_count_pending_and_empty_epoch_replay() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ledger.sqlite3");
        let mut db = StorageCoordinator::open(&path).unwrap();
        for seq in 1..=3 {
            db.commit(&bundle(1, seq)).unwrap();
        }
        db.connection()
            .execute("update committed_bundle set committed_utc=1", [])
            .unwrap();
        newest_window(&db);
        db.connection().execute("update committed_bundle set committed_utc=999999 where writer_epoch=1 and bundle_seq=2", []).unwrap();
        assert_eq!(db.prune_receipts(1000000, 1000, &[]).unwrap(), 1);
        assert_eq!(db.prune_receipts(1000000, 1000, &[]).unwrap(), 0);
        db.connection()
            .execute(
                "update committed_bundle set committed_utc=1 where writer_epoch=1",
                [],
            )
            .unwrap();
        assert_eq!(db.prune_receipts(1000000, 1000, &[(1, 2)]).unwrap(), 0);
        assert_eq!(db.prune_receipts(1000000, 1000, &[]).unwrap(), 2);
        assert_eq!(db.receipt_count().unwrap(), 100000);
        drop(db);
        let mut reopened = StorageCoordinator::open(&path).unwrap();
        assert_eq!(reopened.watermark().unwrap(), 100100);
        assert_eq!(
            reopened
                .connection()
                .query_row("select watermark from data_version where id=1", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            0
        );
        assert_eq!(
            reopened.commit(&bundle(1, 1)).unwrap(),
            CommitOutcome::RetryWindowExpired
        );
        assert_eq!(
            reopened.commit(&bundle(1, 3)).unwrap(),
            CommitOutcome::RetryWindowExpired
        );
        assert_eq!(reopened.prune_receipts(2000000, 1000, &[]).unwrap(), 0);
        let digest: String = reopened
            .connection()
            .query_row(
                "select expired_payload_digest from bundle_epoch where writer_epoch=1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(!digest.is_empty());
        let outcome = reopened.commit(&bundle(100, 1)).unwrap();
        assert!(matches!(
            outcome,
            CommitOutcome::Applied(CommitReceipt {
                data_version: 100101,
                ..
            })
        ));
    }

    #[test]
    fn receipt_gap_cannot_be_mistaken_for_contiguous_commit() {
        let dir = tempdir().unwrap();
        let mut db = StorageCoordinator::open(&dir.path().join("ledger.sqlite3")).unwrap();
        db.commit(&bundle(1, 1)).unwrap();
        db.commit(&bundle(1, 3)).unwrap();
        let contiguous: i64 = db
            .connection()
            .query_row(
                "select highest_contiguous_seq from bundle_epoch where writer_epoch=1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(contiguous, 1);
        db.connection()
            .execute("update committed_bundle set committed_utc=1", [])
            .unwrap();
        newest_window(&db);
        assert_eq!(db.prune_receipts(1000000, 1000, &[]).unwrap(), 1);
        assert_eq!(db.prune_receipts(1000000, 1000, &[]).unwrap(), 0);
        assert!(matches!(
            db.commit(&bundle(1, 2)).unwrap(),
            CommitOutcome::Applied(_)
        ));
        let contiguous: i64 = db
            .connection()
            .query_row(
                "select highest_contiguous_seq from bundle_epoch where writer_epoch=1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(contiguous, 3);
    }

    #[test]
    fn legacy_receipt_requires_owner_retirement_and_full_observation_window() {
        let dir = tempdir().unwrap();
        let mut db = StorageCoordinator::open(&dir.path().join("ledger.sqlite3")).unwrap();
        db.commit(&bundle(1, 1)).unwrap();
        db.connection()
            .execute("update committed_bundle set committed_utc=null", [])
            .unwrap();
        newest_window(&db);
        assert_eq!(db.prune_receipts(1000000, 1000, &[]).unwrap(), 0);
        db.retire_abandoned_generations(99, 1000000).unwrap();
        assert_eq!(db.prune_receipts(1000000 + 86400, 1000, &[]).unwrap(), 0);
        let observed: Option<i64> = db
            .connection()
            .query_row(
                "select committed_utc from committed_bundle where writer_epoch=1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(observed, None);
        assert_eq!(db.prune_receipts(1000000 + 86401, 1000, &[]).unwrap(), 1);
        assert_eq!(
            db.commit(&bundle(1, 2)).unwrap(),
            CommitOutcome::RetryWindowExpired
        );
    }

    #[test]
    fn zero_fact_coverage_cannot_reopen_a_confirmed_day() {
        let dir = tempdir().unwrap();
        let mut db = StorageCoordinator::open(&dir.path().join("ledger.sqlite3")).unwrap();
        db.connection()
            .execute(
                "update retention_watermark set watermark_utc=86400 where layer='raw_delete'",
                [],
            )
            .unwrap();
        let slice = AlertCommitSlice {
            utc: 100,
            observed_interval: Some((99, 100)),
            ..Default::default()
        };
        assert!(db.commit_alert_bundle(&bundle(1, 1), &slice).is_err());
        assert_eq!(db.receipt_count().unwrap(), 0);
        assert_eq!(
            db.connection()
                .query_row("select count(*) from coverage_interval", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn session_fanout_yields_a_durable_prefix_before_hard_deadline() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("fanout.sqlite3");
        let mut db = StorageCoordinator::open(&path).unwrap();
        db.connection().execute_batch("insert into controller_epoch(epoch_id,core_identity,retired_utc) values(1,'old',1);
            with recursive ids(n) as (values(1) union all select n+1 from ids where n<1000)
            insert into connection_session(session_pk,epoch_id,connection_id,started_utc)
                select n,1,'session-'||n,1 from ids;
            insert into connection_session_attr(session_pk,started_utc,ended_utc,chain_key)
                select session_pk,1,2,'entry > exit' from connection_session;
            insert into connection_chain(session_pk,position,node)
                select session_pk,p.n,'node-'||p.n from connection_session
                cross join (select 0 n union all select 1 union all select 2) p;").unwrap();
        // 每个 eligible session 确定地产生耗时，旧的整批事务必定达到 250ms 后全回滚。
        db.connection()
            .create_scalar_function(
                "slow_session_delete",
                0,
                rusqlite::functions::FunctionFlags::SQLITE_UTF8
                    | rusqlite::functions::FunctionFlags::SQLITE_INNOCUOUS,
                |_| {
                    std::thread::sleep(std::time::Duration::from_millis(3));
                    Ok(0)
                },
            )
            .unwrap();
        db.connection().execute_batch("create temp trigger slow_session_delete after delete on connection_session_attr begin select slow_session_delete(); end;").unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let started = std::time::Instant::now();
        db.cleanup_ledger_with_cancel(1000000, 100, 1000, &[], &cancel)
            .unwrap();
        let first_elapsed = started.elapsed();
        let first = cleanup_cursor(db.connection(), "session_cleanup_cursor").unwrap();
        assert!((1..1000).contains(&first));
        assert!(db.connection().is_autocommit());
        assert_eq!(
            db.connection()
                .query_row("select min(session_pk) from connection_session", [], |r| {
                    r.get::<_, i64>(0)
                })
                .unwrap(),
            first + 1
        );
        assert_eq!(
            db.connection()
                .query_row("select count(*) from connection_chain", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            (1000 - first) * 3
        );
        drop(db);
        let mut reopened = StorageCoordinator::open(&path).unwrap();
        assert_eq!(
            cleanup_cursor(reopened.connection(), "session_cleanup_cursor").unwrap(),
            first
        );
        let mut remaining = 1000 - first;
        for _ in 0..100 {
            reopened
                .cleanup_ledger_with_cancel(1000000, 100, 1000, &[], &cancel)
                .unwrap();
            assert!(reopened.connection().is_autocommit());
            let next = reopened
                .connection()
                .query_row("select count(*) from connection_session", [], |r| {
                    r.get::<_, i64>(0)
                })
                .unwrap();
            assert!(next < remaining, "每次恢复 tick 都应推进已收到的清理前缀");
            remaining = next;
            if remaining == 0 {
                break;
            }
        }
        assert_eq!(remaining, 0);
        eprintln!("有界 session 前缀删除 {first} 行，用时 {first_elapsed:?}，reopen 后继续清空");
    }

    #[test]
    fn receipt_yield_persists_only_the_processed_prefix_and_digest() {
        let dir = tempdir().unwrap();
        let mut db = StorageCoordinator::open(&dir.path().join("receipt-prefix.sqlite3")).unwrap();
        for seq in 1..=3 {
            db.commit(&bundle(1, seq)).unwrap();
        }
        db.connection()
            .execute("update committed_bundle set committed_utc=1", [])
            .unwrap();
        newest_window(&db);
        assert_eq!(
            db.prune_receipts_until(1000000, 1000, &[], Some(std::time::Instant::now()))
                .unwrap(),
            1
        );
        assert_eq!(
            cleanup_cursor(db.connection(), "receipt_cleanup_cursor").unwrap(),
            1
        );
        let expected = hex::encode(Sha256::digest(
            format!(":1:{}", bundle(1, 1).payload_hash()).as_bytes(),
        ));
        let proof:(i64,String) = db.connection().query_row("select expired_through_seq,expired_payload_digest from bundle_epoch where writer_epoch=1",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_eq!(proof, (1, expected));
        assert_eq!(db.prune_receipts(1000000, 1000, &[]).unwrap(), 2);
        assert_eq!(db.receipt_count().unwrap(), 100000);
        assert_eq!(
            db.commit(&bundle(1, 1)).unwrap(),
            CommitOutcome::RetryWindowExpired
        );
    }

    #[test]
    fn yielded_protected_session_prefix_advances_without_deleting() {
        let dir = tempdir().unwrap();
        let mut db =
            StorageCoordinator::open(&dir.path().join("protected-prefix.sqlite3")).unwrap();
        db.connection().execute_batch("insert into controller_epoch(epoch_id,core_identity,retired_utc) values(1,'old',1);
            insert into connection_session(session_pk,epoch_id,connection_id,started_utc) values(1,1,'protected',1),(2,1,'eligible',1);").unwrap();
        assert_eq!(
            db.cleanup_retired_sessions_until(
                100,
                1000,
                &["1:protected".into()],
                Some(std::time::Instant::now())
            )
            .unwrap(),
            0
        );
        assert_eq!(
            cleanup_cursor(db.connection(), "session_cleanup_cursor").unwrap(),
            1
        );
        assert_eq!(
            db.cleanup_retired_sessions(100, 1000, &["1:protected".into()])
                .unwrap(),
            1
        );
        assert_eq!(
            cleanup_cursor(db.connection(), "session_cleanup_cursor").unwrap(),
            0
        );
    }

    #[test]
    fn cancellation_during_session_delete_rolls_back_rows_and_scan_cursor() {
        let dir = tempdir().unwrap();
        let mut db = StorageCoordinator::open(&dir.path().join("ledger.sqlite3")).unwrap();
        db.connection().execute_batch("insert into controller_epoch(epoch_id,core_identity,retired_utc) values(1,'old',1000);
            insert into connection_session(session_pk,epoch_id,connection_id,started_utc) values(1,1,'a',1),(2,1,'b',1);").unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancel);
        db.connection()
            .create_scalar_function(
                "cancel_cleanup",
                0,
                rusqlite::functions::FunctionFlags::SQLITE_UTF8
                    | rusqlite::functions::FunctionFlags::SQLITE_INNOCUOUS,
                move |_| {
                    flag.store(true, Ordering::SeqCst);
                    Ok(1_i64)
                },
            )
            .unwrap();
        db.connection().execute_batch("create temp trigger cancel_cleanup after delete on connection_session begin select cancel_cleanup(); end;").unwrap();
        assert!(db
            .cleanup_ledger_with_cancel(1000, 100, 1000, &[], &cancel)
            .is_err());
        assert!(cancel.load(Ordering::SeqCst));
        assert_eq!(
            db.connection()
                .query_row("select count(*) from connection_session", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert_eq!(
            cleanup_cursor(db.connection(), "session_cleanup_cursor").unwrap(),
            0
        );
        assert!(db.connection().is_autocommit());
    }

    #[test]
    fn cancellation_during_receipt_delete_rolls_back_expiry_digest() {
        let dir = tempdir().unwrap();
        let mut db = StorageCoordinator::open(&dir.path().join("ledger.sqlite3")).unwrap();
        db.commit(&bundle(1, 1)).unwrap();
        db.connection()
            .execute("update committed_bundle set committed_utc=1", [])
            .unwrap();
        newest_window(&db);
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancel);
        db.connection()
            .create_scalar_function(
                "cancel_receipt",
                0,
                rusqlite::functions::FunctionFlags::SQLITE_UTF8
                    | rusqlite::functions::FunctionFlags::SQLITE_INNOCUOUS,
                move |_| {
                    flag.store(true, Ordering::SeqCst);
                    Ok(1_i64)
                },
            )
            .unwrap();
        db.connection().execute_batch("create temp trigger cancel_receipt after delete on committed_bundle begin select cancel_receipt(); end;").unwrap();
        assert!(db
            .cleanup_ledger_with_cancel(1000000, 100, 1000, &[], &cancel)
            .is_err());
        assert!(cancel.load(Ordering::SeqCst));
        assert_eq!(db.receipt_count().unwrap(), 100001);
        let state: (i64,String) = db.connection().query_row("select expired_through_seq,expired_payload_digest from bundle_epoch where writer_epoch=1", [], |r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_eq!(state, (0, String::new()));
        assert_eq!(
            cleanup_cursor(db.connection(), "receipt_cleanup_cursor").unwrap(),
            0
        );
        assert!(db.connection().is_autocommit());
    }

    #[test]
    fn bounded_session_scan_advances_past_protected_prefix() {
        let dir = tempdir().unwrap();
        let mut db = StorageCoordinator::open(&dir.path().join("ledger.sqlite3")).unwrap();
        db.connection().execute_batch("insert into controller_epoch(epoch_id,core_identity,retired_utc) values(1,'old',1000);
            with recursive ids(n) as (values(1) union all select n+1 from ids where n<1500)
            insert into connection_session(session_pk,epoch_id,connection_id,started_utc)
                select n,case when n<=1000 then 999 else 1 end,'row-'||n,1 from ids;").unwrap();
        assert_eq!(db.cleanup_retired_sessions(100, 1000, &[]).unwrap(), 0);
        assert_eq!(db.cleanup_retired_sessions(100, 1000, &[]).unwrap(), 500);
        let cancelled = Arc::new(AtomicBool::new(true));
        assert!(db
            .cleanup_ledger_with_cancel(1000, 100, 1000, &[], &cancelled)
            .is_err());
    }

    #[test]
    fn v4_migration_preserves_unknown_time_and_reconstructs_real_prefix() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("old.sqlite3");
        let legacy = legacy_v4_fixture(&path);
        legacy.execute_batch("insert into bundle_epoch(writer_epoch,highest_contiguous_seq,durable_watermark) values(1,3,2);
            insert into committed_bundle(writer_epoch,bundle_seq,payload_hash,data_version) values(1,1,'a',1),(1,3,'b',2);").unwrap();
        drop(legacy);
        let db = StorageCoordinator::open(&path).unwrap();
        let prefix: i64 = db
            .connection()
            .query_row(
                "select highest_contiguous_seq from bundle_epoch where writer_epoch=1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(prefix, 1);
        let unknown: i64 = db
            .connection()
            .query_row(
                "select count(*) from committed_bundle where committed_utc is null",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(unknown, 2);
        assert_eq!(db.watermark().unwrap(), 2);
        let versions: Vec<(i64, i64)> = db
            .connection()
            .prepare("select rowid,data_version from committed_bundle order by data_version")
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(versions, vec![(1, 1), (2, 2)]);
        assert!(db.connection().execute("insert into committed_bundle(writer_epoch,bundle_seq,payload_hash,data_version) values(2,1,'duplicate-version',2)", []).is_err());
        assert!(db.connection().execute("insert into committed_bundle(writer_epoch,bundle_seq,payload_hash,data_version) values(1,1,'duplicate-identity',3)", []).is_err());
    }

    #[test]
    fn invalid_legacy_duplicate_versions_fail_migration_atomically() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("duplicate-version.sqlite3");
        let legacy = legacy_v4_fixture(&path);
        // 真实 v4 是复合主键，允许该损坏状态；v5 必须拒绝，不能重编号收据。
        legacy
            .execute_batch(
                "insert into committed_bundle(writer_epoch,bundle_seq,payload_hash,data_version)
            values(1,1,'first',7),(1,2,'second',7);",
            )
            .unwrap();
        drop(legacy);
        assert!(StorageCoordinator::open(&path).is_err());
        let unchanged = Connection::open(&path).unwrap();
        assert_eq!(
            unchanged
                .query_row("pragma user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            4
        );
        assert_eq!(
            unchanged
                .query_row(
                    "select count(*) from committed_bundle where data_version=7",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            2
        );
        assert_eq!(
            unchanged
                .query_row(
                    "select count(*) from sqlite_master where name='committed_bundle_v5'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        assert_eq!(unchanged.query_row("select count(*) from pragma_table_info('committed_bundle') where name='committed_utc'", [], |r|r.get::<_,i64>(0)).unwrap(),0);
        assert_eq!(
            unchanged
                .query_row(
                    "select count(*) from schema_migration where version=5",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
    }

    #[test]
    fn session_cleanup_protects_legacy_unknown_zero_traffic_raw_and_pending() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ledger.sqlite3");
        let legacy = legacy_v4_fixture(&path);
        legacy.execute_batch("insert into controller_epoch(epoch_id,core_identity) values(1,'old');
            insert into connection_session(session_pk,epoch_id,connection_id,started_utc) values
                (1,1,'legacy',1),(2,1,'raw',1),(3,88,'unknown',1),(4,1,'pending',1);
            insert into connection_session_attr(session_pk,started_utc) values(1,1),(2,1),(3,1),(4,1);
            insert into connection_chain(session_pk,position,node) values(1,0,'DIRECT'),(4,0,'DIRECT');
            insert into connection_minute(utc_minute,session_pk,upload,download) values(1,2,1,1);").unwrap();
        drop(legacy);
        let mut db = StorageCoordinator::open(&path).unwrap();
        assert_eq!(db.cleanup_retired_sessions(100, 100, &[]).unwrap(), 0);
        db.retire_abandoned_generations(1, 1000).unwrap();
        let active = db.replace_controller_epoch(None, "current", 1000).unwrap();
        db.connection().execute("insert into connection_session(session_pk,epoch_id,connection_id,started_utc) values(5,?1,'zero',1)", [active as i64]).unwrap();
        assert_eq!(
            db.cleanup_retired_sessions(100, 100, &["1:pending".into()])
                .unwrap(),
            1
        );
        assert_eq!(
            db.connection()
                .query_row(
                    "select count(*) from connection_chain where session_pk=1",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        assert_eq!(
            db.connection()
                .query_row("select count(*) from connection_session", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            4
        );
        assert_eq!(
            db.cleanup_retired_sessions(100, 100, &["1:pending".into()])
                .unwrap(),
            0
        );
    }
}
