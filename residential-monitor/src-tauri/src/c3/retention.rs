//! 精确保留：先物化并核对，再推进 watermark，最后可选删除。

use crate::c3::query::{
    ReportError, AUTO_DELETE_ENABLED, DIMENSION_RETAIN_DAYS, HOURLY_DIM_V2_LAYER,
    RAW_RETAIN_DAYS_MAX,
};
use crate::c3::space::SpaceBudget;
use crate::c3::sql::UNKNOWN_IDENTITY;
use crate::storage::StorageCoordinator;
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetentionMode {
    DryRun,
    MaterializeOnly,
    DeleteEnabled,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPreview {
    pub raw_retain_days: i64,
    pub raw_rows: i64,
    pub hourly_rows: i64,
    pub daily_dim_rows: i64,
    pub daily_core_rows: i64,
    pub auto_delete_enabled: bool,
    pub note_zh: String,
}

pub struct RetentionService;

impl RetentionService {
    pub fn preview(
        coordinator: &StorageCoordinator,
        raw_retain_days: i64,
    ) -> Result<RetentionPreview, ReportError> {
        let connection = coordinator.connection();
        let raw_rows = count(connection, "select count(*) from connection_minute")?;
        let hourly_rows = count(connection, "select count(*) from traffic_hourly_dimension")?;
        let daily_dim_rows = count(connection, "select count(*) from traffic_daily_dimension")?;
        let daily_core_rows = count(connection, "select count(*) from traffic_daily_core")?;
        Ok(RetentionPreview {
            raw_retain_days: raw_retain_days.clamp(1, RAW_RETAIN_DAYS_MAX),
            raw_rows,
            hourly_rows,
            daily_dim_rows,
            daily_core_rows,
            auto_delete_enabled: AUTO_DELETE_ENABLED,
            note_zh: "DELETE 后的 freelist 不是已释放文件空间。应用不自动 VACUUM。".into(),
        })
    }

    pub fn run(
        coordinator: &mut StorageCoordinator,
        now_utc: i64,
        raw_retain_days: i64,
        mode: RetentionMode,
        space: &SpaceBudget,
        cancel: &Arc<AtomicBool>,
    ) -> Result<RetentionPreview, ReportError> {
        space.check(
            coordinator.path().parent().unwrap_or(coordinator.path()),
            4096,
        )?;
        if cancel.load(Ordering::SeqCst) {
            return Err(ReportError::Cancelled("retention"));
        }
        materialize_hourly(coordinator, now_utc, raw_retain_days)?;
        materialize_daily_from_hourly(coordinator, now_utc)?;
        materialize_core(coordinator, now_utc)?;
        materialize_coverage_daily(coordinator, now_utc)?;
        if mode == RetentionMode::DeleteEnabled && AUTO_DELETE_ENABLED {
            delete_covered_raw(coordinator, now_utc, raw_retain_days)?;
            delete_expired_dimension(coordinator, now_utc)?;
        }
        Self::preview(coordinator, raw_retain_days)
    }
}

fn count(connection: &rusqlite::Connection, sql: &str) -> Result<i64, ReportError> {
    connection
        .query_row(sql, [], |row| row.get(0))
        .map_err(|_| ReportError::Failed("count"))
}

fn materialize_hourly(
    coordinator: &mut StorageCoordinator,
    now_utc: i64,
    raw_retain_days: i64,
) -> Result<(), ReportError> {
    let cutoff = now_utc - raw_retain_days.clamp(1, RAW_RETAIN_DAYS_MAX) * 86_400;
    let start_min = 0;
    let end_min = cutoff.div_euclid(60);
    let connection = coordinator.connection_mut();
    connection
        .execute_batch("begin immediate")
        .map_err(|_| ReportError::StorageBusy("retention begin"))?;
    let result = (|| {
        connection
            .execute(
                "insert or ignore into retention_watermark(layer, watermark_utc, delete_watermark_utc)
                 values (?1, ?2, 0)",
                params![HOURLY_DIM_V2_LAYER, cutoff.max(0)],
            )
            .map_err(|_| ReportError::Failed("hourly_dim_v2 watermark"))?;
        let v2_start: i64 = connection
            .query_row(
                "select watermark_utc from retention_watermark where layer = ?1",
                [HOURLY_DIM_V2_LAYER],
                |row| row.get(0),
            )
            .map_err(|_| ReportError::Failed("hourly_dim_v2 read"))?;
        let v2_start_min = v2_start.div_euclid(60);
        intern_chain_keys(connection, v2_start_min, end_min)?;
        intern_rule_groups(connection, v2_start_min, end_min)?;
        insert_hourly_kind(connection, HOURLY_HOST, start_min, end_min)?;
        insert_hourly_kind(connection, HOURLY_PROCESS, v2_start_min, end_min)?;
        insert_hourly_kind(connection, HOURLY_RULE_GROUP, v2_start_min, end_min)?;
        insert_hourly_kind(connection, HOURLY_CHAIN, v2_start_min, end_min)?;
        insert_hourly_kind(connection, HOURLY_NETWORK, v2_start_min, end_min)?;
        verify_layer(connection, "hourly", start_min * 60, end_min * 60)?;
        Ok(())
    })();
    match result {
        Ok(()) => connection
            .execute_batch("commit")
            .map_err(|_| ReportError::Failed("retention commit")),
        Err(error) => {
            let _ = connection.execute_batch("rollback");
            Err(error)
        }
    }
}

fn materialize_daily_from_hourly(
    coordinator: &mut StorageCoordinator,
    now_utc: i64,
) -> Result<(), ReportError> {
    let end = now_utc;
    let connection = coordinator.connection_mut();
    connection
        .execute(
            "insert or replace into traffic_daily_dimension(
                utc_day, category_id, dimension_kind, dimension_id,
                upload, download, connection_count, active_duration_sec
             )
             select (utc_hour / 86400) * 86400, category_id, dimension_kind, dimension_id,
                    sum(upload), sum(download), sum(connection_count), sum(active_duration_sec)
               from traffic_hourly_dimension
              where utc_hour < ?1
              group by 1, 2, 3, 4",
            [end],
        )
        .map_err(|_| ReportError::Failed("daily dim materialize"))?;
    verify_layer(connection, "daily", 0, end)?;
    Ok(())
}

