//! 生产 schema 容量 fixture。批量生成完整数据行，不冒充 1 Hz 门面回放或安装态证据。

use super::facade::{file_bytes, table_inventory_on};
use super::process;
use crate::c3::query::{default_auto_report_query, Granularity, ReportError, AUTO_DELETE_ENABLED};
use crate::c3::retention::{RetentionMode, RetentionService};
use crate::c3::service::run_uncached;
use crate::c3::space::SpaceBudget;
use crate::storage::StorageCoordinator;
use crate::workload::WorkloadSpec;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

const MANIFEST: &str = "production-corpus.json";
const DAY: i64 = 86_400;

/// days=0 生成三分钟小 fixture。完整容量门必须显式生成 days=30。
pub fn generate_corpus(
    dir: &Path,
    active: u32,
    days: u32,
    seed: u64,
    start_utc: i64,
) -> Result<Value, String> {
    if active == 0 || active > 100_000 || days > 1000 || start_utc < 0 {
        return Err("active 需为 1..100000，days 为 0..1000，start-utc 非负".into());
    }
    std::fs::create_dir_all(dir).map_err(err)?;
    if std::fs::read_dir(dir).map_err(err)?.next().is_some() {
        return Err("容量 fixture 必须使用空隔离目录".into());
    }
    let mut spec = WorkloadSpec::profile_full(active, days);
    spec.seed = seed;
    let minutes = spec.duration_minutes() as i64;
    let start = start_utc.div_euclid(DAY) * DAY;
    let end = start + minutes * 60;
    let mut db = StorageCoordinator::open(&dir.join("monitor.sqlite3")).map_err(err)?;
    let writer = db.reserve_writer_epoch().map_err(err)? as i64;
    let epoch = db
        .reserve_controller_epoch("benchmark-corpus")
        .map_err(err)? as i64;
    let c = db.connection();
    // 仅此批量 fixture writer 扩大页缓存，保留生产 WAL/FULL 与既有事务边界。
    c.execute_batch("pragma cache_size=-65536").map_err(err)?;
    let schema: i64 = c
        .query_row("pragma user_version", [], |r| r.get(0))
        .map_err(err)?;
    if schema < 5 {
        return Err("生产容量 fixture 要求含 receipt 时间的 v5 schema".into());
    }
    let started = Instant::now();
    let native_before = process::sample();
    seed_dimensions(c, &spec)?;
    c.execute_batch("insert or replace into target_set values(1,1); insert or replace into target_item values(1,0,'家宽');").map_err(err)?;
    let mut session = c.prepare(
        "with recursive ids(n) as (values(1) union all select n+1 from ids where n<?1)
         insert into connection_session(session_pk,epoch_id,connection_id,started_utc,host)
         select ?2+n,?3,'corpus-'||(?2+n),?4,
           (select value from dimension_dict where dimension_kind='host' and dimension_id=((?2+n+?5)%800)+1) from ids"
    ).map_err(err)?;
    let mut attr = c.prepare(
        "insert into connection_session_attr(session_pk,host_id,process_id,rule_id,network_id,chain_key,policy_version,primary_category_id,started_utc,ended_utc)
         select session_pk,((session_pk+?1)%800)+1,((session_pk+?1)%120)+1,((session_pk+?1)%40)+1,((session_pk+?1)%4)+1,
          case when (session_pk+?1)%4=0 then '机场' else '家宽' end||' > intermediate-'||((session_pk+?1)%60)||' > exit-'||((session_pk+?1)%60),
          1,case when (session_pk+?1)%4=0 then null else 1 end,started_utc,?4
         from connection_session where session_pk>?2 and session_pk<=?3"
    ).map_err(err)?;
    let mut chain = c.prepare(
        "insert into connection_chain(session_pk,position,node)
         select session_pk,p.n,case p.n when 0 then case when (session_pk+?1)%4=0 then '机场' else '家宽' end
          when 1 then 'intermediate-'||((session_pk+?1)%60) else 'exit-'||((session_pk+?1)%60) end
         from connection_session cross join (select 0 n union all select 1 union all select 2) p
         where session_pk>?2 and session_pk<=?3"
    ).map_err(err)?;
    let mut minute = c
        .prepare(
            "with recursive mins(n) as (values(?1) union all select n+1 from mins where n+1<?2)
         insert into connection_minute(utc_minute,session_pk,upload,download)
         select n,session_pk,8+((session_pk+?3)%13),32+((session_pk+?3)%31)
         from mins cross join connection_session where session_pk>?4 and session_pk<=?5",
        )
        .map_err(err)?;
    let mut receipts = c.prepare(
        "with recursive seq(n) as (values(?1) union all select n+1 from seq where n<?2)
         insert into committed_bundle(writer_epoch,bundle_seq,payload_hash,data_version,committed_utc)
         select ?3,n,?4,n,?5+n-1 from seq"
    ).map_err(err)?;
    // 参数只控制确定性分布；限制在有符号整型内，避免 SQLite 的隐式浮点取模。
    let distribution_seed = (seed % 1_000_000_000) as i64;
    let receipt_hash = hex::encode(Sha256::digest(b"isolated-production-corpus-receipt"));
    let mut pk = 0_i64;
    let mut progress = Vec::new();
    for hour_minute in (0..minutes).step_by(60) {
        c.execute_batch("begin immediate").map_err(err)?;
        let result = (|| -> rusqlite::Result<()> {
            for begin in (hour_minute..(hour_minute + 60).min(minutes)).step_by(5) {
                let finish = (begin + 5).min(minutes);
                let next = pk + i64::from(active);
                session.execute(params![
                    active,
                    pk,
                    epoch,
                    start + begin * 60,
                    distribution_seed
                ])?;
                attr.execute(params![distribution_seed, pk, next, start + finish * 60])?;
                chain.execute(params![distribution_seed, pk, next])?;
                minute.execute(params![
                    start / 60 + begin,
                    start / 60 + finish,
                    distribution_seed,
                    pk,
                    next
                ])?;
                pk = next;
            }
            let through = (hour_minute + 60).min(minutes) * 60;
            receipts.execute(params![
                hour_minute * 60 + 1,
                through,
                writer,
                receipt_hash,
                start
            ])?;
            c.execute("insert into coverage_interval(kind,reason,started_utc,ended_utc) values('covered','synthetic-complete-hour',?1,?2)",params![start+hour_minute*60,start+through])?;
            c.execute("update bundle_epoch set highest_contiguous_seq=?1,durable_watermark=?1 where writer_epoch=?2",params![through,writer])?;
            c.execute_batch("commit")?;
            Ok(())
        })();
        if let Err(error) = result {
            let rollback = c.execute_batch("rollback");
            return Err(format!(
                "容量生成失败：{error}；回滚={rollback:?}；已提交小时保留在隔离目录"
            ));
        }
        if (hour_minute + 60) % 1440 == 0 || hour_minute + 60 >= minutes {
            progress.push(json!({"minutes": (hour_minute+60).min(minutes),"wall_secs": started.elapsed().as_secs_f64(),"native":process::sample()}));
        }
    }
    c.execute(
        "update controller_epoch set retired_utc=?1 where epoch_id=?2",
        params![end, epoch],
    )
    .map_err(err)?;
    c.execute(
        "update bundle_epoch set retired_utc=?1 where writer_epoch=?2",
        params![end, writer],
    )
    .map_err(err)?;
    drop(session);
    drop(attr);
    drop(chain);
    drop(minute);
    drop(receipts);
    c.execute_batch("pragma wal_checkpoint(truncate)")
        .map_err(err)?;
    let wall_secs = started.elapsed().as_secs_f64();
    let native_after = process::sample();
    let count = |table: &str| {
        c.query_row(&format!("select count(*) from {table}"), [], |r| {
            r.get::<_, i64>(0)
        })
        .map_err(err)
    };
    let actual_sessions = count("connection_session")?;
    let actual_minutes = count("connection_minute")?;
    let actual_chains = count("connection_chain")?;
    let actual_receipts = count("committed_bundle")?;
    let counts_match = actual_sessions == pk
        && actual_minutes == i64::from(active) * minutes
        && actual_chains == pk * 3
        && actual_receipts == minutes * 60;
    if !counts_match {
        return Err("完整容量行数与协议不符".into());
    }
    let manifest = json!({"schema_version":1,"kind":"production-corpus","sqlite_schema_version":schema,
        "workload":spec,"spec_hash":spec.manifest_hash(),"start_utc":start,"end_utc":end,
        "minutes":minutes,"full_30_day_input": days==30,"generation_wall_secs":wall_secs,
        "native_before":native_before,"native_after":native_after,"generation_cache_kib":65536,
        "expected":{"sessions":pk,"minutes":i64::from(active)*minutes,"chains":pk*3,"receipts":minutes*60},
        "actual":{"sessions":actual_sessions,"minutes":actual_minutes,"chains":actual_chains,"receipts":actual_receipts},
        "counts_match":counts_match,"files_and_pages":page_inventory(c,dir)?,
        "tables_and_indexes":table_inventory_on(c)?,"progress":progress,
        "limits":["通过生产 migration 建库，批量写入生产表；不代表真实门面提交或1Hz实时耗时。",
        "生成专用连接使用64MiB页缓存；仅用于批量fixture生成，其内存样本不代表生产运行时占用。",
        "固定5分钟会话寿命与3跳chain，复用C0维度基数；所有session已关闭，未伪造活跃保护/legacy证明。",
        "receipt为显式合成的逐秒连续序列，payload_hash是固定fixture摘要；不用于重放安全证明。",
        "进程路径未写入生产schema；host按5%长标签、process按进程名分布。尚未物化任何派生汇总。"]});
    std::fs::write(
        dir.join(MANIFEST),
        serde_json::to_vec_pretty(&manifest).map_err(err)?,
    )
    .map_err(err)?;
    Ok(manifest)
}

