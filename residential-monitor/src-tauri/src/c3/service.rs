//! 统一 ReportService：短读快照内物化，返回前关闭事务。

use crate::c3::query::{
    decode_cursor, empty_result, encode_cursor, plan_capability, plan_capability_ex,
    validate_query, CoverageSlice, DataTier, DimensionKind, PolicyMetadata, RankingRow,
    ReportError, ReportQuery, ReportResult, ReportTotals, SeriesPoint, SessionRow, TargetPolicy,
    HOURLY_DIM_V2_LAYER, UNKNOWN_LABEL_ZH,
};
use crate::c3::snapshot::ReportSnapshotStore;
use crate::c3::sql::{
    dimension_filter_clause, dimension_kind_sql, dimension_kind_sql_layer, filter_clause,
    merge_sql_params, render_sql, COVERAGE_DAILY, COVERAGE_RAW, RANK_DAILY_CATEGORY,
    RANK_DAILY_DIM, RANK_HOURLY, RANK_HOURLY_CATEGORY, RANK_RAW, RANK_RAW_ATTR, RANK_RAW_CHAIN,
    RANK_RAW_RULE, SERIES_DAILY_CORE, SERIES_DAILY_DIM, SERIES_HOURLY, SERIES_RAW,
    TOTALS_DAILY_CORE, TOTALS_DAILY_DIM, TOTALS_HOURLY, TOTALS_RAW, UNKNOWN_IDENTITY,
};
use crate::storage::{open_interruptible_reader, StorageCoordinator};
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
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
        let sql = render_sql(sql, "");
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
    let v2_start = load_hourly_dim_v2_start(connection);
    let plan = plan_capability_ex(query, now_utc, raw_retain_days, v2_start)?;
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
    let (fragment, filter_params) = filter_clause(&query.filters);
    let totals_sql = render_sql(TOTALS_RAW, &fragment);
    let totals_params = merge_sql_params(
        [Value::from(start_min), Value::from(end_min)],
        &filter_params,
        [],
    );
    let (upload, download, count, duration) = load_totals(connection, &totals_sql, &totals_params)?;
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
        let prev_params = merge_sql_params(
            [Value::from(prev_start), Value::from(prev_end)],
            &filter_params,
            [],
        );
        let (prev_up, prev_down, _, _) = load_totals(connection, &totals_sql, &prev_params)?;
        result.totals.previous_upload = Some(prev_up);
        result.totals.previous_download = Some(prev_down);
    }
    let bucket = query.granularity.bucket_minutes();
    let series_sql = render_sql(SERIES_RAW, &fragment);
    let series_params = merge_sql_params(
        [
            Value::from(bucket),
            Value::from(bucket),
            Value::from(start_min),
            Value::from(end_min),
        ],
        &filter_params,
        [],
    );
    result.series = load_series(connection, &series_sql, &series_params, 60)?;
    fill_raw_rank(
        connection,
        query,
        result,
        start_min,
        end_min,
        &fragment,
        &filter_params,
    )?;
    fill_coverage_raw(connection, query, result)?;
    if query.include_sessions {
        fill_sessions(connection, query, result, start_min, end_min)?;
    }
    Ok(())
}

fn fill_raw_rank(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start_min: i64,
    end_min: i64,
    fragment: &str,
    filter_params: &[String],
) -> Result<(), ReportError> {
    let (sql, prefix) = match query.grouping {
        DimensionKind::Host => (
            render_sql(RANK_RAW, fragment),
            vec![Value::from(start_min), Value::from(end_min)],
        ),
        DimensionKind::Chain => (
            render_sql(RANK_RAW_CHAIN, fragment),
            vec![Value::from(start_min), Value::from(end_min)],
        ),
        DimensionKind::Rule => (
            render_sql(RANK_RAW_RULE, fragment),
            vec![Value::from(start_min), Value::from(end_min)],
        ),
        DimensionKind::Process | DimensionKind::Network | DimensionKind::Category => {
            let kind = dimension_kind_sql(query.grouping);
            (
                render_sql(RANK_RAW_ATTR, fragment),
                vec![
                    Value::Text(kind.into()),
                    Value::Text(kind.into()),
                    Value::from(start_min),
                    Value::from(end_min),
                ],
            )
        }
    };
    let params = merge_sql_params(prefix, filter_params, [Value::from(i64::from(query.top_n))]);
    result.rankings = load_rankings(connection, &sql, &params)?;
    Ok(())
}