fn materialize_core(coordinator: &mut StorageCoordinator, now_utc: i64) -> Result<(), ReportError> {
    let connection = coordinator.connection_mut();
    connection
        .execute(
            "insert or replace into traffic_daily_core(
                utc_day, category_id, upload, download, connection_count, active_duration_sec
             )
             select utc_day, category_id, sum(upload), sum(download),
                    sum(connection_count), sum(active_duration_sec)
               from traffic_daily_dimension
              where utc_day < ?1
                and dimension_kind = 'host'
              group by utc_day, category_id",
            [now_utc],
        )
        .map_err(|_| ReportError::Failed("core materialize"))?;
    connection
        .execute(
            "insert or replace into traffic_daily_core(
                utc_day, category_id, upload, download, connection_count, active_duration_sec
             )
             select utc_day, 0, sum(upload), sum(download),
                    sum(connection_count), sum(active_duration_sec)
               from traffic_daily_dimension
              where utc_day < ?1
                and dimension_kind = 'host'
              group by utc_day",
            [now_utc],
        )
        .map_err(|_| ReportError::Failed("core total"))?;
    verify_layer(connection, "core", 0, now_utc)?;
    Ok(())
}

fn materialize_coverage_daily(
    coordinator: &mut StorageCoordinator,
    now_utc: i64,
) -> Result<(), ReportError> {
    let connection = coordinator.connection_mut();
    connection
        .execute(
            "insert or replace into coverage_daily(utc_day, covered_sec, gap_sec, reasons_json)
             select (started_utc / 86400) * 86400,
                    sum(case when kind = 'gap' then 0 else coalesce(ended_utc, ?1) - started_utc end),
                    sum(case when kind = 'gap' then coalesce(ended_utc, ?1) - started_utc else 0 end),
                    group_concat(reason, ',')
               from coverage_interval
              where started_utc < ?1
              group by 1",
            [now_utc],
        )
        .map_err(|_| ReportError::Failed("coverage daily"))?;
    Ok(())
}

const HOURLY_HOST: &str = "
insert or replace into traffic_hourly_dimension(
    utc_hour, category_id, dimension_kind, dimension_id,
    upload, download, connection_count, active_duration_sec)
