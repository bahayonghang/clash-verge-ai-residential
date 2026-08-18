//! ReportQuery / ReportResult 与能力规划。前端不得传 SQL。

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const TOKEN_TTL_SECS: u64 = 600;
pub const MAX_ACTIVE_TOKENS: usize = 8;
pub const MAX_TOKEN_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_SPOOL_BYTES: u64 = 128 * 1024 * 1024;
pub const PAGE_DEADLINE_MS: u64 = 2_000;
pub const REPORT_DEADLINE_MS: u64 = 10_000;
pub const RAW_RETAIN_DAYS_DEFAULT: i64 = 30;
pub const RAW_RETAIN_DAYS_MAX: i64 = 90;
pub const DIMENSION_RETAIN_DAYS: i64 = 396;
pub const PAGE_DEFAULT: u32 = 200;
pub const PAGE_MAX: u32 = 1_000;
pub const TOP_N_DEFAULT: u32 = 20;
pub const TOP_N_MAX: u32 = 100;
pub const MAX_RANGE_SECS: i64 = 400 * 86_400;
pub const AUTO_DELETE_ENABLED: bool = false;
pub const REPORT_DTO_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("{0}")]
    InvalidQuery(&'static str),
    #[error("{0}")]
    CapabilityUnsupported(&'static str),
    #[error("{0}")]
    Cancelled(&'static str),
    #[error("{0}")]
    DeadlineExceeded(&'static str),
    #[error("{0}")]
    TokenExpired(&'static str),
    #[error("{0}")]
    QuotaExceeded(&'static str),
    #[error("{0}")]
    StorageBusy(&'static str),
    #[error("{0}")]
    InsufficientSpace(&'static str),
    #[error("{0}")]
    Failed(&'static str),
}

impl ReportError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidQuery(_) => "invalid_query",
            Self::CapabilityUnsupported(_) => "capability_unsupported",
            Self::Cancelled(_) => "cancelled",
            Self::DeadlineExceeded(_) => "deadline_exceeded",
            Self::TokenExpired(_) => "token_expired",
            Self::QuotaExceeded(_) => "quota_exceeded",
            Self::StorageBusy(_) => "storage_busy",
            Self::InsufficientSpace(_) => "insufficient_space",
            Self::Failed(_) => "storage_failure",
        }
    }

    pub fn message_zh(&self) -> &'static str {
        match self {
            Self::InvalidQuery(_) => "查询参数无效。",
            Self::CapabilityUnsupported(_) => "当前数据层不支持该查询。",
            Self::Cancelled(_) => "查询已取消。",
            Self::DeadlineExceeded(_) => "查询超过时限。",
            Self::TokenExpired(_) => "报告快照已过期，请重新运行。",
            Self::QuotaExceeded(_) => "报告快照配额已满。",
            Self::StorageBusy(_) => "存储正忙。",
            Self::InsufficientSpace(_) => "磁盘空间不足，已停止。",
            Self::Failed(_) => "存储失败。",
        }
    }

    pub fn action_zh(&self) -> &'static str {
        match self {
            Self::InvalidQuery(_) => "检查时间范围、维度和分页",
            Self::CapabilityUnsupported(_) => "缩小范围或改用支持的维度",
            Self::Cancelled(_) => "可重新运行报告",
            Self::DeadlineExceeded(_) => "缩小范围后重试",
            Self::TokenExpired(_) => "重新运行报告",
            Self::QuotaExceeded(_) => "释放旧报告后再试",
            Self::StorageBusy(_) => "等待写入完成后再试",
            Self::InsufficientSpace(_) => "清理磁盘后重试",
            Self::Failed(_) => "打开数据管理检查磁盘",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Granularity {
    Hour,
    Day,
    Month,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DimensionKind {
    Category,
    Host,
    Process,
    Rule,
    Chain,
    Network,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetPolicy {
    Current,
    Historical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SortField {
    Upload,
    Download,
    Name,
    Identity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DataTier {
    Raw,
    HourlyDimension,
    DailyDimension,
    DailyCore,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportFilters {
    pub category: Option<String>,
    pub host: Option<String>,
    pub process: Option<String>,
    pub rule: Option<String>,
    pub chain: Option<String>,
    pub network: Option<String>,
}

impl ReportFilters {
    pub fn dimension_filter_count(&self) -> usize {
        [
            self.host.as_ref(),
            self.process.as_ref(),
            self.rule.as_ref(),
            self.chain.as_ref(),
            self.network.as_ref(),
        ]
        .into_iter()
        .flatten()
        .count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortSpec {
    pub field: SortField,
    pub descending: bool,
}

impl Default for SortSpec {
    fn default() -> Self {
        Self {
            field: SortField::Download,
            descending: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageSpec {
    pub limit: u32,
    pub after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonSpec {
    pub previous_equal_window: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportQuery {
    pub range_start_utc: i64,
    pub range_end_utc: i64,
    pub display_timezone: String,
    pub granularity: Granularity,
    pub filters: ReportFilters,
    pub grouping: DimensionKind,
    pub target_policy: TargetPolicy,
    pub comparison: Option<ComparisonSpec>,
    pub sort: SortSpec,
    pub page: PageSpec,
    pub top_n: u32,
    pub include_sessions: bool,
}

impl Default for ReportQuery {
    fn default() -> Self {
        Self {
            range_start_utc: 0,
            range_end_utc: 3_600,
            display_timezone: "UTC".into(),
            granularity: Granularity::Hour,
            filters: ReportFilters::default(),
            grouping: DimensionKind::Host,
            target_policy: TargetPolicy::Historical,
            comparison: None,
            sort: SortSpec::default(),
            page: PageSpec {
                limit: PAGE_DEFAULT,
                after: None,
            },
            top_n: TOP_N_DEFAULT,
            include_sessions: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportTotals {
    pub upload: i64,
    pub download: i64,
    pub connection_count: i64,
    pub active_duration_sec: i64,
    pub previous_upload: Option<i64>,
    pub previous_download: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesPoint {
    pub bucket_utc: i64,
    pub upload: i64,
    pub download: i64,
    pub connection_count: i64,
    pub active_duration_sec: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankingRow {
    pub identity: String,
    pub label: String,
    pub upload: i64,
    pub download: i64,
    pub connection_count: i64,
    pub active_duration_sec: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRow {
    pub identity: String,
    pub host: Option<String>,
    pub process: Option<String>,
    pub rule: Option<String>,
    pub upload: i64,
    pub download: i64,
    pub started_utc: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageSlice {
    pub kind: String,
    pub reason: String,
    pub started_utc: i64,
    pub ended_utc: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageView {
    pub status: String,
    pub covered_sec: i64,
    pub gap_sec: i64,
    pub slices: Vec<CoverageSlice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrilldownCapability {
    pub sessions: bool,
    pub current_policy: bool,
    pub cross_dimension: bool,
    pub exact_top_n: bool,
    pub note_zh: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyMetadata {
    pub target_policy: TargetPolicy,
    pub policy_version: Option<u32>,
    pub note_zh: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportResult {
    pub schema_version: u32,
    pub data_version: u64,
    pub report_snapshot_token: String,
    pub query_echo: ReportQuery,
    pub totals: ReportTotals,
    pub series: Vec<SeriesPoint>,
    pub rankings: Vec<RankingRow>,
    pub sessions: Vec<SessionRow>,
    pub coverage: CoverageView,
    pub drilldown_capability: DrilldownCapability,
    pub policy_metadata: PolicyMetadata,
    pub data_tier: DataTier,
    pub named_sql: Vec<String>,
    pub next_cursor: Option<String>,
    pub unit: String,
    pub generated_utc: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityPlan {
    pub tier: DataTier,
    pub named_sql: Vec<&'static str>,
    pub drilldown: DrilldownCapability,
    pub deadline_ms: u64,
}

pub fn timezone_offset_secs(name: &str, at_utc: i64) -> Result<i32, ReportError> {
    match name {
        "UTC" | "utc" | "+00:00" | "Z" => Ok(0),
        "Asia/Shanghai" | "+08:00" => Ok(8 * 3600),
        "America/New_York" | "US/Eastern" => Ok(new_york_offset(at_utc)),
        "local" => Ok(local_offset_secs()),
        _ => Err(ReportError::InvalidQuery("unknown timezone")),
    }
}

/// 本地自然日的 UTC 半开区间 `[start, end)`。DST 长短日由偏移重算，不固定 86400。
pub fn local_day_bounds(timezone: &str, at_utc: i64) -> Result<(i64, i64), ReportError> {
    let (year, month, day) = local_ymd(timezone, at_utc)?;
    let start = utc_from_local_naive(timezone, year, month, day, 0, 0, 0)?;
    let (ny, nm, nd) = next_local_day(year, month, day);
    let end = utc_from_local_naive(timezone, ny, nm, nd, 0, 0, 0)?;
    Ok((start, end))
}

/// 本地自然月的 UTC 半开区间 `[start, end)`。月份长度按本地历法。
pub fn local_month_bounds(timezone: &str, at_utc: i64) -> Result<(i64, i64), ReportError> {
    let (year, month, _) = local_ymd(timezone, at_utc)?;
    let start = utc_from_local_naive(timezone, year, month, 1, 0, 0, 0)?;
    let (ny, nm) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let end = utc_from_local_naive(timezone, ny, nm, 1, 0, 0, 0)?;
    Ok((start, end))
}

fn local_ymd(timezone: &str, at_utc: i64) -> Result<(i32, u32, u32), ReportError> {
    use chrono::Datelike;
    let offset = i64::from(timezone_offset_secs(timezone, at_utc)?);
    let local = chrono::DateTime::from_timestamp(at_utc + offset, 0)
        .ok_or(ReportError::InvalidQuery("timestamp"))?;
    let date = local.date_naive();
    Ok((date.year(), date.month(), date.day()))
}

fn utc_from_local_naive(
    timezone: &str,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Result<i64, ReportError> {
    let naive = chrono::NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|date| date.and_hms_opt(hour, minute, second))
        .ok_or(ReportError::InvalidQuery("civil time"))?;
    let as_utc = naive.and_utc().timestamp();
    let mut utc = as_utc - i64::from(timezone_offset_secs(timezone, as_utc)?);
    utc = as_utc - i64::from(timezone_offset_secs(timezone, utc)?);
    Ok(utc)
}

fn next_local_day(year: i32, month: u32, day: u32) -> (i32, u32, u32) {
    use chrono::Datelike;
    let date = chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap_or(chrono::NaiveDate::MIN);
    let next = date.succ_opt().unwrap_or(date);
    (next.year(), next.month(), next.day())
}

fn local_offset_secs() -> i32 {
    let local = chrono::Local::now();
    local.offset().local_minus_utc()
}

fn new_york_offset(at_utc: i64) -> i32 {
    use chrono::Datelike;
    let date = chrono::DateTime::from_timestamp(at_utc, 0)
        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).expect("epoch"));
    let year = date.date_naive().year();
    let dst_start = nth_weekday(year, 3, chrono::Weekday::Sun, 2, 7);
    let dst_end = nth_weekday(year, 11, chrono::Weekday::Sun, 1, 6);
    if at_utc >= dst_start && at_utc < dst_end {
        -4 * 3600
    } else {
        -5 * 3600
    }
}

fn nth_weekday(year: i32, month: u32, weekday: chrono::Weekday, nth: u32, hour: i64) -> i64 {
    use chrono::Datelike;
    let mut count = 0;
    for day in 1..=31 {
        if let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
            if date.weekday() == weekday {
                count += 1;
                if count == nth {
                    return date
                        .and_hms_opt(hour as u32, 0, 0)
                        .expect("hms")
                        .and_utc()
                        .timestamp();
                }
            }
        }
    }
    0
}

pub fn validate_query(query: &ReportQuery) -> Result<(), ReportError> {
    if query.range_end_utc <= query.range_start_utc {
        return Err(ReportError::InvalidQuery("range order"));
    }
    if query.range_end_utc - query.range_start_utc > MAX_RANGE_SECS {
        return Err(ReportError::InvalidQuery("range too large"));
    }
    timezone_offset_secs(&query.display_timezone, query.range_start_utc)?;
    if query.page.limit == 0 || query.page.limit > PAGE_MAX {
        return Err(ReportError::InvalidQuery("page limit"));
    }
    if query.top_n == 0 || query.top_n > TOP_N_MAX {
        return Err(ReportError::InvalidQuery("top n"));
    }
    if let Some(cursor) = &query.page.after {
        if cursor.contains(';') || cursor.to_ascii_lowercase().contains("select") {
            return Err(ReportError::InvalidQuery("cursor"));
        }
    }
    Ok(())
}

pub fn query_fingerprint(query: &ReportQuery) -> String {
    let encoded = serde_json::to_string(query).unwrap_or_default();
    hex::encode(Sha256::digest(encoded.as_bytes()))
}

pub fn needs_raw(query: &ReportQuery) -> bool {
    query.include_sessions
        || query.target_policy == TargetPolicy::Current
        || query.filters.dimension_filter_count() > 1
}

pub fn needs_exact_dimension(query: &ReportQuery) -> bool {
    matches!(
        query.grouping,
        DimensionKind::Host
            | DimensionKind::Process
            | DimensionKind::Rule
            | DimensionKind::Chain
            | DimensionKind::Network
    )
}

pub fn plan_capability(
    query: &ReportQuery,
    now_utc: i64,
    raw_retain_days: i64,
) -> Result<CapabilityPlan, ReportError> {
    validate_query(query)?;
    let raw_days = raw_retain_days.clamp(1, RAW_RETAIN_DAYS_MAX);
    let raw_cutoff = now_utc - raw_days * 86_400;
    let dim_cutoff = now_utc - DIMENSION_RETAIN_DAYS * 86_400;
    let wants_raw = needs_raw(query);
    let wants_dim = needs_exact_dimension(query);
    let deadline = if query.include_sessions {
        PAGE_DEADLINE_MS
    } else {
        REPORT_DEADLINE_MS
    };

    if wants_raw && query.range_start_utc < raw_cutoff {
        return Err(ReportError::CapabilityUnsupported(
            "raw expired for sessions, current policy, or cross-dimension filters",
        ));
    }
    if wants_dim && query.range_start_utc < dim_cutoff {
        return Err(ReportError::CapabilityUnsupported(
            "exact high-cardinality ranking expired",
        ));
    }

    if query.range_start_utc >= raw_cutoff {
        return Ok(CapabilityPlan {
            tier: DataTier::Raw,
            named_sql: raw_named_sql(query),
            drilldown: DrilldownCapability {
                sessions: true,
                current_policy: true,
                cross_dimension: true,
                exact_top_n: true,
                note_zh: "30 天 raw 期内可会话下钻、当前策略重算与受支持组合过滤。".into(),
            },
            deadline_ms: deadline,
        });
    }

    if query.range_start_utc >= dim_cutoff && wants_dim {
        let hourly = matches!(query.granularity, Granularity::Hour);
        return Ok(CapabilityPlan {
            tier: if hourly {
                DataTier::HourlyDimension
            } else {
                DataTier::DailyDimension
            },
            named_sql: if hourly {
                vec![
                    "totals_hourly_dimension",
                    "series_hourly_dimension",
                    "rank_hourly_dimension",
                    "coverage_daily",
                ]
            } else {
                vec![
                    "totals_daily_dimension",
                    "series_daily_dimension",
                    "rank_daily_dimension",
                    "coverage_daily",
                ]
            },
            drilldown: DrilldownCapability {
                sessions: false,
                current_policy: false,
                cross_dimension: false,
                exact_top_n: true,
                note_zh: "13 个月精确层只支持历史主分类加单一分析维度。".into(),
            },
            deadline_ms: deadline,
        });
    }

    if wants_dim || wants_raw {
        return Err(ReportError::CapabilityUnsupported(
            "requested capability is outside retained layers",
        ));
    }

    Ok(CapabilityPlan {
        tier: DataTier::DailyCore,
        named_sql: vec!["totals_daily_core", "series_daily_core", "coverage_daily"],
        drilldown: DrilldownCapability {
            sessions: false,
            current_policy: false,
            cross_dimension: false,
            exact_top_n: false,
            note_zh: "长期 daily 只保留可归因总量、历史主分类和 coverage。".into(),
        },
        deadline_ms: deadline,
    })
}

fn raw_named_sql(query: &ReportQuery) -> Vec<&'static str> {
    let mut names = vec!["totals_raw", "series_raw", "rank_raw", "coverage_raw"];
    if query.include_sessions {
        names.push("sessions_keyset");
    }
    if query.comparison.is_some() {
        names.push("totals_raw_compare");
    }
    names
}

pub fn encode_cursor(sort_value: i64, identity: &str) -> String {
    format!("{sort_value}|{identity}")
}

pub fn decode_cursor(cursor: &str) -> Result<(i64, String), ReportError> {
    let (left, right) = cursor
        .split_once('|')
        .ok_or(ReportError::InvalidQuery("cursor"))?;
    let value = left
        .parse::<i64>()
        .map_err(|_| ReportError::InvalidQuery("cursor"))?;
    if right.is_empty() {
        return Err(ReportError::InvalidQuery("cursor"));
    }
    Ok((value, right.to_string()))
}

pub fn empty_result(query: ReportQuery, plan: &CapabilityPlan, data_version: u64) -> ReportResult {
    ReportResult {
        schema_version: REPORT_DTO_VERSION,
        data_version,
        report_snapshot_token: String::new(),
        query_echo: query,
        totals: ReportTotals {
            upload: 0,
            download: 0,
            connection_count: 0,
            active_duration_sec: 0,
            previous_upload: None,
            previous_download: None,
        },
        series: Vec::new(),
        rankings: Vec::new(),
        sessions: Vec::new(),
        coverage: CoverageView {
            status: "empty".into(),
            covered_sec: 0,
            gap_sec: 0,
            slices: Vec::new(),
        },
        drilldown_capability: plan.drilldown.clone(),
        policy_metadata: PolicyMetadata {
            target_policy: TargetPolicy::Historical,
            policy_version: None,
            note_zh: "观测下界，不是账单。".into(),
        },
        data_tier: plan.tier,
        named_sql: plan
            .named_sql
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
        next_cursor: None,
        unit: "byte".into(),
        generated_utc: 0,
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod query_contract_tests {
    use super::*;

    #[test]
    fn rejects_sql_shaped_cursor_and_bad_range() {
        let mut query = ReportQuery::default();
        query.page.after = Some("1; select * from connection_minute".into());
        assert!(validate_query(&query).is_err());
        query.page.after = None;
        query.range_end_utc = query.range_start_utc;
        assert!(validate_query(&query).is_err());
    }

    #[test]
    fn raw_expired_session_drilldown_is_unsupported() {
        let mut query = ReportQuery::default();
        query.include_sessions = true;
        query.range_start_utc = 0;
        query.range_end_utc = 3_600;
        let now = 90 * 86_400;
        let error = plan_capability(&query, now, 30).expect_err("expired");
        assert_eq!(error.code(), "capability_unsupported");
    }

    #[test]
    fn dimension_expired_top_n_is_unsupported() {
        let mut query = ReportQuery::default();
        query.grouping = DimensionKind::Host;
        query.range_start_utc = 0;
        query.range_end_utc = 86_400;
        let now = 500 * 86_400;
        let error = plan_capability(&query, now, 30).expect_err("dim");
        assert_eq!(error.code(), "capability_unsupported");
    }

    #[test]
    fn long_range_totals_use_daily_core() {
        let mut query = ReportQuery::default();
        query.grouping = DimensionKind::Category;
        query.range_start_utc = 0;
        query.range_end_utc = 86_400;
        let now = 500 * 86_400;
        let plan = plan_capability(&query, now, 30).expect("core");
        assert_eq!(plan.tier, DataTier::DailyCore);
        assert!(!plan.drilldown.exact_top_n);
    }

    #[test]
    fn dst_new_york_switches_offset() {
        let winter = chrono::NaiveDate::from_ymd_opt(2026, 1, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let summer = chrono::NaiveDate::from_ymd_opt(2026, 7, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        assert_eq!(
            timezone_offset_secs("America/New_York", winter).unwrap(),
            -5 * 3600
        );
        assert_eq!(
            timezone_offset_secs("America/New_York", summer).unwrap(),
            -4 * 3600
        );
    }

    #[test]
    fn local_day_bounds_handle_dst_spring_forward() {
        let during = chrono::NaiveDate::from_ymd_opt(2026, 3, 8)
            .unwrap()
            .and_hms_opt(18, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let (start, end) = local_day_bounds("America/New_York", during).expect("day");
        assert_eq!(end - start, 23 * 3600);
        let (again_s, again_e) = local_day_bounds("America/New_York", start).expect("start");
        assert_eq!((again_s, again_e), (start, end));
    }

    #[test]
    fn local_month_bounds_follow_calendar_length() {
        let feb = chrono::NaiveDate::from_ymd_opt(2026, 2, 10)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let (start, end) = local_month_bounds("Asia/Shanghai", feb).expect("month");
        assert_eq!(end - start, 28 * 86_400);
        let mar = chrono::NaiveDate::from_ymd_opt(2026, 3, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let expected_end = mar - 8 * 3600;
        assert_eq!(end, expected_end);
    }

    #[test]
    fn auto_delete_stays_disabled() {
        const { assert!(!AUTO_DELETE_ENABLED) };
    }
}
