//! 统一 ReportService：短读快照内物化，返回前关闭事务。

use crate::c3::query::{
    coverage_union_secs, decode_cursor, empty_result, encode_cursor, plan_capability,
    plan_capability_ex, plan_capability_with_raw_cutoff, validate_query, AttributionQuality,
    CapabilityPlan, CoverageSlice, DataTier, DimensionKind, Granularity, PolicyMetadata,
    RankingRow, ReportError, ReportQuery, ReportResult, ReportTotals, SeriesPoint, SessionRow,
    TargetPolicy, HOURLY_DIM_V2_LAYER, UNKNOWN_LABEL_ZH,
};
use crate::c3::snapshot::ReportSnapshotStore;
#[cfg(test)]
use crate::c3::sql::SERIES_RAW_GROUPED;
use crate::c3::sql::{
    dimension_filter_clause, dimension_kind_sql_layer, filter_clause, merge_sql_params,
    render_rank_sql, render_raw_totals_sql, render_sql, RankLayer, COVERAGE_DAILY, COVERAGE_RAW,
    RANK_DAILY_CATEGORY, RANK_DAILY_DIM, RESIDENTIAL_ACCOUNTING_FILTER, SERIES_DAILY_DIM,
    SERIES_HOURLY, TOTALS_DAILY_CORE_FILTERED, TOTALS_DAILY_DIM, UNKNOWN_IDENTITY,
    USAGE_DAILY_CORE, USAGE_HOURLY, USAGE_RAW,
};
#[cfg(test)]
use crate::c3::sql::{raw_exit_sql, render_exit_sql, RANK_RAW, SERIES_RAW, TOTALS_RAW};
use crate::storage::{open_interruptible_reader, StorageCoordinator};
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
use std::collections::HashSet;
use std::path::Path;
#[cfg(test)]
use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
#[cfg(test)]
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[cfg(test)]
static C3_HOLD_MS: AtomicU64 = AtomicU64::new(0);
#[cfg(test)]
static C3_HOLD_ENTERED: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
static C3_HOLD_LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
pub struct C3HoldGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
}

