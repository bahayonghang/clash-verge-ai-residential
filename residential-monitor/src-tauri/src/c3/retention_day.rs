//! 每次只完成一个关闭的 UTC 日；完整 raw 是 distinct 与 duration 的唯一旁证。

use super::*;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::collections::BTreeSet;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

pub const DAY_EXACT_LAYER: &str = "day_exact_v1";
const DAY: i64 = 86_400;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RetentionChunk {
    pub day_utc: Option<i64>,
    pub deleted_raw_rows: u64,
    pub more_pending: bool,
}

/// `delete_gate` 只由生产常量或本模块隔离测试提供，不是用户可绕过的设置。
#[path = "retention_staged.rs"]
mod staged;
pub(super) fn run_chunk(
    coordinator: &mut StorageCoordinator,
    now_utc: i64,
    raw_days: i64,
    mode: RetentionMode,
    space: &SpaceBudget,
    cancel: &Arc<AtomicBool>,
    delete_gate: bool,
) -> Result<RetentionChunk, ReportError> {
    staged::run(
        coordinator,
        now_utc,
        raw_days,
        mode,
        space,
        cancel,
        delete_gate,
    )
}

fn expired_dimension_day(connection: &Connection, now: i64) -> Result<Option<i64>, ReportError> {
    let cutoff = (now - DIMENSION_RETAIN_DAYS * DAY).div_euclid(DAY) * DAY;
    connection.query_row(
        "select min(utc_day) from traffic_daily_dimension d where utc_day<?1 and exists(
         select 1 from retention_state s where s.layer=?2 and s.chunk_utc=d.utc_day and s.status='deleted')",
        params![cutoff,DAY_EXACT_LAYER],|r|r.get(0),
    ).map_err(sql_error)
}

fn sql_error(error: rusqlite::Error) -> ReportError {
    match error.sqlite_error_code() {
        Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked) => {
            ReportError::StorageBusy("保留维护等待账本空闲")
        }
        Some(rusqlite::ErrorCode::DiskFull) => {
            ReportError::InsufficientSpace("汇总空间不足，已保留原始明细")
        }
        _ => ReportError::Failed("保留维护事务失败"),
    }
}

fn next_day(
    connection: &Connection,
    cutoff: i64,
    deleting: bool,
) -> Result<Option<i64>, ReportError> {
    // 时间索引从最早未确认明细开始；旧版文字 checksum 不参与此判断。
    let start: i64 = connection
        .query_row(
            "select coalesce((select watermark_utc from retention_watermark where layer=?1),0)",
            [DAY_EXACT_LAYER],
            |r| r.get(0),
        )
        .map_err(sql_error)?;
    if deleting {
        let late:bool=connection.query_row("select coalesce((select min(utc_minute) from connection_minute),9223372036854775807)
            < coalesce((select delete_watermark_utc/60 from retention_watermark where layer='raw_delete'),0)",[],|r|r.get(0)).map_err(sql_error)?;
        if late {
            return Err(ReportError::Failed("已封存日期出现迟到明细"));
        }
    }
    let raw: Option<i64> = connection
        .query_row(
            "select min(m.utc_minute) from connection_minute m
         where m.utc_minute < ?1 and m.utc_minute >= ?4 and (?2 or not exists (
           select 1 from retention_state s where s.layer = ?3
             and s.chunk_utc = (m.utc_minute / 1440) * 86400
             and s.status in ('verified', 'deleted')))",
            params![cutoff / 60, deleting, DAY_EXACT_LAYER, start / 60],
            |r| r.get(0),
        )
        .map_err(sql_error)?;
    let raw = raw.map(|minute| (minute * 60).div_euclid(DAY) * DAY);
    let cursor: Option<i64> = connection
        .query_row(
            "select watermark_utc from retention_watermark where layer = 'coverage_exact_v1'",
            [],
            |r| r.get(0),
        )
        .optional()
        .map_err(sql_error)?;
    let span_end = cleanup_cursor(connection, "coverage_span_end")?;
    let coverage: Option<i64> = if cursor.is_some_and(|start| start < span_end && start < cutoff) {
        cursor
    } else {
        connection.query_row(
        "select started_utc from coverage_interval indexed by idx_retention_coverage_source
         where kind in ('covered','gap') and started_utc>=?1 and started_utc<?2 order by started_utc,interval_id limit 1",
        params![cursor.unwrap_or(i64::MIN),cutoff],|r|r.get(0),
    ).optional().map_err(sql_error)?
    };
    let coverage = coverage
        .map(|utc| utc.div_euclid(DAY) * DAY)
        .filter(|d| *d < cutoff);
    let ready: Option<i64> = if deleting {
        connection.query_row(
        "select min(chunk_utc) from retention_state where layer=?1 and status='verified' and chunk_utc<?2",
        params![DAY_EXACT_LAYER,cutoff],|r|r.get(0),
    ).map_err(sql_error)?
    } else {
        None
    };
    Ok([raw, coverage, ready].into_iter().flatten().min())
}

/// 覆盖与 gap 按时间事件求并集，gap 覆盖优先；跨日端点先裁剪。
#[cfg(test)]
fn coverage_for_day(connection: &Connection, day: i64) -> Result<(i64, i64, String), ReportError> {
    let open: bool = connection.query_row(
        "select exists(select 1 from coverage_interval where started_utc < ?1 and ended_utc is null and kind in ('covered','gap'))",
        [day+DAY],|r|r.get(0),
    ).map_err(sql_error)?;
    if open {
        return Err(ReportError::StorageBusy("开放覆盖区间仍受保护"));
    }
    let mut reasons = BTreeSet::new();
    let mut statement = connection.prepare(
        "select max(started_utc,?1) as at,kind,1 as delta,reason from coverage_interval where started_utc<?2 and ended_utc>?1 and kind in ('covered','gap')
         union all select min(ended_utc,?2),kind,-1,reason from coverage_interval where started_utc<?2 and ended_utc>?1 and kind in ('covered','gap')
         order by at"
    ).map_err(sql_error)?;
    let mut rows = statement
        .query(params![day, day + DAY])
        .map_err(sql_error)?;
    let (mut previous, mut cover_depth, mut gap_depth, mut covered, mut gap) = (day, 0, 0, 0, 0);
    while let Some(row) = rows.next().map_err(sql_error)? {
        let at: i64 = row.get(0).map_err(sql_error)?;
        let kind: String = row.get(1).map_err(sql_error)?;
        let delta: i64 = row.get(2).map_err(sql_error)?;
        reasons.insert(row.get::<_, String>(3).map_err(sql_error)?);
        if gap_depth > 0 {
            gap += at - previous;
        } else if cover_depth > 0 {
            covered += at - previous;
        }
        if kind == "gap" {
            gap_depth += delta;
        } else {
            cover_depth += delta;
        }
        previous = at;
    }
    let reasons =
        serde_json::to_string(&reasons).map_err(|_| ReportError::Failed("覆盖原因编码失败"))?;
    Ok((covered, gap, reasons))
}

/// -1 表示引用刚释放，需从头重扫；正数表示有后续页，0 表示最近一轮结束。
pub(super) fn auxiliary_cleanup_pending(connection: &Connection) -> Result<bool, ReportError> {
    connection
        .query_row(
            "select exists(select 1 from retention_watermark
             where layer in ('dictionary_cleanup_cursor','coverage_cleanup_cursor') and watermark_utc<>0)",
            [],
            |row| row.get(0),
        )
        .map_err(sql_error)
}