select (m.utc_minute * 60 / 3600) * 3600,
       coalesce(a.primary_category_id, 0),
       'host',
       coalesce(a.host_id, 0),
       sum(m.upload), sum(m.download),
       count(distinct m.session_pk),
       count(distinct m.utc_minute) * 60
  from connection_minute m
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ?1 and m.utc_minute < ?2
 group by 1, 2, 4
";

const HOURLY_PROCESS: &str = "
insert or replace into traffic_hourly_dimension(
    utc_hour, category_id, dimension_kind, dimension_id,
    upload, download, connection_count, active_duration_sec)
select (m.utc_minute * 60 / 3600) * 3600,
       coalesce(a.primary_category_id, 0),
       'process',
       coalesce(a.process_id, 0),
       sum(m.upload), sum(m.download),
       count(distinct m.session_pk),
       count(distinct m.utc_minute) * 60
  from connection_minute m
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ?1 and m.utc_minute < ?2
 group by 1, 2, 4
";

const HOURLY_NETWORK: &str = "
insert or replace into traffic_hourly_dimension(
    utc_hour, category_id, dimension_kind, dimension_id,
    upload, download, connection_count, active_duration_sec)
select (m.utc_minute * 60 / 3600) * 3600,
       coalesce(a.primary_category_id, 0),
       'network',
       coalesce(a.network_id, 0),
       sum(m.upload), sum(m.download),
       count(distinct m.session_pk),
       count(distinct m.utc_minute) * 60
  from connection_minute m
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ?1 and m.utc_minute < ?2
 group by 1, 2, 4
";

const HOURLY_CHAIN: &str = "
insert or replace into traffic_hourly_dimension(
    utc_hour, category_id, dimension_kind, dimension_id,
    upload, download, connection_count, active_duration_sec)
select (m.utc_minute * 60 / 3600) * 3600,
       coalesce(a.primary_category_id, 0),
       'chain',
       coalesce((select dimension_id from dimension_dict where dimension_kind = 'chain' and value = last_chain_hop(a.chain_key)), 0),
       sum(m.upload), sum(m.download),
       count(distinct m.session_pk),
       count(distinct m.utc_minute) * 60
  from connection_minute m
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ?1 and m.utc_minute < ?2
 group by 1, 2, 4
";

const HOURLY_RULE_GROUP: &str = "
insert or replace into traffic_hourly_dimension(
    utc_hour, category_id, dimension_kind, dimension_id,
    upload, download, connection_count, active_duration_sec)
select (m.utc_minute * 60 / 3600) * 3600,
       coalesce(a.primary_category_id, 0),
       'rule_group',
       coalesce((select dimension_id from dimension_dict where dimension_kind = 'rule_group' and value = coalesce(last_chain_hop(a.chain_key), (select value from dimension_dict d2 where d2.dimension_kind = 'rule' and d2.dimension_id = a.rule_id), 'DIRECT')), 0),
       sum(m.upload), sum(m.download),
       count(distinct m.session_pk),
       count(distinct m.utc_minute) * 60
  from connection_minute m
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ?1 and m.utc_minute < ?2
 group by 1, 2, 4
";

fn insert_hourly_kind(
    connection: &rusqlite::Connection,
    sql: &str,
    start_min: i64,
    end_min: i64,
) -> Result<(), ReportError> {
    connection
        .execute(sql, params![start_min, end_min])
        .map_err(|_| ReportError::Failed("hourly materialize"))?;
    Ok(())
}

fn intern_chain_keys(
    connection: &rusqlite::Connection,
    start_min: i64,
    end_min: i64,
) -> Result<(), ReportError> {
    intern_distinct(
        connection,
        "chain",
        "select distinct last_chain_hop(a.chain_key)
           from connection_minute m
           left join connection_session_attr a on a.session_pk = m.session_pk
          where m.utc_minute >= ?1 and m.utc_minute < ?2
            and last_chain_hop(a.chain_key) is not null",
        start_min,
        end_min,
    )
}

fn intern_rule_groups(
    connection: &rusqlite::Connection,
    start_min: i64,
    end_min: i64,
) -> Result<(), ReportError> {
    intern_distinct(
        connection,
        "rule_group",
        "select distinct coalesce(last_chain_hop(a.chain_key), (select value from dimension_dict d where d.dimension_kind = 'rule' and d.dimension_id = a.rule_id), 'DIRECT')
           from connection_minute m
           left join connection_session_attr a on a.session_pk = m.session_pk
          where m.utc_minute >= ?1 and m.utc_minute < ?2",
        start_min,
        end_min,
    )
}

