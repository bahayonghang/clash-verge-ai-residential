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
pub const UNKNOWN_LABEL_ZH: &str = "未知";
pub const HOURLY_DIM_V2_LAYER: &str = "hourly_dim_v2";

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

    pub fn message_key(&self) -> &'static str {
        match self {
            Self::InvalidQuery(_) => "report.invalid_query",
            Self::CapabilityUnsupported(_) => "report.capability_unsupported",
            Self::Cancelled(_) => "report.cancelled",
            Self::DeadlineExceeded(_) => "report.deadline_exceeded",
            Self::TokenExpired(_) => "report.token_expired",
            Self::QuotaExceeded(_) => "report.quota_exceeded",
            Self::StorageBusy(_) => "report.storage_busy",
            Self::InsufficientSpace(_) => "report.insufficient_space",
            Self::Failed(_) => "report.failed",
        }
    }

    pub fn action_key(&self) -> &'static str {
        match self {
            Self::InvalidQuery(_) => "report.action.invalid_query",
            Self::CapabilityUnsupported(_) => "report.action.capability_unsupported",
            Self::Cancelled(_) => "report.action.cancelled",
            Self::DeadlineExceeded(_) => "report.action.deadline_exceeded",
            Self::TokenExpired(_) => "report.action.token_expired",
            Self::QuotaExceeded(_) => "report.action.quota_exceeded",
            Self::StorageBusy(_) => "report.action.storage_busy",
            Self::InsufficientSpace(_) => "report.action.insufficient_space",
            Self::Failed(_) => "report.action.failed",
        }
    }

    pub fn message(&self, locale: crate::i18n::UiLocale) -> &'static str {
        crate::i18n::t(locale, self.message_key())
    }

    pub fn action(&self, locale: crate::i18n::UiLocale) -> &'static str {
        crate::i18n::t(locale, self.action_key())
    }

    pub fn message_zh(&self) -> &'static str {
        self.message(crate::i18n::UiLocale::Zh)
    }

    pub fn action_zh(&self) -> &'static str {
        self.action(crate::i18n::UiLocale::Zh)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Granularity {
    #[serde(rename = "minute1")]
    Minute1,
    #[serde(rename = "minute2")]
    Minute2,
    #[serde(rename = "minute5")]
    Minute5,
    #[serde(rename = "minute10")]
    Minute10,
    Hour,
    Day,
    Month,
}

impl Granularity {
    pub fn is_minute(self) -> bool {
        matches!(
            self,
            Self::Minute1 | Self::Minute2 | Self::Minute5 | Self::Minute10
        )
    }

    pub fn bucket_minutes(self) -> i64 {
        match self {
            Self::Minute1 => 1,
            Self::Minute2 => 2,
            Self::Minute5 => 5,
            Self::Minute10 => 10,
            Self::Hour => 60,
            Self::Day => 1_440,
            Self::Month => 43_200,
        }
    }
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
            self.category.as_ref(),
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
    #[serde(default)]
    pub primary_exit: Option<String>,
    #[serde(default)]
    pub exit_mixed: bool,
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

/// 缺口按区间并集累加：重叠与相邻的 gap 行先合并再求和，开放行按窗口末端闭合。
/// 结果裁剪到 `[win_start, win_end]`。读取侧（share / summarize）共用，避免口径漂移。
pub fn gap_union_sec(win_start: i64, win_end: i64, slices: &[CoverageSlice]) -> i64 {
    let mut spans: Vec<(i64, i64)> = slices
        .iter()
        .filter(|item| item.kind == "gap")
        .map(|item| {
            (
                item.started_utc.max(win_start),
                item.ended_utc.unwrap_or(win_end).min(win_end),
            )
        })
        .filter(|(start, end)| end > start)
        .collect();
    spans.sort_unstable();
    let mut total = 0i64;
    let mut cursor: Option<(i64, i64)> = None;
    for (start, end) in spans {
        match cursor {
            Some((open_start, open_end)) if start <= open_end => {
                cursor = Some((open_start, open_end.max(end)));
            }
            Some((open_start, open_end)) => {
                total += open_end - open_start;
                cursor = Some((start, end));
            }
            None => cursor = Some((start, end)),
        }
    }
    if let Some((open_start, open_end)) = cursor {
        total += open_end - open_start;
    }
    total
}

#[cfg(test)]
mod gap_union_tests {
    use super::*;

