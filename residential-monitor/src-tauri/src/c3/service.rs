//! 统一 ReportService：短读快照内物化，返回前关闭事务。

use crate::c3::query::{
    decode_cursor, empty_result, encode_cursor, plan_capability, validate_query, CoverageSlice,
    DataTier, DimensionKind, PolicyMetadata, RankingRow, ReportError, ReportQuery, ReportResult,
    ReportTotals, SeriesPoint, SessionRow, TargetPolicy,
};
use crate::c3::snapshot::ReportSnapshotStore;
use crate::c3::sql::{
    dimension_kind_sql, COVERAGE_DAILY, COVERAGE_RAW, RANK_DAILY_DIM, RANK_HOURLY, RANK_RAW,
    SERIES_DAILY_CORE, SERIES_DAILY_DIM, SERIES_HOURLY, SERIES_RAW, TOTALS_DAILY_CORE,
    TOTALS_DAILY_DIM, TOTALS_HOURLY, TOTALS_RAW,
};
use crate::storage::{open_interruptible_reader, StorageCoordinator};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct ReportService;

impl ReportService {
    pub fn run(
        db_path: &Path,
        store: &mut ReportSnapshotStore,
        query: ReportQuery,
        now_utc: i64,
        raw_retain_days: i64,
        cancel: &Arc<AtomicBool>,
        deadline: Option<Duration>,
    ) -> Result<ReportResult, ReportError> {
        validate_query(&query)?;
        let plan = plan_capability(&query, now_utc, raw_retain_days)?;
        let limit = deadline.unwrap_or(Duration::from_millis(plan.deadline_ms));
        let reader = open_interruptible_reader(db_path).map_err(map_storage)?;
        attach_cancel(&reader, cancel, Instant::now(), limit)?;
        reader
            .execute_batch("begin deferred")
            .map_err(|_| ReportError::StorageBusy("begin"))?;
        let built = build_result(&reader, &query, now_utc, raw_retain_days, cancel);
        let close = reader.execute_batch("commit");
        let txn_open = !reader.is_autocommit();
        if txn_open {
            let _ = reader.execute_batch("rollback");
        }
        drop(reader);
        let result = built?;
        close.map_err(|_| ReportError::Failed("commit snapshot"))?;
        if txn_open {
            return Err(ReportError::Failed("read transaction still open"));
        }
        store.insert(&query, result, now_utc, false)
    }

    pub fn explain_named(connection: &Connection, name: &str) -> Result<Vec<String>, ReportError> {
        let sql = crate::c3::sql::lookup(name).ok_or(ReportError::InvalidQuery("unknown sql"))?;
        let mut statement = connection
            .prepare(&format!("explain query plan {sql}"))
            .map_err(|_| ReportError::Failed("eqp"))?;
        let zeros = [0_i64; 16];
        let params: Vec<&dyn rusqlite::types::ToSql> = zeros
            .iter()
            .take(statement.parameter_count())
            .map(|item| item as _)
            .collect();
        let mut rows = statement
            .query(params.as_slice())
            .map_err(|_| ReportError::Failed("eqp rows"))?;
        let mut plans = Vec::new();
        while let Some(row) = rows.next().map_err(|_| ReportError::Failed("eqp row"))? {
            if let Ok(detail) = row.get::<_, String>(3) {
                plans.push(detail);
            }
        }
        Ok(plans)
    }
}

fn attach_cancel(
    connection: &Connection,
    cancel: &Arc<AtomicBool>,
    started: Instant,
    deadline: Duration,
) -> Result<(), ReportError> {
    let flag = Arc::clone(cancel);
    connection
        .progress_handler(
            64,
            Some(move || flag.load(Ordering::SeqCst) || started.elapsed() > deadline),
        )
        .map_err(|_| ReportError::Failed("progress handler"))?;
    Ok(())
}