#[cfg(test)]
impl Drop for C3HoldGuard {
    fn drop(&mut self) {
        C3_HOLD_MS.store(0, Ordering::SeqCst);
        C3_HOLD_ENTERED.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
pub fn hold_c3_runs(ms: u64) -> C3HoldGuard {
    let lock = C3_HOLD_LOCK.lock().expect("c3 hold lock");
    C3_HOLD_ENTERED.store(false, Ordering::SeqCst);
    C3_HOLD_MS.store(ms, Ordering::SeqCst);
    C3HoldGuard { _lock: lock }
}

#[cfg(test)]
pub fn c3_hold_entered() -> bool {
    C3_HOLD_ENTERED.load(Ordering::SeqCst)
}

pub fn poll_interrupt(cancel: &Arc<AtomicBool>, reason: &'static str) -> Result<(), ReportError> {
    #[cfg(test)]
    {
        let hold_ms = C3_HOLD_MS.load(Ordering::SeqCst);
        if hold_ms > 0 {
            C3_HOLD_ENTERED.store(true, Ordering::SeqCst);
            let deadline = Instant::now() + Duration::from_millis(hold_ms);
            while Instant::now() < deadline {
                if cancel.load(Ordering::SeqCst) {
                    return Err(ReportError::Cancelled(reason));
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
    if cancel.load(Ordering::SeqCst) {
        return Err(ReportError::Cancelled(reason));
    }
    Ok(())
}

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
        let result = run_uncached(
            db_path,
            query.clone(),
            now_utc,
            raw_retain_days,
            cancel,
            deadline,
        )?;
        store.insert(&query, result, now_utc, false)
    }

    pub fn explain_named(connection: &Connection, name: &str) -> Result<Vec<String>, ReportError> {
        let sql = crate::c3::sql::lookup(name).ok_or(ReportError::InvalidQuery("unknown sql"))?;
        let sql = if sql.contains("{missing}") {
            render_raw_totals_sql("", DimensionKind::Host)
        } else if sql.contains("{order_by}") {
            render_rank_sql(
                sql,
                "",
                &crate::c3::query::SortSpec::default(),
                if sql.contains("connection_minute m") {
                    RankLayer::Raw
                } else {
                    RankLayer::Dimension
                },
            )
        } else {
            render_sql(sql, "")
        };
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

pub fn run_uncached(
    db_path: &Path,
    query: ReportQuery,
    now_utc: i64,
    raw_retain_days: i64,
    cancel: &Arc<AtomicBool>,
    deadline: Option<Duration>,
) -> Result<ReportResult, ReportError> {
    let started = Instant::now();
    poll_interrupt(cancel, "user")?;
    validate_query(&query)?;
    let plan = plan_capability(&query, now_utc, raw_retain_days)?;
    let limit = deadline.unwrap_or(Duration::from_millis(plan.deadline_ms));
    let reader = open_interruptible_reader(db_path).map_err(map_storage)?;
    attach_cancel(&reader, cancel, started, limit)?;
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
    if cancel.load(Ordering::SeqCst) {
        return Err(ReportError::Cancelled("user"));
    }
    if started.elapsed() >= limit {
        return Err(ReportError::DeadlineExceeded("report query"));
    }
    let result = built?;
    close.map_err(|_| ReportError::Failed("commit snapshot"))?;
    if txn_open {
        return Err(ReportError::Failed("read transaction still open"));
    }
    Ok(result)
}

pub(crate) fn attach_cancel(
    connection: &Connection,
    cancel: &Arc<AtomicBool>,
    started: Instant,
    deadline: Duration,
) -> Result<(), ReportError> {
    let flag = Arc::clone(cancel);
    connection
        .progress_handler(
            1024,
            Some(move || flag.load(Ordering::SeqCst) || started.elapsed() > deadline),
        )
        .map_err(|_| ReportError::Failed("progress handler"))?;
    Ok(())
}

/// 仅供周期告警使用；字节可跨桶相加，不携带无法恢复的 distinct 计数。
#[derive(Debug)]
pub(crate) struct PeriodUsage {
    pub upload: i64,
    pub download: i64,
    pub covered_sec: i64,
    pub gap_sec: i64,
    pub data_version: u64,
    pub policy_note: String,
}

pub(crate) fn query_period_usage(
    db_path: &Path,
    query: &ReportQuery,
    now_utc: i64,
    raw_retain_days: i64,
    cancel: &Arc<AtomicBool>,
) -> Result<PeriodUsage, ReportError> {
    let started = Instant::now();
    poll_interrupt(cancel, "period usage")?;
    plan_capability(query, now_utc, raw_retain_days)?;
    if query.include_sessions
        || query.target_policy != TargetPolicy::Historical
        || query.range_end_utc > now_utc
    {
        return Err(ReportError::CapabilityUnsupported(
            "周期用量仅支持截至观测时刻的历史字节与覆盖",
        ));
    }
    let deadline = Duration::from_millis(crate::c3::query::REPORT_DEADLINE_MS);
    let reader = open_interruptible_reader(db_path).map_err(map_storage)?;
    attach_cancel(&reader, cancel, started, deadline)?;
    reader.execute_batch("begin deferred").map_err(map_sqlite)?;
    let built = build_period_usage(&reader, query, now_utc);
    let closed = reader.execute_batch("commit");
    if !reader.is_autocommit() {
        let _ = reader.execute_batch("rollback");
    }
    drop(reader);
    if cancel.load(Ordering::SeqCst) {
        return Err(ReportError::Cancelled("period usage"));
    }
    if started.elapsed() >= deadline {
        return Err(ReportError::DeadlineExceeded("period usage"));
    }
    let usage = built?;
    closed.map_err(map_sqlite)?;
    Ok(usage)
}

fn build_period_usage(
    connection: &Connection,
    query: &ReportQuery,
    now_utc: i64,
) -> Result<PeriodUsage, ReportError> {
    let first_day = query.range_start_utc.div_euclid(86_400) * 86_400;
    let mut statement = connection.prepare("select chunk_utc from retention_state where layer=?1 and status='deleted' and chunk_utc>=?2 and chunk_utc<?3").map_err(map_sqlite)?;
    let deleted = statement
        .query_map(
            params![
                crate::c3::retention::DAY_EXACT_LAYER,
                first_day,
                query.range_end_utc
            ],
            |row| row.get::<_, i64>(0),
        )
        .map_err(map_sqlite)?
        .collect::<Result<HashSet<_>, _>>()
        .map_err(map_sqlite)?;
    let mut usage = PeriodUsage {
        upload: 0,
        download: 0,
        covered_sec: 0,
        gap_sec: 0,
        data_version: crate::storage::durable_data_version(connection).map_err(map_sqlite)?,
        policy_note: "历史主分类来自写入时策略；字节与覆盖在同一读快照内计算，观测下界不是账单。"
            .into(),
    };
    let mut start = query.range_start_utc;
    while start < query.range_end_utc {
        let day = start.div_euclid(86_400) * 86_400;
        let mut end = (day + 86_400).min(query.range_end_utc);
        let is_deleted = deleted.contains(&day);
        if !is_deleted {
            // 连续保留 raw 合并成一次范围扫描，不按日重复扫描连接。
            while end < query.range_end_utc && !deleted.contains(&end) {
                end = (end + 86_400).min(query.range_end_utc);
            }
        }
        let bytes = if !is_deleted {
            let (filter, values) = filter_clause(&query.filters);
            load_bytes(
                connection,
                USAGE_RAW,
                &filter,
                merge_sql_params(
                    [
                        Value::from(start.div_euclid(60)),
                        Value::from(end.div_euclid(60)),
                    ],
                    &values,
                    [],
                ),
            )?
        } else {
            ensure_matching_filter(query)?;
            ensure_residential_history(connection, query, day)?;
            if query.grouping == DimensionKind::Category && start == day && end == day + 86_400 {
                let (filter, values) = core_filter(query.filters.category.as_deref());
                load_bytes(
                    connection,
                    USAGE_DAILY_CORE,
                    &filter,
                    merge_sql_params([Value::from(start), Value::from(end)], &values, []),
                )?
            } else {
                if start < now_utc - crate::c3::query::DIMENSION_RETAIN_DAYS * 86_400
                    || start.rem_euclid(3_600) != 0
                    || end.rem_euclid(3_600) != 0
                {
                    return Err(ReportError::CapabilityUnsupported(
                        "周期边界细于已保留汇总，或精确维度已经过期",
                    ));
                }
                let (filter, values) = dimension_filter_clause(&query.filters, query.grouping);
                let kind = if query.grouping == DimensionKind::Category {
                    "host"
                } else {
                    dimension_kind_sql_layer(query.grouping)
                };
                load_bytes(
                    connection,
                    USAGE_HOURLY,
                    &filter,
                    merge_sql_params(
                        [
                            Value::from(start),
                            Value::from(end),
                            Value::Text(kind.into()),
                        ],
                        &values,
                        [],
                    ),
                )?
            }
        };
        usage.upload += bytes.0;
        usage.download += bytes.1;
        let (covered, gap) =
            if is_deleted && start < now_utc - crate::c3::query::DIMENSION_RETAIN_DAYS * 86_400 {
                if start != day || end != day + 86_400 {
                    return Err(ReportError::CapabilityUnsupported(
                        "长期覆盖汇总无法恢复部分日覆盖",
                    ));
                }
                connection
                    .query_row(
                        "select covered_sec,gap_sec from coverage_daily where utc_day=?1",
                        [day],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .map_err(map_sqlite)?
            } else {
                coverage_union_secs(start, end, &read_raw_coverage(connection, start, end)?)
            };
        usage.covered_sec += covered;
        usage.gap_sec += gap;
        start = end;
    }
    Ok(usage)
}

fn load_bytes(
    connection: &Connection,
    template: &str,
    filter: &str,
    values: Vec<Value>,
) -> Result<(i64, i64), ReportError> {
    connection
        .query_row(
            &render_sql(template, filter),
            params_from_iter(values.iter()),
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(map_sqlite)
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
    let plan = plan_snapshot_capability(connection, query, now_utc, raw_retain_days)?;
    let data_version = crate::storage::durable_data_version(connection).map_err(map_sqlite)?;
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
    // raw 总量查询已经同时返回归因，避免再次扫描同一明细窗口。
    if result.data_tier != DataTier::Raw {
        result.attribution_quality = load_attribution_quality(connection, query, &result)?;
    }
    if result.attribution_quality.known_upload + result.attribution_quality.missing_upload
        != result.totals.upload
        || result.attribution_quality.known_download + result.attribution_quality.missing_download
            != result.totals.download
        || result.attribution_quality.known_connections
            + result.attribution_quality.missing_connections
            != result.totals.connection_count
    {
        return Err(ReportError::Failed("attribution conservation"));
    }
    if result.coverage.gap_sec > 0 {
        result.coverage.status = "partial".into();
    } else if result.totals.upload == 0
        && result.totals.download == 0
        && result.coverage.slices.is_empty()
    {
        result.coverage.status = "empty".into();
    } else if result.coverage.covered_sec == 0 {
        result.coverage.status = "unknown".into();
    } else if result.coverage.covered_sec < query.range_end_utc - query.range_start_utc {
        result.coverage.status = "partial".into();
    } else {
        result.coverage.status = "covered".into();
    }
    Ok(result)
}

fn plan_snapshot_capability(
    connection: &Connection,
    query: &ReportQuery,
    now_utc: i64,
    raw_retain_days: i64,
) -> Result<CapabilityPlan, ReportError> {
    // 先检查公开保留期能力；仍在库中的过期 raw 不能重新授权会话下钻等能力。
    let mut base = plan_capability(query, now_utc, raw_retain_days)?;
    let first_day = query.range_start_utc.div_euclid(86_400) * 86_400;
    let day_end = (query.range_end_utc - 1).div_euclid(86_400) * 86_400 + 86_400;
    let (verified, deleted) = finalized_days(connection, first_day, day_end)?;
    if base.tier == DataTier::Raw {
        if deleted == 0 {
            return Ok(base);
        }
        // 增大保留天数不能恢复已经删除的明细，必须按实际可用层重新判定能力。
        base = plan_capability_with_raw_cutoff(
            query,
            now_utc,
            query.range_end_utc,
            load_hourly_dim_v2_start(connection),
        )?;
    }
    let complete = verified == (day_end - first_day) / 86_400;
    let exact = if complete {
        ensure_exact_aggregate(connection, query, base.tier)
    } else {
        Err(ReportError::CapabilityUnsupported(
            "该区间的整日汇总尚未全部确认",
        ))
    };
    if let Err(error) = exact {
        if deleted > 0 || !matches!(error, ReportError::CapabilityUnsupported(_)) {
            return Err(error);
        }
        // 自动清理前尚存完整明细，允许用 raw 计算已授权的历史汇总；不读空汇总当零。
        let mut raw = plan_capability_ex(query, query.range_start_utc, raw_retain_days, None)?;
        raw.drilldown = base.drilldown;
        raw.drilldown.exact_top_n = true;
        raw.drilldown.note_zh = "本次从尚存明细计算精确计数与时长；过期会话能力仍不可用。".into();
        raw.deadline_ms = base.deadline_ms;
        return Ok(raw);
    }
    if deleted > 0 {
        return Ok(base);
    }
    let v2_start = load_hourly_dim_v2_start(connection);
    plan_capability_ex(query, now_utc, raw_retain_days, v2_start)
}

fn core_filter(category: Option<&str>) -> (String, Vec<String>) {
    match category {
        None => (" and category_id = 0".into(), Vec::new()),
        Some(RESIDENTIAL_ACCOUNTING_FILTER) => (" and category_id > 0".into(), Vec::new()),
        Some(value) => (
            " and category_id = (select dimension_id from dimension_dict where dimension_kind='category' and value=?)".into(),
            vec![value.into()],
        ),
    }
}

fn identity_filter(query: &ReportQuery) -> bool {
    match query.grouping {
        DimensionKind::Host => query.filters.host.is_some(),
        DimensionKind::Process => query.filters.process.is_some(),
        DimensionKind::Rule => query.filters.rule.is_some(),
        DimensionKind::Chain => query.filters.chain.is_some(),
        DimensionKind::Network => query.filters.network.is_some(),
        DimensionKind::Category => false,
    }
}

fn ensure_matching_filter(query: &ReportQuery) -> Result<(), ReportError> {
    let filters = &query.filters;
    if (filters.host.is_some() && query.grouping != DimensionKind::Host)
        || (filters.process.is_some() && query.grouping != DimensionKind::Process)
        || (filters.rule.is_some() && query.grouping != DimensionKind::Rule)
        || (filters.chain.is_some() && query.grouping != DimensionKind::Chain)
        || (filters.network.is_some() && query.grouping != DimensionKind::Network)
    {
        return Err(ReportError::CapabilityUnsupported(
            "汇总不支持跨分析维度过滤",
        ));
    }
    Ok(())
}

fn ensure_residential_history(
    connection: &Connection,
    query: &ReportQuery,
    day: i64,
) -> Result<(), ReportError> {
    if query.filters.category.as_deref() != Some(RESIDENTIAL_ACCOUNTING_FILTER) {
        return Ok(());
    }
    // legacy NULL 分类曾通过 raw chain 恢复家宽归属；链路退出后不能把这部分当作非家宽零值。
    let unclassified: bool = connection.query_row(
        "select coalesce(sum(case when category_id=0 then upload+download else -(upload+download) end),0)>0
         from traffic_daily_core where utc_day=?1", [day], |row| row.get(0),
    ).map_err(map_sqlite)?;
    if unclassified {
        return Err(ReportError::CapabilityUnsupported(
            "该日仍有未分类历史流量，raw 链路已不可用于精确恢复家宽归属",
        ));
    }
    Ok(())
}

/// 标量去重结果不能跨日、分类或身份相加后冒充原始明细的 distinct。
fn ensure_exact_aggregate(
    connection: &Connection,
    query: &ReportQuery,
    tier: DataTier,
) -> Result<(), ReportError> {
    if query.range_start_utc.rem_euclid(86_400) != 0
        || query.range_end_utc - query.range_start_utc != 86_400
        || !matches!(query.granularity, Granularity::Day | Granularity::Hour)
    {
        return Err(ReportError::CapabilityUnsupported(
            "明细已汇总；精确连接数与时长只支持单个完整 UTC 日，跨日或部分日无法去重",
        ));
    }
    ensure_matching_filter(query)?;
    ensure_residential_history(connection, query, query.range_start_utc)?;
    if !identity_filter(query) {
        let (filter, values) = core_filter(query.filters.category.as_deref());
        let sql = format!("select count(*) from traffic_daily_core where utc_day=? {filter}");
        let params = merge_sql_params([Value::from(query.range_start_utc)], &values, []);
        let count: i64 = connection
            .query_row(&sql, params_from_iter(params.iter()), |row| row.get(0))
            .map_err(map_sqlite)?;
        if count > 1 {
            return Err(ReportError::CapabilityUnsupported(
                "所选分类的活跃分钟可能重叠，现有汇总无法精确去重",
            ));
        }
    }
    if tier == DataTier::DailyCore {
        if query.granularity == Granularity::Hour {
            return Err(ReportError::CapabilityUnsupported(
                "长期汇总没有逐小时精确序列",
            ));
        }
        return Ok(());
    }
    let (filter, values) = dimension_filter_clause(&query.filters, query.grouping);
    let kind = if query.grouping == DimensionKind::Category {
        "host"
    } else {
        dimension_kind_sql_layer(query.grouping)
    };
    let key = if query.grouping == DimensionKind::Category {
        "h.category_id"
    } else {
        "h.dimension_id"
    };
    let sql = format!("select exists(select 1 from traffic_daily_dimension h where utc_day=? and dimension_kind=? {filter} group by {key} having count(*)>1)");
    let params = merge_sql_params(
        [Value::from(query.range_start_utc), Value::Text(kind.into())],
        &values,
        [],
    );
    let ambiguous: bool = connection
        .query_row(&sql, params_from_iter(params.iter()), |row| row.get(0))
        .map_err(map_sqlite)?;
    if ambiguous {
        return Err(ReportError::CapabilityUnsupported(
            "排名身份跨多个分类或身份汇总，活跃分钟无法精确去重",
        ));
    }
    if query.granularity == Granularity::Hour {
        let sql = format!("select exists(select 1 from traffic_hourly_dimension h where utc_hour>=? and utc_hour<? and dimension_kind=? {filter} group by utc_hour having count(*)>1)");
        let params = merge_sql_params(
            [
                Value::from(query.range_start_utc),
                Value::from(query.range_end_utc),
                Value::Text(kind.into()),
            ],
            &values,
            [],
        );
        let ambiguous: bool = connection
            .query_row(&sql, params_from_iter(params.iter()), |row| row.get(0))
            .map_err(map_sqlite)?;
        if ambiguous {
            return Err(ReportError::CapabilityUnsupported(
                "小时内有多个身份汇总，逐小时活跃分钟无法精确去重",
            ));
        }
    }
    Ok(())
}

fn finalized_days(
    connection: &Connection,
    first_day: i64,
    day_end: i64,
) -> Result<(i64, i64), ReportError> {
    connection
        .query_row(
            "select count(*), coalesce(sum(case when status = 'deleted' then 1 else 0 end), 0)
           from retention_state where layer = ?1 and chunk_utc >= ?2 and chunk_utc < ?3
             and status in ('verified', 'deleted')",
            params![crate::c3::retention::DAY_EXACT_LAYER, first_day, day_end],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(map_sqlite)
}

pub(crate) fn raw_range_is_retained(
    connection: &Connection,
    start: i64,
    end: i64,
) -> Result<bool, ReportError> {
    if end <= start {
        return Ok(true);
    }
    let first_day = start.div_euclid(86_400) * 86_400;
    let day_end = (end - 1).div_euclid(86_400) * 86_400 + 86_400;
    Ok(finalized_days(connection, first_day, day_end)?.1 == 0)
}

fn load_attribution_quality(
    connection: &Connection,
    query: &ReportQuery,
    result: &ReportResult,
) -> Result<AttributionQuality, ReportError> {
    match result.data_tier {
        DataTier::Raw => Ok(result.attribution_quality.clone()),
        DataTier::HourlyDimension | DataTier::DailyDimension => {
            load_dimension_attribution_quality(connection, query, DataTier::DailyDimension)
        }
        DataTier::DailyCore => Ok(AttributionQuality::unavailable(&result.totals)),
    }
}

#[cfg(test)]
fn load_raw_attribution_quality(
    connection: &Connection,
    query: &ReportQuery,
) -> Result<AttributionQuality, ReportError> {
    // 保留原先独立聚合算法作为等价性回归的参考。
    let missing = crate::c3::sql::raw_attribution_missing_sql(query.grouping);
    let (fragment, filter_params) = filter_clause(&query.filters);
    let sql = format!(
        "select
            coalesce(sum(case when not ({missing}) then m.upload else 0 end), 0),
            coalesce(sum(case when not ({missing}) then m.download else 0 end), 0),
            coalesce(sum(case when ({missing}) then m.upload else 0 end), 0),
            coalesce(sum(case when ({missing}) then m.download else 0 end), 0),
            count(distinct case when not ({missing}) then m.session_pk end),
            count(distinct case when ({missing}) then m.session_pk end)
         from connection_minute m
         join connection_session s on s.session_pk=m.session_pk
         left join connection_session_attr a on a.session_pk=m.session_pk
         where m.utc_minute >= ? and m.utc_minute < ? {fragment}"
    );
    let params = merge_sql_params(
        [
            Value::from(query.range_start_utc.div_euclid(60)),
            Value::from(query.range_end_utc.div_euclid(60)),
        ],
        &filter_params,
        [],
    );
    load_quality_row(connection, &sql, &params)
}

fn load_dimension_attribution_quality(
    connection: &Connection,
    query: &ReportQuery,
    tier: DataTier,
) -> Result<AttributionQuality, ReportError> {
    let table = match tier {
        DataTier::HourlyDimension => "traffic_hourly_dimension",
        DataTier::DailyDimension => "traffic_daily_dimension",
        _ => return Err(ReportError::Failed("attribution tier")),
    };
    let time_column = match tier {
        DataTier::HourlyDimension => "utc_hour",
        DataTier::DailyDimension => "utc_day",
        _ => return Err(ReportError::Failed("attribution tier")),
    };
    let kind = if query.grouping == DimensionKind::Category {
        "host"
    } else {
        dimension_kind_sql_layer(query.grouping)
    };
    let missing = if query.grouping == DimensionKind::Category {
        "h.category_id = 0"
    } else {
        "h.dimension_id = 0"
    };
    let (fragment, filter_params) = dimension_filter_clause(&query.filters, query.grouping);
    let sql = format!(
        "select
            coalesce(sum(case when not ({missing}) then h.upload else 0 end), 0),
            coalesce(sum(case when not ({missing}) then h.download else 0 end), 0),
            coalesce(sum(case when ({missing}) then h.upload else 0 end), 0),
            coalesce(sum(case when ({missing}) then h.download else 0 end), 0),
            coalesce(sum(case when not ({missing}) then h.connection_count else 0 end), 0),
            coalesce(sum(case when ({missing}) then h.connection_count else 0 end), 0)
         from {table} h
         where h.{time_column} >= ? and h.{time_column} < ?
           and h.dimension_kind = ? {fragment}"
    );
    let params = merge_sql_params(
        [
            Value::from(query.range_start_utc),
            Value::from(query.range_end_utc),
            Value::Text(kind.into()),
        ],
        &filter_params,
        [],
    );
    load_quality_row(connection, &sql, &params)
}

fn load_quality_row(
    connection: &Connection,
    sql: &str,
    params: &[Value],
) -> Result<AttributionQuality, ReportError> {
    connection
        .query_row(sql, params_from_iter(params.iter()), |row| {
            Ok(AttributionQuality::from_parts(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })
        .map_err(map_sqlite)
}

fn fill_raw(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start_min: i64,
    end_min: i64,
) -> Result<(), ReportError> {
    let mut projection_start = start_min;
    let mut previous_min = None;
    if query
        .comparison
        .as_ref()
        .is_some_and(|item| item.previous_equal_window)
    {
        let span = query.range_end_utc - query.range_start_utc;
        let previous_utc = query.range_start_utc - span;
        if raw_range_is_retained(connection, previous_utc, query.range_start_utc)? {
            let minute = previous_utc.div_euclid(60);
            if minute < projection_start {
                projection_start = minute;
            }
            previous_min = Some(minute);
        } else {
            result
                .drilldown_capability
                .note_zh
                .push_str(" 前一区间明细已清理，比较量未知。");
        }
    }
    let sessions =
        crate::c3::raw_fold::load_sessions(connection, query, projection_start, end_min)?;
    let folded =
        crate::c3::raw_fold::fold_window(connection, query, &sessions, start_min, end_min)?;
    result.totals = folded.totals;
    result.attribution_quality = folded.attribution;
    result.series = folded.series;
    result.rankings = folded.rankings;
    if let Some(minute) = previous_min {
        let previous =
            crate::c3::raw_fold::fold_window(connection, query, &sessions, minute, start_min)?;
        result.totals.previous_upload = Some(previous.totals.upload);
        result.totals.previous_download = Some(previous.totals.download);
    }
    result.named_sql = crate::c3::raw_fold::executed_names();
    result.named_sql.push("coverage_raw".to_string());
    fill_coverage_raw(connection, query, result)?;
    if query.include_sessions {
        fill_sessions(connection, query, result, start_min, end_min)?;
        result.named_sql.push("sessions_keyset".to_string());
    }
    Ok(())
}

fn fill_hourly(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start: i64,
    end: i64,
) -> Result<(), ReportError> {
    fill_dimension_layer(connection, query, result, start, end, SERIES_HOURLY)
}

fn fill_daily_dim(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start: i64,
    end: i64,
) -> Result<(), ReportError> {
    fill_dimension_layer(connection, query, result, start, end, SERIES_DAILY_DIM)
}

fn fill_dimension_layer(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start: i64,
    end: i64,
    series_sql: &str,
) -> Result<(), ReportError> {
    let (fragment, filter_params) = dimension_filter_clause(&query.filters, query.grouping);
    let kind = if query.grouping == DimensionKind::Category {
        "host"
    } else {
        dimension_kind_sql_layer(query.grouping)
    };
    let rendered_totals = render_sql(TOTALS_DAILY_DIM, &fragment);
    let totals_params = merge_sql_params(
        [
            Value::from(start),
            Value::from(end),
            Value::Text(kind.into()),
        ],
        &filter_params,
        [],
    );
    let (upload, download, count, duration) = if identity_filter(query) {
        load_totals(connection, &rendered_totals, &totals_params)?
    } else {
        load_core_totals(connection, query)?
    };
    result.totals = ReportTotals {
        upload,
        download,
        connection_count: count,
        active_duration_sec: duration,
        previous_upload: None,
        previous_download: None,
    };
    if query.granularity == Granularity::Day {
        result.series = vec![SeriesPoint {
            bucket_utc: start,
            upload,
            download,
            connection_count: count,
            active_duration_sec: duration,
        }];
    } else {
        let rendered_series = render_sql(series_sql, &fragment);
        result.series = load_series(connection, &rendered_series, &totals_params, 1)?;
    }
    let (rendered_rank, rank_prefix) = if query.grouping == DimensionKind::Category {
        (
            render_rank_sql(
                RANK_DAILY_CATEGORY,
                &fragment,
                &query.sort,
                RankLayer::Dimension,
            ),
            vec![Value::from(start), Value::from(end)],
        )
    } else {
        (
            render_rank_sql(RANK_DAILY_DIM, &fragment, &query.sort, RankLayer::Dimension),
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
    fill_coverage_raw(connection, query, result)?;
    result.named_sql = vec![
        if identity_filter(query) {
            "totals_daily_dimension"
        } else {
            "totals_daily_core_filtered"
        }
        .into(),
        if query.granularity == Granularity::Hour {
            "series_hourly_dimension"
        } else if identity_filter(query) {
            "totals_daily_dimension"
        } else {
            "totals_daily_core_filtered"
        }
        .into(),
        if query.grouping == DimensionKind::Category {
            "rank_daily_category"
        } else {
            "rank_daily_dimension"
        }
        .into(),
        "coverage_raw".into(),
    ];
    Ok(())
}

fn load_core_totals(
    connection: &Connection,
    query: &ReportQuery,
) -> Result<(i64, i64, i64, i64), ReportError> {
    let (filter, values) = core_filter(query.filters.category.as_deref());
    let sql = render_sql(TOTALS_DAILY_CORE_FILTERED, &filter);
    let params = merge_sql_params(
        [
            Value::from(query.range_start_utc),
            Value::from(query.range_end_utc),
        ],
        &values,
        [],
    );
    load_totals(connection, &sql, &params)
}

fn fill_core(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
    start: i64,
    _end: i64,
) -> Result<(), ReportError> {
    let (upload, download, count, duration) = load_core_totals(connection, query)?;
    result.totals = ReportTotals {
        upload,
        download,
        connection_count: count,
        active_duration_sec: duration,
        previous_upload: None,
        previous_download: None,
    };
    result.series = vec![SeriesPoint {
        bucket_utc: start,
        upload,
        download,
        connection_count: count,
        active_duration_sec: duration,
    }];
    result.named_sql = vec!["totals_daily_core_filtered".into(), "coverage_daily".into()];
    result.rankings.clear();
    fill_coverage_daily(connection, query, result)?;
    Ok(())
}

fn fill_coverage_raw(
    connection: &Connection,
    query: &ReportQuery,
    result: &mut ReportResult,
) -> Result<(), ReportError> {
    result.coverage.slices =
        read_raw_coverage(connection, query.range_start_utc, query.range_end_utc)?;
    summarize_coverage(query, result);
    Ok(())
}

fn read_raw_coverage(
    connection: &Connection,
    start: i64,
    end: i64,
) -> Result<Vec<CoverageSlice>, ReportError> {
    let mut statement = connection.prepare(COVERAGE_RAW).map_err(map_sqlite)?;
    let rows = statement
        .query_map(params![start, end], |row| {
            Ok(CoverageSlice {
                kind: row.get(0)?,
                reason: row.get(1)?,
                started_utc: row.get(2)?,
                ended_utc: row.get(3)?,
            })
        })
        .map_err(map_sqlite)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(map_sqlite)
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
            // 日汇总没有 gap 的精确位置，不能伪造整日缺口。
            kind: "daily-summary".into(),
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
    let (covered, gap) = coverage_union_secs(
        query.range_start_utc,
        query.range_end_utc,
        &result.coverage.slices,
    );
    result.coverage.gap_sec = gap;
    result.coverage.covered_sec = covered;
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

#[cfg(test)]
fn load_raw_series(
    connection: &Connection,
    granularity: Granularity,
    start_min: i64,
    end_min: i64,
    fragment: &str,
    filter_params: &[String],
) -> Result<Vec<SeriesPoint>, ReportError> {
    let sql = render_sql(SERIES_RAW, fragment);
    let mut statement = connection.prepare(&sql).map_err(map_sqlite)?;
    let mut values = merge_sql_params(
        [Value::from(0), Value::from(start_min), Value::from(end_min)],
        filter_params,
        [],
    );
    let width = granularity.bucket_minutes();
    let mut cursor = start_min;
    let mut points = Vec::new();
    while cursor < end_min {
        // 与 SQLite 的整数除法一致：负数向零截断，零桶横跨负正分钟。
        let bucket = (cursor / width) * width;
        let upper = if bucket < 0 {
            bucket + 1
        } else {
            bucket.saturating_add(width)
        }
        .min(end_min);
        values[0] = Value::from(bucket);
        values[1] = Value::from(cursor);
        values[2] = Value::from(upper);
        let point = statement
            .query_row(params_from_iter(values.iter()), |row| {
                Ok(SeriesPoint {
                    bucket_utc: row.get::<_, i64>(0)? * 60,
                    upload: row.get(1)?,
                    download: row.get(2)?,
                    connection_count: row.get(3)?,
                    active_duration_sec: row.get(4)?,
                })
            })
            .optional()
            .map_err(map_sqlite)?;
        if let Some(point) = point {
            points.push(point);
        }
        cursor = upper;
    }
    Ok(points)
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
                primary_exit: None,
                exit_mixed: false,
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
mod raw_stage_probe_tests {
    use super::*;
    use sha2::{Digest, Sha256};

    // This proof deliberately lives beside the ignored stage probe.  It is not a
    // production SQL template: the m-first statements remain the source of truth
    // and the session-first text is only used to compare plans/results on a copy.
    fn session_first_sql(sql: &str) -> String {
        let from = "from connection_minute m";
        let attr = "left join connection_session_attr a on a.session_pk = m.session_pk";
        assert_eq!(sql.matches(from).count(), 1, "stage SQL shape changed");
        assert_eq!(sql.matches(attr).count(), 1, "stage SQL shape changed");
        let start = sql.find(from).unwrap();
        let end = sql[start..].find(attr).unwrap() + start + attr.len();
        let mut rewritten = String::with_capacity(sql.len());
        rewritten.push_str(&sql[..start]);
        rewritten.push_str(
            "from connection_session s\nleft join connection_session_attr a on a.session_pk = s.session_pk\njoin connection_minute m on m.session_pk = s.session_pk",
        );
        rewritten.push_str(&sql[end..]);
        rewritten
    }

    fn measure_comparison_select(
        connection: &Connection,
        name: &str,
        sql: &str,
        values: &[Value],
    ) -> serde_json::Value {
        let plans = connection
            .prepare(&format!("explain query plan {sql}"))
            .unwrap()
            .query_map(params_from_iter(values.iter()), |row| {
                row.get::<_, String>(3)
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let started = Instant::now();
        attach_cancel(connection, &cancel, started, Duration::from_secs(60)).unwrap();
        let mut count = 0_u64;
        let mut digest = Sha256::new();
        let execution = (|| -> rusqlite::Result<()> {
            let mut statement = connection.prepare(sql)?;
            let columns = statement.column_count();
            let mut rows = statement.query(params_from_iter(values.iter()))?;
            while let Some(row) = rows.next()? {
                for column in 0..columns {
                    let value: Value = row.get(column)?;
                    digest.update(format!("{value:?}\n").as_bytes());
                }
                count += 1;
            }
            Ok(())
        })();
        let wall_ms = started.elapsed().as_secs_f64() * 1000.0;
        connection
            .progress_handler(0, None::<fn() -> bool>)
            .unwrap();
        let status = match &execution {
            Ok(()) => "ok",
            Err(error)
                if error.sqlite_error_code() == Some(rusqlite::ErrorCode::OperationInterrupted) =>
            {
                "deadline"
            }
            Err(_) => "error",
        };
        serde_json::json!({
            "name": name,
            "sql_sha256": hex::encode(Sha256::digest(sql.as_bytes())),
            "plan": plans,
            "status": status,
            "wall_ms": wall_ms,
            "row_count": count,
            "result_digest": if execution.is_ok() { Some(hex::encode(digest.finalize())) } else { None::<String> },
            "error": execution.err().map(|error| error.to_string()),
            "deadline_ms": 60000,
        })
    }

    fn measure_select(
        connection: &Connection,
        name: &str,
        sql: &str,
        values: &[Value],
    ) -> (serde_json::Value, Vec<String>) {
        let plans = connection
            .prepare(&format!("explain query plan {sql}"))
            .unwrap()
            .query_map(params_from_iter(values.iter()), |row| {
                row.get::<_, String>(3)
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let started = Instant::now();
        attach_cancel(connection, &cancel, started, Duration::from_secs(60)).unwrap();
        let mut count = 0_u64;
        let mut digest = Sha256::new();
        let mut identities = Vec::new();
        let execution = (|| -> rusqlite::Result<()> {
            let mut statement = connection.prepare(sql)?;
            let columns = statement.column_count();
            let mut rows = statement.query(params_from_iter(values.iter()))?;
            while let Some(row) = rows.next()? {
                for column in 0..columns {
                    let value: Value = row.get(column)?;
                    digest.update(format!("{value:?}\n").as_bytes());
                }
                if name == "rank_raw" {
                    identities.push(row.get::<_, String>(0)?);
                }
                count += 1;
            }
            Ok(())
        })();
        let wall_ms = started.elapsed().as_secs_f64() * 1000.0;
        connection
            .progress_handler(0, None::<fn() -> bool>)
            .unwrap();
        let status = match &execution {
            Ok(()) => "ok",
            Err(error)
                if error.sqlite_error_code() == Some(rusqlite::ErrorCode::OperationInterrupted) =>
            {
                "deadline"
            }
            Err(_) => "error",
        };
        println!("SQL阶段 {name}: {status} {wall_ms:.3}ms，输出 {count} 行");
        (
            serde_json::json!({"name":name,"sql_sha256":hex::encode(Sha256::digest(sql.as_bytes())),
            "plan":plans,"status":status,"wall_ms":wall_ms,"row_count":count,
            "result_digest":if execution.is_ok(){Some(hex::encode(digest.finalize()))}else{None},
            "error":execution.err().map(|error|error.to_string()),"deadline_ms":60000}),
            identities,
        )
    }

    fn measure_bucket_series(
        connection: &Connection,
        start_min: i64,
        end_min: i64,
        fragment: &str,
        filter_values: &[String],
    ) -> serde_json::Value {
        let sql = render_sql(SERIES_RAW, fragment);
        let values = merge_sql_params(
            [
                Value::from(start_min),
                Value::from(start_min),
                Value::from(start_min + 60),
            ],
            filter_values,
            [],
        );
        let plans = connection
            .prepare(&format!("explain query plan {sql}"))
            .unwrap()
            .query_map(params_from_iter(values.iter()), |row| {
                row.get::<_, String>(3)
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let started = Instant::now();
        attach_cancel(connection, &cancel, started, Duration::from_secs(60)).unwrap();
        let result = load_raw_series(
            connection,
            Granularity::Hour,
            start_min,
            end_min,
            fragment,
            filter_values,
        );
        let wall_ms = started.elapsed().as_secs_f64() * 1000.0;
        connection
            .progress_handler(0, None::<fn() -> bool>)
            .unwrap();
        let mut digest = Sha256::new();
        if let Ok(points) = &result {
            for point in points {
                for value in [
                    point.bucket_utc / 60,
                    point.upload,
                    point.download,
                    point.connection_count,
                    point.active_duration_sec,
                ] {
                    digest.update(format!("{:?}\n", Value::from(value)).as_bytes());
                }
            }
        }
        let status = match &result {
            Ok(_) => "ok",
            Err(ReportError::Cancelled(_)) => "deadline",
            Err(_) => "error",
        };
        println!("SQL阶段 series_raw_bucket: {status} {wall_ms:.3}ms");
        serde_json::json!({"name":"series_raw_bucket","sql_sha256":hex::encode(Sha256::digest(sql.as_bytes())),
            "plan":plans,"status":status,"wall_ms":wall_ms,"row_count":result.as_ref().ok().map(Vec::len),
            "result_digest":if result.is_ok(){Some(hex::encode(digest.finalize()))}else{None},
            "error":result.err().map(|error|format!("{error:?}")),"deadline_ms":60000})
    }

    #[test]
    #[ignore = "仅显式只读的隔离生成库；逐阶段最多60秒，不输出明细"]
    fn isolated_raw_stage_probe() {
        let db = std::path::PathBuf::from(
            std::env::var_os("RESIWATCH_SQL_STAGE_DB").expect("隔离库路径"),
        );
        let output = std::path::PathBuf::from(
            std::env::var_os("RESIWATCH_SQL_STAGE_OUT").expect("研究结果路径"),
        );
        let start: i64 = std::env::var("RESIWATCH_SQL_STAGE_START")
            .unwrap()
            .parse()
            .unwrap();
        let end: i64 = std::env::var("RESIWATCH_SQL_STAGE_END")
            .unwrap()
            .parse()
            .unwrap();
        let manifest: serde_json::Value = serde_json::from_slice(
            &std::fs::read(db.parent().unwrap().join("production-corpus.json"))
                .expect("生成库标记"),
        )
        .unwrap();
        assert_eq!(manifest["kind"], "production-corpus");
        assert_eq!(start, manifest["start_utc"].as_i64().unwrap() + 86400);
        assert_eq!(end, manifest["end_utc"].as_i64().unwrap());
        let connection = open_interruptible_reader(&db).unwrap();
        connection.execute_batch("begin deferred").unwrap();
        let filters = crate::c3::query::ReportFilters {
            category: Some(RESIDENTIAL_ACCOUNTING_FILTER.into()),
            ..Default::default()
        };
        let (fragment, filter_values) = filter_clause(&filters);
        let start_min = start.div_euclid(60);
        let end_min = end.div_euclid(60);
        let sort = crate::c3::query::SortSpec::default();
        let series_only = std::env::var("RESIWATCH_SQL_STAGE_SERIES_ONLY").as_deref() == Ok("1");
        let version: String = connection
            .query_row("select sqlite_version()", [], |r| r.get(0))
            .unwrap();
        let mut report = serde_json::json!({"kind":"bundled-sqlite-readonly-stage-probe","db":db,"sqlite_version":version,
            "start_utc":start,"end_utc":end,"days":29,"query_only":connection.query_row("pragma query_only",[],|r|r.get::<_,i64>(0)).unwrap(),
            "schema_version":connection.query_row("pragma user_version",[],|r|r.get::<_,i64>(0)).unwrap(),
            "open_flags":"SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_FULL_MUTEX", "series_only":series_only,
            "source_identity":std::env::var("RESIWATCH_SQL_STAGE_SOURCE").ok(),"stages":[],
            "limits":["每个阶段单独60秒诊断上限，不替代生产10秒报告预算。","同一只读快照，未清除OS缓存；只输出计划、行数与摘要，不输出明细。","仅最后29个完整raw日，不是完整30日验收。"]});
        let mut identities = Vec::new();
        for (name, sql, values) in [
            (
                "totals_raw_attribution",
                render_raw_totals_sql(&fragment, DimensionKind::Host),
                merge_sql_params(
                    [Value::from(start_min), Value::from(end_min)],
                    &filter_values,
                    [],
                ),
            ),
            (
                "series_raw",
                render_sql(SERIES_RAW_GROUPED, &fragment),
                merge_sql_params(
                    [
                        Value::from(60),
                        Value::from(60),
                        Value::from(start_min),
                        Value::from(end_min),
                    ],
                    &filter_values,
                    [],
                ),
            ),
            (
                "rank_raw",
                render_rank_sql(RANK_RAW, &fragment, &sort, RankLayer::Raw),
                merge_sql_params(
                    [Value::from(start_min), Value::from(end_min)],
                    &filter_values,
                    [Value::from(20)],
                ),
            ),
        ] {
            if series_only && name != "series_raw" {
                continue;
            }
            let (stage, ids) = measure_select(&connection, name, &sql, &values);
            if name == "rank_raw" && stage["status"] == "ok" {
                identities = ids;
            }
            report["stages"].as_array_mut().unwrap().push(stage);
            if name == "series_raw" {
                report["stages"]
                    .as_array_mut()
                    .unwrap()
                    .push(measure_bucket_series(
                        &connection,
                        start_min,
                        end_min,
                        &fragment,
                        &filter_values,
                    ));
            }
            std::fs::write(&output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        }
        if !identities.is_empty() {
            let sql = render_exit_sql(
                raw_exit_sql(DimensionKind::Host),
                &fragment,
                identities.len(),
            );
            let values = merge_sql_params(
                [Value::from(start_min), Value::from(end_min)],
                &filter_values,
                identities.into_iter().map(Value::Text),
            );
            let (stage, _) = measure_select(&connection, "rank_raw_exits", &sql, &values);
            report["stages"].as_array_mut().unwrap().push(stage);
        } else if !series_only {
            report["stages"].as_array_mut().unwrap().push(serde_json::json!({"name":"rank_raw_exits","status":"skipped","reason":"Top-N阶段未成功产出身份"}));
        }
        connection.execute_batch("commit").unwrap();
        assert!(connection.is_autocommit());
        std::fs::write(output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }

    #[test]
    #[ignore = "仅显式只读的隔离生成库；比较 m-first 与 session-first 的计划、摘要和阶段耗时"]
    fn isolated_session_first_stage_proof() {
        let db = std::path::PathBuf::from(
            std::env::var_os("RESIWATCH_SESSION_FIRST_STAGE_DB").expect("隔离库路径"),
        );
        let output = std::path::PathBuf::from(
            std::env::var_os("RESIWATCH_SESSION_FIRST_STAGE_OUT").expect("研究结果路径"),
        );
        let manifest: serde_json::Value = serde_json::from_slice(
            &std::fs::read(db.parent().unwrap().join("production-corpus.json"))
                .expect("生成库标记"),
        )
        .unwrap();
        assert_eq!(manifest["kind"], "production-corpus");
        let start = manifest["start_utc"].as_i64().unwrap();
        let end = manifest["end_utc"].as_i64().unwrap();
        let retained_start = start + 86_400;
        assert!(retained_start < end);
        let connection = open_interruptible_reader(&db).unwrap();
        connection.execute_batch("begin deferred").unwrap();
        let filters = crate::c3::query::ReportFilters {
            category: Some(RESIDENTIAL_ACCOUNTING_FILTER.into()),
            ..Default::default()
        };
        let (fragment, filter_values) = filter_clause(&filters);
        let sort = crate::c3::query::SortSpec::default();
        let windows = [
            ("broad_29d", retained_start, end),
            ("one_day", end - 86_400, end),
            ("sparse_short_1h", retained_start, retained_start + 3_600),
        ];
        let force_cross =
            std::env::var("RESIWATCH_SESSION_FIRST_FORCE_CROSS").as_deref() == Ok("1");
        let sqlite_version: String = connection
            .query_row("select sqlite_version()", [], |row| row.get(0))
            .unwrap();
        let mut report = serde_json::json!({
            "kind": "bundled-sqlite-session-first-stage-proof",
            "db": db,
            "sqlite_version": sqlite_version,
            "schema_version": connection.query_row("pragma user_version", [], |row| row.get::<_, i64>(0)).unwrap(),
            "open_flags": "SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_FULL_MUTEX",
            "query_only": connection.query_row("pragma query_only", [], |row| row.get::<_, i64>(0)).unwrap(),
            "source_identity": std::env::var("RESIWATCH_SESSION_FIRST_STAGE_SOURCE").ok(),
            "force_cross": force_cross,
            "windows": windows.iter().map(|(name, start_utc, end_utc)| serde_json::json!({"name":name,"start_utc":start_utc,"end_utc":end_utc})).collect::<Vec<_>>(),
            "pairs": [],
            "limits": [
                "每个 SQL 阶段单独 60 秒诊断上限；不改变生产 10 秒报告预算。",
                "同一只读快照，不清除 OS 缓存；只记录计划、摘要、行数和耗时。",
                "session-first 仅为候选证明，不写入生产 SQL 模板。"
            ]
        });
        for (window_name, start_utc, end_utc) in windows {
            let start_min = start_utc.div_euclid(60);
            let end_min = end_utc.div_euclid(60);
            let current_queries = [
                (
                    "totals_raw_attribution",
                    render_raw_totals_sql(&fragment, DimensionKind::Host),
                    merge_sql_params(
                        [Value::from(start_min), Value::from(end_min)],
                        &filter_values,
                        [],
                    ),
                ),
                (
                    "rank_raw_host",
                    render_rank_sql(RANK_RAW, &fragment, &sort, RankLayer::Raw),
                    merge_sql_params(
                        [Value::from(start_min), Value::from(end_min)],
                        &filter_values,
                        [Value::from(20)],
                    ),
                ),
            ];
            for (query_name, current_sql, values) in current_queries {
                let candidate_sql = session_first_sql(&current_sql);
                let current =
                    measure_comparison_select(&connection, query_name, &current_sql, &values);
                let candidate = measure_comparison_select(
                    &connection,
                    "session_first",
                    &candidate_sql,
                    &values,
                );
                let equivalent = current["status"] == "ok"
                    && candidate["status"] == "ok"
                    && current["row_count"] == candidate["row_count"]
                    && current["result_digest"] == candidate["result_digest"];
                report["pairs"]
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::json!({
                        "window": window_name,
                        "query": query_name,
                        "m_first": current,
                        "session_first": candidate,
                        "equivalent": equivalent,
                    }));
                std::fs::write(&output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
            }
        }
        connection.execute_batch("commit").unwrap();
        assert!(connection.is_autocommit());
        std::fs::write(output, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        let pairs = report["pairs"]
            .as_array()
            .expect("阶段证明必须输出 pairs 数组");
        assert_eq!(pairs.len(), windows.len() * 2);
        for pair in pairs {
            assert!(pair["window"].as_str().is_some());
            assert!(pair["query"].as_str().is_some());
            assert!(pair["equivalent"].is_boolean());
            for side in ["m_first", "session_first"] {
                assert!(
                    pair[side]["status"].as_str().is_some(),
                    "{side} 阶段必须记录 status"
                );
            }
        }
    }

    #[test]
    #[ignore = "只读隔离库；记录非空分钟窗口的生产报告阶段，不改变生产 deadline"]
    fn isolated_nonempty_report_stage_proof() {
        let db = std::path::PathBuf::from(
            std::env::var_os("RESIWATCH_NONEMPTY_STAGE_DB").expect("隔离库"),
        );
        let output = std::path::PathBuf::from(
            std::env::var_os("RESIWATCH_NONEMPTY_STAGE_OUT").expect("输出"),
        );
        assert!(!output.exists(), "证据已存在");
        let start_utc: i64 = std::env::var("RESIWATCH_NONEMPTY_STAGE_START")
            .unwrap()
            .parse()
            .unwrap();
        let end_utc: i64 = std::env::var("RESIWATCH_NONEMPTY_STAGE_END")
            .unwrap()
            .parse()
            .unwrap();
        assert!(end_utc > start_utc, "窗口为空");
        let now_utc = end_utc;
        let query = crate::c3::query::default_auto_report_query(
            crate::c3::query::Granularity::Hour,
            start_utc,
            end_utc,
        );
        let cancel = Arc::new(AtomicBool::new(false));
        let first_started = Instant::now();
        let first = run_uncached(&db, query.clone(), now_utc, 30, &cancel, None);
        let first_ms = first_started.elapsed().as_secs_f64() * 1000.0;
        let first_summary = match &first {
            Ok(result) => serde_json::json!({
                "status": "ok",
                "tier": format!("{:?}", result.data_tier),
                "upload": result.totals.upload,
                "download": result.totals.download,
                "connection_count": result.totals.connection_count,
                "series_rows": result.series.len(),
                "data_version": result.data_version,
            }),
            Err(error) => serde_json::json!({
                "status": "error",
                "error": format!("{error:?}"),
            }),
        };

        let connection = open_interruptible_reader(&db).expect("reader");
        connection
            .execute_batch("begin deferred")
            .expect("snapshot");
        let first_day = start_utc.div_euclid(86_400) * 86_400;
        let day_end = (end_utc - 1).div_euclid(86_400) * 86_400 + 86_400;
        let finalized_started = Instant::now();
        let (verified_days, deleted_days) =
            finalized_days(&connection, first_day, day_end).expect("finalized days");
        let finalized_days_ms = finalized_started.elapsed().as_secs_f64() * 1000.0;
        let plan_started = Instant::now();
        let plan = plan_snapshot_capability(&connection, &query, now_utc, 30).expect("plan");
        let plan_snapshot_ms = plan_started.elapsed().as_secs_f64() * 1000.0;
        let version_started = Instant::now();
        let data_version =
            crate::storage::durable_data_version(&connection).expect("durable version");
        let durable_version_ms = version_started.elapsed().as_secs_f64() * 1000.0;
        let mut result = crate::c3::query::empty_result(query.clone(), &plan, data_version);
        let raw_started = Instant::now();
        fill_raw(
            &connection,
            &query,
            &mut result,
            start_utc.div_euclid(60),
            end_utc.div_euclid(60),
        )
        .expect("raw query");
        let raw_query_ms = raw_started.elapsed().as_secs_f64() * 1000.0;
        let close_started = Instant::now();
        connection.execute_batch("commit").expect("close snapshot");
        drop(connection);
        let reader_close_ms = close_started.elapsed().as_secs_f64() * 1000.0;
        assert!(
            result.totals.connection_count > 0,
            "分钟窗口没有连接，不能当作非空证据"
        );
        let report = serde_json::json!({
            "kind": "nonempty-minute-report-stages",
            "db": db,
            "range_start_utc": start_utc,
            "range_end_utc": end_utc,
            "minute_span": end_utc.div_euclid(60) - start_utc.div_euclid(60),
            "now_utc": now_utc,
            "raw_retain_days": 30,
            "query": "default_auto_report_query",
            "os_cache": "not dropped; first run_uncached is the in-process first read",
            "production_first_read_ms": first_ms,
            "production_first_read": first_summary,
            "stage_split_cache": "second connection after the production read",
            "finalized_days_ms": finalized_days_ms,
            "verified_days": verified_days,
            "deleted_days": deleted_days,
            "plan_snapshot_ms": plan_snapshot_ms,
            "plan_snapshot_includes_finalized_days": true,
            "plan_tier": format!("{:?}", plan.tier),
            "durable_version_ms": durable_version_ms,
            "data_version": data_version,
            "raw_query_ms": raw_query_ms,
            "reader_close_ms": reader_close_ms,
            "stage_connection_count": result.totals.connection_count,
            "stage_upload": result.totals.upload,
            "stage_download": result.totals.download,
            "stage_series_rows": result.series.len(),
            "limits": [
                "不驱逐 OS page cache，不能称物理冷读。",
                "plan snapshot 内部会再查一次 finalized days，不能把两次耗时相加当生产路径。",
                "生产 deadline 未放宽；本探针不写安装库。"
            ],
        });
        std::fs::write(&output, serde_json::to_vec_pretty(&report).expect("json")).expect("write");
    }
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
    fn report_and_period_usage_read_latest_receipt_version_without_floor_rewrite() {
        let (_dir, mut coordinator, mut store) = setup();
        coordinator
            .commit(&crate::storage::CommitBundle {
                writer_epoch: 1,
                bundle_seq: 1,
                payload: "report-version".into(),
            })
            .unwrap();
        let query = ReportQuery {
            range_start_utc: 1000,
            range_end_utc: 4000,
            ..Default::default()
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let result = ReportService::run(
            coordinator.path(),
            &mut store,
            query.clone(),
            4000,
            30,
            &cancel,
            None,
        )
        .unwrap();
        let usage = build_period_usage(coordinator.connection(), &query, 4000).unwrap();
        assert_eq!(result.data_version, 1);
        assert_eq!(usage.data_version, 1);
        assert_eq!(
            coordinator
                .connection()
                .query_row("select watermark from data_version where id=1", [], |r| r
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            0
        );
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
    fn incomplete_day_backlog_reads_retained_raw_but_mixed_deleted_range_fails_closed() {
        let (_dir, mut coordinator, _) = setup();
        coordinator
            .connection()
            .execute_batch(
                "insert into connection_minute(utc_minute, session_pk, upload, download)
             values (1450, 1, 7, 11);",
            )
            .expect("second day");
        finalize_fixture_day(&mut coordinator, 0);
        let query = ReportQuery {
            range_start_utc: 0,
            range_end_utc: 2 * 86_400,
            granularity: crate::c3::query::Granularity::Day,
            comparison: None,
            ..ReportQuery::default()
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let report = run_uncached(
            coordinator.path(),
            query.clone(),
            40 * 86_400,
            30,
            &cancel,
            None,
        )
        .expect("retained raw fallback");
        assert_eq!(report.data_tier, DataTier::Raw);
        assert_eq!((report.totals.upload, report.totals.download), (37, 101));
        assert!(!report.drilldown_capability.sessions);
        // 构造已校验且已清理第一天的读取状态；第二天仍是积压。
        coordinator.connection().execute_batch(
            "delete from connection_minute where utc_minute < 1440;
             update retention_state set status='deleted' where layer='day_exact_v1' and chunk_utc=0;",
        ).expect("post-delete reader fixture");
        let error = run_uncached(coordinator.path(), query, 40 * 86_400, 30, &cancel, None)
            .expect_err("no partial aggregate result");
        assert_eq!(error.code(), "capability_unsupported");
    }

    #[test]
    fn postdelete_daily_core_preserves_gap_duration_without_fabricated_gap_span() {
        let (_dir, mut coordinator, _) = setup();
        finalize_fixture_day(&mut coordinator, 0);
        coordinator.connection().execute_batch(
            "delete from connection_minute;
             delete from coverage_interval;
             update retention_state set status='deleted' where layer='day_exact_v1' and chunk_utc=0;",
        ).expect("post-delete reader fixture");
        let query = ReportQuery {
            range_start_utc: 0,
            range_end_utc: 86_400,
            grouping: DimensionKind::Category,
            granularity: crate::c3::query::Granularity::Day,
            comparison: None,
            ..ReportQuery::default()
        };
        let report = run_uncached(
            coordinator.path(),
            query,
            500 * 86_400,
            30,
            &Arc::new(AtomicBool::new(false)),
            None,
        )
        .expect("core report");
        assert_eq!(report.data_tier, DataTier::DailyCore);
        assert_eq!(
            (report.coverage.covered_sec, report.coverage.gap_sec),
            (1_500, 300)
        );
        assert_eq!(report.coverage.status, "partial");
        assert!(report
            .coverage
            .slices
            .iter()
            .all(|slice| slice.kind != "gap"));
        assert_eq!(report.totals.download, 90);
    }

    fn deleted_fixture_day(coordinator: &mut StorageCoordinator) {
        finalize_fixture_day(coordinator, 0);
        coordinator.connection().execute_batch("delete from connection_minute where utc_minute<1440;
            update retention_state set status='deleted' where layer='day_exact_v1' and chunk_utc=0;").expect("已清理状态");
    }

    fn day_query() -> ReportQuery {
        ReportQuery {
            range_start_utc: 0,
            range_end_utc: 86_400,
            granularity: Granularity::Day,
            ..ReportQuery::default()
        }
    }

    #[test]
    fn postdelete_daily_totals_and_category_filter_use_core_without_double_counting() {
        let (_dir, mut coordinator, _) = setup();
        coordinator
            .connection()
            .execute_batch(
                "insert into dimension_dict values('category',2,'另一分类');
            update connection_session_attr set primary_category_id=2 where session_pk=2;",
            )
            .expect("分类");
        deleted_fixture_day(&mut coordinator);
        let cancel = Arc::new(AtomicBool::new(false));
        let mut query = day_query();
        // 增大名义 raw 期限仍必须读取实际汇总。
        let all = run_uncached(
            coordinator.path(),
            query.clone(),
            40 * 86_400,
            90,
            &cancel,
            None,
        )
        .expect("精确日");
        assert_eq!(all.data_tier, DataTier::DailyDimension);
        assert_eq!(
            (
                all.totals.upload,
                all.totals.download,
                all.totals.connection_count,
                all.totals.active_duration_sec
            ),
            (30, 90, 2, 120)
        );
        assert_eq!(
            all.series.iter().map(|p| p.download).sum::<i64>(),
            all.totals.download
        );
        query.grouping = DimensionKind::Category;
        query.filters.category = Some("家宽".into());
        let selected = run_uncached(coordinator.path(), query, 500 * 86_400, 30, &cancel, None)
            .expect("长期分类");
        assert_eq!(selected.data_tier, DataTier::DailyCore);
        assert_eq!(
            (
                selected.totals.upload,
                selected.totals.download,
                selected.totals.connection_count,
                selected.totals.active_duration_sec
            ),
            (10, 30, 1, 60)
        );
        assert_eq!(selected.series.iter().map(|p| p.download).sum::<i64>(), 30);
    }

    #[test]
    fn postdelete_distinct_daily_count_survives_hour_series_and_shared_minutes() {
        let (_dir, mut coordinator, _) = setup();
        coordinator
            .connection()
            .execute_batch(
                "delete from connection_minute; delete from traffic_hourly_dimension;
            insert into connection_minute values(20,1,10,30),(80,1,7,11);",
            )
            .expect("跨小时同连接");
        deleted_fixture_day(&mut coordinator);
        let cancel = Arc::new(AtomicBool::new(false));
        let mut query = day_query();
        query.granularity = Granularity::Hour;
        let report = run_uncached(coordinator.path(), query, 40 * 86_400, 30, &cancel, None)
            .expect("可恢复小时序列");
        assert_eq!(
            (
                report.totals.connection_count,
                report.totals.active_duration_sec
            ),
            (1, 120)
        );
        assert_eq!(report.rankings[0].connection_count, 1);
        assert_eq!(
            report
                .series
                .iter()
                .map(|p| p.connection_count)
                .sum::<i64>(),
            2
        );
        assert_eq!(report.series.iter().map(|p| p.download).sum::<i64>(), 41);
        let (_dir, mut coordinator, _) = setup();
        coordinator
            .connection()
            .execute_batch(
                "delete from connection_minute; delete from traffic_hourly_dimension;
            insert into connection_minute values(20,1,10,30),(20,2,20,60);",
            )
            .expect("同分钟不同 host");
        deleted_fixture_day(&mut coordinator);
        let daily = run_uncached(
            coordinator.path(),
            day_query(),
            40 * 86_400,
            30,
            &cancel,
            None,
        )
        .expect("日总时长");
        assert_eq!(
            (
                daily.totals.connection_count,
                daily.totals.active_duration_sec
            ),
            (2, 60)
        );
        assert_eq!(daily.series[0].active_duration_sec, 60);
        let mut hourly = day_query();
        hourly.granularity = Granularity::Hour;
        assert_eq!(
            run_uncached(coordinator.path(), hourly, 40 * 86_400, 30, &cancel, None)
                .expect_err("小时时长无法去重")
                .code(),
            "capability_unsupported"
        );
    }

    #[test]
    fn deleted_previous_window_is_unknown_and_raw_only_capability_stays_closed() {
        let (_dir, mut coordinator, _) = setup();
        deleted_fixture_day(&mut coordinator);
        coordinator
            .connection()
            .execute("insert into connection_minute values(1450,1,7,11)", [])
            .expect("保留日");
        let cancel = Arc::new(AtomicBool::new(false));
        let mut query = day_query();
        query.range_start_utc = 86_400;
        query.range_end_utc = 2 * 86_400;
        query.comparison = Some(ComparisonSpec {
            previous_equal_window: true,
        });
        let report = run_uncached(coordinator.path(), query, 40 * 86_400, 90, &cancel, None)
            .expect("当前 raw");
        assert_eq!(
            (
                report.totals.previous_upload,
                report.totals.previous_download
            ),
            (None, None)
        );
        assert_eq!(report.totals.download, 11);
        let mut sessions = day_query();
        sessions.include_sessions = true;
        assert_eq!(
            run_uncached(coordinator.path(), sessions, 40 * 86_400, 90, &cancel, None)
                .expect_err("旧 raw 不能恢复")
                .code(),
            "capability_unsupported"
        );
        let mut multiple = day_query();
        multiple.range_end_utc = 2 * 86_400;
        assert_eq!(
            run_uncached(coordinator.path(), multiple, 40 * 86_400, 90, &cancel, None)
                .expect_err("跨日 distinct 未保留")
                .code(),
            "capability_unsupported"
        );
    }

    #[test]
    fn internal_usage_combines_deleted_and_raw_days_without_distinct_claims() {
        let (dir, mut coordinator, _) = setup();
        coordinator.connection().execute_batch("delete from coverage_interval;
            insert into coverage_interval(kind,reason,started_utc,ended_utc) values('covered','running',0,172800);").expect("完整覆盖");
        deleted_fixture_day(&mut coordinator);
        coordinator
            .connection()
            .execute("insert into connection_minute values(1450,1,7,11)", [])
            .expect("第二日");
        let mut query = day_query();
        query.range_end_utc = 2 * 86_400;
        query.granularity = Granularity::Month;
        query.filters.host = Some("a.example".into());
        let usage = query_period_usage(
            coordinator.path(),
            &query,
            40 * 86_400,
            30,
            &Arc::new(AtomicBool::new(false)),
        )
        .expect("可加字节");
        assert_eq!(
            (
                usage.upload,
                usage.download,
                usage.covered_sec,
                usage.gap_sec
            ),
            (17, 41, 172800, 0)
        );
        assert_eq!(
            std::fs::read_dir(dir.path().join("report-spool"))
                .expect("spool目录")
                .count(),
            0
        );
    }

    fn finalize_fixture_day(coordinator: &mut StorageCoordinator, day: i64) {
        for _ in 0..100 {
            crate::c3::retention::RetentionService::run_chunk(
                coordinator,
                40 * 86_400,
                30,
                crate::c3::retention::RetentionMode::MaterializeOnly,
                &crate::c3::SpaceBudget::unlimited(),
                &Arc::new(AtomicBool::new(false)),
            )
            .expect("bounded fixture step");
            let ready: bool = coordinator.connection().query_row(
                "select exists(select 1 from retention_state where layer='day_exact_v1' and chunk_utc=?1 and status='verified')",
                [day], |row| row.get(0),
            ).expect("confirmation");
            if ready {
                return;
            }
        }
        panic!("fixture day did not finish within 100 bounded steps");
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
    fn query_deadline_is_distinct_from_user_cancellation() {
        let (_dir, coordinator, _) = setup();
        let error = run_uncached(
            coordinator.path(),
            ReportQuery::default(),
            3_600,
            30,
            &Arc::new(AtomicBool::new(false)),
            Some(Duration::ZERO),
        )
        .expect_err("deadline");
        assert!(matches!(error, ReportError::DeadlineExceeded(_)));
        assert!(coordinator.connection().is_autocommit());
    }

    #[test]
    fn progress_stride_interrupts_active_sql_on_cancel_and_deadline() {
        for user_cancel in [true, false] {
            let connection = Connection::open_in_memory().expect("reader");
            let cancel = Arc::new(AtomicBool::new(false));
            let calls = Arc::new(AtomicU64::new(0));
            let probe_calls = Arc::clone(&calls);
            let probe_cancel = Arc::clone(&cancel);
            connection
                .create_scalar_function(
                    "report_interrupt_probe",
                    1,
                    rusqlite::functions::FunctionFlags::SQLITE_UTF8,
                    move |ctx| {
                        if probe_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                            if user_cancel {
                                probe_cancel.store(true, Ordering::SeqCst);
                            } else {
                                // 在 SQL 已经运行后越过 deadline，不能只测入口检查。
                                std::thread::sleep(Duration::from_millis(10));
                            }
                        }
                        ctx.get::<i64>(0)
                    },
                )
                .expect("probe");
            attach_cancel(
                &connection,
                &cancel,
                Instant::now(),
                if user_cancel {
                    Duration::from_secs(60)
                } else {
                    Duration::from_millis(5)
                },
            )
            .expect("handler");
            let error = connection
                .query_row(
                    "with recursive work(n) as (values(1) union all select n+1 from work)
                 select sum(report_interrupt_probe(n)) from work",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .expect_err("运行中的 SQL 应被中断");
            assert_eq!(
                error.sqlite_error_code(),
                Some(rusqlite::ErrorCode::OperationInterrupted)
            );
            assert!((1..=1024).contains(&calls.load(Ordering::SeqCst)));
            assert_eq!(cancel.load(Ordering::SeqCst), user_cancel);
            connection
                .progress_handler(0, None::<fn() -> bool>)
                .expect("clear handler");
            assert!(connection.is_autocommit());
            assert_eq!(
                connection
                    .query_row("select 1", [], |row| row.get::<_, i64>(0))
                    .expect("reader reusable"),
                1
            );
        }
    }

    #[test]
    fn eqp_named_queries_have_no_temp_or_autoindex_on_fixture() {
        let (_dir, coordinator, _) = setup();
        let reader = open_interruptible_reader(coordinator.path()).expect("reader");
        for name in [
            "totals_raw",
            "totals_raw_attribution",
            "rank_raw",
            "raw_minute_scan",
            "raw_session_projection",
            "totals_hourly_dimension",
        ] {
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
    use crate::c3::query::{Granularity, SortField};
    use crate::c3::retention::{RetentionMode, RetentionService};
    use crate::c3::space::SpaceBudget;
    use crate::c3::sql::{RESIDENTIAL_ACCOUNTING_FILTER, UNKNOWN_IDENTITY};
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

    fn seed_residential_sort_fixture(connection: &Connection) {
        connection
            .execute_batch(
                "
                insert or ignore into dimension_dict(dimension_kind, dimension_id, value) values
                    ('host', 101, 'upload.example'),
                    ('host', 102, 'download.example'),
                    ('host', 103, 'outside.example'),
                    ('host', 104, '8.8.4.4'),
                    ('category', 101, '家宽测试');
                insert or ignore into connection_session(session_pk, epoch_id, connection_id, started_utc, host)
                values
                    (101, 1, 'upload-first', 600, 'upload.example'),
                    (102, 1, 'download-first', 600, 'download.example'),
                    (103, 1, 'outside', 600, 'outside.example'),
                    (104, 1, 'ip-fallback', 600, '8.8.4.4');
                insert or ignore into connection_session_attr(
                    session_pk, host_id, policy_version, primary_category_id, started_utc, ended_utc
                ) values
                    (101, 101, 1, 101, 600, null),
                    (102, 102, 1, 101, 600, null),
                    (103, 103, 1, null, 600, null),
                    (104, 104, 1, 101, 600, null);
                insert or ignore into connection_minute(utc_minute, session_pk, upload, download)
                values
                    (10, 101, 1000, 1),
                    (10, 102, 1, 1000),
                    (10, 103, 9000, 9000),
                    (10, 104, 40, 50);
                ",
            )
            .expect("residential sort fixture");
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
        // 同一组事实供 raw 与小时汇总对照，汇总窗口使用完整 UTC 小时。
        query.range_end_utc = 7_200;
        query
    }

    #[test]
    fn host_rank_uses_destination_ip_and_keeps_empty_unknown() {
        let (_dir, coordinator, mut store) = setup();
        coordinator
            .connection()
            .execute_batch(
                "
                insert or ignore into connection_session(session_pk, epoch_id, connection_id, started_utc, host)
                values (4, 1, 'ip-only', 700, '8.8.8.8'), (5, 1, 'empty', 800, null);
                insert or ignore into connection_minute(utc_minute, session_pk, upload, download)
                values (12, 4, 1, 40), (12, 5, 2, 80);
                ",
            )
            .expect("seed ip/unknown");
        let mut query = base_query();
        query.grouping = DimensionKind::Host;
        let result = run_now(&coordinator, &mut store, query, 3_600);
        let ids: Vec<_> = result
            .rankings
            .iter()
            .map(|row| row.identity.as_str())
            .collect();
        assert!(ids.contains(&"8.8.8.8"));
        assert!(ids.contains(&UNKNOWN_IDENTITY));
        let mut filtered = base_query();
        filtered.grouping = DimensionKind::Rule;
        filtered.filters.host = Some(UNKNOWN_IDENTITY.into());
        let unknown = run_now(&coordinator, &mut store, filtered, 3_600);
        assert!(unknown.totals.download >= 80);
    }

    #[test]
    fn named_sql_matches_grouping() {
        let (_dir, coordinator, mut store) = setup();
        let mut query = base_query();
        query.grouping = DimensionKind::Chain;
        let chain = run_now(&coordinator, &mut store, query, 3_600);
        assert!(chain.named_sql.iter().any(|name| name == "raw_minute_scan"));
        assert!(chain
            .named_sql
            .iter()
            .all(|name| !name.contains("exits") && name != "rank_raw"));
        let mut query = base_query();
        query.grouping = DimensionKind::Category;
        let category = run_now(&coordinator, &mut store, query, 3_600);
        assert!(category
            .named_sql
            .iter()
            .any(|name| name == "raw_session_projection"));
        assert_ne!(
            chain.rankings[0].identity, category.rankings[0].identity,
            "不同分组不能共用同一排名身份"
        );
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
    fn single_hop_direct_chain_keeps_raw_rule_identity() {
        let (_dir, coordinator, mut store) = setup();
        coordinator
            .connection()
            .execute_batch(
                "insert or ignore into dimension_dict(dimension_kind, dimension_id, value)
                 values ('rule', 2, 'IPCIDR(10.0.0.0/8)');
                 update connection_session_attr set rule_id = 2 where session_pk in (1, 2);",
            )
            .expect("seed IPCIDR");
        let mut query = base_query();
        query.grouping = DimensionKind::Rule;
        let result = run_now(&coordinator, &mut store, query, 3_600);
        let ipc = result
            .rankings
            .iter()
            .find(|row| row.identity == "IPCIDR(10.0.0.0/8)")
            .expect("raw rule fallback");
        assert_eq!((ipc.upload, ipc.download), (30, 90));
    }

    #[test]
    fn attribution_quality_is_exact_and_independent_of_top_n() {
        let (_dir, coordinator, mut store) = setup();
        coordinator
            .connection()
            .execute(
                "update connection_session_attr set process_id = null where session_pk = 2",
                [],
            )
            .expect("one process missing");
        let mut small_query = base_query();
        small_query.grouping = DimensionKind::Process;
        small_query.top_n = 1;
        let small = run_now(&coordinator, &mut store, small_query, 3_600);
        let mut large_query = base_query();
        large_query.grouping = DimensionKind::Process;
        large_query.top_n = 100;
        let large = run_now(&coordinator, &mut store, large_query, 3_600);
        assert_eq!(small.attribution_quality, large.attribution_quality);
        assert_eq!(
            small.attribution_quality.status,
            crate::c3::query::AttributionStatus::Partial
        );
        assert_eq!(
            (
                small.attribution_quality.known_upload,
                small.attribution_quality.known_download,
                small.attribution_quality.missing_upload,
                small.attribution_quality.missing_download,
                small.attribution_quality.known_connections,
                small.attribution_quality.missing_connections,
            ),
            (15, 45, 20, 60, 2, 1)
        );
        assert_eq!(
            small.attribution_quality.known_upload + small.attribution_quality.missing_upload,
            small.totals.upload
        );
        assert_eq!(
            small.attribution_quality.known_download + small.attribution_quality.missing_download,
            small.totals.download
        );

        coordinator
            .connection()
            .execute("update connection_session_attr set process_id = null", [])
            .expect("all process missing");
        let mut unavailable_query = base_query();
        unavailable_query.grouping = DimensionKind::Process;
        let unavailable = run_now(&coordinator, &mut store, unavailable_query, 3_600);
        assert_eq!(
            unavailable.attribution_quality.status,
            crate::c3::query::AttributionStatus::Unavailable
        );
        assert_eq!(unavailable.attribution_quality.known_upload, 0);
        assert_eq!(unavailable.attribution_quality.known_download, 0);
        assert_eq!(
            unavailable.attribution_quality.missing_upload,
            unavailable.totals.upload
        );
        assert_eq!(
            unavailable.attribution_quality.missing_download,
            unavailable.totals.download
        );
    }

    #[test]
    fn fused_raw_summary_matches_independent_totals_and_attribution() {
        let (_dir, coordinator, _) = setup();
        let connection = coordinator.connection();
        connection
            .execute_batch(
                "insert into connection_session(session_pk,epoch_id,connection_id,started_utc,host)
             values (4,1,'missing-attr',600,null),(5,1,'orphan-dimension',600,'orphan.example'),
                    (6,1,'zero-byte-missing',600,'');
             insert into connection_session_attr(session_pk,host_id,process_id,network_id,
                    primary_category_id,chain_key,policy_version,started_utc)
             values (5,999,999,999,999,'',1,600);
             insert into connection_minute(utc_minute,session_pk,upload,download) values
                    (10,4,11,31),(11,4,12,32),(11,5,13,33),(12,5,14,34),
                    (10,6,0,0),(12,6,0,0),(11,1,15,35),(12,1,16,36);",
            )
            .expect("缺失归因、同分钟重叠和零字节连接");
        let filters = [
            crate::c3::query::ReportFilters::default(),
            crate::c3::query::ReportFilters {
                host: Some(UNKNOWN_IDENTITY.into()),
                ..Default::default()
            },
            crate::c3::query::ReportFilters {
                process: Some(UNKNOWN_IDENTITY.into()),
                ..Default::default()
            },
            crate::c3::query::ReportFilters {
                host: Some("a.example".into()),
                process: Some("app.exe".into()),
                ..Default::default()
            },
            crate::c3::query::ReportFilters {
                category: Some(RESIDENTIAL_ACCOUNTING_FILTER.into()),
                ..Default::default()
            },
            crate::c3::query::ReportFilters {
                network: Some("udp".into()),
                ..Default::default()
            },
            crate::c3::query::ReportFilters {
                rule: Some("DIRECT".into()),
                ..Default::default()
            },
            crate::c3::query::ReportFilters {
                chain: Some("DIRECT".into()),
                ..Default::default()
            },
            crate::c3::query::ReportFilters {
                host: Some("no-matching-host".into()),
                ..Default::default()
            },
        ];
        for grouping in [
            DimensionKind::Host,
            DimensionKind::Process,
            DimensionKind::Rule,
            DimensionKind::Chain,
            DimensionKind::Network,
            DimensionKind::Category,
        ] {
            for filter in &filters {
                let mut query = base_query();
                query.grouping = grouping;
                query.filters = filter.clone();
                query.top_n = 1;
                let (fragment, values) = filter_clause(filter);
                let params = merge_sql_params([Value::from(0), Value::from(120)], &values, []);
                let expected_totals =
                    load_totals(connection, &render_sql(TOTALS_RAW, &fragment), &params)
                        .expect("原独立总量");
                let expected_quality =
                    load_raw_attribution_quality(connection, &query).expect("原独立归因");
                let report = run_uncached(
                    coordinator.path(),
                    query.clone(),
                    7_200,
                    30,
                    &Arc::new(AtomicBool::new(false)),
                    None,
                )
                .expect("融合报告");
                assert_eq!(
                    (
                        report.totals.upload,
                        report.totals.download,
                        report.totals.connection_count,
                        report.totals.active_duration_sec
                    ),
                    expected_totals,
                    "{query:?}"
                );
                assert_eq!(report.attribution_quality, expected_quality, "{query:?}");
                if *filter == crate::c3::query::ReportFilters::default() {
                    assert_eq!(report.totals.connection_count, 6);
                    assert_eq!(report.totals.active_duration_sec, 5 * 60);
                    assert_eq!(
                        (report.coverage.covered_sec, report.coverage.gap_sec),
                        (1500, 300)
                    );
                }
            }
        }
    }

    #[test]
    fn unknown_process_filter_keeps_only_missing_process_sessions() {
        let (_dir, coordinator, mut store) = setup();
        coordinator
            .connection()
            .execute(
                "update connection_session_attr set process_id = null where session_pk = 2",
                [],
            )
            .expect("one process missing");
        let mut query = base_query();
        query.grouping = DimensionKind::Host;
        query.filters.process = Some(UNKNOWN_IDENTITY.into());
        let result = run_now(&coordinator, &mut store, query, 3_600);
        assert_eq!(result.totals.upload, 20);
        assert_eq!(result.totals.download, 60);
        assert_eq!(result.totals.connection_count, 1);
        assert_eq!(
            result.attribution_quality.known_upload + result.attribution_quality.missing_upload,
            result.totals.upload
        );
    }

    #[test]
    fn residential_accounting_filter_keeps_tagged_sessions() {
        let (_dir, coordinator, mut store) = setup();
        coordinator
            .connection()
            .execute(
                "update connection_session_attr set primary_category_id = null where session_pk = 2",
                [],
            )
            .expect("untag one session");
        let mut global = base_query();
        global.grouping = DimensionKind::Process;
        let all = run_now(&coordinator, &mut store, global, 3_600);
        let mut query = base_query();
        query.grouping = DimensionKind::Process;
        query.filters.category = Some(RESIDENTIAL_ACCOUNTING_FILTER.into());
        let filtered = run_now(&coordinator, &mut store, query, 3_600);
        assert!(filtered.totals.download < all.totals.download);
        assert_eq!(filtered.totals.download, all.totals.download - 60);
        assert_eq!(
            filtered
                .series
                .iter()
                .map(|point| point.download)
                .sum::<i64>(),
            filtered.totals.download
        );
    }

    #[test]
    fn residential_raw_recovery_restores_null_categories_without_multiplying_traffic() {
        let (_dir, coordinator, mut store) = setup();
        seed_residential_sort_fixture(coordinator.connection());
        coordinator
            .connection()
            .execute_batch(
                "insert or replace into target_item(set_id, position, name) values
                    (1, 0, '家宽'), (1, 1, '备用');
                 update connection_session_attr
                    set primary_category_id = null
                  where session_pk in (101, 102, 104);
                 insert into connection_chain(session_pk, position, node) values
                    (101, 0, 'AI-家宽'),
                    (101, 1, '家宽-SOCKS5'),
                    (101, 2, '备用'),
                    (102, 0, 'PROXY>家宽节点'),
                    (103, 0, 'DIRECT'),
                    (104, 0, 'AI-家宽');",
            )
            .expect("legacy raw fixture");

        let mut global_query = base_query();
        global_query.grouping = DimensionKind::Host;
        global_query.top_n = 100;
        let global = run_now(&coordinator, &mut store, global_query, 3_600);

        let mut query = base_query();
        query.grouping = DimensionKind::Host;
        query.filters.category = Some(RESIDENTIAL_ACCOUNTING_FILTER.into());
        query.top_n = 1;
        query.sort.field = SortField::Upload;
        let upload = run_now(&coordinator, &mut store, query.clone(), 3_600);
        assert_eq!(upload.rankings[0].identity, "upload.example");

        query.sort.field = SortField::Download;
        let download = run_now(&coordinator, &mut store, query.clone(), 3_600);
        assert_eq!(download.rankings[0].identity, "download.example");

        query.top_n = 100;
        let complete = run_now(&coordinator, &mut store, query, 3_600);
        assert!(complete
            .rankings
            .iter()
            .any(|row| row.identity == "c.example"));
        assert!(!complete
            .rankings
            .iter()
            .any(|row| row.identity == "outside.example"));
        assert_eq!(
            complete.rankings.iter().map(|row| row.upload).sum::<i64>(),
            complete.totals.upload
        );
        assert_eq!(
            complete
                .rankings
                .iter()
                .map(|row| row.download)
                .sum::<i64>(),
            complete.totals.download
        );
        assert_eq!(
            complete
                .series
                .iter()
                .map(|point| point.upload)
                .sum::<i64>(),
            complete.totals.upload
        );
        assert_eq!(
            complete
                .series
                .iter()
                .map(|point| point.download)
                .sum::<i64>(),
            complete.totals.download
        );
        assert_eq!(
            global.totals.upload - complete.totals.upload,
            9_000,
            "未分类且不命中 target 的行仍排除"
        );
        assert_eq!(global.totals.download - complete.totals.download, 9_000);
    }

    #[test]
    fn residential_history_without_raw_or_materialized_category_is_unsupported() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("legacy-null.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        coordinator
            .connection()
            .execute_batch(
                "update connection_session_attr set primary_category_id = null;
                 delete from traffic_hourly_dimension;
                 insert into target_item(set_id, position, name) values (1, 0, '家宽');
                 insert into connection_chain(session_pk, position, node) values
                    (1, 0, 'AI-家宽'), (2, 0, 'PROXY>家宽节点');
                 insert or replace into retention_watermark(layer, watermark_utc, delete_watermark_utc)
                 values ('hourly_dim_v2', 0, 0);",
            )
            .expect("legacy null fixture");
        materialize(&mut coordinator, 40 * 86_400);
        let mut store = ReportSnapshotStore::open(dir.path());
        let mut query = base_query();
        query.grouping = DimensionKind::Host;
        query.filters.category = Some(RESIDENTIAL_ACCOUNTING_FILTER.into());
        let result = run_now(&coordinator, &mut store, query.clone(), 40 * 86_400);
        assert_eq!(result.data_tier, DataTier::Raw);
        assert_eq!(result.totals.upload, 30);
        coordinator.connection().execute_batch("delete from connection_minute;
            update retention_state set status='deleted' where layer='day_exact_v1' and chunk_utc=0;").expect("明细已清理");
        query.range_end_utc = 86_400;
        query.granularity = Granularity::Day;
        assert_eq!(
            query_period_usage(
                coordinator.path(),
                &query,
                40 * 86_400,
                30,
                &Arc::new(AtomicBool::new(false))
            )
            .expect_err("告警也不能把 legacy 未知变成零")
            .code(),
            "capability_unsupported"
        );
        assert_eq!(
            run_uncached(
                coordinator.path(),
                query,
                40 * 86_400,
                30,
                &Arc::new(AtomicBool::new(false)),
                None
            )
            .expect_err("无法恢复 raw membership")
            .code(),
            "capability_unsupported"
        );
    }

    #[test]
    fn residential_host_rank_sorts_before_raw_top_n_and_conserves_filtered_traffic() {
        let (_dir, coordinator, mut store) = setup();
        seed_residential_sort_fixture(coordinator.connection());
        let mut query = base_query();
        query.grouping = DimensionKind::Host;
        query.filters.category = Some(RESIDENTIAL_ACCOUNTING_FILTER.into());
        query.top_n = 1;
        query.sort.field = SortField::Upload;
        let upload = run_now(&coordinator, &mut store, query.clone(), 3_600);
        assert_eq!(upload.rankings[0].identity, "upload.example");
        assert!(!upload
            .rankings
            .iter()
            .any(|row| row.identity == "outside.example"));

        query.sort.field = SortField::Download;
        let download = run_now(&coordinator, &mut store, query.clone(), 3_600);
        assert_eq!(download.rankings[0].identity, "download.example");

        query.sort = Default::default();
        let default_download = run_now(&coordinator, &mut store, query.clone(), 3_600);
        assert_eq!(default_download.rankings[0].identity, "download.example");

        query.top_n = 100;
        let complete = run_now(&coordinator, &mut store, query, 3_600);
        assert!(complete
            .rankings
            .iter()
            .any(|row| row.identity == "8.8.4.4"));
        assert!(!complete
            .rankings
            .iter()
            .any(|row| row.identity == "outside.example"));
        assert_eq!(
            complete.rankings.iter().map(|row| row.upload).sum::<i64>(),
            complete.totals.upload
        );
        assert_eq!(
            complete
                .rankings
                .iter()
                .map(|row| row.download)
                .sum::<i64>(),
            complete.totals.download
        );
        assert_eq!(
            complete
                .series
                .iter()
                .map(|point| point.upload)
                .sum::<i64>(),
            complete.totals.upload
        );
        assert_eq!(
            complete
                .series
                .iter()
                .map(|point| point.download)
                .sum::<i64>(),
            complete.totals.download
        );
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

    #[test]
    fn raw_bucket_series_matches_grouped_oracle_with_negative_minutes_and_filters() {
        use crate::c3::query::ReportFilters;
        let (_dir, coordinator, _) = setup();
        let connection = coordinator.connection();
        connection
            .execute_batch(
                "insert into connection_session(session_pk,epoch_id,connection_id,started_utc,host)
             values (4,1,'missing-attr',0,null),(5,1,'zero-only',0,'zero.example');
             insert into connection_chain(session_pk,position,node) values(4,0,'家宽');
             insert into target_set(set_id,policy_version) values(1,1)
                 on conflict(set_id) do nothing;
             insert or ignore into target_item(set_id,position,name) values(1,0,'家宽');
             insert into connection_minute(utc_minute,session_pk,upload,download)
             values(-121,5,0,0),(0,5,0,0),(121,5,0,0);",
            )
            .expect("缺失元数据、家宽回退与零字节样本");
        let granularities = [
            Granularity::Minute1,
            Granularity::Minute2,
            Granularity::Minute5,
            Granularity::Minute10,
            Granularity::Hour,
            Granularity::Day,
            Granularity::Month,
        ];
        for granularity in granularities {
            let width = granularity.bucket_minutes();
            for minute in [
                -2 * width - 1,
                -2 * width,
                -width - 1,
                -width,
                -width + 1,
                -1,
                0,
                1,
                width - 1,
                width,
                width + 1,
                2 * width,
                2 * width + 1,
            ] {
                for session in 1..=4 {
                    connection.execute("insert or ignore into connection_minute(utc_minute,session_pk,upload,download) values(?1,?2,?2,?2*3)",
                        params![minute, session]).unwrap();
                }
            }
        }
        let filters = [
            ReportFilters::default(),
            ReportFilters {
                host: Some(UNKNOWN_IDENTITY.into()),
                ..Default::default()
            },
            ReportFilters {
                process: Some(UNKNOWN_IDENTITY.into()),
                ..Default::default()
            },
            ReportFilters {
                host: Some("a.example".into()),
                process: Some("app.exe".into()),
                ..Default::default()
            },
            ReportFilters {
                category: Some(RESIDENTIAL_ACCOUNTING_FILTER.into()),
                ..Default::default()
            },
            ReportFilters {
                category: Some("机场".into()),
                ..Default::default()
            },
            ReportFilters {
                network: Some("udp".into()),
                ..Default::default()
            },
            ReportFilters {
                rule: Some("DIRECT".into()),
                ..Default::default()
            },
            ReportFilters {
                chain: Some("家宽".into()),
                ..Default::default()
            },
            ReportFilters {
                host: Some("zero.example".into()),
                ..Default::default()
            },
            ReportFilters {
                host: Some("no-match.example".into()),
                ..Default::default()
            },
        ];
        for granularity in granularities {
            let width = granularity.bucket_minutes();
            for (start, end) in [
                (-2 * width - 1, 2 * width + 2),
                (-width, -width + 1),
                (-width + 1, width),
                (1, width + 1),
                (width, width),
                (10 * width, 11 * width),
            ] {
                for filter in &filters {
                    let (fragment, filter_params) = filter_clause(filter);
                    let values = merge_sql_params(
                        [
                            Value::from(width),
                            Value::from(width),
                            Value::from(start),
                            Value::from(end),
                        ],
                        &filter_params,
                        [],
                    );
                    let expected = load_series(
                        connection,
                        &render_sql(SERIES_RAW_GROUPED, &fragment),
                        &values,
                        60,
                    )
                    .expect("原始全窗口SQL");
                    let actual = load_raw_series(
                        connection,
                        granularity,
                        start,
                        end,
                        &fragment,
                        &filter_params,
                    )
                    .expect("索引分桶");
                    assert_eq!(
                        actual, expected,
                        "{granularity:?} {start}..{end} {filter:?}"
                    );
                }
            }
        }
        let (fragment, values) = filter_clause(&ReportFilters {
            host: Some("zero.example".into()),
            ..Default::default()
        });
        let zero =
            load_raw_series(connection, Granularity::Hour, -122, 122, &fragment, &values).unwrap();
        assert_eq!(
            zero.iter()
                .map(|point| point.bucket_utc / 60)
                .collect::<Vec<_>>(),
            vec![-120, 0, 120]
        );
        assert!(zero.iter().all(|point| point.upload == 0
            && point.download == 0
            && point.connection_count == 1
            && point.active_duration_sec == 60));
    }

    #[test]
    fn raw_bucket_series_uses_minute_range_without_group_sort() {
        let (_dir, coordinator, _) = setup();
        let plans = ReportService::explain_named(coordinator.connection(), "series_raw").unwrap();
        assert!(
            plans
                .iter()
                .any(|plan| plan.contains("sqlite_autoindex_connection_minute_1")
                    && plan.contains("utc_minute>?")
                    && plan.contains("utc_minute<?")),
            "{plans:?}"
        );
        assert!(
            !plans
                .iter()
                .any(|plan| plan.contains("GROUP BY") || plan.contains("ORDER BY")),
            "{plans:?}"
        );
        let scan =
            ReportService::explain_named(coordinator.connection(), "raw_minute_scan").unwrap();
        assert!(
            scan.iter()
                .any(|plan| plan.contains("sqlite_autoindex_connection_minute_1")
                    && plan.contains("utc_minute>?")
                    && plan.contains("utc_minute<?")),
            "{scan:?}"
        );
        assert!(
            !scan
                .iter()
                .any(|plan| plan.contains("GROUP BY") || plan.contains("ORDER BY")),
            "{scan:?}"
        );
    }

    #[test]
    fn raw_bucket_series_keeps_registered_cancel_and_deadline_across_buckets() {
        let (_dir, coordinator, _) = setup();
        let connection = coordinator.connection();
        let cancel = Arc::new(AtomicBool::new(false));
        let seen = Arc::new(AtomicU64::new(0));
        let seen_rows = Arc::clone(&seen);
        let flag = Arc::clone(&cancel);
        connection
            .create_scalar_function(
                "observe_series",
                0,
                rusqlite::functions::FunctionFlags::SQLITE_UTF8,
                move |_| {
                    if seen_rows.fetch_add(1, Ordering::SeqCst) == 20 {
                        flag.store(true, Ordering::SeqCst);
                    }
                    Ok(1)
                },
            )
            .unwrap();
        for minute in 0..600 {
            connection.execute("insert or ignore into connection_minute(utc_minute,session_pk,upload,download) values(?1,1,1,1)", [minute]).unwrap();
        }
        attach_cancel(connection, &cancel, Instant::now(), Duration::from_secs(10)).unwrap();
        let cancelled = load_raw_series(
            connection,
            Granularity::Minute1,
            0,
            600,
            "and observe_series() = 1",
            &[],
        );
        connection
            .progress_handler(0, None::<fn() -> bool>)
            .unwrap();
        assert!(
            matches!(cancelled, Err(ReportError::Cancelled(_))),
            "{cancelled:?}"
        );
        assert!(seen.load(Ordering::SeqCst) < 600);
        cancel.store(false, Ordering::SeqCst);
        attach_cancel(
            connection,
            &cancel,
            Instant::now() - Duration::from_secs(1),
            Duration::from_millis(1),
        )
        .unwrap();
        let expired = load_raw_series(connection, Granularity::Minute1, 0, 600, "", &[]);
        connection
            .progress_handler(0, None::<fn() -> bool>)
            .unwrap();
        assert!(
            matches!(expired, Err(ReportError::Cancelled(_))),
            "{expired:?}"
        );
        assert!(connection.is_autocommit());
        assert!(
            !load_raw_series(connection, Granularity::Hour, 0, 600, "", &[])
                .unwrap()
                .is_empty()
        );
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
        for _ in 0..100 {
            let chunk = RetentionService::run_chunk(
                coordinator,
                now,
                30,
                RetentionMode::MaterializeOnly,
                &SpaceBudget::unlimited(),
                &Arc::new(AtomicBool::new(false)),
            )
            .expect("bounded step");
            if !chunk.more_pending {
                return;
            }
        }
        panic!("fixture materialization did not finish within 100 bounded steps");
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
        dim_q.range_end_utc = 86_400;
        dim_q.granularity = Granularity::Day;
        let dim_chain = run_now(&coordinator, &mut store, dim_q, 40 * 86_400);
        assert_eq!(dim_chain.data_tier, DataTier::DailyDimension);
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
        dim_rule_q.range_end_utc = 86_400;
        dim_rule_q.granularity = Granularity::Day;
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
    fn residential_host_rank_sorts_before_dimension_top_n() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("sort-dim.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        seed_residential_sort_fixture(coordinator.connection());
        enable_v2_from_epoch(&coordinator);
        materialize(&mut coordinator, 40 * 86_400);
        let mut store = ReportSnapshotStore::open(dir.path());
        let mut query = base_query();
        query.grouping = DimensionKind::Host;
        query.filters.category = Some(RESIDENTIAL_ACCOUNTING_FILTER.into());
        query.top_n = 1;
        query.sort.field = SortField::Upload;
        let upload = run_now(&coordinator, &mut store, query.clone(), 40 * 86_400);
        assert_eq!(upload.data_tier, DataTier::Raw);
        assert_eq!(upload.rankings[0].identity, "upload.example");

        query.sort.field = SortField::Download;
        let download = run_now(&coordinator, &mut store, query, 40 * 86_400);
        assert_eq!(download.data_tier, DataTier::Raw);
        assert_eq!(download.rankings[0].identity, "download.example");
        assert!(!download
            .rankings
            .iter()
            .any(|row| row.identity == "outside.example"));
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
            query.range_end_utc = 86_400;
            query.granularity = Granularity::Day;
            let result = run_now(&coordinator, &mut store, query, 40 * 86_400);
            assert!(
                !result.rankings.is_empty(),
                "{grouping:?} should have ranks"
            );
            assert_eq!(
                result.data_tier,
                if grouping == DimensionKind::Category {
                    DataTier::Raw
                } else {
                    DataTier::DailyDimension
                }
            );
        }
    }

    #[test]
    fn first_materialization_starts_at_actual_retained_raw() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("wm.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        materialize(&mut coordinator, 40 * 86_400);
        let mut store = ReportSnapshotStore::open(dir.path());
        let mut query = base_query();
        query.grouping = DimensionKind::Process;
        let cancel = Arc::new(AtomicBool::new(false));
        query.range_end_utc = 86_400;
        query.granularity = Granularity::Day;
        let report = ReportService::run(
            coordinator.path(),
            &mut store,
            query,
            40 * 86_400,
            30,
            &cancel,
            None,
        )
        .expect("first materialization");
        assert_eq!(report.data_tier, DataTier::DailyDimension);
        assert_eq!(report.totals.download, 90);
        assert!(report.drilldown_capability.exact_top_n);
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

    fn ranking_named<'a>(result: &'a ReportResult, identity: &str) -> &'a RankingRow {
        result
            .rankings
            .iter()
            .find(|row| row.identity == identity)
            .unwrap_or_else(|| panic!("missing {identity}"))
    }

    #[test]
    fn raw_host_direct_exit_is_not_mixed() {
        let (_dir, coordinator, mut store) = setup();
        let mut query = base_query();
        query.grouping = DimensionKind::Host;
        let result = run_now(&coordinator, &mut store, query, 3_600);
        assert_eq!(result.data_tier, DataTier::Raw);
        assert!(result
            .named_sql
            .iter()
            .any(|name| name == "raw_minute_scan"));
        let row = ranking_named(&result, "b.example");
        assert_eq!(row.primary_exit.as_deref(), Some("DIRECT"));
        assert!(!row.exit_mixed);
    }

    #[test]
    fn raw_host_mixed_exits_pick_download_then_chain_key_asc() {
        let (_dir, coordinator, mut store) = setup();
        coordinator
            .connection()
            .execute_batch(
                "
                insert or ignore into connection_session(session_pk, epoch_id, connection_id, started_utc, host)
                values
                    (201, 1, 'mix-a', 900, 'mix.example'),
                    (202, 1, 'mix-b', 900, 'mix.example'),
                    (203, 1, 'win-a', 900, 'win.example'),
                    (204, 1, 'win-b', 900, 'win.example');
                insert or ignore into dimension_dict(dimension_kind, dimension_id, value) values
                    ('host', 201, 'mix.example'),
                    ('host', 202, 'win.example');
                insert or ignore into connection_session_attr(
                    session_pk, host_id, process_id, rule_id, network_id, chain_key,
                    policy_version, primary_category_id, started_utc, ended_utc
                ) values
                    (201, 201, 1, null, 1, 'PROXY', 1, 1, 900, null),
                    (202, 201, 1, null, 1, 'DIRECT', 1, 1, 900, null),
                    (203, 202, 1, null, 1, 'PROXY', 1, 1, 900, null),
                    (204, 202, 1, null, 1, 'DIRECT', 1, 1, 900, null);
                insert or ignore into connection_minute(utc_minute, session_pk, upload, download)
                values
                    (15, 201, 1, 50),
                    (15, 202, 1, 50),
                    (15, 203, 1, 100),
                    (15, 204, 1, 40);
                ",
            )
            .expect("mixed exits");
        let mut query = base_query();
        query.grouping = DimensionKind::Host;
        let result = run_now(&coordinator, &mut store, query, 3_600);
        let tied = ranking_named(&result, "mix.example");
        assert_eq!(tied.primary_exit.as_deref(), Some("DIRECT"));
        assert!(tied.exit_mixed);
        let won = ranking_named(&result, "win.example");
        assert_eq!(won.primary_exit.as_deref(), Some("PROXY"));
        assert!(won.exit_mixed);
    }

    #[test]
    fn empty_chain_key_is_unknown_not_direct() {
        let (_dir, coordinator, mut store) = setup();
        coordinator
            .connection()
            .execute_batch(
                "
                insert or ignore into connection_session(session_pk, epoch_id, connection_id, started_utc, host)
                values (210, 1, 'blank', 900, 'blank.example'), (211, 1, 'null-exit', 900, 'nulle.example');
                insert or ignore into connection_session_attr(
                    session_pk, host_id, process_id, rule_id, network_id, chain_key,
                    policy_version, primary_category_id, started_utc, ended_utc
                ) values
                    (210, null, 1, null, 1, '   ', 1, 1, 900, null),
                    (211, null, 1, null, 1, null, 1, 1, 900, null);
                insert or ignore into connection_minute(utc_minute, session_pk, upload, download)
                values (15, 210, 1, 70), (15, 211, 1, 80);
                ",
            )
            .expect("empty exits");
        let mut query = base_query();
        query.grouping = DimensionKind::Host;
        let result = run_now(&coordinator, &mut store, query, 3_600);
        let blank = ranking_named(&result, "blank.example");
        assert_eq!(blank.primary_exit, None);
        assert!(!blank.exit_mixed);
        let missing = ranking_named(&result, "nulle.example");
        assert_eq!(missing.primary_exit, None);
        assert!(!missing.exit_mixed);
    }

    #[test]
    fn chain_grouping_does_not_fill_exits() {
        let (_dir, coordinator, mut store) = setup();
        let mut query = base_query();
        query.grouping = DimensionKind::Chain;
        let result = run_now(&coordinator, &mut store, query, 3_600);
        assert!(!result.rankings.is_empty());
        assert!(result.named_sql.iter().all(|name| !name.contains("exits")));
        assert!(result
            .rankings
            .iter()
            .all(|row| row.primary_exit.is_none() && !row.exit_mixed));
    }

    #[test]
    fn daily_dimension_leaves_exits_unknown() {
        let dir = tempdir().expect("dir");
        let mut coordinator =
            StorageCoordinator::open(&dir.path().join("hourly-exit.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        seed_extra_dimensions(coordinator.connection());
        enable_v2_from_epoch(&coordinator);
        materialize(&mut coordinator, 40 * 86_400);
        let mut store = ReportSnapshotStore::open(dir.path());
        let mut query = base_query();
        query.grouping = DimensionKind::Host;
        query.range_end_utc = 86_400;
        query.granularity = Granularity::Day;
        let result = run_now(&coordinator, &mut store, query, 40 * 86_400);
        assert_eq!(result.data_tier, DataTier::DailyDimension);
        assert!(!result.rankings.is_empty());
        assert!(result
            .rankings
            .iter()
            .all(|row| row.primary_exit.is_none() && !row.exit_mixed));
    }

    #[test]
    fn raw_process_exit_uses_attr_identity() {
        let (_dir, coordinator, mut store) = setup();
        let mut query = base_query();
        query.grouping = DimensionKind::Process;
        let result = run_now(&coordinator, &mut store, query, 3_600);
        assert!(result
            .named_sql
            .iter()
            .any(|name| name == "raw_minute_scan"));
        let row = ranking_named(&result, "app.exe");
        assert_eq!(row.primary_exit.as_deref(), Some("DIRECT"));
        assert!(!row.exit_mixed);
    }

    #[test]
    fn raw_rule_exit_uses_rule_identity() {
        let (_dir, coordinator, mut store) = setup();
        let mut query = base_query();
        query.grouping = DimensionKind::Rule;
        let result = run_now(&coordinator, &mut store, query, 3_600);
        assert!(result
            .named_sql
            .iter()
            .any(|name| name == "raw_minute_scan"));
        let row = ranking_named(&result, "DIRECT");
        assert_eq!(row.primary_exit.as_deref(), Some("DIRECT"));
        assert!(!row.exit_mixed);
    }
}
