//! 单日持久构建器：有界 raw 双遍、精确成员集合、确认后有界删除。

use super::*;
use rusqlite::OptionalExtension;

const ROWS: i64 = 128;
const DELETE_ROWS: i64 = 2_048;
type Key = (String, i64, i64, String, i64);
type Values = (i64, i64, i64, i64);

struct Job {
    day: i64,
    phase: String,
    minute: i64,
    session: i64,
    build_hash: String,
    check_hash: String,
    cursor: String,
    invalidated: bool,
}
struct Raw {
    minute: i64,
    session: i64,
    upload: i64,
    download: i64,
    category: i64,
    ids: [i64; 5],
}

pub(super) fn run(
    coordinator: &mut StorageCoordinator,
    now: i64,
    raw_days: i64,
    mode: RetentionMode,
    space: &SpaceBudget,
    cancel: &Arc<AtomicBool>,
    gate: bool,
) -> Result<RetentionChunk, ReportError> {
    crate::c3::service::poll_interrupt(cancel, "retention")?;
    let cutoff = now
        .saturating_sub(raw_days.clamp(1, RAW_RETAIN_DAYS_MAX) * DAY)
        .div_euclid(DAY)
        * DAY;
    let deleting = mode == RetentionMode::DeleteEnabled && gate;
    if mode != RetentionMode::DryRun {
        space.check(
            coordinator.path().parent().unwrap_or(coordinator.path()),
            512 * 1024,
        )?;
    }
    let started = Instant::now();
    let flag = Arc::clone(cancel);
    let idle_writer = coordinator.connection().is_autocommit();
    coordinator
        .connection()
        .busy_timeout(Duration::ZERO)
        .map_err(sql_error)?;
    coordinator
        .connection()
        .progress_handler(
            256,
            Some(move || {
                flag.load(Ordering::SeqCst) || started.elapsed() >= Duration::from_millis(1_000)
            }),
        )
        .map_err(sql_error)?;
    let result = (|| {
        if mode == RetentionMode::DryRun {
            let day = load(coordinator.connection())?
                .map(|job| job.day)
                .or(next_day(coordinator.connection(), cutoff, deleting)?);
            return Ok(RetentionChunk {
                day_utc: day,
                more_pending: day.is_some(),
                ..Default::default()
            });
        }
        let tx = coordinator
            .connection_mut()
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(sql_error)?;
        if load(&tx)?.is_none() {
            let candidate = next_day(&tx, cutoff, deleting)?;
            let expired = if candidate.is_none() && deleting {
                expired_dimension_day(&tx, now)?
            } else {
                None
            };
            let Some(day) = candidate.or(expired) else {
                if deleting {
                    let changed = cleanup_expired(
                        &tx,
                        (now - DIMENSION_RETAIN_DAYS * DAY).div_euclid(DAY) * DAY,
                        now,
                    )?;
                    tx.commit().map_err(sql_error)?;
                    return Ok(RetentionChunk {
                        more_pending: changed,
                        ..Default::default()
                    });
                }
                return Ok(RetentionChunk::default());
            };
            if candidate.is_some() {
                let deleted:bool=tx.query_row("select exists(select 1 from retention_state where layer=?1 and chunk_utc=?2 and status='deleted')",
                    params![DAY_EXACT_LAYER,day],|r|r.get(0)).map_err(sql_error)?;
                if deleted {
                    return Err(ReportError::Failed("已封存日期出现迟到明细"));
                }
            }
            tx.execute("insert into retention_build(job_id,day_utc,phase,cursor_minute,cursor_session,output_protected) values (1,?1,?2,?3,-1,?4)",
                params![day,if expired.is_some(){"prune_check"}else{"build"},day/60-1,expired.is_some()]).map_err(sql_error)?;
        }
        let mut job = load(&tx)?.ok_or(ReportError::Failed("保留构建状态丢失"))?;
        if !deleting && (job.phase == "delete" || job.phase.starts_with("prune_")) {
            return Ok(RetentionChunk {
                day_utc: Some(job.day),
                ..Default::default()
            });
        }
        if job.invalidated && job.phase != "discard" {
            tx.execute(
                "update retention_build set phase='discard' where job_id=1",
                [],
            )
            .map_err(sql_error)?;
            tx.commit().map_err(sql_error)?;
            return Err(ReportError::Failed("保留旁证变化，已停止清理并安排重建"));
        }
        let mut deleted = 0;
        match job.phase.as_str() {
            "build" | "check" => raw_step(&tx, &mut job)?,
            "legacy_hour" | "legacy_day" | "legacy_core" => legacy_step(&tx, &job)?,
            "publish" | "audit" => publish_step(&tx, &job)?,
            "coverage_scan" => coverage_scan(&tx, &job)?,
            "coverage_union" => coverage_union(&tx, &job)?,
            "confirm" => confirm(&tx, &job, now, deleting)?,
            "delete" => {
                deleted=tx.execute("delete from connection_minute where (utc_minute,session_pk) in
                    (select utc_minute,session_pk from connection_minute where utc_minute>=?1 and utc_minute<?2 order by utc_minute,session_pk limit ?3)",
                    params![job.day/60,(job.day+DAY)/60,DELETE_ROWS]).map_err(sql_error)? as u64;
                if deleted == 0 {
                    phase(&tx, "clear")?;
                }
            }
            "clear" | "discard" => {
                if job.phase == "discard" && discard_output(&tx, job.day)? {
                    tx.commit().map_err(sql_error)?;
                    return Ok(RetentionChunk {
                        day_utc: Some(job.day),
                        more_pending: true,
                        ..Default::default()
                    });
                }
                let n=tx.execute("delete from retention_build_member where rowid in(select rowid from retention_build_member limit ?1)",[DELETE_ROWS]).map_err(sql_error)?;
                if n == 0 {
                    let n=tx.execute("delete from retention_build_aggregate where rowid in(select rowid from retention_build_aggregate limit ?1)",[DELETE_ROWS]).map_err(sql_error)?;
                    if n == 0 {
                        tx.execute("delete from retention_build where job_id=1", [])
                            .map_err(sql_error)?;
                        if deleting {
                            save_cleanup_cursor(&tx, "dictionary_cleanup_cursor", -1)?;
                            save_cleanup_cursor(&tx, "coverage_cleanup_cursor", -1)?;
                        }
                    }
                }
            }
            "prune_check" => prune_check(&tx, &job)?,
            "prune_hour" | "prune_day" => {
                tx.execute(
                    "update retention_build set phase='_prune' where job_id=1",
                    [],
                )
                .map_err(sql_error)?;
                let (table, time, next) = if job.phase == "prune_hour" {
                    ("traffic_hourly_dimension", "utc_hour", "prune_day")
                } else {
                    ("traffic_daily_dimension", "utc_day", "prune_finish")
                };
                let n=tx.execute(&format!("delete from {table} where rowid in(select rowid from {table} where {time}>=?1 and {time}<?2 limit ?3)"),
                    params![job.day,job.day+DAY,DELETE_ROWS]).map_err(sql_error)?;
                if n == 0 {
                    phase(&tx, next)?;
                } else {
                    tx.execute(
                        "update retention_build set phase=?1 where job_id=1",
                        [&job.phase],
                    )
                    .map_err(sql_error)?;
                }
            }
            "prune_finish" => {
                // 较早扫描过的字典页可能刚失去最后一日引用，必须完整重扫。
                save_cleanup_cursor(&tx, "dictionary_cleanup_cursor", -1)?;
                save_cleanup_cursor(&tx, "coverage_cleanup_cursor", -1)?;
                tx.execute("delete from retention_build where job_id=1", [])
                    .map_err(sql_error)?;
            }
            _ => return Err(ReportError::Failed("未知保留构建阶段")),
        }
        crate::c3::service::poll_interrupt(cancel, "retention commit")?;
        if load(&tx)?.is_some_and(|current| current.invalidated && current.phase != "discard") {
            return Err(ReportError::Failed("保留块提交前旁证发生变化"));
        }
        tx.commit().map_err(sql_error)?;
        let pending = load(coordinator.connection())?.is_some()
            || next_day(coordinator.connection(), cutoff, deleting)?.is_some()
            || (deleting
                && (expired_dimension_day(coordinator.connection(), now)?.is_some()
                    || auxiliary_cleanup_pending(coordinator.connection())?));
        Ok(RetentionChunk {
            day_utc: Some(job.day),
            deleted_raw_rows: deleted,
            more_pending: pending,
        })
    })();
    coordinator
        .connection()
        .progress_handler(0, None::<fn() -> bool>)
        .map_err(sql_error)?;
    // 取消 handler 可能同时中断 RAII 回滚；清除后再归还干净的 writer。
    let rollback = if result.is_err() && idle_writer && !coordinator.connection().is_autocommit() {
        coordinator.connection().execute_batch("rollback")
    } else {
        Ok(())
    };
    coordinator
        .connection()
        .busy_timeout(Duration::from_millis(u64::from(
            crate::c0_contract::BUSY_TIMEOUT_MS,
        )))
        .map_err(sql_error)?;
    rollback.map_err(sql_error)?;
    match result {
        Err(_) if cancel.load(Ordering::SeqCst) => Err(ReportError::Cancelled("保留维护已取消")),
        Err(ReportError::Failed(_)) if started.elapsed() >= Duration::from_millis(1_000) => {
            Err(ReportError::DeadlineExceeded("保留块让出采集时间"))
        }
        other => other,
    }
}