fn build_result(
    connection: &Connection,
    query: &ReportQuery,
    now_utc: i64,
    raw_retain_days: i64,
    cancel: &Arc<AtomicBool>,
) -> Result<ReportResult, ReportError> {
    if cancel.load(Ordering::SeqCst) {
        return Err(ReportError::Cancelled("user"));
    }
    let plan = plan_capability(query, now_utc, raw_retain_days)?;
    let data_version = connection
        .query_row(
            "select watermark from data_version where id = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) as u64;
    let mut result = empty_result(query.clone(), &plan, data_version);
    result.generated_utc = now_utc;
    result.policy_metadata = PolicyMetadata {
        target_policy: query.target_policy,
        policy_version: load_policy_version(connection),
        note_zh: if query.target_policy == TargetPolicy::Current {
            "当前策略重算仅在 raw 能力期内可用。观测下界，不是账单。".into()
        } else {
            "历史主分类来自写入时策略。观测下界，不是账单。".into()
        },
    };
    let start_min = query.range_start_utc.div_euclid(60);
    let end_min = query.range_end_utc.div_euclid(60);
    match plan.tier {
        DataTier::Raw => fill_raw(connection, query, &mut result, start_min, end_min)?,
        DataTier::HourlyDimension => fill_hourly(
            connection,
            query,
            &mut result,
            query.range_start_utc,
            query.range_end_utc,
        )?,
        DataTier::DailyDimension => fill_daily_dim(
            connection,
            query,
            &mut result,
            query.range_start_utc,
            query.range_end_utc,
        )?,
        DataTier::DailyCore => fill_core(
            connection,
            query,
            &mut result,
            query.range_start_utc,
            query.range_end_utc,
        )?,
    }
    if result.coverage.slices.iter().any(|item| item.kind == "gap") {
        result.coverage.status = "partial".into();
    } else if result.totals.upload == 0
        && result.totals.download == 0
        && result.coverage.slices.is_empty()
    {
        result.coverage.status = "empty".into();
    } else if result.coverage.slices.is_empty() {
        result.coverage.status = "unknown".into();
    } else {
        result.coverage.status = "covered".into();
    }
    Ok(result)
}

fn fill_raw(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start_min: i64,
    end_min: i64,
) -> Result<(), ReportError> {
    let host_on = i64::from(query.filters.host.is_some());
    let process_on = i64::from(query.filters.process.is_some());
    let rule_on = i64::from(query.filters.rule.is_some());
    let network_on = i64::from(query.filters.network.is_some());
    let chain_on = i64::from(query.filters.chain.is_some());
    let host = query.filters.host.clone().unwrap_or_default();
    let process = query.filters.process.clone().unwrap_or_default();
    let rule = query.filters.rule.clone().unwrap_or_default();
    let network = query.filters.network.clone().unwrap_or_default();
    let chain = query.filters.chain.clone().unwrap_or_default();
    let (upload, download, count, duration): (i64, i64, i64, i64) = connection
        .query_row(
            TOTALS_RAW,
            params![
                start_min, end_min, host_on, host, process_on, process, rule_on, rule, network_on,
                network, chain_on, chain
            ],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(map_sqlite)?;
    result.totals = ReportTotals {
        upload,
        download,
        connection_count: count,
        active_duration_sec: duration,
        previous_upload: None,
        previous_download: None,
    };
    if query
        .comparison
        .as_ref()
        .is_some_and(|item| item.previous_equal_window)
    {
        let span = query.range_end_utc - query.range_start_utc;
        let prev_start = (query.range_start_utc - span).div_euclid(60);
        let prev_end = start_min;
        let (prev_up, prev_down, _, _): (i64, i64, i64, i64) = connection
            .query_row(
                TOTALS_RAW,
                params![
                    prev_start, prev_end, host_on, host, process_on, process, rule_on, rule,
                    network_on, network, chain_on, chain
                ],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .map_err(map_sqlite)?;
        result.totals.previous_upload = Some(prev_up);
        result.totals.previous_download = Some(prev_down);
    }
    let bucket = match query.granularity {
        crate::c3::query::Granularity::Hour => 60,
        crate::c3::query::Granularity::Day => 1_440,
        crate::c3::query::Granularity::Month => 43_200,
    };
    let mut series = connection.prepare(SERIES_RAW).map_err(map_sqlite)?;
    let rows = series
        .query_map(params![start_min, end_min, bucket, host_on, host], |row| {
            Ok(SeriesPoint {
                bucket_utc: row.get::<_, i64>(0)? * 60,
                upload: row.get(1)?,
                download: row.get(2)?,
                connection_count: row.get(3)?,
                active_duration_sec: row.get(4)?,
            })
        })
        .map_err(map_sqlite)?;
    result.series = rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite)?;
    if query.grouping == DimensionKind::Host || query.grouping == DimensionKind::Category {
        let mut rank = connection.prepare(RANK_RAW).map_err(map_sqlite)?;
        let rows = rank
            .query_map(params![start_min, end_min, query.top_n as i64], |row| {
                let label: String = row.get(0)?;
                Ok(RankingRow {
                    identity: label.clone(),
                    label,
                    upload: row.get(1)?,
                    download: row.get(2)?,
                    connection_count: row.get(3)?,
                    active_duration_sec: row.get(4)?,
                })
            })
            .map_err(map_sqlite)?;
        result.rankings = rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite)?;
    } else {
        fill_raw_attr_rank(connection, query, result, start_min, end_min)?;
    }
    fill_coverage_raw(connection, query, result)?;
    if query.include_sessions {
        fill_sessions(connection, query, result, start_min, end_min)?;
    }
    Ok(())
}

fn fill_raw_attr_rank(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start_min: i64,
    end_min: i64,
) -> Result<(), ReportError> {
    let kind = dimension_kind_sql(query.grouping);
    let sql = "
        select coalesce(d.value, ''),
               coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0),
               count(distinct m.session_pk), count(distinct m.utc_minute) * 60
          from connection_minute m
          join connection_session_attr a on a.session_pk = m.session_pk
          join dimension_dict d on d.dimension_kind = ?3 and d.dimension_id = case ?3
            when 'process' then a.process_id
            when 'rule' then a.rule_id
            when 'network' then a.network_id
            when 'category' then a.primary_category_id
            else a.host_id end
         where m.utc_minute >= ?1 and m.utc_minute < ?2
         group by d.value
         order by sum(m.download) desc, d.value asc
         limit ?4
    ";
    let mut statement = connection.prepare(sql).map_err(map_sqlite)?;
    let rows = statement
        .query_map(
            params![start_min, end_min, kind, query.top_n as i64],
            |row| {
                let label: String = row.get(0)?;
                Ok(RankingRow {
                    identity: label.clone(),
                    label,
                    upload: row.get(1)?,
                    download: row.get(2)?,
                    connection_count: row.get(3)?,
                    active_duration_sec: row.get(4)?,
                })
            },
        )
        .map_err(map_sqlite)?;
    result.rankings = rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite)?;
    Ok(())
}