/// 游标约束扫描的源行数；未命中任何可删对象的页也会前进。
fn cleanup_expired(connection: &Connection, _day: i64, now: i64) -> Result<bool, ReportError> {
    let cutoff = (now - DIMENSION_RETAIN_DAYS * DAY).div_euclid(DAY) * DAY;
    let coverage_cursor = cleanup_cursor(connection, "coverage_cleanup_cursor")?;
    let intervals = {
        let mut statement=connection.prepare("select interval_id,started_utc,ended_utc from coverage_interval where interval_id>?1 order by interval_id limit 128").map_err(sql_error)?;
        let rows = statement
            .query_map([coverage_cursor], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, Option<i64>>(2)?,
                ))
            })
            .map_err(sql_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql_error)?
    };
    for (id, start, end) in &intervals {
        let Some(end) = end.filter(|end| *end <= cutoff && *end > *start) else {
            continue;
        };
        let days:i64=connection.query_row("select count(*) from retention_state where layer=?1 and status in ('verified','deleted') and chunk_utc>=?2 and chunk_utc<?3",
            params![DAY_EXACT_LAYER,start.div_euclid(DAY)*DAY,end],|r|r.get(0)).map_err(sql_error)?;
        if days == (end - 1).div_euclid(DAY) - start.div_euclid(DAY) + 1 {
            connection
                .execute("delete from coverage_interval where interval_id=?1", [id])
                .map_err(sql_error)?;
        }
    }
    save_cleanup_cursor(
        connection,
        "coverage_cleanup_cursor",
        if intervals.len() == 128 {
            intervals.last().map_or(0, |r| r.0)
        } else {
            0
        },
    )?;
    let dictionary_cursor = cleanup_cursor(connection, "dictionary_cleanup_cursor")?;
    let dictionaries = {
        let mut statement=connection.prepare("select rowid,dimension_kind,dimension_id,value from dimension_dict where rowid>?1 order by rowid limit 128").map_err(sql_error)?;
        let rows = statement
            .query_map([dictionary_cursor], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, String>(3)?,
                ))
            })
            .map_err(sql_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql_error)?
    };
    for (rowid, kind, id, value) in &dictionaries {
        if !dictionary_referenced(connection, kind, *id, value)? {
            connection
                .execute("delete from dimension_dict where rowid=?1", [rowid])
                .map_err(sql_error)?;
        }
    }
    save_cleanup_cursor(
        connection,
        "dictionary_cleanup_cursor",
        if dictionaries.len() == 128 {
            dictionaries.last().map_or(0, |r| r.0)
        } else {
            0
        },
    )?;
    Ok(intervals.len() == 128 || dictionaries.len() == 128)
}