fn fill_hourly(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start: i64,
    end: i64,
) -> Result<(), ReportError> {
    fill_dimension_layer(
        connection,
        query,
        result,
        start,
        end,
        TOTALS_HOURLY,
        SERIES_HOURLY,
        RANK_HOURLY,
        RANK_HOURLY_CATEGORY,
    )
}

fn fill_daily_dim(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start: i64,
    end: i64,
) -> Result<(), ReportError> {
    fill_dimension_layer(
        connection,
        query,
        result,
        start,
        end,
        TOTALS_DAILY_DIM,
        SERIES_DAILY_DIM,
        RANK_DAILY_DIM,
        RANK_DAILY_CATEGORY,
    )
}

#[allow(clippy::too_many_arguments)]
fn fill_dimension_layer(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start: i64,
    end: i64,
    totals_sql: &str,
    series_sql: &str,
    rank_sql: &str,
    rank_category_sql: &str,
) -> Result<(), ReportError> {
    let (fragment, filter_params) = dimension_filter_clause(&query.filters, query.grouping);
    let kind = if query.grouping == DimensionKind::Category {
        "host"
    } else {
        dimension_kind_sql_layer(query.grouping)
    };
    let rendered_totals = render_sql(totals_sql, &fragment);
    let totals_params = merge_sql_params(
        [
            Value::from(start),
            Value::from(end),
            Value::Text(kind.into()),
        ],
        &filter_params,
        [],
    );
    let (upload, download, count, duration) =
        load_totals(connection, &rendered_totals, &totals_params)?;
    result.totals = ReportTotals {
        upload,
        download,
        connection_count: count,
        active_duration_sec: duration,
        previous_upload: None,
        previous_download: None,
    };
    let rendered_series = render_sql(series_sql, &fragment);
    result.series = load_series(connection, &rendered_series, &totals_params, 1)?;
    let (rendered_rank, rank_prefix) = if query.grouping == DimensionKind::Category {
        (
            render_sql(rank_category_sql, &fragment),
            vec![Value::from(start), Value::from(end)],
        )
    } else {
        (
            render_sql(rank_sql, &fragment),
            vec![
                Value::from(start),
                Value::from(end),
                Value::Text(kind.into()),
            ],
        )
    };
    let rank_params = merge_sql_params(
        rank_prefix,
        &filter_params,
        [Value::from(i64::from(query.top_n))],
    );
    result.rankings = load_rankings(connection, &rendered_rank, &rank_params)?;
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

fn load_hourly_dim_v2_start(connection: &Connection) -> Option<i64> {
    connection
        .query_row(
            "select watermark_utc from retention_watermark where layer = ?1",
            [HOURLY_DIM_V2_LAYER],
            |row| row.get(0),
        )
        .optional()
        .ok()
        .flatten()
}

fn load_totals(
    connection: &Connection,
    sql: &str,
    params: &[Value],
) -> Result<(i64, i64, i64, i64), ReportError> {
    connection
        .query_row(sql, params_from_iter(params.iter()), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(map_sqlite)
}

fn load_series(
    connection: &Connection,
    sql: &str,
    params: &[Value],
    bucket_scale: i64,
) -> Result<Vec<SeriesPoint>, ReportError> {
    let mut statement = connection.prepare(sql).map_err(map_sqlite)?;
    let rows = statement
        .query_map(params_from_iter(params.iter()), |row| {
            Ok(SeriesPoint {
                bucket_utc: row.get::<_, i64>(0)? * bucket_scale,
                upload: row.get(1)?,
                download: row.get(2)?,
                connection_count: row.get(3)?,
                active_duration_sec: row.get(4)?,
            })
        })
        .map_err(map_sqlite)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite)
}

fn load_rankings(
    connection: &Connection,
    sql: &str,
    params: &[Value],
) -> Result<Vec<RankingRow>, ReportError> {
    let mut statement = connection.prepare(sql).map_err(map_sqlite)?;
    let rows = statement
        .query_map(params_from_iter(params.iter()), |row| {
            let identity: String = row.get(0)?;
            let label = if identity == UNKNOWN_IDENTITY {
                UNKNOWN_LABEL_ZH.to_string()
            } else {
                identity.clone()
            };
            Ok(RankingRow {
                identity,
                label,
                upload: row.get(1)?,
                download: row.get(2)?,
                connection_count: row.get(3)?,
                active_duration_sec: row.get(4)?,
            })
        })
        .map_err(map_sqlite)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite)
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

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod dimension_capability_tests {
    use super::*;
    use crate::c3::query::Granularity;
    use crate::c3::retention::{RetentionMode, RetentionService};
    use crate::c3::space::SpaceBudget;
    use crate::c3::sql::UNKNOWN_IDENTITY;
    use crate::storage::StorageCoordinator;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, StorageCoordinator, ReportSnapshotStore) {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("dim.sqlite3");
        let coordinator = StorageCoordinator::open(&path).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        seed_extra_dimensions(coordinator.connection());
        let store = ReportSnapshotStore::open(dir.path());
        (dir, coordinator, store)
    }

    fn seed_extra_dimensions(connection: &Connection) {
        connection
            .execute_batch(
                "
                insert or ignore into connection_session(session_pk, epoch_id, connection_id, started_utc, host)
                values (3, 1, 'gamma', 600, 'c.example');
                insert or ignore into dimension_dict(dimension_kind, dimension_id, value) values
                    ('host', 3, 'c.example'),
                    ('process', 2, 'other.exe'),
                    ('rule', 1, 'RuleSet'),
                    ('network', 2, 'udp'),
                    ('category', 2, '机场');
                insert or ignore into connection_session_attr(
                    session_pk, host_id, process_id, rule_id, network_id, chain_key,
                    policy_version, primary_category_id, started_utc, ended_utc
                ) values (3, 3, 2, 1, 2, 'PROXY>家宽', 1, 2, 600, null);
                insert or ignore into connection_minute(utc_minute, session_pk, upload, download)
                values (10, 3, 5, 15);
                ",
            )
            .expect("extra seed");
    }

    fn run_now(
        coordinator: &StorageCoordinator,
        store: &mut ReportSnapshotStore,
        query: ReportQuery,
        now: i64,
    ) -> ReportResult {
        let cancel = Arc::new(AtomicBool::new(false));
        ReportService::run(coordinator.path(), store, query, now, 30, &cancel, None).expect("run")
    }

    fn base_query() -> ReportQuery {
        let mut query = ReportQuery::default();
        query.range_start_utc = 0;
        query.range_end_utc = 4_000;
        query
    }

    #[test]
    fn named_sql_matches_grouping() {
        let (_dir, coordinator, mut store) = setup();
        let mut query = base_query();
        query.grouping = DimensionKind::Chain;
        let result = run_now(&coordinator, &mut store, query, 3_600);
        assert!(result.named_sql.iter().any(|name| name == "rank_raw_chain"));
        assert!(!result.named_sql.iter().any(|name| name == "rank_raw"));
        let mut query = base_query();
        query.grouping = DimensionKind::Category;
        let result = run_now(&coordinator, &mut store, query, 3_600);
        assert!(result.named_sql.iter().any(|name| name == "rank_raw_attr"));
    }

    #[test]
    fn chain_rank_differs_from_host() {
        let (_dir, coordinator, mut store) = setup();
        let mut host_q = base_query();
        host_q.grouping = DimensionKind::Host;
        let mut chain_q = base_query();
        chain_q.grouping = DimensionKind::Chain;
        let host = run_now(&coordinator, &mut store, host_q, 3_600);
        let chain = run_now(&coordinator, &mut store, chain_q, 3_600);
        let host_ids: Vec<_> = host
            .rankings
            .iter()
            .map(|row| row.identity.as_str())
            .collect();
        let chain_ids: Vec<_> = chain
            .rankings
            .iter()
            .map(|row| row.identity.as_str())
            .collect();
        assert!(host_ids.contains(&"c.example"));
        assert!(chain_ids.contains(&"家宽"));
        assert_ne!(host_ids, chain_ids);
    }

    #[test]
    fn category_rank_differs_from_host() {
        let (_dir, coordinator, mut store) = setup();
        let mut host_q = base_query();
        host_q.grouping = DimensionKind::Host;
        let mut cat_q = base_query();
        cat_q.grouping = DimensionKind::Category;
        let host = run_now(&coordinator, &mut store, host_q, 3_600);
        let category = run_now(&coordinator, &mut store, cat_q, 3_600);
        let host_ids: Vec<_> = host
            .rankings
            .iter()
            .map(|row| row.identity.as_str())
            .collect();
        let cat_ids: Vec<_> = category
            .rankings
            .iter()
            .map(|row| row.identity.as_str())
            .collect();
        assert!(cat_ids.contains(&"家宽") || cat_ids.contains(&"机场"));
        assert_ne!(host_ids, cat_ids);
    }

    #[test]
    fn rule_rank_uses_policy_group_not_raw_type() {
        let (_dir, coordinator, mut store) = setup();
        let mut query = base_query();
        query.grouping = DimensionKind::Rule;
        let result = run_now(&coordinator, &mut store, query, 3_600);
        let ids: Vec<_> = result
            .rankings
            .iter()
            .map(|row| row.identity.as_str())
            .collect();
        assert!(ids.contains(&"家宽"));
        assert!(!ids.contains(&"RuleSet"));
        assert!(!ids.contains(&"Match"));
    }

    #[test]
    fn drilldown_filter_chain_and_rule_take_subset() {
        let (_dir, coordinator, mut store) = setup();
        let global = run_now(&coordinator, &mut store, base_query(), 3_600);
        let mut chain_q = base_query();
        chain_q.grouping = DimensionKind::Chain;
        let chain = run_now(&coordinator, &mut store, chain_q, 3_600);
        let hop = chain
            .rankings
            .iter()
            .find(|row| row.identity == "家宽")
            .expect("hop");
        let mut filtered = base_query();
        filtered.filters.chain = Some(hop.identity.clone());
        let subset = run_now(&coordinator, &mut store, filtered, 3_600);
        assert_eq!(subset.totals.download, hop.download);
        assert!(subset.totals.download < global.totals.download);

        let mut rule_q = base_query();
        rule_q.grouping = DimensionKind::Rule;
        let rules = run_now(&coordinator, &mut store, rule_q, 3_600);
        let rule_row = rules
            .rankings
            .iter()
            .find(|row| row.identity == "家宽")
            .expect("rule");
        let mut rule_filtered = base_query();
        rule_filtered.filters.rule = Some(rule_row.identity.clone());
        let rule_subset = run_now(&coordinator, &mut store, rule_filtered, 3_600);
        assert_eq!(rule_subset.totals.download, rule_row.download);
        assert!(rule_subset.totals.download < global.totals.download);
    }

    fn assert_filter_reduces(field: &str, value: &str) {
        let (_dir, coordinator, mut store) = setup();
        let global = run_now(&coordinator, &mut store, base_query(), 3_600);
        let mut query = base_query();
        match field {
            "host" => query.filters.host = Some(value.into()),
            "process" => query.filters.process = Some(value.into()),
            "rule" => query.filters.rule = Some(value.into()),
            "network" => query.filters.network = Some(value.into()),
            "chain" => query.filters.chain = Some(value.into()),
            "category" => query.filters.category = Some(value.into()),
            _ => panic!("field"),
        }
        let filtered = run_now(&coordinator, &mut store, query, 3_600);
        let series_up: i64 = filtered.series.iter().map(|point| point.upload).sum();
        let series_down: i64 = filtered.series.iter().map(|point| point.download).sum();
        assert_eq!(series_up, filtered.totals.upload, "{field} series upload");
        assert_eq!(
            series_down, filtered.totals.download,
            "{field} series download"
        );
        assert!(
            filtered.totals.download < global.totals.download,
            "{field} should reduce"
        );
    }

    #[test]
    fn host_filter_reduces_and_series_matches_totals() {
        assert_filter_reduces("host", "c.example");
    }

    #[test]
    fn process_filter_reduces_and_series_matches_totals() {
        assert_filter_reduces("process", "other.exe");
    }

    #[test]
    fn rule_filter_reduces_and_series_matches_totals() {
        assert_filter_reduces("rule", "家宽");
    }

    #[test]
    fn network_filter_reduces_and_series_matches_totals() {
        assert_filter_reduces("network", "udp");
    }

    #[test]
    fn chain_filter_reduces_and_series_matches_totals() {
        assert_filter_reduces("chain", "家宽");
    }

    #[test]
    fn category_filter_reduces_and_series_matches_totals() {
        assert_filter_reduces("category", "机场");
    }

    #[test]
    fn minute_granularity_raw_series_buckets() {
        let (_dir, coordinator, mut store) = setup();
        let mut query = base_query();
        query.granularity = Granularity::Minute1;
        let result = run_now(&coordinator, &mut store, query, 3_600);
        assert!(result.series.len() >= 2);
        assert_eq!(result.data_tier, DataTier::Raw);
    }

    fn enable_v2_from_epoch(coordinator: &StorageCoordinator) {
        coordinator
            .connection()
            .execute(
                "insert or replace into retention_watermark(layer, watermark_utc, delete_watermark_utc)
                 values ('hourly_dim_v2', 0, 0)",
                [],
            )
            .expect("v2");
    }

    fn materialize(coordinator: &mut StorageCoordinator, now: i64) {
        RetentionService::run(
            coordinator,
            now,
            30,
            RetentionMode::MaterializeOnly,
            &SpaceBudget::unlimited(),
            &Arc::new(AtomicBool::new(false)),
        )
        .expect("materialize");
    }

    #[test]
    fn five_kinds_materialize_and_keys_match_raw() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("m.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        seed_extra_dimensions(coordinator.connection());
        enable_v2_from_epoch(&coordinator);
        let mut store = ReportSnapshotStore::open(dir.path());
        let mut raw_q = base_query();
        raw_q.grouping = DimensionKind::Chain;
        let raw_chain = run_now(&coordinator, &mut store, raw_q, 3_600);
        let mut raw_rule_q = base_query();
        raw_rule_q.grouping = DimensionKind::Rule;
        let raw_rule = run_now(&coordinator, &mut store, raw_rule_q, 3_600);
        materialize(&mut coordinator, 40 * 86_400);
        let kinds: Vec<String> = {
            let mut statement = coordinator
                .connection()
                .prepare("select distinct dimension_kind from traffic_hourly_dimension order by 1")
                .expect("kinds");
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
        let mut dim_q = base_query();
        dim_q.grouping = DimensionKind::Chain;
        dim_q.range_start_utc = 0;
        dim_q.range_end_utc = 4_000;
        let dim_chain = run_now(&coordinator, &mut store, dim_q, 40 * 86_400);
        assert_eq!(dim_chain.data_tier, DataTier::HourlyDimension);
        let mut raw_keys: Vec<_> = raw_chain
            .rankings
            .iter()
            .map(|row| row.identity.clone())
            .collect();
        let mut dim_keys: Vec<_> = dim_chain
            .rankings
            .iter()
            .map(|row| row.identity.clone())
            .collect();
        raw_keys.sort();
        dim_keys.sort();
        assert_eq!(raw_keys, dim_keys);
        let mut dim_rule_q = base_query();
        dim_rule_q.grouping = DimensionKind::Rule;
        let dim_rule = run_now(&coordinator, &mut store, dim_rule_q, 40 * 86_400);
        let mut raw_rule_keys: Vec<_> = raw_rule
            .rankings
            .iter()
            .map(|row| row.identity.clone())
            .collect();
        let mut dim_rule_keys: Vec<_> = dim_rule
            .rankings
            .iter()
            .map(|row| row.identity.clone())
            .collect();
        raw_rule_keys.sort();
        dim_rule_keys.sort();
        assert_eq!(raw_rule_keys, dim_rule_keys);
    }

    #[test]
    fn process_rule_chain_network_category_rank_outside_raw() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("old.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        seed_extra_dimensions(coordinator.connection());
        enable_v2_from_epoch(&coordinator);
        materialize(&mut coordinator, 40 * 86_400);
        let mut store = ReportSnapshotStore::open(dir.path());
        for grouping in [
            DimensionKind::Process,
            DimensionKind::Rule,
            DimensionKind::Chain,
            DimensionKind::Network,
            DimensionKind::Category,
        ] {
            let mut query = base_query();
            query.grouping = grouping;
            let result = run_now(&coordinator, &mut store, query, 40 * 86_400);
            assert!(
                !result.rankings.is_empty(),
                "{grouping:?} should have ranks"
            );
            assert_eq!(result.data_tier, DataTier::HourlyDimension);
        }
    }

    #[test]
    fn range_before_v2_watermark_returns_chinese_capability() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("wm.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        materialize(&mut coordinator, 40 * 86_400);
        let mut store = ReportSnapshotStore::open(dir.path());
        let mut query = base_query();
        query.grouping = DimensionKind::Process;
        let cancel = Arc::new(AtomicBool::new(false));
        let error = ReportService::run(
            coordinator.path(),
            &mut store,
            query,
            40 * 86_400,
            30,
            &cancel,
            None,
        )
        .expect_err("before v2");
        assert_eq!(error.code(), "capability_unsupported");
        assert!(error.to_string().contains("五维物化水位"));
    }

    #[test]
    fn unknown_rank_row_closes_totals() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("unk.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        coordinator
            .connection()
            .execute_batch(
                "
                insert or ignore into connection_session(session_pk, epoch_id, connection_id, started_utc, host)
                values (9, 1, 'none', 100, 'orphan.example');
                insert or ignore into connection_minute(utc_minute, session_pk, upload, download)
                values (2, 9, 7, 11);
                ",
            )
            .expect("orphan");
        enable_v2_from_epoch(&coordinator);
        materialize(&mut coordinator, 40 * 86_400);
        let mut store = ReportSnapshotStore::open(dir.path());
        let mut query = base_query();
        query.grouping = DimensionKind::Process;
        query.top_n = 100;
        let result = run_now(&coordinator, &mut store, query, 40 * 86_400);
        let rank_down: i64 = result.rankings.iter().map(|row| row.download).sum();
        assert_eq!(rank_down, result.totals.download);
        assert!(result
            .rankings
            .iter()
            .any(|row| row.identity == UNKNOWN_IDENTITY && row.label == "未知"));
        let exists: i64 = coordinator
            .connection()
            .query_row(
                "select count(*) from dimension_dict where value = ?1",
                [UNKNOWN_IDENTITY],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(exists, 0);
    }

    #[test]
    fn category_rank_sum_does_not_exceed_totals() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("cat.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        seed_extra_dimensions(coordinator.connection());
        enable_v2_from_epoch(&coordinator);
        materialize(&mut coordinator, 40 * 86_400);
        let mut store = ReportSnapshotStore::open(dir.path());
        let mut query = base_query();
        query.grouping = DimensionKind::Category;
        query.top_n = 100;
        let result = run_now(&coordinator, &mut store, query, 40 * 86_400);
        let rank_down: i64 = result.rankings.iter().map(|row| row.download).sum();
        assert!(rank_down <= result.totals.download);
    }
}
