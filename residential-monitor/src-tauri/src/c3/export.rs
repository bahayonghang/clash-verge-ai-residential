//! 流式导出：只消费 snapshot token，不重新查询。

use crate::c3::query::{ReportError, ReportResult, TargetPolicy};
use crate::c3::space::SpaceBudget;
use crate::i18n::{t, UiLocale};
use serde::{Deserialize, Serialize};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const WRITE_BUFFER: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportFormat {
    Csv,
    Json,
    Html,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RedactMode {
    None,
    Mask,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSpec {
    pub format: ExportFormat,
    pub include_series: bool,
    pub include_rankings: bool,
    pub include_sessions: bool,
    pub redact_host: RedactMode,
    pub redact_process: RedactMode,
    #[serde(default)]
    pub ui_locale: UiLocale,
}

impl Default for ExportSpec {
    fn default() -> Self {
        Self {
            format: ExportFormat::Csv,
            include_series: true,
            include_rankings: true,
            include_sessions: false,
            redact_host: RedactMode::None,
            redact_process: RedactMode::None,
            ui_locale: UiLocale::Zh,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPreview {
    pub format: ExportFormat,
    pub row_count: usize,
    pub sample_labels: Vec<String>,
    pub metadata_zh: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HtmlDocument {
    pub html: String,
}

pub struct ExportService;

impl ExportService {
    pub fn preview(result: &ReportResult, spec: &ExportSpec) -> Result<ExportPreview, ReportError> {
        reject_secret(result)?;
        let labels = result
            .rankings
            .iter()
            .take(5)
            .map(|row| redact_label(&row.label, spec.redact_host))
            .collect();
        Ok(ExportPreview {
            format: spec.format,
            row_count: result.series.len() + result.rankings.len() + result.sessions.len(),
            sample_labels: labels,
            metadata_zh: metadata_line(result),
        })
    }

    pub fn render_html(
        result: &ReportResult,
        spec: &ExportSpec,
        cancel: &Arc<AtomicBool>,
    ) -> Result<String, ReportError> {
        reject_secret(result)?;
        let mut buf = Vec::new();
        write_html(&mut buf, result, spec, cancel)?;
        String::from_utf8(buf).map_err(|_| ReportError::Failed("html utf8"))
    }

    pub fn export_to_path(
        result: &ReportResult,
        spec: &ExportSpec,
        dest: &Path,
        space: &SpaceBudget,
        cancel: &Arc<AtomicBool>,
    ) -> Result<PathBuf, ReportError> {
        reject_secret(result)?;
        if dest.exists() {
            return Err(ReportError::Failed("destination exists"));
        }
        let parent = dest.parent().unwrap_or_else(|| Path::new("."));
        space.check(parent, 64 * 1024)?;
        let partial = parent.join(format!(
            "{}.partial",
            dest.file_name()
                .and_then(|item| item.to_str())
                .unwrap_or("export")
        ));
        let _ = std::fs::remove_file(&partial);
        let outcome = write_partial(result, spec, &partial, cancel);
        if outcome.is_err() {
            let _ = std::fs::remove_file(&partial);
            return outcome.map(|_| dest.to_path_buf());
        }
        std::fs::rename(&partial, dest).map_err(|_| ReportError::Failed("atomic rename"))?;
        Ok(dest.to_path_buf())
    }
}

fn write_partial(
    result: &ReportResult,
    spec: &ExportSpec,
    partial: &Path,
    cancel: &Arc<AtomicBool>,
) -> Result<(), ReportError> {
    let file = std::fs::File::create(partial).map_err(|_| ReportError::Failed("create partial"))?;
    let mut writer = BufWriter::with_capacity(WRITE_BUFFER, file);
    match spec.format {
        ExportFormat::Csv => write_csv(&mut writer, result, spec, cancel)?,
        ExportFormat::Json => write_json(&mut writer, result, spec, cancel)?,
        ExportFormat::Html => write_html(&mut writer, result, spec, cancel)?,
    }
    writer.flush().map_err(|_| ReportError::Failed("flush"))?;
    Ok(())
}

fn write_csv<W: Write>(
    writer: &mut W,
    result: &ReportResult,
    spec: &ExportSpec,
    cancel: &Arc<AtomicBool>,
) -> Result<(), ReportError> {
    writeln!(writer, "# {}", metadata_line(result)).map_err(|_| ReportError::Failed("csv"))?;
    writeln!(
        writer,
        "kind,identity,label,bucket_utc,upload,download,connection_count,active_duration_sec"
    )
    .map_err(|_| ReportError::Failed("csv"))?;
    writeln!(
        writer,
        "totals,,,{},{},{},{},{}",
        result.generated_utc,
        result.totals.upload,
        result.totals.download,
        result.totals.connection_count,
        result.totals.active_duration_sec
    )
    .map_err(|_| ReportError::Failed("csv"))?;
    if spec.include_series {
        for point in &result.series {
            check_cancel(cancel)?;
            writeln!(
                writer,
                "series,,,{},{},{},{},{}",
                point.bucket_utc,
                point.upload,
                point.download,
                point.connection_count,
                point.active_duration_sec
            )
            .map_err(|_| ReportError::Failed("csv"))?;
        }
    }
    if spec.include_rankings {
        for row in &result.rankings {
            check_cancel(cancel)?;
            let label = csv_escape(&redact_label(&row.label, spec.redact_host));
            writeln!(
                writer,
                "ranking,{},{},,{},{},{},{}",
                csv_escape(&row.identity),
                label,
                row.upload,
                row.download,
                row.connection_count,
                row.active_duration_sec
            )
            .map_err(|_| ReportError::Failed("csv"))?;
        }
    }
    if spec.include_sessions {
        for row in &result.sessions {
            check_cancel(cancel)?;
            let host = row
                .host
                .as_deref()
                .map(|value| redact_label(value, spec.redact_host))
                .unwrap_or_default();
            writeln!(
                writer,
                "session,{},{},{},{},{},,",
                csv_escape(&row.identity),
                csv_escape(&host),
                row.started_utc,
                row.upload,
                row.download
            )
            .map_err(|_| ReportError::Failed("csv"))?;
        }
    }
    Ok(())
}

fn write_json<W: Write>(
    writer: &mut W,
    result: &ReportResult,
    spec: &ExportSpec,
    cancel: &Arc<AtomicBool>,
) -> Result<(), ReportError> {
    check_cancel(cancel)?;
    let mut clone = result.clone();
    if spec.redact_host == RedactMode::Mask {
        for row in &mut clone.rankings {
            row.label = redact_label(&row.label, RedactMode::Mask);
            row.identity = redact_label(&row.identity, RedactMode::Mask);
        }
        for row in &mut clone.sessions {
            if let Some(host) = &row.host {
                row.host = Some(redact_label(host, RedactMode::Mask));
            }
        }
    }
    if !spec.include_series {
        clone.series.clear();
    }
    if !spec.include_rankings {
        clone.rankings.clear();
    }
    if !spec.include_sessions {
        clone.sessions.clear();
    }
    serde_json::to_writer(writer, &clone).map_err(|_| ReportError::Failed("json"))?;
    Ok(())
}

fn write_html<W: Write>(
    writer: &mut W,
    result: &ReportResult,
    spec: &ExportSpec,
    cancel: &Arc<AtomicBool>,
) -> Result<(), ReportError> {
    check_cancel(cancel)?;
    let locale = spec.ui_locale;
    let tz = &result.query_echo.display_timezone;
    write!(
        writer,
        "<!DOCTYPE html><html lang=\"{}\"><head><meta charset=\"utf-8\"><title>{}</title>\
<style>body{{font-family:sans-serif;background:#12161c;color:#e8eef6;margin:24px;max-width:960px}}\
table{{border-collapse:collapse;width:100%}}\
td,th{{border-bottom:1px solid #2a3340;padding:6px;font-variant-numeric:tabular-nums;text-align:left}}\
td:nth-child(n+2),th:nth-child(n+2){{text-align:right}}\
pre{{white-space:pre-wrap;color:#9aa7b5;font-size:12px}}\
@media print{{body{{background:#fff;color:#000}}td,th{{border-bottom:1px solid #000}}}}</style></head><body>",
        locale.html_lang(),
        t(locale, "export.html_title")
    )
    .map_err(|_| ReportError::Failed("html"))?;
    let window = t(locale, "export.html_window")
        .replacen(
            "{}",
            &format_html_time(result.query_echo.range_start_utc, tz),
            1,
        )
        .replacen(
            "{}",
            &format_html_time(result.query_echo.range_end_utc, tz),
            1,
        );
    let generated = t(locale, "export.html_generated").replacen(
        "{}",
        &format_html_time(result.generated_utc, tz),
        1,
    );
    let version = result
        .policy_metadata
        .policy_version
        .map(|value| format!("v{value}"))
        .unwrap_or_default();
    let policy = t(locale, "export.html_policy")
        .replacen("{}", policy_label(result.policy_metadata.target_policy), 1)
        .replacen("{}", &version, 1);
    let totals = t(locale, "export.html_totals")
        .replacen("{}", &result.totals.upload.to_string(), 1)
        .replacen("{}", &result.totals.download.to_string(), 1)
        .replacen("{}", &html_escape(&result.coverage.status), 1);
    writeln!(
        writer,
        "<h1>{}</h1><p>{}</p><p>{}</p><p>{}</p><p>{}</p><pre>{}</pre>",
        html_escape(t(locale, "export.html_title")),
        html_escape(&window),
        html_escape(&generated),
        html_escape(&policy),
        totals,
        html_escape(&metadata_line(result))
    )
    .map_err(|_| ReportError::Failed("html"))?;
    if spec.include_rankings {
        writeln!(
            writer,
            "<h2>{}</h2><table><thead><tr><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody>",
            t(locale, "export.html_rankings"),
            t(locale, "export.col_name"),
            t(locale, "export.col_upload"),
            t(locale, "export.col_download")
        )
        .map_err(|_| ReportError::Failed("html"))?;
        for row in &result.rankings {
            check_cancel(cancel)?;
            writeln!(
                writer,
                "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                html_escape(&redact_label(&row.label, spec.redact_host)),
                row.upload,
                row.download
            )
            .map_err(|_| ReportError::Failed("html"))?;
        }
        writeln!(writer, "</tbody></table>").map_err(|_| ReportError::Failed("html"))?;
    }
    if spec.include_series {
        writeln!(
            writer,
            "<h2>{}</h2><table><thead><tr><th>{}</th><th>{}</th><th>{}</th></tr></thead><tbody>",
            t(locale, "export.html_series"),
            t(locale, "export.col_time"),
            t(locale, "export.col_upload"),
            t(locale, "export.col_download")
        )
        .map_err(|_| ReportError::Failed("html"))?;
        for point in &result.series {
            writeln!(
                writer,
                "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                html_escape(&format_html_time(point.bucket_utc, tz)),
                point.upload,
                point.download
            )
            .map_err(|_| ReportError::Failed("html"))?;
        }
        writeln!(writer, "</tbody></table>").map_err(|_| ReportError::Failed("html"))?;
    }
    writeln!(writer, "</body></html>").map_err(|_| ReportError::Failed("html"))?;
    Ok(())
}

fn format_html_time(ts: i64, timezone: &str) -> String {
    let Some(utc) = chrono::DateTime::from_timestamp(ts, 0) else {
        return ts.to_string();
    };
    if timezone.eq_ignore_ascii_case("utc") {
        return utc.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    }
    utc.with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn policy_label(policy: TargetPolicy) -> &'static str {
    match policy {
        TargetPolicy::Current => "current",
        TargetPolicy::Historical => "historical",
    }
}

fn metadata_line(result: &ReportResult) -> String {
    format!(
        "utc={}..{} tz={} unit={} policy={:?} policy_version={:?} schema={} data={} generated={} coverage={} covered={} gap={} sessions={} currentPolicy={} crossDimension={} exactTopN={} token={}",
        result.query_echo.range_start_utc,
        result.query_echo.range_end_utc,
        result.query_echo.display_timezone,
        result.unit,
        result.policy_metadata.target_policy,
        result.policy_metadata.policy_version,
        result.schema_version,
        result.data_version,
        result.generated_utc,
        result.coverage.status,
        result.coverage.covered_sec,
        result.coverage.gap_sec,
        result.drilldown_capability.sessions,
        result.drilldown_capability.current_policy,
        result.drilldown_capability.cross_dimension,
        result.drilldown_capability.exact_top_n,
        result.report_snapshot_token
    )
}

fn redact_label(value: &str, mode: RedactMode) -> String {
    if mode == RedactMode::None || value.is_empty() {
        return value.to_string();
    }
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 2 {
        return "*".repeat(chars.len());
    }
    format!("{}***{}", chars[0], chars[chars.len() - 1])
}

fn reject_secret(result: &ReportResult) -> Result<(), ReportError> {
    let blob = serde_json::to_string(result).unwrap_or_default();
    let lower = blob.to_ascii_lowercase();
    if lower.contains("bearer ") || lower.contains("password=") || lower.contains("secret=") {
        return Err(ReportError::Failed("secret in report"));
    }
    Ok(())
}

fn check_cancel(cancel: &Arc<AtomicBool>) -> Result<(), ReportError> {
    if cancel.load(Ordering::SeqCst) {
        return Err(ReportError::Cancelled("export"));
    }
    Ok(())
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod export_tests {
    use super::*;
    use crate::c3::query::{empty_result, plan_capability, ReportQuery};
    use crate::c3::space::SpaceBudget;
    use tempfile::tempdir;

    fn sample() -> ReportResult {
        let query = ReportQuery::default();
        let plan = plan_capability(&query, 4_000, 30).expect("plan");
        let mut result = empty_result(query, &plan, 7);
        result.report_snapshot_token = "tok".into();
        result.totals.upload = 10;
        result.totals.download = 20;
        result.rankings.push(crate::c3::query::RankingRow {
            identity: "a.example".into(),
            label: "a.example".into(),
            upload: 10,
            download: 20,
            connection_count: 1,
            active_duration_sec: 60,
        });
        result
    }

    #[test]
    fn three_formats_share_totals_and_do_not_overwrite() {
        let dir = tempdir().expect("dir");
        let result = sample();
        let spec = ExportSpec::default();
        let cancel = Arc::new(AtomicBool::new(false));
        let csv = ExportService::export_to_path(
            &result,
            &spec,
            &dir.path().join("r.csv"),
            &SpaceBudget::unlimited(),
            &cancel,
        )
        .expect("csv");
        let json = ExportService::export_to_path(
            &result,
            &ExportSpec {
                format: ExportFormat::Json,
                ..spec.clone()
            },
            &dir.path().join("r.json"),
            &SpaceBudget::unlimited(),
            &cancel,
        )
        .expect("json");
        let html = ExportService::export_to_path(
            &result,
            &ExportSpec {
                format: ExportFormat::Html,
                ..spec
            },
            &dir.path().join("r.html"),
            &SpaceBudget::unlimited(),
            &cancel,
        )
        .expect("html");
        let csv_text = std::fs::read_to_string(csv).expect("read");
        let json_text = std::fs::read_to_string(json).expect("read");
        let html_text = std::fs::read_to_string(html).expect("read");
        assert!(csv_text.contains(",10,20,"));
        assert!(csv_text.contains("policy_version="));
        assert!(csv_text.contains("coverage="));
        assert!(csv_text.contains("currentPolicy="));
        assert!(json_text.contains("\"download\":20"));
        assert!(json_text.contains("\"policyVersion\""));
        assert!(json_text.contains("\"drilldownCapability\""));
        assert!(html_text.contains("下行 20"));
        assert!(html_text.contains("policy_version="));
        assert!(html_text.contains("@media print"));
        assert!(html_text.contains("创建时间"));
        assert!(html_text.contains("统计窗口"));
        assert!(!html_text.contains("http://"));
        assert!(!html_text.to_ascii_lowercase().contains("<script"));
        let rendered = ExportService::render_html(
            &result,
            &ExportSpec {
                format: ExportFormat::Html,
                ..ExportSpec::default()
            },
            &cancel,
        )
        .expect("render");
        assert_eq!(rendered, html_text);
        assert!(!csv_text.to_ascii_lowercase().contains("bearer "));
        assert!(!csv_text.to_ascii_lowercase().contains("password="));
        assert!(!json_text.to_ascii_lowercase().contains("secret="));
        let error = ExportService::export_to_path(
            &result,
            &ExportSpec::default(),
            &dir.path().join("r.csv"),
            &SpaceBudget::unlimited(),
            &cancel,
        )
        .expect_err("exists");
        assert_eq!(error.code(), "storage_failure");
    }

    #[test]
    fn cancel_cleans_partial_and_low_space_fails() {
        let dir = tempdir().expect("dir");
        let result = sample();
        let cancel = Arc::new(AtomicBool::new(true));
        let dest = dir.path().join("x.csv");
        let error = ExportService::export_to_path(
            &result,
            &ExportSpec::default(),
            &dest,
            &SpaceBudget::unlimited(),
            &cancel,
        )
        .expect_err("cancel");
        assert_eq!(error.code(), "cancelled");
        assert!(!dest.exists());
        assert!(!dir.path().join("x.csv.partial").exists());
        let error = ExportService::export_to_path(
            &result,
            &ExportSpec::default(),
            &dir.path().join("y.csv"),
            &SpaceBudget::exhausted(),
            &Arc::new(AtomicBool::new(false)),
        )
        .expect_err("space");
        assert_eq!(error.code(), "insufficient_space");
    }
}