    fn slice(kind: &str, start: i64, end: Option<i64>) -> CoverageSlice {
        CoverageSlice {
            kind: kind.into(),
            reason: "disconnect_or_sleep".into(),
            started_utc: start,
            ended_utc: end,
        }
    }

    #[test]
    fn overlapping_open_gaps_count_once() {
        // 0.2.x 断连风暴形状：29k 条重叠开放行只算一份窗口末端。
        let slices: Vec<_> = (0..29).map(|i| slice("gap", i, None)).collect();
        assert_eq!(gap_union_sec(0, 100, &slices), 100);
    }

    #[test]
    fn disjoint_gaps_sum_and_closed_rows_are_clipped() {
        let slices = vec![
            slice("gap", 10, Some(20)),
            slice("gap", 15, Some(25)),
            slice("gap", 40, Some(70)),
            slice("covered", 0, Some(100)),
        ];
        assert_eq!(gap_union_sec(0, 100, &slices), 45);
    }

    #[test]
    fn open_gap_extends_only_to_window_end() {
        let slices = vec![slice("gap", 50, None)];
        assert_eq!(gap_union_sec(0, 60, &slices), 10);
        // 断连自窗口前开始且仍未恢复：整个窗口都是缺口。
        assert_eq!(gap_union_sec(60, 120, &slices), 60);
        assert_eq!(gap_union_sec(120, 180, &slices), 60);
    }

