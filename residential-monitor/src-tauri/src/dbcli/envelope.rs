//! 查询与维护命令共用的 JSON envelope。

use crate::c3::query::{AttributionQuality, AttributionStatus, CoverageView, ReportError};
use serde::Serialize;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope {
    pub schema_version: u32,
    pub command: String,
    pub generated_utc: i64,
    pub window: WindowEcho,
    pub data_version: u64,
    pub capability: CapabilityEcho,
    pub coverage: CoverageEcho,
    pub attribution_quality: AttributionEcho,
    pub truncation: TruncationEcho,
    pub named_sql: Vec<String>,
    pub result: serde_json::Value,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowEcho {
    pub start_utc: i64,
    pub end_utc: i64,
    pub timezone: String,
    pub granularity: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityEcho {
    pub layer: String,
    pub supported: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageEcho {
    pub observed_sec: Option<i64>,
    pub gap_sec: Option<i64>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributionEcho {
    pub status: String,
    pub known_bytes: Option<i64>,
    pub missing_bytes: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TruncationEcho {
    pub status: String,
    pub row_cap: i64,
    pub rows: i64,
}

impl Envelope {
    pub fn unsupported(
        command: &str,
        window: WindowEcho,
        now_utc: i64,
        reason: &str,
        notes: Vec<String>,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            command: command.into(),
            generated_utc: now_utc,
            window,
            data_version: 0,
            capability: CapabilityEcho {
                layer: "raw".into(),
                supported: false,
                reason: Some(reason.into()),
            },
            coverage: CoverageEcho {
                observed_sec: None,
                gap_sec: None,
                status: "unknown".into(),
            },
            attribution_quality: AttributionEcho {
                status: "unavailable".into(),
                known_bytes: None,
                missing_bytes: None,
            },
            truncation: TruncationEcho {
                status: "complete".into(),
                row_cap: 0,
                rows: 0,
            },
            named_sql: Vec::new(),
            result: serde_json::Value::Null,
            notes,
        }
    }
}

pub fn coverage_from_view(view: &CoverageView) -> CoverageEcho {
    CoverageEcho {
        observed_sec: Some(view.covered_sec),
        gap_sec: Some(view.gap_sec),
        status: view.status.clone(),
    }
}

pub fn coverage_from_status(
    status: &str,
    observed_sec: Option<i64>,
    gap_sec: Option<i64>,
) -> CoverageEcho {
    CoverageEcho {
        observed_sec,
        gap_sec,
        status: status.into(),
    }
}

pub fn attribution_from_quality(quality: &AttributionQuality) -> AttributionEcho {
    let known = quality.known_upload.saturating_add(quality.known_download);
    let missing = quality
        .missing_upload
        .saturating_add(quality.missing_download);
    AttributionEcho {
        status: match quality.status {
            AttributionStatus::Complete => "complete",
            AttributionStatus::Partial => "partial",
            AttributionStatus::Unavailable => "unavailable",
        }
        .into(),
        known_bytes: Some(known),
        missing_bytes: Some(missing),
    }
}

#[allow(dead_code)]
pub fn capability_reason(error: &ReportError) -> Option<String> {
    match error {
        ReportError::CapabilityUnsupported(reason) => Some((*reason).into()),
        _ => None,
    }
}