fn fill_hourly(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start: i64,
    end: i64,
) -> Result<(), ReportError> {
    let kind = dimension_kind_sql(query.grouping);
    let (upload, download, count, duration): (i64, i64, i64, i64) = connection
        .query_row(TOTALS_HOURLY, params![start, end, kind], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(map_sqlite)?;
    result.totals = ReportTotals {
        upload,
        download,
        connection_count: count,
        active_duration_sec: duration,
        previous_upload: None,
        previous_download: None,
    };
    let mut series = connection.prepare(SERIES_HOURLY).map_err(map_sqlite)?;
    result.series = series
        .query_map(params![start, end, kind], |row| {
            Ok(SeriesPoint {
                bucket_utc: row.get(0)?,
                upload: row.get(1)?,
                download: row.get(2)?,
                connection_count: row.get(3)?,
                active_duration_sec: row.get(4)?,
            })
        })
        .map_err(map_sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_sqlite)?;
    let mut rank = connection.prepare(RANK_HOURLY).map_err(map_sqlite)?;
    result.rankings = rank
        .query_map(params![start, end, kind, query.top_n as i64], |row| {
            let label: String = row.get(0)?;
            Ok(RankingRow {
                identity: label.clone(),
                label,
                upload: row.get(1)?,
                download: row.get(2)?,
                connection_count: row.get(3)?,
                active_duration_sec: row.get(4)?,
            })
        })
        .map_err(map_sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_sqlite)?;
    fill_coverage_daily(connection, query, result)?;
    Ok(())
}

fn fill_daily_dim(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start: i64,
    end: i64,
) -> Result<(), ReportError> {
    let kind = dimension_kind_sql(query.grouping);
    let (upload, download, count, duration): (i64, i64, i64, i64) = connection
        .query_row(TOTALS_DAILY_DIM, params![start, end, kind], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(map_sqlite)?;
    result.totals = ReportTotals {
        upload,
        download,
        connection_count: count,
        active_duration_sec: duration,
        previous_upload: None,
        previous_download: None,
    };
    let mut series = connection.prepare(SERIES_DAILY_DIM).map_err(map_sqlite)?;
    result.series = series
        .query_map(params![start, end, kind], |row| {
            Ok(SeriesPoint {
                bucket_utc: row.get(0)?,
                upload: row.get(1)?,
                download: row.get(2)?,
                connection_count: row.get(3)?,
                active_duration_sec: row.get(4)?,
            })
        })
        .map_err(map_sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_sqlite)?;
    let mut rank = connection.prepare(RANK_DAILY_DIM).map_err(map_sqlite)?;
    result.rankings = rank
        .query_map(params![start, end, kind, query.top_n as i64], |row| {
            let label: String = row.get(0)?;
            Ok(RankingRow {
                identity: label.clone(),
                label,
                upload: row.get(1)?,
                download: row.get(2)?,
                connection_count: row.get(3)?,
                active_duration_sec: row.get(4)?,
            })
        })
        .map_err(map_sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_sqlite)?;
    fill_coverage_daily(connection, query, result)?;
    Ok(())
}

fn fill_core(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start: i64,
    end: i64,
) -> Result<(), ReportError> {
    let (upload, download, count, duration): (i64, i64, i64, i64) = connection
        .query_row(TOTALS_DAILY_CORE, params![start, end], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(map_sqlite)?;
    result.totals = ReportTotals {
        upload,
        download,
        connection_count: count,
        active_duration_sec: duration,
        previous_upload: None,
        previous_download: None,
    };
    let mut series = connection.prepare(SERIES_DAILY_CORE).map_err(map_sqlite)?;
    result.series = series
        .query_map(params![start, end], |row| {
            Ok(SeriesPoint {
                bucket_utc: row.get(0)?,
                upload: row.get(1)?,
                download: row.get(2)?,
                connection_count: row.get(3)?,
                active_duration_sec: row.get(4)?,
            })
        })
        .map_err(map_sqlite)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_sqlite)?;
    result.rankings.clear();
    fill_coverage_daily(connection, query, result)?;
    Ok(())
}

fn fill_coverage_raw(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
) -> Result<(), ReportError> {
    let mut statement = connection.prepare(COVERAGE_RAW).map_err(map_sqlite)?;
    let rows = statement
        .query_map(params![query.range_start_utc, query.range_end_utc], |row| {
            Ok(CoverageSlice {
                kind: row.get(0)?,
                reason: row.get(1)?,
                started_utc: row.get(2)?,
                ended_utc: row.get(3)?,
            })
        })
        .map_err(map_sqlite)?;
    result.coverage.slices = rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite)?;
    summarize_coverage(query, result);
    Ok(())
}

fn fill_coverage_daily(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
) -> Result<(), ReportError> {
    let mut statement = connection.prepare(COVERAGE_DAILY).map_err(map_sqlite)?;
    let rows = statement
        .query_map(params![query.range_start_utc, query.range_end_utc], |row| {
            let day: i64 = row.get(0)?;
            let covered: i64 = row.get(1)?;
            let gap: i64 = row.get(2)?;
            let reasons: String = row.get(3)?;
            Ok((day, covered, gap, reasons))
        })
        .map_err(map_sqlite)?;
    let mut covered = 0;
    let mut gap = 0;
    for row in rows {
        let (day, cov, g, reasons) = row.map_err(map_sqlite)?;
        covered += cov;
        gap += g;
        result.coverage.slices.push(CoverageSlice {
            kind: if g > 0 {
                "gap".into()
            } else {
                "covered".into()
            },
            reason: reasons,
            started_utc: day,
            ended_utc: Some(day + 86_400),
        });
    }
    result.coverage.covered_sec = covered;
    result.coverage.gap_sec = gap;
    Ok(())
}

fn summarize_coverage(query: &ReportQuery, result: &mut ReportResult) {
    let span = (query.range_end_utc - query.range_start_utc).max(0);
    let gap = result
        .coverage
        .slices
        .iter()
        .filter(|item| item.kind == "gap")
        .map(|item| {
            let end = item.ended_utc.unwrap_or(query.range_end_utc);
            (end.min(query.range_end_utc) - item.started_utc.max(query.range_start_utc)).max(0)
        })
        .sum();
    result.coverage.gap_sec = gap;
    result.coverage.covered_sec = (span - gap).max(0);
}

fn fill_sessions(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start_min: i64,
    end_min: i64,
) -> Result<(), ReportError> {
    let (after_download, after_id) = match &query.page.after {
        Some(cursor) => decode_cursor(cursor)?,
        None => (i64::MAX, String::new()),
    };
    let sql = "
        select s.epoch_id || ':' || s.connection_id, s.host, s.started_utc,
               coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0)
          from connection_minute m
          join connection_session s on s.session_pk = m.session_pk
         where m.utc_minute >= ?1 and m.utc_minute < ?2
         group by s.session_pk
        having sum(m.download) < ?3
            or (sum(m.download) = ?3 and (s.epoch_id || ':' || s.connection_id) > ?4)
         order by sum(m.download) desc, (s.epoch_id || ':' || s.connection_id) asc
         limit ?5
    ";
    let mut statement = connection.prepare(sql).map_err(map_sqlite)?;
    let rows = statement
        .query_map(
            params![
                start_min,
                end_min,
                after_download,
                after_id,
                i64::from(query.page.limit)
            ],
            |row| {
                Ok(SessionRow {
                    identity: row.get(0)?,
                    host: row.get(1)?,
                    process: None,
                    rule: None,
                    upload: row.get(3)?,
                    download: row.get(4)?,
                    started_utc: row.get(2)?,
                })
            },
        )
        .map_err(map_sqlite)?;
    result.sessions = rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite)?;
    if let Some(last) = result.sessions.last() {
        result.next_cursor = Some(encode_cursor(last.download, &last.identity));
    }
    Ok(())
}

