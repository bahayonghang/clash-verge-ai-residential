//! 单事务家宽审计：死规则、越界展开、开关聚合。

use super::envelope::{
    attribution_from_quality, coverage_from_status, CapabilityEcho, Envelope, TruncationEcho,
    WindowEcho, SCHEMA_VERSION,
};
use super::patterns::{
    match_host, parse_rules, pattern_key, DomainPattern, RulesFile, SwitchesFile,
};
use super::{data_version, map_report, open_reader, redact_host, DbCliError, QueryOptions};
use crate::c3::query::{
    plan_capability, AttributionQuality, DimensionKind, Granularity, ReportError, ReportFilters,
    ReportQuery, RAW_RETAIN_DAYS_DEFAULT, REPORT_DEADLINE_MS,
};
use crate::c3::service::attach_cancel;
use crate::c3::share::query_residential_share_on;
use crate::c3::sql::{
    render_residential_membership_sql, AUDIT_MAX_ROWS, AUDIT_RESIDENTIAL_HOST_RULE_PROCESS,
    RESIDENTIAL_ACCOUNTING_FILTER,
};
use rusqlite::params;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

const UNKNOWN: &str = "__unknown__";

#[derive(Debug, Clone)]
struct ProjectionRow {
    host: String,
    rule: String,
    process: String,
    upload: i64,
    download: i64,
    connections: i64,
}

