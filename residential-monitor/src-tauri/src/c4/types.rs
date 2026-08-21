//! C4 跨层 DTO。前端不计算滚动窗口或周期累计。

use crate::c3::query::{ReportError, ReportQuery};
use serde::{Deserialize, Serialize};

pub const ALERT_DTO_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlertKind {
    Health,
    Rate,
    PeriodUsage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SelectorKind {
    HealthKind,
    PrimaryCategory,
    Domain,
    Process,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlertDirection {
    Upload,
    Download,
    Combined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlertPeriod {
    Rolling1h,
    LocalDay,
    LocalMonth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstanceStatus {
    Inactive,
    Active,
    NotEvaluable,
    Resolved,
    Superseded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventKind {
    Activated,
    Recovered,
    EvaluationGap,
    Superseded,
    NotEvaluable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutboxStatus {
    Pending,
    Leased,
    Retry,
    Sent,
    Failed,
    Suppressed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertRule {
    pub rule_id: String,
    pub version: i64,
    pub enabled: bool,
    pub kind: AlertKind,
    pub selector_kind: SelectorKind,
    pub selector_value: Option<String>,
    pub direction: Option<AlertDirection>,
    pub threshold_value: i64,
    pub recovery_threshold: Option<i64>,
    pub period: Option<AlertPeriod>,
    pub timezone: String,
    pub cooldown_sec: i64,
    pub quiet_start_min: Option<i64>,
    pub quiet_end_min: Option<i64>,
    pub created_utc: i64,
    pub updated_utc: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertEvidence {
    pub rule_id: String,
    pub rule_version: i64,
    pub data_version: Option<u64>,
    pub evaluated_at_utc: i64,
    pub window_start_utc: Option<i64>,
    pub window_end_utc: Option<i64>,
    pub display_timezone: String,
    pub selector: String,
    pub direction: Option<AlertDirection>,
    pub observed_value: Option<i64>,
    pub trigger_threshold: i64,
    pub recovery_threshold: Option<i64>,
    pub coverage_summary: String,
    pub policy_metadata: Option<String>,
    pub report_query: Option<ReportQuery>,
    pub not_evaluable_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertInstance {
    pub instance_id: String,
    pub rule_id: String,
    pub rule_version: i64,
    pub selector_identity: String,
    pub status: InstanceStatus,
    pub started_utc: Option<i64>,
    pub resolved_utc: Option<i64>,
    pub last_eval_utc: i64,
    pub last_observed: Option<i64>,
    pub evidence: AlertEvidence,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertEvent {
    pub event_id: String,
    pub instance_id: String,
    pub bundle_id: String,
    pub kind: EventKind,
    pub at_utc: i64,
    pub evidence: AlertEvidence,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboxIntent {
    pub outbox_id: String,
    pub event_id: String,
    pub bundle_id: String,
    pub status: OutboxStatus,
    pub attempt: i64,
    pub next_attempt_at: i64,
    pub lease_until: Option<i64>,
    pub lease_token: Option<String>,
    pub error_class: Option<String>,
    pub error_summary: Option<String>,
    pub idempotency_key: String,
    pub created_utc: i64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AlertWriteSet {
    pub instances: Vec<AlertInstance>,
    pub events: Vec<AlertEvent>,
    pub outbox: Vec<OutboxIntent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertCenterPage {
    pub schema_version: u32,
    pub items: Vec<AlertInstance>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertSummary {
    pub schema_version: u32,
    pub active_count: u32,
    pub not_evaluable_count: u32,
    pub outbox_backlog: u32,
    pub last_event_utc: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertError {
    InvalidRule(&'static str),
    NotFound,
    Storage,
    NotEvaluable(&'static str),
}

impl AlertError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidRule(_) => "invalid_rule",
            Self::NotFound => "not_found",
            Self::Storage => "storage_failure",
            Self::NotEvaluable(_) => "not_evaluable",
        }
    }

    pub fn message_zh(&self) -> &'static str {
        match self {
            Self::InvalidRule(_) => "规则参数无效。",
            Self::NotFound => "告警不存在。",
            Self::Storage => "告警存储失败。",
            Self::NotEvaluable(_) => "当前无法评估该规则。",
        }
    }

    pub fn action_zh(&self) -> &'static str {
        match self {
            Self::InvalidRule(_) => "检查阈值、滞回、时区和周期",
            Self::NotFound => "刷新告警中心",
            Self::Storage => "打开数据管理检查磁盘",
            Self::NotEvaluable(_) => "查看覆盖或缩小查询范围",
        }
    }
}

pub fn map_report_to_not_evaluable(error: &ReportError) -> &'static str {
    match error {
        ReportError::CapabilityUnsupported(_) => "capability_unsupported",
        ReportError::DeadlineExceeded(_) => "deadline_exceeded",
        ReportError::Cancelled(_) => "cancelled",
        ReportError::TokenExpired(_) => "token_expired",
        ReportError::QuotaExceeded(_) => "quota_exceeded",
        ReportError::StorageBusy(_) => "storage_busy",
        ReportError::InvalidQuery(_) => "invalid_query",
        ReportError::InsufficientSpace(_) => "insufficient_space",
        ReportError::Failed(_) => "report_failed",
    }
}

pub fn validate_rule(rule: &AlertRule) -> Result<(), AlertError> {
    if rule.rule_id.is_empty() || rule.rule_id.len() > 128 {
        return Err(AlertError::InvalidRule("rule id"));
    }
    if rule.cooldown_sec < 0 || rule.cooldown_sec > 86_400 {
        return Err(AlertError::InvalidRule("cooldown"));
    }
    if rule.threshold_value <= 0 {
        return Err(AlertError::InvalidRule("threshold"));
    }
    match rule.kind {
        AlertKind::Rate | AlertKind::PeriodUsage => {
            if rule.direction.is_none() {
                return Err(AlertError::InvalidRule("direction"));
            }
            let recovery = rule
                .recovery_threshold
                .ok_or(AlertError::InvalidRule("recovery"))?;
            if recovery <= 0 || recovery >= rule.threshold_value {
                return Err(AlertError::InvalidRule("hysteresis"));
            }
        }
        AlertKind::Health => {
            if rule.selector_kind != SelectorKind::HealthKind {
                return Err(AlertError::InvalidRule("health selector"));
            }
            if rule.selector_value.as_deref().is_none_or(str::is_empty) {
                return Err(AlertError::InvalidRule("health kind"));
            }
        }
    }
    if rule.kind == AlertKind::PeriodUsage {
        if rule.period.is_none() {
            return Err(AlertError::InvalidRule("period"));
        }
        if rule.timezone.is_empty() {
            return Err(AlertError::InvalidRule("timezone"));
        }
    }
    Ok(())
}

pub fn selector_identity(kind: SelectorKind, value: &str) -> String {
    format!("{}:{value}", selector_kind_name(kind))
}

pub fn selector_kind_name(kind: SelectorKind) -> &'static str {
    match kind {
        SelectorKind::HealthKind => "health_kind",
        SelectorKind::PrimaryCategory => "primary_category",
        SelectorKind::Domain => "domain",
        SelectorKind::Process => "process",
    }
}

pub fn health_root_keys() -> &'static [&'static str] {
    &[
        "disconnect",
        "tcp_auth",
        "protocol",
        "collection_gap",
        "storage",
        "migration",
        "backup",
    ]
}