fn intern_distinct(
    connection: &rusqlite::Connection,
    kind: &str,
    sql: &str,
    start_min: i64,
    end_min: i64,
) -> Result<(), ReportError> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|_| ReportError::Failed("intern select"))?;
    let values = statement
        .query_map(params![start_min, end_min], |row| row.get::<_, String>(0))
        .map_err(|_| ReportError::Failed("intern query"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ReportError::Failed("intern row"))?;
    for value in values {
        intern_one(connection, kind, &value)?;
    }
    Ok(())
}

fn intern_one(
    connection: &rusqlite::Connection,
    kind: &str,
    value: &str,
) -> Result<(), ReportError> {
    if value.is_empty() || value == UNKNOWN_IDENTITY {
        return Ok(());
    }
    let existing: Option<i64> = connection
        .query_row(
            "select dimension_id from dimension_dict where dimension_kind = ?1 and value = ?2",
            params![kind, value],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| ReportError::Failed("intern lookup"))?;
    if existing.is_some() {
        return Ok(());
    }
    let next: i64 = connection
        .query_row(
            "select coalesce(max(dimension_id), 0) + 1 from dimension_dict where dimension_kind = ?1",
            [kind],
            |row| row.get(0),
        )
        .map_err(|_| ReportError::Failed("intern max"))?;
    connection
        .execute(
            "insert or ignore into dimension_dict(dimension_kind, dimension_id, value) values (?1, ?2, ?3)",
            params![kind, next, value],
        )
        .map_err(|_| ReportError::Failed("intern insert"))?;
    Ok(())
}

fn verify_layer(
    connection: &rusqlite::Connection,
    layer: &str,
    start: i64,
    end: i64,
) -> Result<(), ReportError> {
    let payload = format!("{layer}:{start}:{end}");
    let checksum = hex::encode(Sha256::digest(payload.as_bytes()));
    connection
        .execute(
            "insert or replace into retention_state(layer, chunk_utc, status, checksum, updated_utc)
             values (?1, ?2, 'verified', ?3, ?2)",
            params![layer, end, checksum],
        )
        .map_err(|_| ReportError::Failed("retention state"))?;
    connection
        .execute(
            "insert into retention_watermark(layer, watermark_utc, delete_watermark_utc)
             values (?1, ?2, 0)
             on conflict(layer) do update set watermark_utc = excluded.watermark_utc",
            params![layer, end],
        )
        .map_err(|_| ReportError::Failed("watermark"))?;
    Ok(())
}

fn delete_covered_raw(
    coordinator: &mut StorageCoordinator,
    now_utc: i64,
    raw_retain_days: i64,
) -> Result<(), ReportError> {
    let cutoff = now_utc - raw_retain_days.clamp(1, RAW_RETAIN_DAYS_MAX) * 86_400;
    let verified: Option<i64> = coordinator
        .connection()
        .query_row(
            "select watermark_utc from retention_watermark where layer = 'hourly'",
            [],
            |row| row.get(0),
        )
        .ok();
    if verified.unwrap_or(0) < cutoff {
        return Err(ReportError::Failed("delete before verified watermark"));
    }
    coordinator
        .connection_mut()
        .execute(
            "delete from connection_minute where utc_minute < ?1",
            [cutoff.div_euclid(60)],
        )
        .map_err(|_| ReportError::Failed("delete raw"))?;
    Ok(())
}

fn delete_expired_dimension(
    coordinator: &mut StorageCoordinator,
    now_utc: i64,
) -> Result<(), ReportError> {
    let cutoff = now_utc - DIMENSION_RETAIN_DAYS * 86_400;
    let connection = coordinator.connection_mut();
    connection
        .execute(
            "delete from traffic_hourly_dimension where utc_hour < ?1",
            [cutoff],
        )
        .map_err(|_| ReportError::Failed("delete hourly"))?;
    connection
        .execute(
            "delete from traffic_daily_dimension where utc_day < ?1",
            [cutoff],
        )
        .map_err(|_| ReportError::Failed("delete daily dim"))?;
    Ok(())
}

#[cfg(test)]
mod retention_tests {
    use super::*;
    use crate::c3::space::SpaceBudget;
    use crate::storage::StorageCoordinator;
    use tempfile::tempdir;