impl ProjectionRow {
    fn bytes(&self) -> i64 {
        self.upload.saturating_add(self.download)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PatternRow {
    pattern: String,
    hosts: i64,
    upload: Option<i64>,
    download: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    zero_flow: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HostRow {
    host: String,
    upload: Option<i64>,
    download: Option<i64>,
    unknown: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SwitchRow {
    switch: String,
    upload: Option<i64>,
    download: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    zero_flow: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    switches: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct OutboundRow {
    source: String,
    rule: String,
    process: String,
    upload: i64,
    download: i64,
    connections: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuditResult {
    covered: Vec<PatternRow>,
    dead: Vec<PatternRow>,
    unsupported_pattern: Vec<PatternRow>,
    uncovered: Vec<HostRow>,
    mapped: Vec<SwitchRow>,
    shared: Vec<SwitchRow>,
    unmapped: Vec<SwitchRow>,
    unsupported_switch: Vec<SwitchRow>,
    outbound: Vec<OutboundRow>,
    residential_upload: Option<i64>,
    residential_download: Option<i64>,
    attributed_upload: Option<i64>,
    attributed_download: Option<i64>,
    conservation_holds: Option<bool>,
}

pub fn run_audit(
    db: &Path,
    options: &QueryOptions,
    rules_path: &Path,
    map_path: Option<&Path>,
    cancel: &Arc<AtomicBool>,
) -> Result<(Envelope, i32), DbCliError> {
    let rules: RulesFile = read_json(rules_path)?;
    let switches = match map_path {
        Some(path) => Some(read_json::<SwitchesFile>(path)?),
        None => None,
    };
    let parsed = parse_rules(&rules);
    let probe = residential_probe(options);
    let now_utc = options.now_utc;
    match plan_capability(&probe, now_utc, RAW_RETAIN_DAYS_DEFAULT) {
        Err(error @ ReportError::CapabilityUnsupported(_)) => {
            let reason = error.to_string();
            return Ok((
                Envelope::unsupported(
                    "audit",
                    window_echo(options),
                    now_utc,
                    &reason,
                    vec!["能力不支持时仍返回完整 envelope。".into()],
                ),
                6,
            ));
        }
        Err(error) => return Err(map_report(error)),
        Ok(_) => {}
    }

    let reader = open_reader(db)?;
    attach_cancel(
        &reader,
        cancel,
        Instant::now(),
        Duration::from_millis(REPORT_DEADLINE_MS),
    )
    .map_err(map_report)?;
    reader
        .execute_batch("begin deferred")
        .map_err(|_| DbCliError::Busy("begin deferred".into()))?;
    let built = build_audit(&reader, options, &parsed, switches.as_ref());
    let close = reader.execute_batch("commit");
    let txn_open = !reader.is_autocommit();
    if txn_open {
        let _ = reader.execute_batch("rollback");
    }
    drop(reader);
    let (mut envelope, code) = built?;
    close.map_err(|_| DbCliError::FailClosed("提交读事务失败".into()))?;
    if txn_open {
        return Err(DbCliError::FailClosed("读事务仍未关闭".into()));
    }
    if options.redact {
        redact_result(&mut envelope);
    }
    Ok((envelope, code))
}

fn build_audit(
    connection: &rusqlite::Connection,
    options: &QueryOptions,
    parsed: &super::patterns::ParsedRules,
    switches: Option<&SwitchesFile>,
) -> Result<(Envelope, i32), DbCliError> {
    let start_min = options.start_utc.div_euclid(60);
    let end_min = options.end_utc.div_euclid(60);
    let data_version = data_version(connection)?;
    let sql = render_residential_membership_sql(AUDIT_RESIDENTIAL_HOST_RULE_PROCESS);
    let mut statement = connection
        .prepare(&sql)
        .map_err(|_| DbCliError::FailClosed("准备审计投影失败".into()))?;
    let rows = statement
        .query_map(params![start_min, end_min, AUDIT_MAX_ROWS], |row| {
            Ok(ProjectionRow {
                host: row.get::<_, String>(0)?,
                rule: row.get::<_, String>(1)?,
                process: row.get::<_, String>(2)?,
                upload: row.get(3)?,
                download: row.get(4)?,
                connections: row.get(5)?,
            })
        })
        .map_err(map_sqlite_cli)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_sqlite_cli)?;
    let truncated = rows.len() as i64 == AUDIT_MAX_ROWS;
    let share = query_residential_share_on(
        connection,
        options.start_utc,
        options.end_utc,
        options.now_utc,
    )
    .map_err(map_report)?;

    let mut notes = vec![
        "模式优先级 exact > 最长 suffix > regex > 输入顺序；这是字节归属规则，不模拟 Mihomo 首个规则命中。".into(),
        format!("过期 DELETE 关闭：auto_delete_enabled={}", crate::c3::query::AUTO_DELETE_ENABLED),
    ];
    if parsed.skipped_group > 0 {
        notes.push(format!(
            "已跳过 {} 条目标组不匹配的规则",
            parsed.skipped_group
        ));
    }

    let uncovered_status = share.coverage_status == "uncovered";
    let null_bytes = truncated || uncovered_status;
    let mut result = bucketize(&rows, parsed, switches, null_bytes, options.redact);
    if !null_bytes {
        result.residential_upload = share.residential_upload.map(|v| v as i64);
        result.residential_download = share.residential_download.map(|v| v as i64);
        result.attributed_upload = share.attributed_upload.map(|v| v as i64);
        result.attributed_download = share.attributed_download.map(|v| v as i64);
        let projected: i64 = rows.iter().map(ProjectionRow::bytes).sum();
        let share_bytes = result
            .residential_upload
            .unwrap_or(0)
            .saturating_add(result.residential_download.unwrap_or(0));
        if projected != share_bytes {
            notes.push(format!(
                "投影字节之和 {projected} 与份额分子 {share_bytes} 不一致"
            ));
        }
        result.conservation_holds = Some(conservation_holds(&result));
    } else {
        result.residential_upload = None;
        result.residential_download = None;
        result.attributed_upload = None;
        result.attributed_download = None;
        result.conservation_holds = None;
        null_conservation(&mut result);
        if truncated {
            notes.push("投影截断，守恒字段为 null。".into());
        }
    }

    let known = result
        .residential_upload
        .unwrap_or(0)
        .saturating_add(result.residential_download.unwrap_or(0));
    let envelope = Envelope {
        schema_version: SCHEMA_VERSION,
        command: "audit".into(),
        generated_utc: options.now_utc,
        window: window_echo(options),
        data_version,
        capability: CapabilityEcho {
            layer: "raw".into(),
            supported: true,
            reason: None,
        },
        coverage: coverage_from_status(
            &share.coverage_status,
            if uncovered_status { None } else { Some(1) },
            None,
        ),
        attribution_quality: attribution_from_quality(&if uncovered_status {
            AttributionQuality::default()
        } else {
            AttributionQuality::from_parts(known, 0, 0, 0, 1, 0)
        }),
        truncation: TruncationEcho {
            status: if truncated { "truncated" } else { "complete" }.into(),
            row_cap: AUDIT_MAX_ROWS,
            rows: rows.len() as i64,
        },
        named_sql: vec![
            "coverage_raw".into(),
            "share_residential_raw".into(),
            "audit_residential_host_rule_process".into(),
        ],
        result: serde_json::to_value(&result).unwrap_or(serde_json::Value::Null),
        notes,
    };
    Ok((envelope, 0))
}

fn bucketize(
    rows: &[ProjectionRow],
    parsed: &super::patterns::ParsedRules,
    switches: Option<&SwitchesFile>,
    null_bytes: bool,
    redact: bool,
) -> AuditResult {
    let mut covered_idx: BTreeSet<usize> = BTreeSet::new();
    let mut pattern_bytes: BTreeMap<usize, (i64, i64, BTreeSet<String>)> = BTreeMap::new();
    let mut uncovered_hosts: BTreeMap<String, (i64, i64)> = BTreeMap::new();
    let mut uncovered_rows: Vec<&ProjectionRow> = Vec::new();
    for row in rows {
        match match_host(&row.host, &parsed.domain) {
            Some(index) => {
                covered_idx.insert(index);
                let entry = pattern_bytes
                    .entry(index)
                    .or_insert((0, 0, BTreeSet::new()));
                entry.0 += row.upload;
                entry.1 += row.download;
                entry.2.insert(row.host.clone());
            }
            None => {
                let entry = uncovered_hosts.entry(row.host.clone()).or_insert((0, 0));
                entry.0 += row.upload;
                entry.1 += row.download;
                uncovered_rows.push(row);
            }
        }
    }

    let covered = parsed
        .domain
        .iter()
        .enumerate()
        .filter(|(index, _)| covered_idx.contains(index))
        .map(|(index, pattern)| {
            let (upload, download, hosts) =
                pattern_bytes
                    .get(&index)
                    .cloned()
                    .unwrap_or((0, 0, BTreeSet::new()));
            PatternRow {
                pattern: display_pattern(pattern, redact),
                hosts: hosts.len() as i64,
                upload: if null_bytes { None } else { Some(upload) },
                download: if null_bytes { None } else { Some(download) },
                zero_flow: None,
                reason: None,
            }
        })
        .collect::<Vec<_>>();
    let dead = parsed
        .domain
        .iter()
        .enumerate()
        .filter(|(index, _)| !covered_idx.contains(index))
        .map(|(_, pattern)| PatternRow {
            pattern: display_pattern(pattern, redact),
            hosts: 0,
            upload: if null_bytes { None } else { Some(0) },
            download: if null_bytes { None } else { Some(0) },
            zero_flow: if null_bytes { None } else { Some(true) },
            reason: None,
        })
        .collect::<Vec<_>>();
    let unsupported_pattern = parsed
        .unsupported
        .iter()
        .map(|item| PatternRow {
            pattern: if redact {
                super::render::redact_identity(&item.raw)
            } else {
                item.raw.clone()
            },
            hosts: 0,
            upload: if null_bytes { None } else { Some(0) },
            download: if null_bytes { None } else { Some(0) },
            zero_flow: if null_bytes { None } else { Some(true) },
            reason: Some(item.reason.clone()),
        })
        .collect::<Vec<_>>();
    let uncovered = uncovered_hosts
        .into_iter()
        .map(|(host, (upload, download))| HostRow {
            unknown: host == UNKNOWN || host.is_empty(),
            host: redact_host(&host, redact),
            upload: if null_bytes { None } else { Some(upload) },
            download: if null_bytes { None } else { Some(download) },
        })
        .collect::<Vec<_>>();

    let (mapped, shared, unmapped, unsupported_switch) =
        switch_buckets(&covered, &parsed.domain, switches, null_bytes);
    let outbound = outbound_from(&uncovered_rows, redact, null_bytes);

    AuditResult {
        covered,
        dead,
        unsupported_pattern,
        uncovered,
        mapped,
        shared,
        unmapped,
        unsupported_switch,
        outbound,
        residential_upload: None,
        residential_download: None,
        attributed_upload: None,
        attributed_download: None,
        conservation_holds: None,
    }
}

fn switch_buckets(
    covered: &[PatternRow],
    domain: &[DomainPattern],
    switches: Option<&SwitchesFile>,
    null_bytes: bool,
) -> (
    Vec<SwitchRow>,
    Vec<SwitchRow>,
    Vec<SwitchRow>,
    Vec<SwitchRow>,
) {
    let Some(switches) = switches else {
        let unmapped = if covered.is_empty() {
            Vec::new()
        } else {
            vec![SwitchRow {
                switch: "unmapped".into(),
                upload: if null_bytes {
                    None
                } else {
                    Some(covered.iter().map(|row| row.upload.unwrap_or(0)).sum())
                },
                download: if null_bytes {
                    None
                } else {
                    Some(covered.iter().map(|row| row.download.unwrap_or(0)).sum())
                },
                zero_flow: None,
                status: None,
                switches: None,
            }]
        };
        return (Vec::new(), Vec::new(), unmapped, Vec::new());
    };

    let mut owners: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, patterns) in &switches.supported {
        for pattern in patterns {
            owners
                .entry(pattern.to_ascii_lowercase())
                .or_default()
                .push(name.clone());
        }
    }
    let mut mapped_acc: BTreeMap<String, (i64, i64)> = BTreeMap::new();
    for name in switches.supported.keys() {
        mapped_acc.insert(name.clone(), (0, 0));
    }
    let mut shared = Vec::new();
    let mut unmapped_up = 0i64;
    let mut unmapped_down = 0i64;
    for (index, row) in covered.iter().enumerate() {
        let key = domain
            .iter()
            .find(|pattern| {
                display_pattern(pattern, false) == row.pattern || pattern.raw == row.pattern
            })
            .map(pattern_key)
            .or_else(|| domain.get(index).map(pattern_key))
            .unwrap_or_else(|| row.pattern.clone());
        let key = key.to_ascii_lowercase();
        let names = owners.get(&key).cloned().unwrap_or_default();
        let unique = {
            let mut names = names;
            names.sort();
            names.dedup();
            names
        };
        match unique.len() {
            0 => {
                unmapped_up += row.upload.unwrap_or(0);
                unmapped_down += row.download.unwrap_or(0);
            }
            1 => {
                let entry = mapped_acc.entry(unique[0].clone()).or_insert((0, 0));
                entry.0 += row.upload.unwrap_or(0);
                entry.1 += row.download.unwrap_or(0);
            }
            _ => shared.push(SwitchRow {
                switch: "shared".into(),
                upload: if null_bytes { None } else { row.upload },
                download: if null_bytes { None } else { row.download },
                zero_flow: if null_bytes {
                    None
                } else {
                    Some(row.upload.unwrap_or(0) == 0 && row.download.unwrap_or(0) == 0)
                },
                status: None,
                switches: Some(unique),
            }),
        }
    }
    let mapped = mapped_acc
        .into_iter()
        .map(|(switch, (upload, download))| {
            let zero = upload == 0 && download == 0;
            SwitchRow {
                switch,
                upload: if null_bytes { None } else { Some(upload) },
                download: if null_bytes { None } else { Some(download) },
                zero_flow: if null_bytes { None } else { Some(zero) },
                status: None,
                switches: None,
            }
        })
        .collect();
    let unmapped = vec![SwitchRow {
        switch: "unmapped".into(),
        upload: if null_bytes { None } else { Some(unmapped_up) },
        download: if null_bytes {
            None
        } else {
            Some(unmapped_down)
        },
        zero_flow: if null_bytes {
            None
        } else {
            Some(unmapped_up == 0 && unmapped_down == 0)
        },
        status: None,
        switches: None,
    }];
    let unsupported_switch = switches
        .unsupported
        .iter()
        .map(|name| SwitchRow {
            switch: name.clone(),
            upload: None,
            download: None,
            zero_flow: None,
            status: Some("unsupportedSwitch".into()),
            switches: None,
        })
        .collect();
    (mapped, shared, unmapped, unsupported_switch)
}

fn outbound_from(rows: &[&ProjectionRow], redact: bool, null_bytes: bool) -> Vec<OutboundRow> {
    if null_bytes {
        return Vec::new();
    }
    let mut grouped: BTreeMap<(String, String, String), (i64, i64, i64)> = BTreeMap::new();
    for row in rows {
        let source = classify_rule(&row.rule);
        let key = (source.to_string(), row.rule.clone(), row.process.clone());
        let entry = grouped.entry(key).or_insert((0, 0, 0));
        entry.0 += row.upload;
        entry.1 += row.download;
        entry.2 += row.connections;
    }
    grouped
        .into_iter()
        .map(
            |((source, rule, process), (upload, download, connections))| OutboundRow {
                source,
                rule: redact_host(&rule, redact),
                process: redact_host(&process, redact),
                upload,
                download,
                connections,
            },
        )
        .collect()
}

fn classify_rule(rule: &str) -> &'static str {
    let upper = rule.to_ascii_uppercase();
    if upper.starts_with("PROCESS") {
        "process"
    } else if upper.contains("IPCIDR") || upper.contains("IP-CIDR") {
        "ip"
    } else {
        "other"
    }
}

fn conservation_holds(result: &AuditResult) -> bool {
    let mapped: i64 = result
        .mapped
        .iter()
        .map(|row| row.upload.unwrap_or(0) + row.download.unwrap_or(0))
        .sum();
    let shared: i64 = result
        .shared
        .iter()
        .map(|row| row.upload.unwrap_or(0) + row.download.unwrap_or(0))
        .sum();
    let unmapped: i64 = result
        .unmapped
        .iter()
        .map(|row| row.upload.unwrap_or(0) + row.download.unwrap_or(0))
        .sum();
    let unsupported: i64 = result
        .unsupported_pattern
        .iter()
        .map(|row| row.upload.unwrap_or(0) + row.download.unwrap_or(0))
        .sum();
    let uncovered: i64 = result
        .uncovered
        .iter()
        .map(|row| row.upload.unwrap_or(0) + row.download.unwrap_or(0))
        .sum();
    let total = result
        .residential_upload
        .unwrap_or(0)
        .saturating_add(result.residential_download.unwrap_or(0));
    mapped + shared + unmapped + unsupported + uncovered == total
}

fn null_conservation(result: &mut AuditResult) {
    for row in result
        .covered
        .iter_mut()
        .chain(result.dead.iter_mut())
        .chain(result.unsupported_pattern.iter_mut())
    {
        row.upload = None;
        row.download = None;
        row.zero_flow = None;
    }
    for row in &mut result.uncovered {
        row.upload = None;
        row.download = None;
    }
}

fn display_pattern(pattern: &DomainPattern, redact: bool) -> String {
    if redact {
        super::render::redact_identity(&pattern.raw)
    } else {
        pattern.raw.clone()
    }
}

fn redact_result(envelope: &mut Envelope) {
    let _ = envelope;
}

fn residential_probe(options: &QueryOptions) -> ReportQuery {
    ReportQuery {
        range_start_utc: options.start_utc,
        range_end_utc: options.end_utc,
        display_timezone: options.timezone.clone(),
        granularity: Granularity::Hour,
        grouping: DimensionKind::Host,
        include_sessions: true,
        filters: ReportFilters {
            category: Some(RESIDENTIAL_ACCOUNTING_FILTER.into()),
            ..ReportFilters::default()
        },
        ..ReportQuery::default()
    }
}

fn window_echo(options: &QueryOptions) -> WindowEcho {
    WindowEcho {
        start_utc: options.start_utc,
        end_utc: options.end_utc,
        timezone: options.timezone.clone(),
        granularity: "hour".into(),
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, DbCliError> {
    let raw = std::fs::read(path).map_err(|error| {
        DbCliError::InvalidArgs(format!("读取 {} 失败: {error}", path.display()))
    })?;
    serde_json::from_slice(&raw)
        .map_err(|error| DbCliError::InvalidArgs(format!("解析 {} 失败: {error}", path.display())))
}

fn map_sqlite_cli(error: rusqlite::Error) -> DbCliError {
    if crate::sqlite_probe::map_sqlite_error(&error) == "cancelled" {
        DbCliError::Cancelled("sqlite interrupt".into())
    } else {
        DbCliError::FailClosed(format!("sqlite query: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageCoordinator;
    use std::sync::atomic::AtomicBool;
    use tempfile::tempdir;

    fn write_json(path: &Path, value: &serde_json::Value) {
        std::fs::write(path, serde_json::to_vec_pretty(value).expect("json")).expect("write");
    }

    fn seeded_with_vertex() -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempdir().expect("dir");
        let path = dir.path().join("monitor.sqlite3");
        let mut coordinator = StorageCoordinator::open(&path).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        coordinator
            .connection_mut()
            .execute_batch(
                "
                insert into connection_session(session_pk, epoch_id, connection_id, started_utc, host)
                values (8, 1, 'vertex', 1500, 'us-central1-aiplatform.googleapis.com');
                insert into dimension_dict(dimension_kind, dimension_id, value) values
                    ('host', 8, 'us-central1-aiplatform.googleapis.com'),
                    ('rule', 8, 'ProcessName');
                insert into connection_session_attr(
                    session_pk, host_id, process_id, rule_id, network_id, chain_key,
                    policy_version, primary_category_id, started_utc, ended_utc
                ) values (8, 8, 1, 8, 1, 'DIRECT', 1, 1, 1500, null);
                insert into connection_minute(utc_minute, session_pk, upload, download)
                values (25, 8, 1, 2);
                ",
            )
            .expect("vertex");
        let minutes: i64 = coordinator
            .connection()
            .query_row(
                "select count(*) from connection_minute where session_pk = 8",
                [],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(minutes, 1);
        drop(coordinator);
        (dir, path)
    }

    fn options() -> QueryOptions {
        QueryOptions {
            start_utc: 0,
            end_utc: 3_600,
            timezone: "UTC".into(),
            now_utc: 3_600,
            redact: false,
        }
    }

    #[test]
    fn audit_buckets_are_exclusive_and_cover_patterns() {
        let (dir, db) = seeded_with_vertex();
        let rules_path = dir.path().join("rules.json");
        let map_path = dir.path().join("switches.json");
        write_json(
            &rules_path,
            &serde_json::json!({
                "schemaVersion": 1,
                "group": "AI-家宽",
                "rules": [
                    "DOMAIN,a.example,AI-家宽",
                    "DOMAIN,dead.example,AI-家宽",
                    "DOMAIN-REGEX,^[a-z0-9-]+-aiplatform\\.googleapis\\.com$,AI-家宽",
                    "DOMAIN-REGEX,[invalid,AI-家宽",
                    "DOMAIN-SUFFIX,claude.ai,AI-家宽"
                ]
            }),
        );
        write_json(
            &map_path,
            &serde_json::json!({
                "schemaVersion": 1,
                "supported": {
                    "openai_core": ["a.example"],
                    "vertex_ai_endpoints": ["^[a-z0-9-]+-aiplatform\\.googleapis\\.com$"]
                },
                "unsupported": ["openai_shared_dependencies"]
            }),
        );
        let cancel = Arc::new(AtomicBool::new(false));
        let (envelope, code) =
            run_audit(&db, &options(), &rules_path, Some(&map_path), &cancel).expect("audit");
        assert_eq!(code, 0);
        let result = &envelope.result;
        let covered: Vec<&str> = result["covered"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|row| row["pattern"].as_str())
            .collect();
        let dead: Vec<&str> = result["dead"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|row| row["pattern"].as_str())
            .collect();
        let unsupported: Vec<&str> = result["unsupportedPattern"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|row| row["pattern"].as_str())
            .collect();
        assert!(
            covered.iter().any(|item| item.contains("a.example")),
            "covered={covered:?} dead={dead:?} unsupported={unsupported:?} result={result}"
        );
        assert!(
            covered
                .iter()
                .any(|item| item.contains("aiplatform") && item.contains("DOMAIN-REGEX")),
            "covered={covered:?} dead={dead:?} unsupported={unsupported:?} result={result}"
        );
        assert!(dead.iter().any(|item| item.contains("dead.example")));
        assert!(dead.iter().any(|item| item.contains("claude.ai")));
        assert!(unsupported.iter().any(|item| item.contains("[invalid")));
        let mut all = covered.clone();
        all.extend(dead.iter().copied());
        all.extend(unsupported.iter().copied());
        all.sort();
        all.dedup();
        assert_eq!(all.len(), 5);
        let uncovered: Vec<&str> = result["uncovered"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|row| row["host"].as_str())
            .collect();
        assert!(uncovered.contains(&"b.example"));
        assert!(!uncovered.contains(&"a.example"));
        let outbound = result["outbound"].as_array().unwrap();
        assert!(outbound
            .iter()
            .all(|row| row["upload"].as_i64().unwrap() >= 0));
        assert_eq!(
            result["unsupportedSwitch"][0]["status"],
            "unsupportedSwitch"
        );
        assert_eq!(
            result["unsupportedSwitch"][0]["upload"],
            serde_json::Value::Null
        );
        assert_eq!(result["conservationHolds"], true);
    }
}