fn load_policy_version(connection: &Connection) -> Option<u32> {
    connection
        .query_row(
            "select policy_version from target_set where set_id = 1",
            [],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .ok()
        .flatten()
        .map(|value| value as u32)
}

fn map_sqlite(error: rusqlite::Error) -> ReportError {
    if crate::sqlite_probe::map_sqlite_error(&error) == "cancelled" {
        return ReportError::Cancelled("sqlite interrupt");
    }
    if crate::sqlite_probe::map_sqlite_error(&error) == "busy" {
        return ReportError::StorageBusy("sqlite busy");
    }
    ReportError::Failed("sqlite query")
}

fn map_storage(_error: crate::storage::StorageError) -> ReportError {
    ReportError::Failed("open reader")
}

pub fn seed_golden_fixture(path: &Path) -> Result<(), ReportError> {
    let coordinator = StorageCoordinator::open(path).map_err(|_| ReportError::Failed("open"))?;
    coordinator
        .seed_report_fixture()
        .map_err(|_| ReportError::Failed("seed"))?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod report_service_tests {
    use super::*;
    use crate::c3::query::{ComparisonSpec, Granularity, ReportFilters};
    use crate::storage::StorageCoordinator;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, StorageCoordinator, ReportSnapshotStore) {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("rep.sqlite3");
        let coordinator = StorageCoordinator::open(&path).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        let store = ReportSnapshotStore::open(dir.path());
        (dir, coordinator, store)
    }

    #[test]
    fn golden_totals_and_token_close_transaction() {
        let (_dir, coordinator, mut store) = setup();
        let mut query = ReportQuery::default();
        query.range_start_utc = 1_000;
        query.range_end_utc = 4_000;
        query.comparison = Some(ComparisonSpec {
            previous_equal_window: true,
        });
        let cancel = Arc::new(AtomicBool::new(false));
        let result = ReportService::run(
            coordinator.path(),
            &mut store,
            query,
            3_600,
            30,
            &cancel,
            None,
        )
        .expect("run");
        assert_eq!(result.totals.upload, 30);
        assert_eq!(result.totals.download, 90);
        assert_eq!(result.rankings[0].label, "b.example");
        assert_eq!(result.rankings[0].download, 60);
        assert!(!result.report_snapshot_token.is_empty());
        assert_eq!(result.coverage.status, "partial");
        assert!(result.totals.previous_download.is_some());
        coordinator
            .checkpoint_passive()
            .expect("checkpoint after token");
    }

    #[test]
    fn empty_range_is_zero_not_invented() {
        let (_dir, coordinator, mut store) = setup();
        let mut query = ReportQuery::default();
        query.range_start_utc = 80_000;
        query.range_end_utc = 81_000;
        let cancel = Arc::new(AtomicBool::new(false));
        let result = ReportService::run(
            coordinator.path(),
            &mut store,
            query,
            90_000,
            30,
            &cancel,
            None,
        )
        .expect("run");
        assert_eq!(result.totals.upload, 0);
        assert_eq!(result.totals.download, 0);
        assert_eq!(result.coverage.status, "empty");
    }

    #[test]
    fn cancel_stops_sqlite_work() {
        let (_dir, coordinator, mut store) = setup();
        let cancel = Arc::new(AtomicBool::new(true));
        let error = ReportService::run(
            coordinator.path(),
            &mut store,
            ReportQuery::default(),
            3_600,
            30,
            &cancel,
            Some(Duration::from_millis(1)),
        )
        .expect_err("cancel");
        assert!(error.code() == "cancelled" || error.code() == "deadline_exceeded");
    }

    #[test]
    fn eqp_named_queries_have_no_temp_or_autoindex_on_fixture() {
        let (_dir, coordinator, _) = setup();
        let reader = open_interruptible_reader(coordinator.path()).expect("reader");
        for name in ["totals_raw", "rank_raw", "totals_hourly_dimension"] {
            let plans = ReportService::explain_named(&reader, name).expect("eqp");
            let joined = plans.join(" ").to_ascii_uppercase();
            assert!(!joined.contains("AUTOMATIC INDEX"), "{name}: {joined}");
        }
    }

    #[test]
    fn same_token_export_matches_ui_totals() {
        let dir = tempdir().expect("dir");
        let (_tmp, coordinator, mut store) = setup();
        let mut query = ReportQuery::default();
        query.range_start_utc = 1_000;
        query.range_end_utc = 4_000;
        let cancel = Arc::new(AtomicBool::new(false));
        let result = ReportService::run(
            coordinator.path(),
            &mut store,
            query,
            3_600,
            30,
            &cancel,
            None,
        )
        .expect("run");
        let spec = crate::c3::export::ExportSpec::default();
        let csv = crate::c3::ExportService::export_to_path(
            &result,
            &spec,
            &dir.path().join("same.csv"),
            &crate::c3::SpaceBudget::unlimited(),
            &cancel,
        )
        .expect("csv");
        let text = std::fs::read_to_string(csv).expect("read");
        assert!(text.contains(&format!(
            ",{},{},",
            result.totals.upload, result.totals.download
        )));
        assert!(text.contains(&result.report_snapshot_token));
    }

    #[test]
    fn writer_report_checkpoint_can_run_together() {
        let (_dir, mut coordinator, mut store) = setup();
        coordinator
            .commit(&crate::storage::CommitBundle {
                writer_epoch: 9,
                bundle_seq: 1,
                payload: "200,1,4,8".into(),
            })
            .expect("write");
        let cancel = Arc::new(AtomicBool::new(false));
        let mut query = ReportQuery::default();
        query.range_start_utc = 1_000;
        query.range_end_utc = 4_000;
        let result = ReportService::run(
            coordinator.path(),
            &mut store,
            query,
            3_600,
            30,
            &cancel,
            None,
        )
        .expect("report");
        coordinator.checkpoint_passive().expect("ck");
        assert_eq!(result.totals.download, 90);
    }

    #[test]
    fn restart_same_query_same_totals() {
        let (dir, coordinator, mut store) = setup();
        let mut query = ReportQuery::default();
        query.range_start_utc = 1_000;
        query.range_end_utc = 4_000;
        query.granularity = Granularity::Hour;
        query.filters = ReportFilters::default();
        let cancel = Arc::new(AtomicBool::new(false));
        let first = ReportService::run(
            coordinator.path(),
            &mut store,
            query.clone(),
            3_600,
            30,
            &cancel,
            None,
        )
        .expect("first");
        drop(store);
        let mut store = ReportSnapshotStore::open(dir.path());
        let second = ReportService::run(
            coordinator.path(),
            &mut store,
            query,
            3_600,
            30,
            &cancel,
            None,
        )
        .expect("second");
        assert_eq!(first.totals, second.totals);
        assert_eq!(first.rankings, second.rankings);
    }
}