fn discard_output(c: &Connection, day: i64) -> Result<bool, ReportError> {
    let published: bool = c
        .query_row(
            "select output_started from retention_build where job_id=1",
            [],
            |r| r.get(0),
        )
        .map_err(sql_error)?;
    if !published {
        return Ok(false);
    }
    let confirmed: bool = c
        .query_row(
            "select exists(select 1 from retention_state where layer=?1 and chunk_utc=?2)",
            params![DAY_EXACT_LAYER, day],
            |r| r.get(0),
        )
        .map_err(sql_error)?;
    if confirmed {
        return Ok(false);
    }
    for (table, time) in [
        ("traffic_hourly_dimension", "utc_hour"),
        ("traffic_daily_dimension", "utc_day"),
        ("traffic_daily_core", "utc_day"),
        ("coverage_daily", "utc_day"),
    ] {
        let n=c.execute(&format!("delete from {table} where rowid in(select rowid from {table} where {time}>=?1 and {time}<?2 limit ?3)"),params![day,day+DAY,DELETE_ROWS]).map_err(sql_error)?;
        if n > 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

fn load(c: &Connection) -> Result<Option<Job>, ReportError> {
    c.query_row("select day_utc,phase,cursor_minute,cursor_session,build_hash,check_hash,output_cursor,invalidated from retention_build where job_id=1",[],|r|Ok(Job{
        day:r.get(0)?,phase:r.get(1)?,minute:r.get(2)?,session:r.get(3)?,build_hash:r.get(4)?,check_hash:r.get(5)?,cursor:r.get(6)?,invalidated:r.get(7)?,
    })).optional().map_err(sql_error)
}

fn phase(c: &Connection, next: &str) -> Result<(), ReportError> {
    c.execute(
        "update retention_build set phase=?1,output_cursor='' where job_id=1",
        [next],
    )
    .map_err(sql_error)?;
    Ok(())
}

fn digest(previous: &str, value: &impl serde::Serialize) -> Result<String, ReportError> {
    let bytes = serde_json::to_vec(value).map_err(|_| ReportError::Failed("保留旁证编码失败"))?;
    let mut hash = Sha256::new();
    hash.update(previous.as_bytes());
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
    Ok(hex::encode(hash.finalize()))
}

fn raw_rows(c: &Connection, job: &Job) -> Result<Vec<Raw>, ReportError> {
    let mut stmt=c.prepare_cached("select m.utc_minute,m.session_pk,m.upload,m.download,coalesce(a.primary_category_id,0),
        coalesce(a.host_id,0),coalesce(a.process_id,0),coalesce(a.network_id,0),
        chain_identity(a.chain_key),coalesce(last_chain_hop(a.chain_key),r.value,'DIRECT')
        from connection_minute m left join connection_session_attr a on a.session_pk=m.session_pk
        left join dimension_dict r on r.dimension_kind='rule' and r.dimension_id=a.rule_id
        where m.utc_minute>=?1 and m.utc_minute<?2 and (m.utc_minute,m.session_pk)>(?3,?4)
        order by m.utc_minute,m.session_pk limit ?5").map_err(sql_error)?;
    let mut rows = stmt
        .query(params![
            job.day / 60,
            (job.day + DAY) / 60,
            job.minute,
            job.session,
            ROWS
        ])
        .map_err(sql_error)?;
    let mut result = Vec::new();
    while let Some(row) = rows.next().map_err(sql_error)? {
        let chain: Option<String> = row.get(8).map_err(sql_error)?;
        let rule: String = row.get(9).map_err(sql_error)?;
        let mut ids = [
            row.get(5).map_err(sql_error)?,
            row.get(6).map_err(sql_error)?,
            0,
            0,
            row.get(7).map_err(sql_error)?,
        ];
        for (index, kind, value) in [
            (2, "rule_group", Some(rule.as_str())),
            (3, "chain", chain.as_deref()),
        ] {
            if let Some(value) = value {
                intern_one(c, kind, value)?;
                ids[index]=c.prepare_cached("select dimension_id from dimension_dict where dimension_kind=?1 and value=?2").map_err(sql_error)?
                    .query_row(params![kind,value],|r|r.get(0))
                    .optional().map_err(sql_error)?.unwrap_or(0);
            }
        }
        result.push(Raw {
            minute: row.get(0).map_err(sql_error)?,
            session: row.get(1).map_err(sql_error)?,
            upload: row.get(2).map_err(sql_error)?,
            download: row.get(3).map_err(sql_error)?,
            category: row.get(4).map_err(sql_error)?,
            ids,
        });
    }
    Ok(result)
}

fn raw_step(c: &Connection, job: &mut Job) -> Result<(), ReportError> {
    let checking = job.phase == "check";
    let rows = raw_rows(c, job)?;
    if rows.is_empty() {
        if checking {
            if job.build_hash != job.check_hash {
                return Err(ReportError::Failed("完整 raw 双遍旁证不一致"));
            }
            phase(c, "legacy_hour")?;
            c.execute(
                "update retention_build set output_protected=1 where job_id=1",
                [],
            )
            .map_err(sql_error)?;
        } else {
            c.execute("insert or ignore into retention_build_aggregate(granularity,bucket_utc,category_id,dimension_kind,dimension_id)
                values ('core',?1,0,'',0)",[job.day]).map_err(sql_error)?;
            c.execute("update retention_build set phase='check',cursor_minute=?1,cursor_session=-1 where job_id=1",[job.day/60-1]).map_err(sql_error)?;
        }
        return Ok(());
    }
    let mut hash = if checking {
        job.check_hash.clone()
    } else {
        job.build_hash.clone()
    };
    for row in &rows {
        hash = digest(
            &hash,
            &(
                row.minute,
                row.session,
                row.upload,
                row.download,
                row.category,
                row.ids,
            ),
        )?;
        for (granularity, bucket) in [
            ("hour", (row.minute * 60).div_euclid(3600) * 3600),
            ("day", job.day),
        ] {
            for (kind, id) in ["host", "process", "rule_group", "chain", "network"]
                .into_iter()
                .zip(row.ids)
            {
                apply_raw(
                    c,
                    &(granularity.into(), bucket, row.category, kind.into(), id),
                    row,
                    checking,
                )?;
            }
        }
        apply_raw(
            c,
            &("core".into(), job.day, 0, String::new(), 0),
            row,
            checking,
        )?;
        if row.category > 0 {
            apply_raw(
                c,
                &("core".into(), job.day, row.category, String::new(), 0),
                row,
                checking,
            )?;
        }
    }
    let last = rows.last().ok_or(ReportError::Failed("保留游标缺失"))?;
    let column = if checking { "check_hash" } else { "build_hash" };
    c.execute(&format!("update retention_build set cursor_minute=?1,cursor_session=?2,{column}=?3,raw_rows=raw_rows+?4 where job_id=1"),
        params![last.minute,last.session,hash,if checking{0}else{rows.len() as i64}]).map_err(sql_error)?;
    Ok(())
}

fn apply_raw(c: &Connection, key: &Key, row: &Raw, checking: bool) -> Result<(), ReportError> {
    let mut additions = [0_i64; 2];
    for (i, kind, id) in [(0, "session", row.session), (1, "minute", row.minute)] {
        let sql = if checking {
            "update retention_build_member set checked=1 where granularity=?1 and bucket_utc=?2 and category_id=?3 and dimension_kind=?4 and dimension_id=?5 and member_kind=?6 and member_id=?7 and checked=0"
        } else {
            "insert or ignore into retention_build_member(granularity,bucket_utc,category_id,dimension_kind,dimension_id,member_kind,member_id) values (?1,?2,?3,?4,?5,?6,?7)"
        };
        additions[i] = c
            .prepare_cached(sql)
            .map_err(sql_error)?
            .execute(params![key.0, key.1, key.2, key.3, key.4, kind, id])
            .map_err(sql_error)? as i64;
        if checking && additions[i] == 0 {
            let exists:bool=c.prepare_cached("select exists(select 1 from retention_build_member where granularity=?1 and bucket_utc=?2 and category_id=?3 and dimension_kind=?4 and dimension_id=?5 and member_kind=?6 and member_id=?7)")
                .map_err(sql_error)?.query_row(params![key.0,key.1,key.2,key.3,key.4,kind,id],|r|r.get(0)).map_err(sql_error)?;
            if !exists {
                return Err(ReportError::Failed("精确成员旁证缺失"));
            }
        }
    }
    let sql = if checking {
        "update retention_build_aggregate set check_upload=check_upload+?6,check_download=check_download+?7,
        check_connection_count=check_connection_count+?8,check_active_duration_sec=check_active_duration_sec+?9
        where granularity=?1 and bucket_utc=?2 and category_id=?3 and dimension_kind=?4 and dimension_id=?5"
    } else {
        "insert into retention_build_aggregate(granularity,bucket_utc,category_id,dimension_kind,dimension_id,upload,download,connection_count,active_duration_sec)
        values (?1,?2,?3,?4,?5,?6,?7,?8,?9) on conflict(granularity,bucket_utc,category_id,dimension_kind,dimension_id)
        do update set upload=upload+excluded.upload,download=download+excluded.download,
        connection_count=connection_count+excluded.connection_count,active_duration_sec=active_duration_sec+excluded.active_duration_sec"
    };
    let changed = c
        .prepare_cached(sql)
        .map_err(sql_error)?
        .execute(params![
            key.0,
            key.1,
            key.2,
            key.3,
            key.4,
            row.upload,
            row.download,
            additions[0],
            additions[1] * 60
        ])
        .map_err(sql_error)?;
    if checking && changed != 1 {
        return Err(ReportError::Failed("汇总旁证缺失"));
    }
    Ok(())
}

fn parse_cursor(text: &str) -> Result<Key, ReportError> {
    if text.is_empty() {
        return Ok((String::new(), i64::MIN, i64::MIN, String::new(), i64::MIN));
    }
    serde_json::from_str(text).map_err(|_| ReportError::Failed("保留游标损坏"))
}
fn save_cursor(c: &Connection, key: &Key) -> Result<(), ReportError> {
    c.execute(
        "update retention_build set output_cursor=?1 where job_id=1",
        [serde_json::to_string(key).map_err(|_| ReportError::Failed("保留游标编码失败"))?],
    )
    .map_err(sql_error)?;
    Ok(())
}
fn source(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "hour" => ("traffic_hourly_dimension", "utc_hour"),
        "day" => ("traffic_daily_dimension", "utc_day"),
        _ => ("traffic_daily_core", "utc_day"),
    }
}
fn stage_values(c: &Connection, key: &Key) -> Result<Option<Values>, ReportError> {
    c.prepare_cached("select upload,download,connection_count,active_duration_sec from retention_build_aggregate
        where granularity=?1 and bucket_utc=?2 and category_id=?3 and dimension_kind=?4 and dimension_id=?5").map_err(sql_error)?
        .query_row(params![key.0,key.1,key.2,key.3,key.4],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(sql_error)
}
fn existing_rows(c: &Connection, job: &Job, kind: &str) -> Result<Vec<(Key, Values)>, ReportError> {
    let key = parse_cursor(&job.cursor)?;
    let (table, time) = source(kind);
    let (columns, predicate) = if kind == "core" {
        ("'',0", "(utc_day,category_id)>(?3,?4)")
    } else {
        (
            "dimension_kind,dimension_id",
            "(utc_day,category_id,dimension_kind,dimension_id)>(?3,?4,?5,?6)",
        )
    };
    let predicate = predicate.replace("utc_day", time);
    let sql=format!("select {time},category_id,{columns},upload,download,connection_count,active_duration_sec from {table}
        where {time}>=?1 and {time}<?2 and {predicate} order by 1,2,3,4 limit {ROWS}");
    let mut stmt = c.prepare(&sql).map_err(sql_error)?;
    let mut rows = if kind == "core" {
        stmt.query(params![job.day, job.day + DAY, key.1, key.2])
    } else {
        stmt.query(params![job.day, job.day + DAY, key.1, key.2, key.3, key.4])
    }
    .map_err(sql_error)?;
    let mut result = Vec::new();
    while let Some(r) = rows.next().map_err(sql_error)? {
        result.push((
            (
                kind.into(),
                r.get(0).map_err(sql_error)?,
                r.get(1).map_err(sql_error)?,
                r.get(2).map_err(sql_error)?,
                r.get(3).map_err(sql_error)?,
            ),
            (
                r.get(4).map_err(sql_error)?,
                r.get(5).map_err(sql_error)?,
                r.get(6).map_err(sql_error)?,
                r.get(7).map_err(sql_error)?,
            ),
        ));
    }
    Ok(result)
}
fn legacy_step(c: &Connection, job: &Job) -> Result<(), ReportError> {
    let (kind, next) = match job.phase.as_str() {
        "legacy_hour" => ("hour", "legacy_day"),
        "legacy_day" => ("day", "legacy_core"),
        _ => ("core", "publish"),
    };
    let strict: bool = c
        .query_row(
            "select exists(select 1 from retention_state where layer=?1 and chunk_utc=?2)",
            params![DAY_EXACT_LAYER, job.day],
            |r| r.get(0),
        )
        .map_err(sql_error)?;
    let rows = existing_rows(c, job, kind)?;
    for (key, values) in &rows {
        let expected = stage_values(c, key)?.ok_or(ReportError::Failed("已有汇总无原始旁证"))?;
        if (values.0, values.1) != (expected.0, expected.1) || (strict && values != &expected) {
            return Err(ReportError::Failed("已有汇总与完整原始旁证不守恒"));
        }
    }
    if let Some((key, _)) = rows.last() {
        save_cursor(c, key)?;
    } else {
        phase(c, next)?;
        if next == "publish" {
            c.execute(
                "update retention_build set check_hash='' where job_id=1",
                [],
            )
            .map_err(sql_error)?;
        }
    }
    Ok(())
}

fn publish_step(c: &Connection, job: &Job) -> Result<(), ReportError> {
    let auditing = job.phase == "audit";
    let strict: bool = c
        .query_row(
            "select exists(select 1 from retention_state where layer=?1 and chunk_utc=?2)",
            params![DAY_EXACT_LAYER, job.day],
            |r| r.get(0),
        )
        .map_err(sql_error)?;
    if !auditing {
        c.execute(
            "update retention_build set phase='_publish',output_started=1,output_protected=1 where job_id=1",
            [],
        )
        .map_err(sql_error)?;
    }
    let cursor = parse_cursor(&job.cursor)?;
    let mut stmt=c.prepare("select granularity,bucket_utc,category_id,dimension_kind,dimension_id,upload,download,connection_count,active_duration_sec,
        check_upload,check_download,check_connection_count,check_active_duration_sec from retention_build_aggregate
        where (granularity,bucket_utc,category_id,dimension_kind,dimension_id)>(?1,?2,?3,?4,?5)
        order by 1,2,3,4,5 limit ?6").map_err(sql_error)?;
    let mut rows = stmt
        .query(params![
            cursor.0, cursor.1, cursor.2, cursor.3, cursor.4, ROWS
        ])
        .map_err(sql_error)?;
    let mut last = None;
    let mut hash = job.check_hash.clone();
    while let Some(r) = rows.next().map_err(sql_error)? {
        let key: Key = (
            r.get(0).map_err(sql_error)?,
            r.get(1).map_err(sql_error)?,
            r.get(2).map_err(sql_error)?,
            r.get(3).map_err(sql_error)?,
            r.get(4).map_err(sql_error)?,
        );
        let values: Values = (
            r.get(5).map_err(sql_error)?,
            r.get(6).map_err(sql_error)?,
            r.get(7).map_err(sql_error)?,
            r.get(8).map_err(sql_error)?,
        );
        let check: Values = (
            r.get(9).map_err(sql_error)?,
            r.get(10).map_err(sql_error)?,
            r.get(11).map_err(sql_error)?,
            r.get(12).map_err(sql_error)?,
        );
        if values != check {
            return Err(ReportError::Failed("分类、维度字节或精确成员数量不守恒"));
        }
        let old = read_actual(c, &key)?;
        if auditing || strict {
            if old != Some(values) {
                return Err(ReportError::Failed("已确认或已发布汇总缺失或损坏"));
            }
        } else if old.is_some_and(|value| (value.0, value.1) != (values.0, values.1)) {
            return Err(ReportError::Failed("已有汇总与完整原始旁证不守恒"));
        }
        if auditing {
            hash = digest(&hash, &(&key, values))?;
        } else {
            write_actual(c, &key, &values)?;
        }
        last = Some(key);
    }
    if let Some(key) = last {
        save_cursor(c, &key)?;
        c.execute(
            "update retention_build set check_hash=?1,phase=?2 where job_id=1",
            params![hash, if auditing { "audit" } else { "publish" }],
        )
        .map_err(sql_error)?;
    } else {
        phase(c, if auditing { "coverage_scan" } else { "audit" })?;
        if auditing {
            c.execute(
                "update retention_build set cursor_minute=?1,cursor_session=-1 where job_id=1",
                [i64::MIN],
            )
            .map_err(sql_error)?;
        }
        if !auditing {
            c.execute(
                "update retention_build set check_hash='' where job_id=1",
                [],
            )
            .map_err(sql_error)?;
        }
    }
    Ok(())
}

fn write_actual(c: &Connection, key: &Key, values: &Values) -> Result<(), ReportError> {
    let (table, _) = source(&key.0);
    if key.0 == "core" {
        c.prepare_cached(&format!(
            "insert or replace into {table} values (?1,?2,?3,?4,?5,?6)"
        ))
        .map_err(sql_error)?
        .execute(params![
            key.1, key.2, values.0, values.1, values.2, values.3
        ])
        .map_err(sql_error)?;
    } else {
        c.prepare_cached(&format!(
            "insert or replace into {table} values (?1,?2,?3,?4,?5,?6,?7,?8)"
        ))
        .map_err(sql_error)?
        .execute(params![
            key.1, key.2, key.3, key.4, values.0, values.1, values.2, values.3
        ])
        .map_err(sql_error)?;
    }
    if read_actual(c, key)? != Some(*values) {
        return Err(ReportError::Failed("汇总写入后校验失败"));
    }
    Ok(())
}

fn read_actual(c: &Connection, key: &Key) -> Result<Option<Values>, ReportError> {
    let (table, time) = source(&key.0);
    let predicate = if key.0 == "core" {
        String::new()
    } else {
        " and dimension_kind=?3 and dimension_id=?4".into()
    };
    let sql=format!("select upload,download,connection_count,active_duration_sec from {table} where {time}=?1 and category_id=?2{predicate}");
    let mut statement = c.prepare_cached(&sql).map_err(sql_error)?;
    let read = |r: &rusqlite::Row<'_>| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?));
    let stored = if key.0 == "core" {
        statement.query_row(params![key.1, key.2], read)
    } else {
        statement.query_row(params![key.1, key.2, key.3, key.4], read)
    }
    .optional()
    .map_err(sql_error)?;
    Ok(stored)
}

// 只按索引读取固定数量源行，在 Rust 侧筛除不相交区间；LIMIT 不依赖命中数量。
fn coverage_scan(c: &Connection, job: &Job) -> Result<(), ReportError> {
    let mut stmt = c
        .prepare(
            "select interval_id,kind,reason,started_utc,ended_utc from coverage_interval indexed by idx_retention_coverage_source
        where kind in ('covered','gap') and started_utc<?1 and (started_utc,interval_id)>(?2,?3)
        order by started_utc,interval_id limit ?4",
        )
        .map_err(sql_error)?;
    let mut rows = stmt
        .query(params![job.day + DAY, job.minute, job.session, ROWS])
        .map_err(sql_error)?;
    let mut last = None;
    let mut span_end = 0;
    while let Some(row) = rows.next().map_err(sql_error)? {
        let id: i64 = row.get(0).map_err(sql_error)?;
        let kind: String = row.get(1).map_err(sql_error)?;
        let reason: String = row.get(2).map_err(sql_error)?;
        let started: i64 = row.get(3).map_err(sql_error)?;
        let ended: Option<i64> = row.get(4).map_err(sql_error)?;
        last = Some((started, id));
        let Some(ended) = ended else {
            return Err(ReportError::StorageBusy("开放覆盖区间仍受保护"));
        };
        span_end = span_end.max(ended);
        let start = started.max(job.day);
        let end = ended.min(job.day + DAY);
        if end <= start {
            continue;
        }
        let category = i64::from(kind == "gap");
        for (at, event, delta) in [(start, "start", 1), (end, "end", -1)] {
            c.prepare_cached("insert into retention_build_member(granularity,bucket_utc,category_id,dimension_kind,dimension_id,member_kind,member_id,checked)
                values ('coverage_event',?1,?2,'',?3,?4,0,?5)").map_err(sql_error)?.execute(params![at,category,id,event,delta]).map_err(sql_error)?;
        }
        c.prepare_cached("insert or ignore into retention_build_member(granularity,bucket_utc,category_id,dimension_kind,dimension_id,member_kind,member_id)
            values ('coverage_reason',0,0,?1,0,'reason',0)").map_err(sql_error)?.execute([reason]).map_err(sql_error)?;
    }
    if let Some((at, id)) = last {
        c.execute("insert into retention_watermark values ('coverage_span_end',?1,0) on conflict(layer) do update set watermark_utc=max(watermark_utc,excluded.watermark_utc)",[span_end]).map_err(sql_error)?;
        c.execute(
            "update retention_build set cursor_minute=?1,cursor_session=?2 where job_id=1",
            params![at, id],
        )
        .map_err(sql_error)?;
    } else {
        c.execute("insert into retention_build_aggregate(granularity,bucket_utc,category_id,dimension_kind,dimension_id,upload)
            values ('coverage_state',?1,0,'',0,?1)",[job.day]).map_err(sql_error)?;
        phase(c, "coverage_union")?;
    }
    Ok(())
}

fn coverage_union(c: &Connection, job: &Job) -> Result<(), ReportError> {
    let key = parse_cursor(&job.cursor)?;
    let (mut previous,mut cover_depth,mut gap_depth,mut covered,mut gap):(i64,i64,i64,i64,i64)=c.query_row(
        "select upload,download,connection_count,active_duration_sec,check_upload from retention_build_aggregate where granularity='coverage_state'",[],
        |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).map_err(sql_error)?;
    let mut stmt=c.prepare("select bucket_utc,category_id,dimension_id,checked from retention_build_member
        where granularity='coverage_event' and (bucket_utc,category_id,dimension_kind,dimension_id)>(?1,?2,?3,?4)
        order by bucket_utc,category_id,dimension_kind,dimension_id limit ?5").map_err(sql_error)?;
    let mut rows = stmt
        .query(params![key.1, key.2, key.3, key.4, ROWS])
        .map_err(sql_error)?;
    let mut last = None;
    while let Some(row) = rows.next().map_err(sql_error)? {
        let at: i64 = row.get(0).map_err(sql_error)?;
        let kind: i64 = row.get(1).map_err(sql_error)?;
        let id: i64 = row.get(2).map_err(sql_error)?;
        let delta: i64 = row.get(3).map_err(sql_error)?;
        if gap_depth > 0 {
            gap += at - previous;
        } else if cover_depth > 0 {
            covered += at - previous;
        }
        if kind == 1 {
            gap_depth += delta;
        } else {
            cover_depth += delta;
        }
        previous = at;
        last = Some(("coverage_event".into(), at, kind, String::new(), id));
    }
    c.execute("update retention_build_aggregate set upload=?1,download=?2,connection_count=?3,active_duration_sec=?4,check_upload=?5
        where granularity='coverage_state'",params![previous,cover_depth,gap_depth,covered,gap]).map_err(sql_error)?;
    if let Some(key) = last {
        save_cursor(c, &key)?;
    } else {
        if cover_depth != 0 || gap_depth != 0 {
            return Err(ReportError::Failed("覆盖端点旁证不平衡"));
        }
        phase(c, "confirm")?;
    }
    Ok(())
}

fn staged_coverage(c: &Connection) -> Result<(i64, i64, String), ReportError> {
    let (covered,gap)=c.query_row("select active_duration_sec,check_upload from retention_build_aggregate where granularity='coverage_state'",[],
        |r|Ok((r.get(0)?,r.get(1)?))).map_err(sql_error)?;
    let mut stmt=c.prepare("select dimension_kind from retention_build_member where granularity='coverage_reason' order by dimension_kind").map_err(sql_error)?;
    let reasons = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(sql_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_error)?;
    let reasons =
        serde_json::to_string(&reasons).map_err(|_| ReportError::Failed("覆盖原因编码失败"))?;
    Ok((covered, gap, reasons))
}

fn confirm(c: &Connection, job: &Job, now: i64, deleting: bool) -> Result<(), ReportError> {
    let coverage = staged_coverage(c)?;
    c.execute(
        "update retention_build set phase='_confirm' where job_id=1",
        [],
    )
    .map_err(sql_error)?;
    let old: Option<(i64, i64, String)> = c
        .query_row(
            "select covered_sec,gap_sec,reasons_json from coverage_daily where utc_day=?1",
            [job.day],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
        .map_err(sql_error)?;
    if old.as_ref().is_some_and(|value| value != &coverage) {
        return Err(ReportError::Failed("覆盖汇总与区间并集不一致"));
    }
    c.execute(
        "insert or replace into coverage_daily values (?1,?2,?3,?4)",
        params![job.day, coverage.0, coverage.1, coverage.2],
    )
    .map_err(sql_error)?;
    let stored: (i64, i64, String) = c
        .query_row(
            "select covered_sec,gap_sec,reasons_json from coverage_daily where utc_day=?1",
            [job.day],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(sql_error)?;
    if stored != coverage {
        return Err(ReportError::Failed("覆盖汇总写入后校验失败"));
    }
    let hash = format!(
        "staged-v1:{}",
        digest(&job.check_hash, &(job.day, &coverage))?
    );
    c.execute(
        "insert or replace into retention_state values (?1,?2,?3,?4,?5)",
        params![
            DAY_EXACT_LAYER,
            job.day,
            if deleting { "deleted" } else { "verified" },
            hash,
            now
        ],
    )
    .map_err(sql_error)?;
    for layer in [
        DAY_EXACT_LAYER,
        "hourly",
        "daily",
        "core",
        "coverage_exact_v1",
    ] {
        c.execute("insert into retention_watermark values (?1,?2,?3) on conflict(layer) do update set watermark_utc=max(watermark_utc,excluded.watermark_utc),delete_watermark_utc=max(delete_watermark_utc,excluded.delete_watermark_utc)",
            params![layer,job.day+DAY,if deleting{job.day+DAY}else{0}]).map_err(sql_error)?;
    }
    c.execute("insert into retention_watermark values (?1,?2,0) on conflict(layer) do update set watermark_utc=min(watermark_utc,excluded.watermark_utc)",params![HOURLY_DIM_V2_LAYER,job.day]).map_err(sql_error)?;
    if deleting {
        c.execute("update retention_watermark set watermark_utc=max(watermark_utc,?1),delete_watermark_utc=max(delete_watermark_utc,?1) where layer='raw_delete'",[job.day+DAY]).map_err(sql_error)?;
    }
    phase(c, if deleting { "delete" } else { "clear" })
}

fn prune_check(c: &Connection, job: &Job) -> Result<(), ReportError> {
    let cursor = parse_cursor(&job.cursor)?;
    let kind = if cursor.0.is_empty() {
        "core"
    } else {
        cursor.0.as_str()
    };
    let rows = existing_rows(c, job, kind)?;
    let mut hash = job.check_hash.clone();
    for row in &rows {
        hash = digest(&hash, row)?;
    }
    if let Some((key, _)) = rows.last() {
        save_cursor(c, key)?;
        c.execute(
            "update retention_build set check_hash=?1 where job_id=1",
            [hash],
        )
        .map_err(sql_error)?;
    } else if kind != "hour" {
        save_cursor(
            c,
            &(
                if kind == "core" { "day" } else { "hour" }.into(),
                i64::MIN,
                i64::MIN,
                String::new(),
                i64::MIN,
            ),
        )?;
    } else {
        let coverage: (i64, i64, String) = c
            .query_row(
                "select covered_sec,gap_sec,reasons_json from coverage_daily where utc_day=?1",
                [job.day],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(sql_error)?;
        let expected: String = c
            .query_row(
                "select checksum from retention_state where layer=?1 and chunk_utc=?2",
                params![DAY_EXACT_LAYER, job.day],
                |r| r.get(0),
            )
            .map_err(sql_error)?;
        if format!("staged-v1:{}", digest(&hash, &(job.day, coverage))?) != expected {
            return Err(ReportError::Failed("冻结汇总校验失败，保留精确维度"));
        }
        phase(c, "prune_hour")?;
    }
    Ok(())
}