    #[test]
    fn empty_slices_are_zero() {
        assert_eq!(gap_union_sec(0, 100, &[]), 0);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageView {
    pub status: String,
    pub covered_sec: i64,
    pub gap_sec: i64,
    pub slices: Vec<CoverageSlice>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum AttributionStatus {
    Complete,
    Partial,
    #[default]
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributionQuality {
    pub known_upload: i64,
    pub known_download: i64,
    pub missing_upload: i64,
    pub missing_download: i64,
    pub known_connections: i64,
    pub missing_connections: i64,
    pub status: AttributionStatus,
}

impl Default for AttributionQuality {
    fn default() -> Self {
        Self::from_parts(0, 0, 0, 0, 0, 0)
    }
}

impl AttributionQuality {
    pub fn from_parts(
        known_upload: i64,
        known_download: i64,
        missing_upload: i64,
        missing_download: i64,
        known_connections: i64,
        missing_connections: i64,
    ) -> Self {
        let has_known = known_upload > 0 || known_download > 0 || known_connections > 0;
        let has_missing = missing_upload > 0 || missing_download > 0 || missing_connections > 0;
        let status = match (has_known, has_missing) {
            (_, false) => AttributionStatus::Complete,
            (false, true) => AttributionStatus::Unavailable,
            (true, true) => AttributionStatus::Partial,
        };
        Self {
            known_upload,
            known_download,
            missing_upload,
            missing_download,
            known_connections,
            missing_connections,
            status,
        }
    }

    pub fn unavailable(totals: &ReportTotals) -> Self {
        Self::from_parts(
            0,
            0,
            totals.upload,
            totals.download,
            0,
            totals.connection_count,
        )
    }
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
    #[serde(default)]
    pub attribution_quality: AttributionQuality,
    pub drilldown_capability: DrilldownCapability,
    pub policy_metadata: PolicyMetadata,
    pub data_tier: DataTier,
    pub named_sql: Vec<String>,
    pub next_cursor: Option<String>,
    pub unit: String,
    pub generated_utc: i64,
}

impl ReportResult {
    pub fn reconcile_legacy_attribution_quality(&mut self) {
        let quality = &self.attribution_quality;
        let is_default = quality.known_upload == 0
            && quality.known_download == 0
            && quality.missing_upload == 0
            && quality.missing_download == 0
            && quality.known_connections == 0
            && quality.missing_connections == 0;
        if is_default
            && (self.totals.upload != 0
                || self.totals.download != 0
                || self.totals.connection_count != 0)
        {
            self.attribution_quality = AttributionQuality::unavailable(&self.totals);
        }
    }
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

/// 本地小时的 UTC 半开区间 `[start, end)`。DST 不固定 3600。
pub fn local_hour_bounds(timezone: &str, at_utc: i64) -> Result<(i64, i64), ReportError> {
    let (_, _, _, hour) = local_ymdh(timezone, at_utc)?;
    let start = find_local_hour_start(timezone, at_utc, hour)?;
    let end = find_local_hour_end(timezone, start, hour)?;
    if end <= start {
        return Err(ReportError::InvalidQuery("hour bounds"));
    }
    Ok((start, end))
}

/// 当前本地小时起点之前的已闭合小时。
pub fn closed_local_hour_bounds(timezone: &str, now_utc: i64) -> Result<(i64, i64), ReportError> {
    let (current_start, _) = local_hour_bounds(timezone, now_utc)?;
    local_hour_bounds(timezone, current_start.saturating_sub(1))
}

/// 当前本地自然日起点之前的已闭合日。
pub fn closed_local_day_bounds(timezone: &str, now_utc: i64) -> Result<(i64, i64), ReportError> {
    let (current_start, _) = local_day_bounds(timezone, now_utc)?;
    local_day_bounds(timezone, current_start.saturating_sub(1))
}

/// 自动小时 / 日档案的默认查询。fingerprint 对同一窗口稳定。
pub fn default_auto_report_query(
    granularity: Granularity,
    range_start_utc: i64,
    range_end_utc: i64,
) -> ReportQuery {
    ReportQuery {
        range_start_utc,
        range_end_utc,
        display_timezone: "local".into(),
        granularity,
        filters: ReportFilters::default(),
        grouping: DimensionKind::Host,
        target_policy: TargetPolicy::Historical,
        comparison: Some(ComparisonSpec {
            previous_equal_window: true,
        }),
        sort: SortSpec::default(),
        page: PageSpec {
            limit: PAGE_DEFAULT,
            after: None,
        },
        top_n: TOP_N_DEFAULT,
        include_sessions: false,
    }
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
    let (year, month, day, _) = local_ymdh(timezone, at_utc)?;
    Ok((year, month, day))
}

fn local_ymdh(timezone: &str, at_utc: i64) -> Result<(i32, u32, u32, u32), ReportError> {
    use chrono::{Datelike, Timelike};
    let offset = i64::from(timezone_offset_secs(timezone, at_utc)?);
    let local = chrono::DateTime::from_timestamp(at_utc + offset, 0)
        .ok_or(ReportError::InvalidQuery("timestamp"))?;
    let date = local.date_naive();
    Ok((date.year(), date.month(), date.day(), local.hour()))
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

fn find_local_hour_start(timezone: &str, at_utc: i64, hour: u32) -> Result<i64, ReportError> {
    let mut lo = at_utc.saturating_sub(3 * 3600);
    let mut hi = at_utc;
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let (_, _, _, found) = local_ymdh(timezone, mid)?;
        if found == hour {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    Ok(lo)
}

fn find_local_hour_end(timezone: &str, start: i64, hour: u32) -> Result<i64, ReportError> {
    let mut lo = start.saturating_add(1);
    let mut hi = start.saturating_add(3 * 3600);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let (_, _, _, found) = local_ymdh(timezone, mid)?;
        if found == hour {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    Ok(lo)
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
            | DimensionKind::Category
    )
}

pub fn plan_capability(
    query: &ReportQuery,
    now_utc: i64,
    raw_retain_days: i64,
) -> Result<CapabilityPlan, ReportError> {
    plan_capability_ex(query, now_utc, raw_retain_days, None)
}

pub fn plan_capability_ex(
    query: &ReportQuery,
    now_utc: i64,
    raw_retain_days: i64,
    hourly_dim_v2_start: Option<i64>,
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

    if query.granularity.is_minute() && query.range_start_utc < raw_cutoff {
        return Err(ReportError::CapabilityUnsupported(
            "分钟粒度只在 raw 保留期内可用",
        ));
    }
    if wants_raw && query.range_start_utc < raw_cutoff {
        return Err(ReportError::CapabilityUnsupported(
            "raw expired for sessions, current policy, or cross-dimension filters",
        ));
    }
    if wants_dim && query.grouping != DimensionKind::Category && query.range_start_utc < dim_cutoff
    {
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
        if query.grouping != DimensionKind::Host {
            if let Some(start) = hourly_dim_v2_start {
                if query.range_start_utc < start {
                    return Err(ReportError::CapabilityUnsupported(
                        "该维度的精确层从五维物化水位起可用",
                    ));
                }
            }
        }
        let hourly = matches!(query.granularity, Granularity::Hour);
        let exact_top_n = query.grouping == DimensionKind::Host
            || hourly_dim_v2_start.is_some_and(|start| query.range_start_utc >= start);
        let note_zh = if exact_top_n {
            "13 个月精确层只支持历史主分类加单一分析维度。".into()
        } else {
            "该维度尚未五维物化，精确 Top N 不可用。".into()
        };
        return Ok(CapabilityPlan {
            tier: if hourly {
                DataTier::HourlyDimension
            } else {
                DataTier::DailyDimension
            },
            named_sql: dim_named_sql(query, hourly),
            drilldown: DrilldownCapability {
                sessions: false,
                current_policy: false,
                cross_dimension: false,
                exact_top_n,
                note_zh,
            },
            deadline_ms: deadline,
        });
    }

    if wants_raw || (wants_dim && query.grouping != DimensionKind::Category) {
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
    let mut names = vec![
        "totals_raw",
        "series_raw",
        crate::c3::sql::raw_rank_sql(query.grouping),
        "coverage_raw",
    ];
    if matches!(
        query.grouping,
        DimensionKind::Host | DimensionKind::Rule | DimensionKind::Process
    ) {
        names.push(crate::c3::sql::raw_exit_sql_name(query.grouping));
    }
    if query.include_sessions {
        names.push("sessions_keyset");
    }
    if query.comparison.is_some() {
        names.push("totals_raw_compare");
    }
    names
}

fn dim_named_sql(query: &ReportQuery, hourly: bool) -> Vec<&'static str> {
    if hourly {
        vec![
            "totals_hourly_dimension",
            "series_hourly_dimension",
            crate::c3::sql::dim_rank_sql(query.grouping, true),
            "coverage_daily",
        ]
    } else {
        vec![
            "totals_daily_dimension",
            "series_daily_dimension",
            crate::c3::sql::dim_rank_sql(query.grouping, false),
            "coverage_daily",
        ]
    }
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
        attribution_quality: AttributionQuality::default(),
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
    fn empty_attribution_quality_is_complete_but_legacy_traffic_is_unavailable() {
        let empty = AttributionQuality::default();
        assert_eq!(empty.status, AttributionStatus::Complete);

        let query = ReportQuery::default();
        let plan = plan_capability(&query, 0, RAW_RETAIN_DAYS_DEFAULT).expect("plan");
        let mut legacy = empty_result(query, &plan, 1);
        legacy.totals.upload = 10;
        legacy.totals.connection_count = 1;
        legacy.reconcile_legacy_attribution_quality();
        assert_eq!(
            legacy.attribution_quality.status,
            AttributionStatus::Unavailable
        );
        assert_eq!(legacy.attribution_quality.missing_upload, 10);
        assert_eq!(legacy.attribution_quality.missing_connections, 1);
    }

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
    fn validate_query_rejects_bounds_page_top_n_timezone_and_semicolon_cursor() {
        let mut query = ReportQuery::default();
        query.range_end_utc = query.range_start_utc - 1;
        assert_eq!(
            validate_query(&query).expect_err("inverted").code(),
            "invalid_query"
        );

        query = ReportQuery::default();
        query.range_end_utc = query.range_start_utc + MAX_RANGE_SECS + 1;
        assert_eq!(
            validate_query(&query).expect_err("range").code(),
            "invalid_query"
        );

        query = ReportQuery::default();
        query.page.limit = 0;
        assert_eq!(
            validate_query(&query).expect_err("limit0").code(),
            "invalid_query"
        );

        query = ReportQuery::default();
        query.page.limit = PAGE_MAX + 1;
        assert_eq!(
            validate_query(&query).expect_err("limitmax").code(),
            "invalid_query"
        );

        query = ReportQuery::default();
        query.top_n = 0;
        assert_eq!(
            validate_query(&query).expect_err("top0").code(),
            "invalid_query"
        );

        query = ReportQuery::default();
        query.top_n = TOP_N_MAX + 1;
        assert_eq!(
            validate_query(&query).expect_err("topmax").code(),
            "invalid_query"
        );

        query = ReportQuery::default();
        query.display_timezone = "Not/A_Zone".into();
        assert_eq!(
            validate_query(&query).expect_err("tz").code(),
            "invalid_query"
        );

        query = ReportQuery::default();
        query.page.after = Some("cursor;more".into());
        assert_eq!(
            validate_query(&query).expect_err("cursor").code(),
            "invalid_query"
        );

        query = ReportQuery::default();
        query.page.after = Some("select".into());
        assert_eq!(
            validate_query(&query).expect_err("select").code(),
            "invalid_query"
        );
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
    fn local_hour_bounds_utc_and_shanghai() {
        let utc_at = chrono::NaiveDate::from_ymd_opt(2026, 1, 15)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let (start, end) = local_hour_bounds("UTC", utc_at).expect("utc");
        assert_eq!(end - start, 3_600);
        assert_eq!(start, utc_at - 1_800);
        let shanghai_ten = chrono::NaiveDate::from_ymd_opt(2026, 1, 15)
            .unwrap()
            .and_hms_opt(2, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let (closed_s, closed_e) =
            closed_local_hour_bounds("Asia/Shanghai", shanghai_ten).expect("closed");
        assert_eq!(closed_e, shanghai_ten);
        assert_eq!(closed_s, shanghai_ten - 3_600);
        let (open_s, open_e) = local_hour_bounds("Asia/Shanghai", shanghai_ten).expect("open");
        assert_eq!((open_s, open_e), (shanghai_ten, shanghai_ten + 3_600));
    }

    #[test]
    fn local_hour_bounds_chain_across_new_york_dst() {
        let spring_four = chrono::NaiveDate::from_ymd_opt(2026, 3, 8)
            .unwrap()
            .and_hms_opt(8, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let (cur_s, _) = local_hour_bounds("America/New_York", spring_four).expect("spring");
        let (closed_s, closed_e) =
            local_hour_bounds("America/New_York", cur_s.saturating_sub(1)).expect("closed");
        assert_eq!(closed_e, cur_s);
        assert!(closed_e > closed_s);
        let (prev_s, prev_e) =
            local_hour_bounds("America/New_York", closed_s.saturating_sub(1)).expect("prev");
        assert_eq!(prev_e, closed_s);
        assert!(prev_e > prev_s);
        let fall_two = chrono::NaiveDate::from_ymd_opt(2026, 11, 1)
            .unwrap()
            .and_hms_opt(7, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let (fall_s, fall_e) = local_hour_bounds("America/New_York", fall_two).expect("fall");
        assert!(fall_e > fall_s);
        let (fall_prev_s, fall_prev_e) =
            local_hour_bounds("America/New_York", fall_s.saturating_sub(1)).expect("fall prev");
        assert_eq!(fall_prev_e, fall_s);
        assert!(fall_prev_e > fall_prev_s);
        assert!(fall_prev_e - fall_prev_s >= 3_600);
        let local_now = chrono::Utc::now().timestamp();
        let (local_s, local_e) = local_hour_bounds("local", local_now).expect("local");
        assert!(local_e > local_s);
    }

    #[test]
    fn default_auto_report_query_fingerprint_is_stable() {
        let first = default_auto_report_query(Granularity::Hour, 3_600, 7_200);
        let second = default_auto_report_query(Granularity::Hour, 3_600, 7_200);
        assert_eq!(query_fingerprint(&first), query_fingerprint(&second));
        assert_eq!(first.display_timezone, "local");
        assert_eq!(first.grouping, DimensionKind::Host);
        assert_eq!(first.target_policy, TargetPolicy::Historical);
        assert_eq!(first.top_n, 20);
        assert!(!first.include_sessions);
        assert_eq!(
            first.comparison,
            Some(ComparisonSpec {
                previous_equal_window: true
            })
        );
        let day = default_auto_report_query(Granularity::Day, 0, 86_400);
        assert_eq!(day.granularity, Granularity::Day);
        assert_ne!(query_fingerprint(&first), query_fingerprint(&day));
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

    #[test]
    fn granularity_kebab_case_keeps_hour_day_month() {
        assert_eq!(
            serde_json::to_string(&Granularity::Hour).unwrap(),
            "\"hour\""
        );
        assert_eq!(serde_json::to_string(&Granularity::Day).unwrap(), "\"day\"");
        assert_eq!(
            serde_json::to_string(&Granularity::Month).unwrap(),
            "\"month\""
        );
        assert_eq!(
            serde_json::to_string(&Granularity::Minute1).unwrap(),
            "\"minute1\""
        );
        assert_eq!(
            serde_json::to_string(&Granularity::Minute2).unwrap(),
            "\"minute2\""
        );
        assert_eq!(
            serde_json::to_string(&Granularity::Minute5).unwrap(),
            "\"minute5\""
        );
        assert_eq!(
            serde_json::to_string(&Granularity::Minute10).unwrap(),
            "\"minute10\""
        );
        assert_eq!(Granularity::Minute1.bucket_minutes(), 1);
        assert_eq!(Granularity::Hour.bucket_minutes(), 60);
        assert_eq!(Granularity::Day.bucket_minutes(), 1_440);
        assert_eq!(Granularity::Month.bucket_minutes(), 43_200);
        assert_eq!(
            serde_json::from_str::<Granularity>("\"hour\"").unwrap(),
            Granularity::Hour
        );
        assert_eq!(
            serde_json::from_str::<Granularity>("\"minute1\"").unwrap(),
            Granularity::Minute1
        );
    }

    #[test]
    fn minute_granularity_outside_raw_is_unsupported() {
        let mut query = ReportQuery::default();
        query.granularity = Granularity::Minute1;
        query.range_start_utc = 0;
        query.range_end_utc = 3_600;
        let error = plan_capability(&query, 90 * 86_400, 30).expect_err("minute");
        assert_eq!(error.code(), "capability_unsupported");
        assert!(error.to_string().contains("分钟粒度"));
    }

    #[test]
    fn category_filter_counts_and_triggers_needs_raw() {
        let mut query = ReportQuery::default();
        query.filters.category = Some("家宽".into());
        assert_eq!(query.filters.dimension_filter_count(), 1);
        assert!(!needs_raw(&query));
        query.filters.host = Some("a.example".into());
        assert_eq!(query.filters.dimension_filter_count(), 2);
        assert!(needs_raw(&query));
    }

    #[test]
    fn process_dimension_without_v2_is_not_exact_top_n() {
        let mut query = ReportQuery::default();
        query.grouping = DimensionKind::Process;
        query.range_start_utc = 0;
        query.range_end_utc = 86_400;
        let now = 40 * 86_400;
        let plan = plan_capability_ex(&query, now, 30, None).expect("plan");
        assert_eq!(plan.tier, DataTier::HourlyDimension);
        assert!(!plan.drilldown.exact_top_n);
        assert!(plan.drilldown.note_zh.contains("尚未五维物化"));
    }

    #[test]
    fn process_before_v2_watermark_is_unsupported() {
        let mut query = ReportQuery::default();
        query.grouping = DimensionKind::Process;
        query.range_start_utc = 0;
        query.range_end_utc = 86_400;
        let now = 40 * 86_400;
        let error = plan_capability_ex(&query, now, 30, Some(10 * 86_400)).expect_err("watermark");
        assert_eq!(error.code(), "capability_unsupported");
        assert!(error.to_string().contains("五维物化水位"));
    }
}