fn seed_dimensions(c: &Connection, spec: &WorkloadSpec) -> Result<(), String> {
    let mut insert = c
        .prepare("insert into dimension_dict(dimension_kind,dimension_id,value) values(?1,?2,?3)")
        .map_err(err)?;
    for (kind, n) in [
        ("host", spec.domain_cardinality),
        ("process", spec.process_cardinality),
        ("rule", spec.rule_cardinality),
        ("network", spec.network_cardinality),
        ("chain", spec.chain_cardinality),
        ("rule_group", spec.chain_cardinality),
    ] {
        for id in 1..=n {
            let value = match kind {
                "host" if id % 20 == 0 => format!("{}.host-{id}.example.test", "a".repeat(40)),
                "host" => format!("host-{id}.example.test"),
                "process" => format!("process-{id}.exe"),
                "chain" | "rule_group" => format!("exit-{}", id - 1),
                _ => format!("{kind}-{id}"),
            };
            insert.execute(params![kind, id, value]).map_err(err)?;
        }
    }
    insert
        .execute(params!["category", 1, "家宽"])
        .map_err(err)?;
    Ok(())
}

pub fn retain_corpus(
    dir: &Path,
    now_utc: i64,
    raw_days: i64,
    max_chunks: u32,
    delete: bool,
) -> Result<Value, String> {
    retain_corpus_inner(
        dir,
        now_utc,
        raw_days,
        max_chunks,
        delete,
        CorpusGate::Production,
    )
}