fn cleanup_cursor(connection: &Connection, layer: &str) -> Result<i64, ReportError> {
    connection
        .query_row(
            "select watermark_utc from retention_watermark where layer=?1",
            [layer],
            |r| r.get(0),
        )
        .optional()
        .map_err(sql_error)
        .map(|v| v.unwrap_or(0))
}
fn save_cleanup_cursor(
    connection: &Connection,
    layer: &str,
    cursor: i64,
) -> Result<(), ReportError> {
    if cleanup_cursor(connection, layer)? != cursor {
        connection.execute("insert into retention_watermark values (?1,?2,0) on conflict(layer) do update set watermark_utc=excluded.watermark_utc",params![layer,cursor]).map_err(sql_error)?;
    }
    Ok(())
}
fn dictionary_referenced(
    connection: &Connection,
    kind: &str,
    id: i64,
    value: &str,
) -> Result<bool, ReportError> {
    let derived:bool=connection.query_row("select exists(select 1 from traffic_hourly_dimension where dimension_kind=?1 and dimension_id=?2)
        or exists(select 1 from traffic_daily_dimension where dimension_kind=?1 and dimension_id=?2)",params![kind,id],|r|r.get(0)).map_err(sql_error)?;
    if derived {
        return Ok(true);
    }
    let field = match kind {
        "host" => Some("host_id"),
        "process" => Some("process_id"),
        "rule" => Some("rule_id"),
        "network" => Some("network_id"),
        "category" => Some("primary_category_id"),
        _ => None,
    };
    if let Some(field) = field {
        let raw: bool = connection
            .query_row(
                &format!("select exists(select 1 from connection_session_attr where {field}=?1)"),
                [id],
                |r| r.get(0),
            )
            .map_err(sql_error)?;
        if raw {
            return Ok(true);
        }
        if kind == "category" {
            return connection
                .query_row(
                    "select exists(select 1 from traffic_daily_core where category_id=?1)
                or exists(select 1 from traffic_hourly_dimension where category_id=?1)
                or exists(select 1 from traffic_daily_dimension where category_id=?1)",
                    [id],
                    |r| r.get(0),
                )
                .map_err(sql_error);
        }
        return Ok(false);
    }
    match kind {
        "chain"=>connection.query_row("select exists(select 1 from connection_session_attr where chain_identity(chain_key)=?1)",[value],|r|r.get(0)).map_err(sql_error),
        "rule_group"=>connection.query_row("select exists(select 1 from connection_session_attr where last_chain_hop(chain_key)=?1)
            or exists(select 1 from connection_session_attr where last_chain_hop(chain_key) is null and rule_id=(select dimension_id from dimension_dict where dimension_kind='rule' and value=?1))
            or (?1='DIRECT' and exists(select 1 from connection_session_attr where last_chain_hop(chain_key) is null and rule_id is null))",[value],|r|r.get(0)).map_err(sql_error),
        _=>Ok(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, TempDir};

    fn advance_to(db: &mut StorageCoordinator, now: i64, phase: &str) {
        for _ in 0..2000 {
            let found: Option<String> = db
                .connection()
                .query_row(
                    "select phase from retention_build where job_id=1",
                    [],
                    |r| r.get(0),
                )
                .optional()
                .expect("阶段");
            if found.as_deref() == Some(phase) {
                return;
            }
            run_chunk(
                db,
                now,
                30,
                RetentionMode::DeleteEnabled,
                &SpaceBudget::unlimited(),
                &Arc::new(AtomicBool::new(false)),
                true,
            )
            .expect("推进");
        }
        panic!("阶段未到达: {phase}");
    }

    #[test]
    fn corruption_after_publication_before_confirmation_blocks_all_deletion() {
        let (_dir, mut db) = fixture();
        advance_to(&mut db, 40 * DAY, "publish");
        run_chunk(
            &mut db,
            40 * DAY,
            30,
            RetentionMode::DeleteEnabled,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
            true,
        )
        .expect("首批发布");
        db.connection()
            .execute(
                "update traffic_daily_core set upload=upload+1 where category_id=0",
                [],
            )
            .expect("发布后损坏");
        assert!(run_chunk(
            &mut db,
            40 * DAY,
            30,
            RetentionMode::DeleteEnabled,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
            true
        )
        .is_err());
        assert_eq!(scalar(&db, "select count(*) from connection_minute"), 4);
        assert_eq!(
            scalar(
                &db,
                "select count(*) from retention_state where layer='day_exact_v1'"
            ),
            0
        );
    }

    #[test]
    fn resume_delete_honors_current_mode_and_gate() {
        let (_dir, mut db) = fixture();
        advance_to(&mut db, 40 * DAY, "delete");
        for (mode, gate) in [
            (RetentionMode::MaterializeOnly, true),
            (RetentionMode::DeleteEnabled, false),
        ] {
            let chunk = run_chunk(
                &mut db,
                40 * DAY,
                30,
                mode,
                &SpaceBudget::unlimited(),
                &Arc::new(AtomicBool::new(false)),
                gate,
            )
            .expect("关闭门恢复");
            assert_eq!(chunk.deleted_raw_rows, 0);
            assert_eq!(scalar(&db, "select count(*) from connection_minute"), 4);
        }
        assert_eq!(
            run(&mut db, 40 * DAY, true)
                .expect("重新开放门")
                .deleted_raw_rows,
            4
        );
    }

    #[test]
    fn changed_metadata_discards_unconfirmed_output_and_rebuilds() {
        let (_dir, mut db) = fixture();
        advance_to(&mut db, 40 * DAY, "publish");
        run_chunk(
            &mut db,
            40 * DAY,
            30,
            RetentionMode::DeleteEnabled,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
            true,
        )
        .expect("发布一批");
        db.connection()
            .execute(
                "update connection_session_attr set primary_category_id=2 where session_pk=1",
                [],
            )
            .expect("规范元数据变化");
        assert!(run_chunk(
            &mut db,
            40 * DAY,
            30,
            RetentionMode::DeleteEnabled,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
            true
        )
        .is_err());
        assert_eq!(scalar(&db, "select count(*) from connection_minute"), 4);
        assert_eq!(
            run(&mut db, 40 * DAY, true)
                .expect("丢弃并重建")
                .deleted_raw_rows,
            4
        );
        assert_eq!(
            scalar(
                &db,
                "select download from traffic_daily_core where category_id=2"
            ),
            41
        );
        assert_eq!(
            scalar(
                &db,
                "select download from traffic_daily_core where category_id=0"
            ),
            106
        );
    }

    #[test]
    fn coverage_only_day_and_more_than_one_cleanup_page_are_reclaimed() {
        let dir = tempdir().expect("目录");
        let mut db =
            StorageCoordinator::open(&dir.path().join("coverage-only.sqlite3")).expect("打开");
        for i in 0..700 {
            db.connection().execute("insert into coverage_interval(kind,reason,started_utc,ended_utc) values ('gap','lost',?1,?2)",params![i,i+1]).expect("区间");
        }
        run(&mut db, 40 * DAY, true).expect("无流量日守恒");
        assert_eq!(
            scalar(&db, "select count(*) from traffic_daily_dimension"),
            0
        );
        assert_eq!(scalar(&db, "select gap_sec from coverage_daily"), 700);
        run(&mut db, 400 * DAY, true).expect("独立覆盖清理");
        assert_eq!(scalar(&db, "select count(*) from coverage_interval"), 0);
        assert_eq!(scalar(&db, "select gap_sec from coverage_daily"), 700);
    }

    #[test]
    fn final_dimension_prune_finishes_a_fresh_multi_page_dictionary_sweep() {
        let dir = tempdir().expect("目录");
        let mut db =
            StorageCoordinator::open(&dir.path().join("dictionary-prune.sqlite3")).expect("打开");
        db.connection().execute_batch(
            "insert into dimension_dict values('category',1,'保留分类');
             with recursive ids(n) as(values(1) union all select n+1 from ids where n<260)
             insert into dimension_dict select 'host',n,'host-'||n from ids;
             insert into connection_session(session_pk,epoch_id,connection_id,started_utc,host)
             select dimension_id,1,'session-'||dimension_id,0,value from dimension_dict where dimension_kind='host';
             insert into connection_session_attr(session_pk,host_id,primary_category_id,policy_version,started_utc,ended_utc)
             select session_pk,session_pk,1,1,0,172800 from connection_session;
             insert into connection_minute select 10,session_pk,1,2 from connection_session;
             insert into connection_minute select 1450,session_pk,1,2 from connection_session;"
        ).expect("两个完整日共享超过一页的字典引用");
        run(&mut db, 40 * DAY, true).expect("封存两日");
        assert_eq!(
            db.cleanup_retired_sessions(10 * DAY, 1000, &[])
                .expect("清理会话"),
            260
        );
        run(&mut db, 40 * DAY, true).expect("仍受精确汇总引用保护");
        assert_eq!(
            scalar(
                &db,
                "select count(*) from dimension_dict where dimension_kind='host'"
            ),
            260
        );

        // 首日清理时旧字典页仍受第二日保护，最后一日退出后必须从头再扫。
        run(&mut db, 400 * DAY, true).expect("首次无待处理工作应已完成字典全遍");
        assert_eq!(
            scalar(&db, "select count(*) from traffic_daily_dimension"),
            0
        );
        assert_eq!(
            scalar(&db, "select count(*) from traffic_hourly_dimension"),
            0
        );
        assert_eq!(scalar(&db, "select count(*) from dimension_dict"), 1);
        assert_eq!(scalar(&db, "select count(*) from dimension_dict where dimension_kind='category' and dimension_id=1"), 1);
        assert_eq!(
            scalar(
                &db,
                "select sum(upload) from traffic_daily_core where category_id=0"
            ),
            520
        );
        assert_eq!(scalar(&db, "select count(*) from coverage_daily"), 2);
        assert!(!RetentionService::auxiliary_cleanup_pending(db.connection()).expect("游标结束"));
    }

    #[test]
    fn ledger_deletion_invalidates_completed_dictionary_sweep_and_preserves_protected_refs() {
        let dir = tempdir().expect("目录");
        let mut db =
            StorageCoordinator::open(&dir.path().join("dictionary-ledger.sqlite3")).expect("打开");
        db.connection().execute_batch(
            "insert into dimension_dict values('category',1,'长期分类');
             insert into traffic_daily_core values(0,1,25,50,1,60);
             with recursive ids(n) as(values(1) union all select n+1 from ids where n<201)
             insert into dimension_dict select 'host',n,'host-'||n from ids;
             insert into connection_session(session_pk,epoch_id,connection_id,started_utc,host)
             select dimension_id,1,'session-'||dimension_id,0,value from dimension_dict where dimension_kind='host';
             insert into connection_session_attr(session_pk,host_id,policy_version,started_utc,ended_utc)
             select session_pk,session_pk,1,0,1 from connection_session;"
        ).expect("辅助清理之后才释放的引用");
        run(&mut db, 400 * DAY, true).expect("字典全遍结束但仍有会话保护");
        assert!(!RetentionService::auxiliary_cleanup_pending(db.connection()).expect("已结束"));
        assert_eq!(
            scalar(
                &db,
                "select count(*) from dimension_dict where dimension_kind='host'"
            ),
            201
        );
        let protected = ["1:session-201".to_string()];
        assert_eq!(
            db.cleanup_retired_sessions(1000, 50, &protected)
                .expect("之后释放首批引用"),
            50
        );
        assert_eq!(
            cleanup_cursor(db.connection(), "dictionary_cleanup_cursor").expect("失效游标"),
            -1
        );
        assert!(
            RetentionService::auxiliary_cleanup_pending(db.connection()).expect("旧完成信号失效")
        );

        let cancel = Arc::new(AtomicBool::new(false));
        for (mode, gate) in [
            (RetentionMode::MaterializeOnly, true),
            (RetentionMode::DeleteEnabled, false),
        ] {
            let chunk = run_chunk(
                &mut db,
                400 * DAY,
                30,
                mode,
                &SpaceBudget::unlimited(),
                &cancel,
                gate,
            )
            .expect("关闭删除门");
            assert!(!chunk.more_pending, "关闭删除门不因旧游标持续调度");
            assert_eq!(
                cleanup_cursor(db.connection(), "dictionary_cleanup_cursor").expect("保留游标"),
                -1
            );
        }
        let mut complete = false;
        for _ in 0..30 {
            let chunk = run_chunk(
                &mut db,
                400 * DAY,
                30,
                RetentionMode::DeleteEnabled,
                &SpaceBudget::unlimited(),
                &cancel,
                true,
            )
            .expect("有界字典页");
            // 与生产/driver 相同：辅助账本在 retention 块返回后运行。
            db.cleanup_retired_sessions(1000, 50, &protected)
                .expect("后续引用释放");
            let pending =
                RetentionService::auxiliary_cleanup_pending(db.connection()).expect("提交后复核");
            if !chunk.more_pending
                && !pending
                && scalar(
                    &db,
                    "select count(*) from connection_session where session_pk<=200",
                ) == 0
            {
                complete = true;
                break;
            }
        }
        assert!(complete, "有限游标扫描应收敛");
        assert_eq!(scalar(&db, "select count(*) from dimension_dict"), 2);
        assert_eq!(scalar(&db, "select count(*) from dimension_dict where dimension_kind='host' and dimension_id=201"), 1);
        assert_eq!(
            scalar(&db, "select count(*) from connection_session_attr"),
            1
        );
        assert_eq!(
            scalar(
                &db,
                "select upload from traffic_daily_core where category_id=1"
            ),
            25
        );
    }

    #[test]
    fn frozen_core_mutation_mid_prune_preserves_exact_layers() {
        let (_dir, mut db) = fixture();
        run(&mut db, 40 * DAY, true).expect("清理 raw");
        advance_to(&mut db, 400 * DAY, "prune_check");
        run_chunk(
            &mut db,
            400 * DAY,
            30,
            RetentionMode::DeleteEnabled,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
            true,
        )
        .expect("核对 core");
        db.connection()
            .execute(
                "update traffic_daily_core set download=download+1 where category_id=0",
                [],
            )
            .expect("核对后损坏");
        assert!(run_chunk(
            &mut db,
            400 * DAY,
            30,
            RetentionMode::DeleteEnabled,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
            true
        )
        .is_err());
        assert!(scalar(&db, "select count(*) from traffic_daily_dimension") > 0);
    }

    #[test]
    fn nonoverlapping_coverage_source_pages_still_advance() {
        let (_dir, mut db) = fixture();
        db.connection().execute_batch("update connection_minute set utc_minute=utc_minute+2880; delete from coverage_interval;
            insert into retention_watermark values ('coverage_exact_v1',172800,0);").expect("历史覆盖已处理");
        for i in 0..200 {
            db.connection().execute("insert into coverage_interval(kind,reason,started_utc,ended_utc) values ('gap','old',?1,?2)",params![i,i+1]).expect("旧区间");
        }
        advance_to(&mut db, 40 * DAY, "coverage_scan");
        run_chunk(
            &mut db,
            40 * DAY,
            30,
            RetentionMode::DeleteEnabled,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
            true,
        )
        .expect("无命中源页");
        assert_eq!(
            scalar(&db, "select cursor_minute from retention_build"),
            127
        );
        assert_eq!(
            scalar(
                &db,
                "select count(*) from retention_build_member where granularity='coverage_event'"
            ),
            0
        );
        assert_eq!(
            run(&mut db, 40 * DAY, true)
                .expect("继续到完成")
                .deleted_raw_rows,
            4
        );
    }

    #[test]
    fn audited_output_is_protected_while_coverage_builds() {
        for phase in ["coverage_scan", "coverage_union"] {
            let (_dir, mut db) = fixture();
            advance_to(&mut db, 40 * DAY, phase);
            db.connection()
                .execute(
                    "update traffic_daily_core set download=download+1 where category_id=0",
                    [],
                )
                .expect("已审计结果损坏");
            assert!(run_chunk(
                &mut db,
                40 * DAY,
                30,
                RetentionMode::DeleteEnabled,
                &SpaceBudget::unlimited(),
                &Arc::new(AtomicBool::new(false)),
                true
            )
            .is_err());
            assert_eq!(scalar(&db, "select count(*) from connection_minute"), 4);
            assert_eq!(
                scalar(
                    &db,
                    "select count(*) from retention_state where layer='day_exact_v1'"
                ),
                0
            );
        }
    }

    #[test]
    fn invalidation_before_publication_keeps_unproven_legacy_output() {
        let (_dir, mut db) = fixture();
        db.connection()
            .execute(
                "insert into traffic_hourly_dimension values (3600,1,'host',999,99,99,1,60)",
                [],
            )
            .expect("无 raw 旁证的旧汇总");
        run_chunk(
            &mut db,
            40 * DAY,
            30,
            RetentionMode::DeleteEnabled,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
            true,
        )
        .expect("首批 raw");
        db.connection()
            .execute(
                "update connection_session_attr set primary_category_id=2 where session_pk=1",
                [],
            )
            .expect("构建中变化");
        assert!(run(&mut db, 40 * DAY, true).is_err());
        assert!(run(&mut db, 40 * DAY, true).is_err());
        assert_eq!(
            scalar(
                &db,
                "select download from traffic_hourly_dimension where dimension_id=999"
            ),
            99
        );
        assert_eq!(scalar(&db, "select count(*) from connection_minute"), 4);
    }

    #[test]
    fn coverage_source_page_uses_range_index_without_temp_sort() {
        let (_dir, db) = fixture();
        let mut stmt=db.connection().prepare("explain query plan select interval_id,kind,reason,started_utc,ended_utc from coverage_interval indexed by idx_retention_coverage_source
            where kind in ('covered','gap') and started_utc<?1 and (started_utc,interval_id)>(?2,?3) order by started_utc,interval_id limit ?4").expect("计划");
        let details = stmt
            .query_map(params![86400, 0, 0, 128], |r| r.get::<_, String>(3))
            .expect("解释")
            .collect::<Result<Vec<_>, _>>()
            .expect("计划行");
        assert!(details
            .iter()
            .any(|row| row.contains("idx_retention_coverage_source")));
        assert!(
            details.iter().all(|row| !row.contains("TEMP B-TREE")),
            "{details:?}"
        );
    }

    #[test]
    fn coverage_write_fault_cannot_damage_audited_core_and_advance_watermark() {
        let (_dir, mut db) = fixture();
        db.connection()
            .execute_batch(
                "create trigger corrupt_core_on_coverage after insert on coverage_daily
            begin update traffic_daily_core set download=download+1 where category_id=0; end;",
            )
            .expect("跨表故障");
        assert!(run(&mut db, 40 * DAY, true).is_err());
        assert_eq!(scalar(&db, "select count(*) from connection_minute"), 4);
        assert_eq!(
            scalar(
                &db,
                "select count(*) from retention_state where layer='day_exact_v1'"
            ),
            0
        );
        assert_eq!(
            scalar(
                &db,
                "select download from traffic_daily_core where category_id=0"
            ),
            106
        );
    }

    fn fixture() -> (TempDir, StorageCoordinator) {
        let dir = tempdir().expect("目录");
        let db = StorageCoordinator::open(&dir.path().join("retention.sqlite3")).expect("打开");
        db.seed_report_fixture().expect("基础数据");
        db.connection()
            .execute_batch(
                "delete from traffic_hourly_dimension;
            insert into connection_minute values (80,1,7,11),(80,2,3,5);",
            )
            .expect("跨小时数据");
        (dir, db)
    }

    fn run(
        db: &mut StorageCoordinator,
        now: i64,
        delete: bool,
    ) -> Result<RetentionChunk, ReportError> {
        drain(db, now, delete, &Arc::new(AtomicBool::new(false)))
    }
    fn drain(
        db: &mut StorageCoordinator,
        now: i64,
        delete: bool,
        cancel: &Arc<AtomicBool>,
    ) -> Result<RetentionChunk, ReportError> {
        let mut total = RetentionChunk::default();
        for _ in 0..300 {
            let chunk = run_chunk(
                db,
                now,
                30,
                if delete {
                    RetentionMode::DeleteEnabled
                } else {
                    RetentionMode::MaterializeOnly
                },
                &SpaceBudget::unlimited(),
                cancel,
                true,
            )?;
            total.day_utc = total.day_utc.or(chunk.day_utc);
            total.deleted_raw_rows += chunk.deleted_raw_rows;
            if !chunk.more_pending {
                return Ok(total);
            }
        }
        Err(ReportError::Failed("测试构建未结束"))
    }

    fn scalar(db: &StorageCoordinator, sql: &str) -> i64 {
        db.connection()
            .query_row(sql, [], |r| r.get(0))
            .expect("标量")
    }

    #[test]
    fn full_day_distinct_duration_and_all_five_dimensions_survive_real_delete() {
        let (_dir, mut db) = fixture();
        let chunk = run(&mut db, 40 * DAY, false).expect("物化");
        assert_eq!(chunk.day_utc, Some(0));
        assert_eq!(scalar(&db, "select count(*) from connection_minute"), 4);
        assert_eq!(
            scalar(
                &db,
                "select connection_count from traffic_daily_core where category_id=0"
            ),
            2
        );
        assert_eq!(
            scalar(
                &db,
                "select active_duration_sec from traffic_daily_core where category_id=0"
            ),
            180
        );
        assert_eq!(scalar(&db,"select connection_count from traffic_daily_dimension where dimension_kind='host' and dimension_id=1"),1);
        assert_eq!(
            scalar(
                &db,
                "select count(distinct dimension_kind) from traffic_hourly_dimension"
            ),
            5
        );
        assert_eq!(
            scalar(
                &db,
                "select watermark_utc from retention_watermark where layer='hourly_dim_v2'"
            ),
            0
        );
        let deleted = run(&mut db, 40 * DAY, true).expect("真实删除分支");
        assert_eq!(deleted.deleted_raw_rows, 4);
        assert_eq!(scalar(&db, "select count(*) from connection_minute"), 0);
        assert_eq!(
            scalar(
                &db,
                "select download from traffic_daily_core where category_id=0"
            ),
            106
        );
        assert_eq!(
            run(&mut db, 40 * DAY, true).expect("重复删除"),
            RetentionChunk::default()
        );
        assert_eq!(
            scalar(
                &db,
                "select download from traffic_daily_core where category_id=0"
            ),
            106
        );
    }

    #[test]
    fn last_hour_corruption_rolls_back_entire_day_and_all_raw_hours() {
        let (_dir, mut db) = fixture();
        db.connection().execute_batch("create trigger corrupt_last_hour after insert on traffic_hourly_dimension
            when new.utc_hour=3600 and new.dimension_kind='network'
            begin update traffic_hourly_dimension set download=download+1 where utc_hour=new.utc_hour and dimension_kind='network'; end;")
            .expect("故障注入");
        assert!(run(&mut db, 40 * DAY, true).is_err());
        assert_eq!(scalar(&db, "select count(*) from connection_minute"), 4);
        assert_eq!(
            scalar(&db, "select count(*) from traffic_hourly_dimension"),
            0
        );
        assert_eq!(scalar(&db, "select count(*) from traffic_daily_core"), 0);
        assert_eq!(
            scalar(
                &db,
                "select count(*) from retention_state where layer='day_exact_v1'"
            ),
            0
        );
        assert_eq!(
            scalar(
                &db,
                "select count(*) from retention_watermark where layer='day_exact_v1'"
            ),
            0
        );
    }

    #[test]
    fn damaged_dimension_or_coverage_blocks_delete_and_preserves_confirmation() {
        for corrupt in [
            "update traffic_daily_dimension set upload=upload+1 where dimension_kind='process'",
            "update coverage_daily set gap_sec=gap_sec+1",
        ] {
            let (_dir, mut db) = fixture();
            run(&mut db, 40 * DAY, false).expect("物化");
            db.connection().execute_batch(corrupt).expect("损坏");
            assert!(run(&mut db, 40 * DAY, true).is_err());
            assert_eq!(scalar(&db, "select count(*) from connection_minute"), 4);
            assert_eq!(scalar(&db,"select delete_watermark_utc from retention_watermark where layer='day_exact_v1'"),0);
            assert_eq!(scalar(&db,"select count(*) from retention_state where layer='day_exact_v1' and status='verified'"),1);
        }
    }

    #[test]
    fn cross_midnight_overlap_is_clipped_and_open_intervals_block_finalization() {
        let (_dir, mut db) = fixture();
        db.connection().execute_batch("delete from coverage_interval;
            insert into coverage_interval(kind,reason,started_utc,ended_utc) values
            ('covered','running',86300,86700),('gap','disconnect',86360,86460),('gap','disconnect',86400,86480);")
            .expect("跨日覆盖");
        assert_eq!(
            coverage_for_day(db.connection(), 0).expect("前一天"),
            (60, 40, "[\"disconnect\",\"running\"]".into())
        );
        assert_eq!(
            coverage_for_day(db.connection(), DAY).expect("后一天"),
            (220, 80, "[\"disconnect\",\"running\"]".into())
        );
        run(&mut db, 40 * DAY, false).expect("前一天物化");
        run(&mut db, 40 * DAY, false).expect("后一天无流量也物化");
        assert_eq!(scalar(&db, "select count(*) from coverage_daily"), 2);
        let (_dir, mut db) = fixture();
        db.connection()
            .execute(
                "insert into coverage_interval(kind,reason,started_utc) values ('gap','unknown',0)",
                [],
            )
            .expect("开放区间");
        let error = run(&mut db, 40 * DAY, true).expect_err("开放区间保护");
        assert_eq!(error.code(), "storage_busy");
        assert_eq!(scalar(&db, "select count(*) from connection_minute"), 4);
        assert_eq!(
            scalar(
                &db,
                "select count(*) from coverage_interval where ended_utc is null"
            ),
            1
        );
    }

    #[test]
    fn busy_low_space_and_cancel_never_commit_partial_day() {
        let (dir, mut db) = fixture();
        let cancelled = Arc::new(AtomicBool::new(true));
        let error = run_chunk(
            &mut db,
            40 * DAY,
            30,
            RetentionMode::DeleteEnabled,
            &SpaceBudget::unlimited(),
            &cancelled,
            true,
        )
        .expect_err("取消");
        assert_eq!(error.code(), "cancelled");
        let error = run_chunk(
            &mut db,
            40 * DAY,
            30,
            RetentionMode::DeleteEnabled,
            &SpaceBudget::exhausted(),
            &Arc::new(AtomicBool::new(false)),
            true,
        )
        .expect_err("低空间");
        assert_eq!(error.code(), "insufficient_space");
        let lock = Connection::open(dir.path().join("retention.sqlite3")).expect("占用连接");
        lock.execute_batch("begin immediate").expect("占用 writer");
        let error = run(&mut db, 40 * DAY, true).expect_err("busy");
        assert_eq!(error.code(), "storage_busy");
        lock.execute_batch("rollback").expect("解锁");
        assert_eq!(scalar(&db, "select count(*) from connection_minute"), 4);
        assert_eq!(
            scalar(
                &db,
                "select count(*) from retention_state where layer='day_exact_v1'"
            ),
            0
        );
        run(&mut db, 40 * DAY, true).expect("恢复后成功");
    }

    #[test]
    fn mid_statement_cancel_rolls_back_and_can_resume() {
        let (_dir, mut db) = fixture();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancel);
        db.connection()
            .create_scalar_function(
                "cancel_retention",
                0,
                rusqlite::functions::FunctionFlags::SQLITE_UTF8,
                move |_| {
                    flag.store(true, Ordering::SeqCst);
                    Ok(1_i64)
                },
            )
            .expect("取消回调");
        db.connection().execute_batch("create trigger interrupt_final_day after insert on traffic_daily_core begin select cancel_retention(); end;").expect("中途取消");
        let error = drain(&mut db, 40 * DAY, true, &cancel).expect_err("取消生效");
        assert_eq!(error.code(), "cancelled");
        assert!(db.connection().is_autocommit());
        assert_eq!(scalar(&db, "select count(*) from connection_minute"), 4);
        assert_eq!(scalar(&db, "select count(*) from traffic_daily_core"), 0);
        db.connection()
            .execute_batch("drop trigger interrupt_final_day")
            .expect("解除故障");
        assert_eq!(
            run(&mut db, 40 * DAY, true).expect("恢复").deleted_raw_rows,
            4
        );
    }

    #[test]
    fn cancel_during_raw_delete_preserves_rows_and_writer_transaction() {
        let (_dir, mut db) = fixture();
        advance_to(&mut db, 40 * DAY, "delete");
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancel);
        db.connection()
            .create_scalar_function(
                "cancel_raw_delete",
                0,
                rusqlite::functions::FunctionFlags::SQLITE_UTF8,
                move |_| {
                    flag.store(true, Ordering::SeqCst);
                    Ok(1_i64)
                },
            )
            .expect("取消回调");
        db.connection().execute_batch("create temp trigger interrupt_raw_delete after delete on connection_minute begin select cancel_raw_delete(); end;").expect("删除途中取消");
        let error = run_chunk(
            &mut db,
            40 * DAY,
            30,
            RetentionMode::DeleteEnabled,
            &SpaceBudget::unlimited(),
            &cancel,
            true,
        )
        .expect_err("取消删除");
        assert_eq!(error.code(), "cancelled");
        assert!(db.connection().is_autocommit());
        assert_eq!(scalar(&db, "select count(*) from connection_minute"), 4);
        db.connection()
            .execute_batch("drop trigger interrupt_raw_delete")
            .expect("解除取消");
        assert_eq!(
            run(&mut db, 40 * DAY, true)
                .expect("恢复删除")
                .deleted_raw_rows,
            4
        );
    }

    #[test]
    fn restart_after_confirmation_then_expiry_preserves_long_term_core() {
        let (dir, mut db) = fixture();
        run(&mut db, 40 * DAY, false).expect("确认");
        drop(db);
        let mut db = StorageCoordinator::open(&dir.path().join("retention.sqlite3")).expect("重启");
        assert_eq!(
            run(&mut db, 40 * DAY, true)
                .expect("恢复删除")
                .deleted_raw_rows,
            4
        );
        assert!(scalar(&db, "select count(*) from traffic_daily_dimension") > 0);
        run(&mut db, 400 * DAY, true).expect("396 天维度回收");
        assert_eq!(
            scalar(&db, "select count(*) from traffic_daily_dimension"),
            0
        );
        assert_eq!(
            scalar(&db, "select count(*) from traffic_hourly_dimension"),
            0
        );
        assert_eq!(scalar(&db, "select count(*) from coverage_interval"), 0);
        assert_eq!(
            scalar(
                &db,
                "select download from traffic_daily_core where category_id=0"
            ),
            106
        );
        assert_eq!(scalar(&db, "select count(*) from coverage_daily"), 1);
    }

    #[test]
    fn dry_run_has_no_side_effects_and_late_raw_cannot_rebuild_deleted_day() {
        let (_dir, mut db) = fixture();
        let changes = db.connection().total_changes();
        run_chunk(
            &mut db,
            40 * DAY,
            30,
            RetentionMode::DryRun,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
            true,
        )
        .expect("预览");
        assert_eq!(db.connection().total_changes(), changes);
        run(&mut db, 40 * DAY, true).expect("删除");
        db.connection()
            .execute("insert into connection_minute values (20,1,1,1)", [])
            .expect("迟到明细");
        assert!(run(&mut db, 40 * DAY, true).is_err());
        assert_eq!(
            scalar(
                &db,
                "select download from traffic_daily_core where category_id=0"
            ),
            106
        );
        assert_eq!(scalar(&db, "select count(*) from connection_minute"), 1);
    }
}