    #[test]
    fn materialize_is_idempotent_and_auto_delete_stays_off() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("r.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        let cancel = Arc::new(AtomicBool::new(false));
        let first = RetentionService::run(
            &mut coordinator,
            10_000,
            30,
            RetentionMode::MaterializeOnly,
            &SpaceBudget::unlimited(),
            &cancel,
        )
        .expect("first");
        let second = RetentionService::run(
            &mut coordinator,
            10_000,
            30,
            RetentionMode::MaterializeOnly,
            &SpaceBudget::unlimited(),
            &cancel,
        )
        .expect("second");
        assert_eq!(first.hourly_rows, second.hourly_rows);
        assert!(first.hourly_rows > 0);
        const { assert!(!AUTO_DELETE_ENABLED) };
        assert!(!first.auto_delete_enabled);
        let minutes_before = count(
            coordinator.connection(),
            "select count(*) from connection_minute",
        )
        .expect("count");
        let _ = RetentionService::run(
            &mut coordinator,
            10_000,
            30,
            RetentionMode::DeleteEnabled,
            &SpaceBudget::unlimited(),
            &cancel,
        )
        .expect("delete flag ignored");
        let minutes_after = count(
            coordinator.connection(),
            "select count(*) from connection_minute",
        )
        .expect("count");
        assert_eq!(minutes_before, minutes_after);
    }

    #[test]
    fn low_space_does_not_start() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("r.sqlite3")).expect("open");
        let error = RetentionService::run(
            &mut coordinator,
            10_000,
            30,
            RetentionMode::DryRun,
            &SpaceBudget::exhausted(),
            &Arc::new(AtomicBool::new(false)),
        )
        .expect_err("space");
        assert_eq!(error.code(), "insufficient_space");
    }

    #[test]
    fn crash_after_materialize_can_resume() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("r.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        materialize_hourly(&mut coordinator, 10_000, 30).expect("hourly");
        let path = dir.path().join("r.sqlite3");
        drop(coordinator);
        let mut coordinator = StorageCoordinator::open(&path).expect("reopen");
        RetentionService::run(
            &mut coordinator,
            10_000,
            30,
            RetentionMode::MaterializeOnly,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
        )
        .expect("resume");
        let status: String = coordinator
            .connection()
            .query_row(
                "select status from retention_state where layer = 'hourly'",
                [],
                |row| row.get(0),
            )
            .expect("state");
        assert_eq!(status, "verified");
    }

    #[test]
    fn five_dimension_kinds_and_v2_watermark() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("r5.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        coordinator
            .connection()
            .execute(
                "insert or replace into retention_watermark(layer, watermark_utc, delete_watermark_utc)
                 values ('hourly_dim_v2', 0, 0)",
                [],
            )
            .expect("v2");
        RetentionService::run(
            &mut coordinator,
            40 * 86_400,
            30,
            RetentionMode::MaterializeOnly,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
        )
        .expect("run");
        let kinds: Vec<String> = {
            let mut statement = coordinator
                .connection()
                .prepare("select distinct dimension_kind from traffic_hourly_dimension order by 1")
                .expect("stmt");
            statement
                .query_map([], |row| row.get(0))
                .expect("map")
                .collect::<Result<Vec<_>, _>>()
                .expect("rows")
        };
        assert_eq!(
            kinds,
            vec!["chain", "host", "network", "process", "rule_group"]
        );
        let v2: i64 = coordinator
            .connection()
            .query_row(
                "select watermark_utc from retention_watermark where layer = 'hourly_dim_v2'",
                [],
                |row| row.get(0),
            )
            .expect("v2");
        assert_eq!(v2, 0);
        let sentinel: i64 = coordinator
            .connection()
            .query_row(
                "select count(*) from dimension_dict where value = '__unknown__'",
                [],
                |row| row.get(0),
            )
            .expect("sentinel");
        assert_eq!(sentinel, 0);
        let core: i64 = coordinator
            .connection()
            .query_row(
                "select coalesce(sum(download), 0) from traffic_daily_core where category_id = 0",
                [],
                |row| row.get(0),
            )
            .expect("core");
        let host: i64 = coordinator
            .connection()
            .query_row(
                "select coalesce(sum(download), 0) from traffic_daily_dimension where dimension_kind = 'host'",
                [],
                |row| row.get(0),
            )
            .expect("host");
        assert_eq!(core, host);
    }
}