#[derive(Clone, Copy)]
enum CorpusGate {
    Production,
    #[cfg(test)]
    IsolatedTest,
}

fn retain_corpus_inner(
    dir: &Path,
    now_utc: i64,
    raw_days: i64,
    max_chunks: u32,
    delete: bool,
    gate: CorpusGate,
) -> Result<Value, String> {
    let (deletion_enabled, isolated_test_gate) = match gate {
        CorpusGate::Production => (AUTO_DELETE_ENABLED, false),
        #[cfg(test)]
        CorpusGate::IsolatedTest => (true, true),
    };
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(dir.join(MANIFEST)).map_err(err)?).map_err(err)?;
    if manifest["kind"] != "production-corpus" || max_chunks == 0 {
        return Err("只接受此工具生成的容量 fixture 与正数 chunks".into());
    }
    let mut db = StorageCoordinator::open(&dir.join("monitor.sqlite3")).map_err(err)?;
    let before = page_inventory(db.connection(), dir)?;
    let cutoff = (now_utc - raw_days.clamp(1, 90) * DAY).div_euclid(DAY) * DAY;
    let ledger_before = lifecycle_inventory(db.connection(), now_utc, cutoff)?;
    let cancel = Arc::new(AtomicBool::new(false));
    let mut chunks = Vec::new();
    let started = Instant::now();
    let mut complete = false;
    let mut cleanup_pass = CleanupPass::default();
    let mut completion_inventory_checks = 0_u64;
    let mut retry_attempts = 0_u32;
    let mut staging_bytes_sampled_max = None::<u64>;
    let mut wal_bytes_sampled_max = 0;
    for index in 0..max_chunks {
        let tick = Instant::now();
        let mode = if delete {
            RetentionMode::DeleteEnabled
        } else {
            RetentionMode::MaterializeOnly
        };
        let result = match gate {
            CorpusGate::Production => RetentionService::run_chunk(
                &mut db,
                now_utc,
                raw_days,
                mode,
                &SpaceBudget::default(),
                &cancel,
            ),
            #[cfg(test)]
            CorpusGate::IsolatedTest => RetentionService::run_chunk_for_benchmark(
                &mut db,
                now_utc,
                raw_days,
                mode,
                &SpaceBudget::default(),
                &cancel,
            ),
        };
        match result {
            Ok(chunk) => {
                complete = !chunk.more_pending;
                if delete && deletion_enabled {
                    // fixture 明确没有活跃连接或 pending bundle，复用生产的取消/时间预算及清理入口。
                    if let Err(error) =
                        db.cleanup_ledger_with_cancel(now_utc, cutoff, 1000, &[], &cancel)
                    {
                        let mapped = error.maintenance_report_error(cancel.load(Ordering::SeqCst));
                        let writer_autocommit = db.connection().is_autocommit();
                        let retryable = maintenance_retryable(&mapped, &cancel, writer_autocommit);
                        chunks.push(json!({"error":"ledger_cleanup","error_code":mapped.code(),"message":error.to_string(),
                            "retryable":retryable,"writer_autocommit":writer_autocommit,"day_utc":chunk.day_utc,"deleted_raw_rows":chunk.deleted_raw_rows,
                            "more_pending":chunk.more_pending,"wall_ms":tick.elapsed().as_secs_f64()*1000.0}));
                        complete = false;
                        if retryable {
                            // session 前缀可能已提交，随后收据步骤超时；仍记录持久游标归零。
                            cleanup_pass.observe(db.connection())?;
                            retry_attempts += 1;
                            continue;
                        }
                        break;
                    }
                    cleanup_pass.observe(db.connection())?;
                }
                chunks.push(json!({"day_utc":chunk.day_utc,"deleted_raw_rows":chunk.deleted_raw_rows,"more_pending":chunk.more_pending,"wall_ms":tick.elapsed().as_secs_f64()*1000.0}));
                wal_bytes_sampled_max =
                    wal_bytes_sampled_max.max(file_bytes(&dir.join("monitor.sqlite3-wal"))?);
                if index % 32 == 0 || complete {
                    if let Some(bytes) = staging_bytes(db.connection())? {
                        staging_bytes_sampled_max =
                            Some(staging_bytes_sampled_max.unwrap_or(0).max(bytes));
                    }
                }
                if delete && deletion_enabled {
                    // 各扫描异步结束；账本完成信号保留到 raw/字典/覆盖均允许旁证时再消费。
                    let auxiliary_pending =
                        RetentionService::auxiliary_cleanup_pending(db.connection())
                            .map_err(err)?;
                    complete = cleanup_pass.take_completed(chunk.more_pending || auxiliary_pending);
                    if complete {
                        completion_inventory_checks += 1;
                        let ledger = lifecycle_inventory(db.connection(), now_utc, cutoff)?;
                        complete = ledger["reclaimable_unreferenced_sessions"] == 0
                            && ledger["eligible_receipts"] == 0;
                    }
                }
                if complete {
                    break;
                }
            }
            Err(error) => {
                let writer_autocommit = db.connection().is_autocommit();
                let retryable = maintenance_retryable(&error, &cancel, writer_autocommit);
                chunks.push(json!({"error":error.code(),"message":error.to_string(),"retryable":retryable,"writer_autocommit":writer_autocommit,"wall_ms":tick.elapsed().as_secs_f64()*1000.0}));
                complete = false;
                if retryable {
                    retry_attempts += 1;
                    continue;
                }
                break;
            }
        }
    }
    let end = manifest["end_utc"].as_i64().ok_or("fixture end 缺失")?;
    let first = manifest["start_utc"].as_i64().ok_or("fixture start 缺失")?;
    let query = default_auto_report_query(Granularity::Hour, (end - DAY).max(first), end);
    let query_start = Instant::now();
    let query_outcome = match run_uncached(db.path(), query, now_utc, raw_days, &cancel, None) {
        Ok(result) => json!({"ok":true,"totals":result.totals,"tier":result.data_tier}),
        Err(error) => json!({"ok":false,"error":error.code(),"message":error.to_string()}),
    };
    let query_wall_ms = query_start.elapsed().as_secs_f64() * 1000.0;
    let quick: String = db
        .connection()
        .query_row("pragma quick_check", [], |r| r.get(0))
        .map_err(err)?;
    Ok(
        json!({"kind":"production-corpus-retention","now_utc":now_utc,"raw_retain_days":raw_days,
        "delete_requested":delete,"auto_delete_enabled":AUTO_DELETE_ENABLED,
        "isolated_test_delete_gate":isolated_test_gate,
        "complete":complete && (!delete || deletion_enabled),"work_queue_empty":complete,
        "wall_secs":started.elapsed().as_secs_f64(),"chunks":chunks,"before":before,"after":page_inventory(db.connection(),dir)?,
        "ledger_before":ledger_before,"ledger_after":lifecycle_inventory(db.connection(),now_utc,cutoff)?,
        "staging_bytes_sampled_max":staging_bytes_sampled_max,"wal_bytes_sampled_max":wal_bytes_sampled_max,
        "completion_inventory_checks":completion_inventory_checks,
        "retry_attempts":retry_attempts,"max_chunks":max_chunks,
        "query_wall_ms":query_wall_ms,"query":query_outcome,"quick_check":quick,
        "tables_and_indexes":table_inventory_on(db.connection())?,
        "limits":[if isolated_test_gate { "本次仅用cfg(test)隔离删除门；生产AUTO_DELETE_ENABLED状态单列，不构成开放生产删除的授权。" } else { "调用生产RetentionService与cleanup_ledger_with_cancel，删除仍受AUTO_DELETE_ENABLED控制。complete=false不构成容量门通过。" },
        "staging字节每32块及raw队列完成时采样，仅是观测最大值，不能证明块内瞬时峰值。",
        "仅重试生产分类中的deadline_exceeded/storage_busy，每次占用既有chunks上限；这是加速维护驱动，不模拟生产错误后的60秒调度间隔。",
        "此命令不自动VACUUM，不声称freelist已释放到操作系统，不代替故障/legacy/安装态soak验证。"]}),
    )
}

fn maintenance_retryable(
    error: &ReportError,
    cancel: &AtomicBool,
    writer_autocommit: bool,
) -> bool {
    writer_autocommit
        && !cancel.load(Ordering::SeqCst)
        && matches!(
            error,
            ReportError::DeadlineExceeded(_) | ReportError::StorageBusy(_)
        )
}

#[derive(Default)]
struct CleanupPass {
    session_end_seen: bool,
    receipt_end_seen: bool,
}

impl CleanupPass {
    fn observe(&mut self, connection: &Connection) -> Result<(), String> {
        let (session_cursor, receipt_cursor): (i64, i64) = connection
            .query_row(
                "select
                  coalesce((select cast(value as integer) from machine_setting where key='session_cleanup_cursor'),0),
                  coalesce((select cast(value as integer) from machine_setting where key='receipt_cleanup_cursor'),0)",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(err)?;
        self.session_end_seen |= session_cursor == 0;
        self.receipt_end_seen |= receipt_cursor == 0;
        Ok(())
    }

    fn take_completed(&mut self, more_pending: bool) -> bool {
        let finished = !more_pending && self.session_end_seen && self.receipt_end_seen;
        if finished {
            *self = Self::default();
        }
        finished
    }
}

fn staging_bytes(c: &Connection) -> Result<Option<u64>, String> {
    let Ok(mut bytes) = c.prepare("select coalesce(sum(pgsize),0) from dbstat where name=?1")
    else {
        return Ok(None);
    };
    let mut names=c.prepare("select name from sqlite_master where name like 'retention_build%' or (type='index' and tbl_name like 'retention_build%')").map_err(err)?;
    let mut total = 0_u64;
    for row in names
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(err)?
    {
        total += bytes
            .query_row([row.map_err(err)?], |r| {
                r.get::<_, i64>(0).map(|n| n as u64)
            })
            .map_err(err)?;
    }
    Ok(Some(total))
}

fn lifecycle_inventory(c: &Connection, now: i64, cutoff: i64) -> Result<Value, String> {
    let (total,unreferenced,reclaimable,closed):(i64,i64,i64,i64)=c.query_row(
        "select count(*),
          coalesce(sum(not exists(select 1 from connection_minute m where m.session_pk=s.session_pk)),0),
          coalesce(sum(not exists(select 1 from connection_minute m where m.session_pk=s.session_pk)
            and ((a.ended_utc is not null and a.ended_utc<=?1) or (a.ended_utc is null and e.retired_utc is not null and e.retired_utc<=?1))),0),
          coalesce(sum(a.ended_utc is not null),0)
         from connection_session s left join connection_session_attr a on a.session_pk=s.session_pk
         left join controller_epoch e on e.epoch_id=s.epoch_id",[cutoff],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(err)?;
    let receipts: i64 = c
        .query_row("select count(*) from committed_bundle", [], |r| r.get(0))
        .map_err(err)?;
    let eligible:i64=c.query_row(
        "select count(*) from committed_bundle where committed_utc<?1 and data_version<
          (select data_version from committed_bundle order by data_version desc limit 1 offset 99999)",
        [now-86400],|r|r.get(0)).map_err(err)?;
    Ok(
        json!({"sessions":total,"closed_sessions":closed,"unreferenced_sessions":unreferenced,
        "reclaimable_unreferenced_sessions":reclaimable,"protected_by_raw_sessions":total-unreferenced,
        "unreferenced_without_retirement_or_cutoff_proof":unreferenced-reclaimable,
        "receipts":receipts,"eligible_receipts":eligible,"active_or_pending_fixture_references":0}),
    )
}

fn page_inventory(c: &Connection, dir: &Path) -> Result<Value, String> {
    let n = |sql: &str| c.query_row(sql, [], |r| r.get::<_, i64>(0)).map_err(err);
    let pages = n("pragma page_count")?;
    let page_size = n("pragma page_size")?;
    let free = n("pragma freelist_count")?;
    Ok(
        json!({"db_bytes":file_bytes(&dir.join("monitor.sqlite3"))?,"wal_bytes":file_bytes(&dir.join("monitor.sqlite3-wal"))?,
        "page_count":pages,"page_size":page_size,"freelist_count":free,"active_page_bytes":(pages-free)*page_size,"reusable_page_bytes":free*page_size}),
    )
}
fn err(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maintenance_retry_uses_production_mapping_and_never_retries_cancellation() {
        let cancel = AtomicBool::new(false);
        for (sqlite_code, expected) in [
            (rusqlite::ffi::SQLITE_INTERRUPT, true),
            (rusqlite::ffi::SQLITE_BUSY, true),
            (rusqlite::ffi::SQLITE_LOCKED, true),
            (rusqlite::ffi::SQLITE_FULL, false),
            (rusqlite::ffi::SQLITE_CORRUPT, false),
            (rusqlite::ffi::SQLITE_IOERR, false),
        ] {
            let error = crate::storage::StorageError::Sqlite(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(sqlite_code),
                None,
            ));
            let mapped = error.maintenance_report_error(false);
            assert_eq!(maintenance_retryable(&mapped, &cancel, true), expected);
            assert!(!maintenance_retryable(&mapped, &cancel, false));
            assert!(!maintenance_retryable(
                &error.maintenance_report_error(true),
                &cancel,
                true
            ));
            // 分类之后才收到取消，也不开始下一次尝试。
            cancel.store(true, Ordering::SeqCst);
            assert!(!maintenance_retryable(&mapped, &cancel, true));
            cancel.store(false, Ordering::SeqCst);
        }
        for error in [
            ReportError::Cancelled("cancel"),
            ReportError::Failed("integrity"),
            ReportError::InsufficientSpace("space"),
            ReportError::InvalidQuery("query"),
            ReportError::CapabilityUnsupported("capability"),
            ReportError::TokenExpired("token"),
            ReportError::QuotaExceeded("quota"),
        ] {
            assert!(!maintenance_retryable(&error, &cancel, true));
        }
    }

    #[test]
    fn completion_inventory_waits_for_both_persisted_scan_ends() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("create table machine_setting(key text primary key,value text);
            insert into machine_setting values('session_cleanup_cursor','1000'),('receipt_cleanup_cursor','0');").unwrap();
        let mut pass = CleanupPass::default();
        // 收据先结束，session 仍在扫描受 raw 保护的行；每块只读游标。
        for _ in 0..10_000 {
            pass.observe(&db).unwrap();
            assert!(!pass.take_completed(false));
        }
        db.execute_batch("update machine_setting set value=case key when 'session_cleanup_cursor' then '0' else '1000' end;").unwrap();
        pass.observe(&db).unwrap();
        assert!(pass.take_completed(false));
        // 旁证失败后必须再等完整一轮，不能复用上轮的收据完成标记。
        pass.observe(&db).unwrap();
        assert!(!pass.take_completed(false));
        db.execute_batch("update machine_setting set value=case key when 'session_cleanup_cursor' then '1000' else '0' end;").unwrap();
        pass.observe(&db).unwrap();
        assert!(pass.take_completed(false));
        db.execute_batch("drop table machine_setting").unwrap();
        assert!(pass.observe(&db).is_err());
    }

    #[test]
    fn completion_latches_async_scan_ends_until_other_maintenance_finishes() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("create table machine_setting(key text primary key,value text);
            insert into machine_setting values('session_cleanup_cursor','1000'),('receipt_cleanup_cursor','2000');").unwrap();
        let mut pass = CleanupPass::default();
        pass.observe(&db).unwrap();
        assert!(!pass.take_completed(false));
        // session EOF 恰逢字典扫描仍有后续页，不能丢掉该完成信号。
        db.execute_batch(
            "update machine_setting set value='0' where key='session_cleanup_cursor';",
        )
        .unwrap();
        pass.observe(&db).unwrap();
        assert!(!pass.take_completed(true));
        db.execute_batch("update machine_setting set value=case key when 'session_cleanup_cursor' then '1000' else '0' end;").unwrap();
        pass.observe(&db).unwrap();
        assert!(!pass.take_completed(true));
        // 两个游标均开始下一轮后，其它扫描才结束；之前的 EOF 仍应触发一次完整旁证。
        db.execute_batch("update machine_setting set value='2000';")
            .unwrap();
        pass.observe(&db).unwrap();
        assert!(!pass.take_completed(true));
        assert!(pass.take_completed(false));
        assert!(!pass.take_completed(false));
        pass.observe(&db).unwrap();
        assert!(!pass.take_completed(false));
    }

    #[test]
    fn corpus_has_production_schema_churn_receipts_and_exact_rows() {
        let dir = tempfile::tempdir().unwrap();
        let result = generate_corpus(dir.path(), 4, 0, 7, 1_800_000_000).unwrap();
        assert_eq!(result["actual"]["minutes"], 12);
        assert_eq!(result["actual"]["sessions"], 4);
        assert_eq!(result["actual"]["chains"], 12);
        assert_eq!(result["actual"]["receipts"], 180);
        assert_eq!(result["full_30_day_input"], false);
        let db = StorageCoordinator::open(&dir.path().join("monitor.sqlite3")).unwrap();
        assert_eq!(db.watermark().unwrap(), 180);
        assert_eq!(
            db.connection()
                .query_row("select watermark from data_version where id=1", [], |r| r
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            0
        );
        assert_eq!(
            db.connection()
                .query_row("pragma integrity_check", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
        assert!(generate_corpus(dir.path(), 4, 0, 7, 1_800_000_000).is_err());
    }

    #[test]
    #[ignore = "仅显式指定的隔离容量库；完整规模可能耗时数分钟"]
    fn isolated_corpus_retention_capacity_gate() {
        let dir = std::path::PathBuf::from(
            std::env::var_os("RESIWATCH_BENCH_CORPUS_DIR")
                .expect("设置此工具生成的隔离 corpus 目录"),
        );
        let manifest: Value =
            serde_json::from_slice(&std::fs::read(dir.join(MANIFEST)).expect("fixture manifest"))
                .expect("json");
        assert_eq!(manifest["kind"], "production-corpus");
        let now: i64 = std::env::var("RESIWATCH_BENCH_CORPUS_NOW_UTC")
            .expect("显式设置维护虚拟时间")
            .parse()
            .expect("UTC 秒");
        let chunks: u32 = std::env::var("RESIWATCH_BENCH_CORPUS_CHUNKS")
            .unwrap_or_else(|_| "100000".into())
            .parse()
            .expect("chunks");
        let before = {
            let db = StorageCoordinator::open(&dir.join("monitor.sqlite3")).expect("open");
            combined_totals(db.connection())
        };
        let mut result = retain_corpus_inner(&dir, now, 30, chunks, true, CorpusGate::IsolatedTest)
            .expect("retention");
        let (after, remaining) = {
            let db = StorageCoordinator::open(&dir.join("monitor.sqlite3")).expect("reopen");
            let cutoff = (now - 30 * DAY).div_euclid(DAY) * DAY;
            let remaining: i64 = db
                .connection()
                .query_row(
                    "select count(*) from connection_minute where utc_minute<?1",
                    [cutoff / 60],
                    |r| r.get(0),
                )
                .expect("raw count");
            (combined_totals(db.connection()), remaining)
        };
        result["conservation"] = json!({
            "combined_before": {"upload": before.0, "download": before.1},
            "combined_after": {"upload": after.0, "download": after.1},
            "conserved": before == after,
            "remaining_expired_raw": remaining,
        });
        std::fs::write(
            dir.join("retention-test-result.json"),
            serde_json::to_vec_pretty(&result).expect("json"),
        )
        .expect("write evidence");
        assert_eq!(
            result["complete"], true,
            "未完成；保留完整 evidence，不能宣布容量门通过"
        );
        assert_eq!(result["quick_check"], "ok");
        assert_eq!(after, before, "raw 与已删日 core 合计守恒");
        assert_eq!(remaining, 0, "过期 raw 全部退出");
        assert_eq!(
            result["ledger_after"]["reclaimable_unreferenced_sessions"],
            0
        );
        assert_eq!(result["ledger_after"]["eligible_receipts"], 0);
    }

    fn combined_totals(c: &Connection) -> (i64, i64) {
        c.query_row("select
            (select coalesce(sum(upload),0) from connection_minute)+(select coalesce(sum(upload),0) from traffic_daily_core d where category_id=0 and exists(select 1 from retention_state s where s.layer='day_exact_v1' and s.chunk_utc=d.utc_day and s.status='deleted')),
            (select coalesce(sum(download),0) from connection_minute)+(select coalesce(sum(download),0) from traffic_daily_core d where category_id=0 and exists(select 1 from retention_state s where s.layer='day_exact_v1' and s.chunk_utc=d.utc_day and s.status='deleted'))",[],|r|Ok((r.get(0)?,r.get(1)?))).expect("conserved totals")
    }
}
