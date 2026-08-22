//! 稳定 sort tuple + identity 的 keyset 列表。

use crate::c2::contract::{LIST_PAGE_DEFAULT, LIST_PAGE_MAX};
use crate::c2::hub::LiveConnectionView;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionFilter {
    pub category: Option<String>,
    pub host: Option<String>,
    pub process: Option<String>,
    pub rule: Option<String>,
    pub chain: Option<String>,
    pub network: Option<String>,
    #[serde(default)]
    pub residential_only: bool,
    #[serde(default)]
    pub clauses: Vec<FilterClause>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterClause {
    pub field: String,
    pub mode: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCursor {
    pub sort_key: String,
    pub identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionQuery {
    pub filter: ConnectionFilter,
    pub sort_field: String,
    pub descending: bool,
    pub cursor: Option<ConnectionCursor>,
    pub limit: u32,
}

impl Default for ConnectionQuery {
    fn default() -> Self {
        Self {
            filter: ConnectionFilter::default(),
            sort_field: "identity".into(),
            descending: false,
            cursor: None,
            limit: LIST_PAGE_DEFAULT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionPage {
    pub rows: Vec<LiveConnectionView>,
    pub next_cursor: Option<ConnectionCursor>,
    pub matched_count: u32,
    pub sample_utc: Option<i64>,
    pub summary: ConnectionSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionSummary {
    pub top_download: Option<ConnectionHotspot>,
    pub top_upload: Option<ConnectionHotspot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionHotspot {
    pub identity: String,
    pub label: String,
    pub host: Option<String>,
    pub process: Option<String>,
    pub destination: Option<String>,
    pub value: u64,
}

pub fn sanitize_limit(limit: u32) -> u32 {
    if limit == 0 {
        LIST_PAGE_DEFAULT
    } else {
        limit.min(LIST_PAGE_MAX)
    }
}

fn matches_filter(row: &LiveConnectionView, filter: &ConnectionFilter, targets: &[String]) -> bool {
    if let Some(category) = &filter.category {
        let hit = row.primary.as_deref() == Some(category.as_str())
            || row.tags.iter().any(|tag| tag == category);
        if !hit {
            return false;
        }
    }
    if filter.residential_only && !is_residential(row, targets) {
        return false;
    }
    if !filter
        .clauses
        .iter()
        .take(8)
        .all(|clause| matches_clause(row, clause))
    {
        return false;
    }
    contains_opt(row.host.as_deref(), filter.host.as_deref())
        && contains_opt(row.process_name.as_deref(), filter.process.as_deref())
        && contains_opt(row.rule.as_deref(), filter.rule.as_deref())
        && contains_opt(row.network.as_deref(), filter.network.as_deref())
        && filter
            .chain
            .as_ref()
            .is_none_or(|chain| row.chains.iter().any(|item| item.contains(chain)))
}

fn is_residential(row: &LiveConnectionView, targets: &[String]) -> bool {
    crate::residential::is_residential_filter(targets, &row.chains)
}

fn is_numeric_field(field: &str) -> bool {
    matches!(
        field,
        "download" | "upload" | "rateDownload" | "rateUpload" | "duration"
    )
}

fn numeric_row_value(row: &LiveConnectionView, field: &str) -> Option<u64> {
    match field {
        "download" => Some(row.download),
        "upload" => Some(row.upload),
        "rateDownload" => row.rate_download,
        "rateUpload" => row.rate_upload,
        "duration" => row.duration_ms,
        _ => None,
    }
}

fn matches_numeric(row: &LiveConnectionView, field: &str, mode: &str, raw: &str) -> bool {
    if !matches!(mode, "gt" | "gte" | "lt" | "lte" | "eq") {
        return true;
    }
    let Ok(threshold) = raw.parse::<u64>() else {
        return true;
    };
    let Some(actual) = numeric_row_value(row, field) else {
        return false;
    };
    match mode {
        "gt" => actual > threshold,
        "gte" => actual >= threshold,
        "lt" => actual < threshold,
        "lte" => actual <= threshold,
        "eq" => actual == threshold,
        _ => true,
    }
}

fn matches_clause(row: &LiveConnectionView, clause: &FilterClause) -> bool {
    let value = clause.value.trim();
    if value.is_empty() {
        return true;
    }
    if is_numeric_field(&clause.field) {
        return matches_numeric(row, &clause.field, &clause.mode, value);
    }
    if !matches!(clause.mode.as_str(), "exact" | "contains") {
        return true;
    }
    let candidates = clause_candidates(row, &clause.field);
    match clause.mode.as_str() {
        "exact" => candidates.iter().any(|item| item == value),
        _ => candidates.iter().any(|item| item.contains(value)),
    }
}

fn clause_candidates(row: &LiveConnectionView, field: &str) -> Vec<String> {
    match field {
        "host" => joined_host(row.host.as_deref(), row.destination_port.as_deref()),
        "process" => opt_vec(row.process_name.as_deref()),
        "rule" => {
            let mut items = opt_vec(row.rule.as_deref());
            if let (Some(rule), Some(payload)) = (row.rule.as_deref(), row.rule_payload.as_deref())
            {
                items.push(format!("{rule}({payload})"));
            }
            items
        }
        "chain" => row.chains.clone(),
        "source" => joined_host(row.source_ip.as_deref(), row.source_port.as_deref()),
        "destination" => joined_host(
            row.destination_ip.as_deref(),
            row.destination_port.as_deref(),
        ),
        "type" => {
            let mut items = opt_vec(row.inbound.as_deref());
            items.extend(opt_vec(row.network.as_deref()));
            if let (Some(inbound), Some(network)) = (row.inbound.as_deref(), row.network.as_deref())
            {
                items.push(format!("{inbound}({network})"));
            }
            items
        }
        _ => Vec::new(),
    }
}

fn joined_host(host: Option<&str>, port: Option<&str>) -> Vec<String> {
    let mut items = opt_vec(host);
    if let (Some(host), Some(port)) = (host, port) {
        items.push(format!("{host}:{port}"));
    }
    items
}

fn opt_vec(value: Option<&str>) -> Vec<String> {
    value.map(|item| vec![item.to_string()]).unwrap_or_default()
}

fn contains_opt(value: Option<&str>, needle: Option<&str>) -> bool {
    match needle {
        None => true,
        Some(text) => value.is_some_and(|value| value.contains(text)),
    }
}

fn normalize_sort_field(field: &str) -> &str {
    match field {
        "host" | "download" | "upload" | "rateDownload" | "rateUpload" | "chain" | "rule"
        | "process" | "duration" | "source" | "destination" | "type" | "identity" => field,
        _ => "identity",
    }
}

fn known_text(value: Option<&str>) -> (u8, String) {
    match value {
        Some(text) if !text.is_empty() => (0, text.to_string()),
        _ => (1, String::new()),
    }
}

fn known_u64(value: Option<u64>) -> (u8, String) {
    match value {
        Some(number) => (0, format!("{number:020}")),
        None => (1, String::new()),
    }
}

fn join_endpoint(host: Option<&str>, port: Option<&str>) -> Option<String> {
    let host = host.filter(|item| !item.is_empty())?;
    match port.filter(|item| !item.is_empty()) {
        Some(port) => Some(format!("{host}:{port}")),
        None => Some(host.to_string()),
    }
}

fn sort_slot(row: &LiveConnectionView, field: &str) -> (u8, String) {
    match normalize_sort_field(field) {
        "host" => known_text(row.host.as_deref()),
        "process" => known_text(row.process_name.as_deref()),
        "rule" => {
            let display = match (row.rule.as_deref(), row.rule_payload.as_deref()) {
                (Some(rule), Some(payload)) => Some(format!("{rule}({payload})")),
                (Some(rule), None) => Some(rule.to_string()),
                _ => None,
            };
            known_text(display.as_deref())
        }
        "chain" => {
            if row.chains.is_empty() {
                (1, String::new())
            } else {
                (0, row.chains.join(" / "))
            }
        }
        "upload" => known_u64(Some(row.upload)),
        "download" => known_u64(Some(row.download)),
        "rateDownload" => known_u64(row.rate_download),
        "rateUpload" => known_u64(row.rate_upload),
        "duration" => known_u64(row.duration_ms),
        "source" => known_text(
            join_endpoint(row.source_ip.as_deref(), row.source_port.as_deref()).as_deref(),
        ),
        "destination" => known_text(
            join_endpoint(
                row.destination_ip.as_deref(),
                row.destination_port.as_deref(),
            )
            .as_deref(),
        ),
        "type" => {
            let display = match (row.inbound.as_deref(), row.network.as_deref()) {
                (Some(inbound), Some(network)) => Some(format!("{inbound}({network})")),
                (Some(inbound), None) => Some(inbound.to_string()),
                (None, Some(network)) => Some(network.to_string()),
                _ => None,
            };
            known_text(display.as_deref())
        }
        _ => (0, row.identity.clone()),
    }
}

fn encode_sort_key(slot: &(u8, String)) -> String {
    format!("{}:{}", slot.0, slot.1)
}

fn decode_sort_key(raw: &str) -> (u8, String) {
    if let Some(rest) = raw.strip_prefix("1:") {
        (1, rest.to_string())
    } else if let Some(rest) = raw.strip_prefix("0:") {
        (0, rest.to_string())
    } else {
        (0, raw.to_string())
    }
}

fn cmp_slots(
    left: &(u8, String),
    left_id: &str,
    right: &(u8, String),
    right_id: &str,
    descending: bool,
) -> std::cmp::Ordering {
    match left.0.cmp(&right.0) {
        std::cmp::Ordering::Equal if left.0 == 1 => {
            if descending {
                right_id.cmp(left_id)
            } else {
                left_id.cmp(right_id)
            }
        }
        std::cmp::Ordering::Equal => {
            let keys = if descending {
                right.1.cmp(&left.1)
            } else {
                left.1.cmp(&right.1)
            };
            keys.then_with(|| {
                if descending {
                    right_id.cmp(left_id)
                } else {
                    left_id.cmp(right_id)
                }
            })
        }
        rank => rank,
    }
}

fn sort_key(row: &LiveConnectionView, field: &str) -> String {
    encode_sort_key(&sort_slot(row, field))
}

pub fn query_connections(rows: &[LiveConnectionView], query: &ConnectionQuery) -> ConnectionPage {
    query_connections_with_targets_at(rows, query, &[], None)
}

pub fn query_connections_with_targets(
    rows: &[LiveConnectionView],
    query: &ConnectionQuery,
    targets: &[String],
) -> ConnectionPage {
    query_connections_with_targets_at(rows, query, targets, None)
}

pub fn query_connections_with_targets_at(
    rows: &[LiveConnectionView],
    query: &ConnectionQuery,
    targets: &[String],
    sample_utc: Option<i64>,
) -> ConnectionPage {
    let limit = sanitize_limit(query.limit) as usize;
    let mut matched: Vec<&LiveConnectionView> = rows
        .iter()
        .filter(|row| matches_filter(row, &query.filter, targets))
        .collect();
    let matched_count = u32::try_from(matched.len()).unwrap_or(u32::MAX);
    let summary = ConnectionSummary {
        top_download: hotspot(&matched, |row| row.download),
        top_upload: hotspot(&matched, |row| row.upload),
    };
    matched.sort_by(|left, right| {
        cmp_slots(
            &sort_slot(left, &query.sort_field),
            left.identity.as_str(),
            &sort_slot(right, &query.sort_field),
            right.identity.as_str(),
            query.descending,
        )
    });
    let start = query.cursor.as_ref().map(|cursor| {
        let cursor_slot = decode_sort_key(&cursor.sort_key);
        matched
            .iter()
            .position(|row| {
                cmp_slots(
                    &sort_slot(row, &query.sort_field),
                    row.identity.as_str(),
                    &cursor_slot,
                    cursor.identity.as_str(),
                    query.descending,
                ) == std::cmp::Ordering::Greater
            })
            .unwrap_or(matched.len())
    });
    let start = start.unwrap_or(0);
    let page: Vec<LiveConnectionView> = matched
        .iter()
        .skip(start)
        .take(limit)
        .map(|row| (*row).clone())
        .collect();
    let next_cursor = if start + page.len() < matched.len() {
        page.last().map(|row| ConnectionCursor {
            sort_key: sort_key(row, &query.sort_field),
            identity: row.identity.clone(),
        })
    } else {
        None
    };
    ConnectionPage {
        rows: page,
        next_cursor,
        matched_count,
        sample_utc,
        summary,
    }
}

fn hotspot(
    rows: &[&LiveConnectionView],
    value: impl Fn(&LiveConnectionView) -> u64,
) -> Option<ConnectionHotspot> {
    rows.iter()
        .min_by(|left, right| {
            value(right)
                .cmp(&value(left))
                .then_with(|| left.identity.cmp(&right.identity))
        })
        .map(|row| ConnectionHotspot {
            identity: row.identity.clone(),
            label: hotspot_label(row),
            host: row.host.clone(),
            process: row.process_name.clone(),
            destination: join_endpoint(
                row.destination_ip.as_deref(),
                row.destination_port.as_deref(),
            ),
            value: value(row),
        })
}

fn hotspot_label(row: &LiveConnectionView) -> String {
    let destination = join_endpoint(
        row.destination_ip.as_deref(),
        row.destination_port.as_deref(),
    );
    row.host
        .as_deref()
        .filter(|item| !item.is_empty())
        .or_else(|| row.process_name.as_deref().filter(|item| !item.is_empty()))
        .or(destination.as_deref())
        .unwrap_or(&row.identity)
        .to_string()
}

#[cfg(test)]
mod connection_query_tests {
    use super::*;

    fn row(id: &str, host: &str) -> LiveConnectionView {
        LiveConnectionView {
            identity: format!("0:{id}"),
            connection_id: id.into(),
            epoch: 0,
            upload: id.bytes().next().unwrap_or(0) as u64,
            download: 0,
            rate_upload: None,
            rate_download: None,
            duration_ms: None,
            primary: None,
            tags: Vec::new(),
            host: Some(host.into()),
            source_ip: None,
            destination_ip: None,
            process_name: None,
            process_path: None,
            network: Some("tcp".into()),
            rule: None,
            rule_payload: None,
            chains: Vec::new(),
            ..LiveConnectionView::default()
        }
    }

    #[test]
    fn random_order_does_not_change_identity_page() {
        let mut rows = vec![row("b", "b.test"), row("a", "a.test"), row("c", "c.test")];
        let query = ConnectionQuery {
            limit: 2,
            ..ConnectionQuery::default()
        };
        let first = query_connections(&rows, &query);
        rows.reverse();
        let second = query_connections(&rows, &query);
        assert_eq!(first.rows[0].identity, "0:a");
        assert_eq!(first.rows[1].identity, "0:b");
        assert_eq!(first.rows[0].identity, second.rows[0].identity);
        let next = query_connections(
            &rows,
            &ConnectionQuery {
                cursor: first.next_cursor.clone(),
                limit: 2,
                ..ConnectionQuery::default()
            },
        );
        assert_eq!(next.rows[0].identity, "0:c");
        assert!(next.next_cursor.is_none());
    }

    #[test]
    fn residential_only_matches_target_or_name() {
        let mut home = row("a", "a.test");
        home.chains = vec!["AI-家宽".into(), "家宽-SOCKS5".into()];
        let mut other = row("b", "b.test");
        other.chains = vec!["DIRECT".into()];
        let query = ConnectionQuery {
            filter: ConnectionFilter {
                residential_only: true,
                ..ConnectionFilter::default()
            },
            ..ConnectionQuery::default()
        };
        let page = query_connections_with_targets(&[home, other], &query, &["家宽".into()]);
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows[0].connection_id, "a");
    }

    /// 改造前 `is_residential` 原文。用于断言「只看家宽」选中集合不变。
    fn legacy_is_residential(row: &LiveConnectionView, targets: &[String]) -> bool {
        row.chains
            .iter()
            .any(|node| targets.iter().any(|target| target == node) || node.contains("家宽"))
    }

    #[test]
    fn residential_only_selected_set_matches_legacy() {
        let mut exact = row("a", "a.test");
        exact.chains = vec!["家宽-SOCKS5".into()];
        let mut substring = row("b", "b.test");
        substring.chains = vec!["AI-家宽".into()];
        let mut miss = row("c", "c.test");
        miss.chains = vec!["DIRECT".into()];
        let mut both = row("d", "d.test");
        both.chains = vec!["家宽".into(), "其它".into()];
        let rows = [exact, substring, miss, both];
        let query = ConnectionQuery {
            filter: ConnectionFilter {
                residential_only: true,
                ..ConnectionFilter::default()
            },
            sort_field: "identity".into(),
            ..ConnectionQuery::default()
        };
        for targets in [
            vec!["家宽-SOCKS5".into()],
            vec!["家宽".into()],
            Vec::<String>::new(),
            vec!["DIRECT".into()],
        ] {
            let page = query_connections_with_targets(&rows, &query, &targets);
            let mut got: Vec<String> = page.rows.iter().map(|item| item.identity.clone()).collect();
            let mut expected: Vec<String> = rows
                .iter()
                .filter(|item| legacy_is_residential(item, &targets))
                .map(|item| item.identity.clone())
                .collect();
            got.sort();
            expected.sort();
            assert_eq!(got, expected, "targets={targets:?}");
        }
    }

    #[test]
    fn exact_clause_does_not_match_substring() {
        let mut row = row("a", "ws.chatgpt.com");
        row.chains = vec!["AI-家宽".into()];
        let query = ConnectionQuery {
            filter: ConnectionFilter {
                clauses: vec![FilterClause {
                    field: "host".into(),
                    mode: "exact".into(),
                    value: "chatgpt.com".into(),
                }],
                ..ConnectionFilter::default()
            },
            ..ConnectionQuery::default()
        };
        let page = query_connections(&[row.clone()], &query);
        assert!(page.rows.is_empty());
        let mut contains = query.clone();
        contains.filter.clauses[0].mode = "contains".into();
        let page = query_connections(&[row], &contains);
        assert_eq!(page.rows.len(), 1);
    }

    #[test]
    fn download_descending_puts_largest_first() {
        let mut low = row("a", "a.test");
        low.download = 10;
        let mut high = row("b", "b.test");
        high.download = 30;
        let page = query_connections(
            &[low, high],
            &ConnectionQuery {
                sort_field: "download".into(),
                descending: true,
                ..ConnectionQuery::default()
            },
        );
        assert_eq!(page.rows[0].connection_id, "b");
        assert_eq!(page.rows[1].connection_id, "a");
    }

    #[test]
    fn unknown_rate_sorts_after_known_in_both_directions() {
        let mut known = row("a", "a.test");
        known.rate_download = Some(8);
        let mut unknown = row("b", "b.test");
        unknown.rate_download = None;
        let asc = query_connections(
            &[unknown.clone(), known.clone()],
            &ConnectionQuery {
                sort_field: "rateDownload".into(),
                descending: false,
                ..ConnectionQuery::default()
            },
        );
        assert_eq!(
            asc.rows
                .iter()
                .map(|item| item.connection_id.as_str())
                .collect::<Vec<_>>(),
            ["a", "b"]
        );
        let desc = query_connections(
            &[unknown, known],
            &ConnectionQuery {
                sort_field: "rateDownload".into(),
                descending: true,
                ..ConnectionQuery::default()
            },
        );
        assert_eq!(
            desc.rows
                .iter()
                .map(|item| item.connection_id.as_str())
                .collect::<Vec<_>>(),
            ["a", "b"]
        );
    }

    #[test]
    fn unknown_sort_field_falls_back_to_identity() {
        let page = query_connections(
            &[row("b", "b.test"), row("a", "a.test")],
            &ConnectionQuery {
                sort_field: "nope".into(),
                ..ConnectionQuery::default()
            },
        );
        assert_eq!(page.rows[0].connection_id, "a");
        assert_eq!(page.rows[1].connection_id, "b");
    }

    #[test]
    fn zero_download_sorts_before_positive() {
        let mut zero = row("a", "a.test");
        zero.download = 0;
        let mut other = row("b", "b.test");
        other.download = 5;
        let page = query_connections(
            &[other, zero],
            &ConnectionQuery {
                sort_field: "download".into(),
                descending: false,
                ..ConnectionQuery::default()
            },
        );
        assert_eq!(page.rows[0].connection_id, "a");
        assert_eq!(page.rows[1].connection_id, "b");
    }

    #[test]
    fn numeric_eq_zero_matches_download_zero() {
        let mut zero = row("a", "a.test");
        zero.download = 0;
        let mut other = row("b", "b.test");
        other.download = 10;
        let page = query_connections(
            &[zero, other],
            &ConnectionQuery {
                filter: ConnectionFilter {
                    clauses: vec![FilterClause {
                        field: "download".into(),
                        mode: "eq".into(),
                        value: "0".into(),
                    }],
                    ..ConnectionFilter::default()
                },
                ..ConnectionQuery::default()
            },
        );
        assert_eq!(page.rows.len(), 1);
        assert_eq!(page.rows[0].connection_id, "a");
    }

    #[test]
    fn numeric_gt_does_not_use_contains() {
        let mut row = row("a", "a.test");
        row.download = 100;
        let page = query_connections(
            &[row],
            &ConnectionQuery {
                filter: ConnectionFilter {
                    clauses: vec![FilterClause {
                        field: "download".into(),
                        mode: "gt".into(),
                        value: "50".into(),
                    }],
                    ..ConnectionFilter::default()
                },
                ..ConnectionQuery::default()
            },
        );
        assert_eq!(page.rows.len(), 1);
    }

    #[test]
    fn unknown_rate_does_not_match_numeric_clause() {
        let mut row = row("a", "a.test");
        row.rate_download = None;
        let page = query_connections(
            &[row],
            &ConnectionQuery {
                filter: ConnectionFilter {
                    clauses: vec![FilterClause {
                        field: "rateDownload".into(),
                        mode: "gte".into(),
                        value: "0".into(),
                    }],
                    ..ConnectionFilter::default()
                },
                ..ConnectionQuery::default()
            },
        );
        assert!(page.rows.is_empty());
    }

    #[test]
    fn numeric_mode_on_text_field_is_ignored() {
        let row = row("a", "a.test");
        let page = query_connections(
            &[row],
            &ConnectionQuery {
                filter: ConnectionFilter {
                    clauses: vec![FilterClause {
                        field: "host".into(),
                        mode: "gt".into(),
                        value: "1".into(),
                    }],
                    ..ConnectionFilter::default()
                },
                ..ConnectionQuery::default()
            },
        );
        assert_eq!(page.rows.len(), 1);
    }

    #[test]
    fn hotspots_use_complete_filtered_set_with_identity_tie_breaks() {
        let mut alpha = row("alpha", "alpha.test");
        alpha.download = 90;
        alpha.upload = 40;
        let mut beta = row("beta", "beta.test");
        beta.download = 120;
        beta.upload = 80;
        let mut gamma = row("gamma", "gamma.test");
        gamma.download = 120;
        gamma.upload = 80;
        gamma.process_name = Some("worker.exe".into());
        gamma.destination_ip = Some("203.0.113.9".into());
        gamma.destination_port = Some("443".into());
        let mut ignored = row("ignored", "ignored.test");
        ignored.download = 999;
        ignored.upload = 999;

        let query = ConnectionQuery {
            filter: ConnectionFilter {
                host: Some(".test".into()),
                clauses: vec![FilterClause {
                    field: "host".into(),
                    mode: "contains".into(),
                    value: "a.test".into(),
                }],
                ..ConnectionFilter::default()
            },
            limit: 1,
            ..ConnectionQuery::default()
        };
        let rows = [ignored, gamma, beta, alpha];
        let first = query_connections_with_targets_at(&rows, &query, &[], Some(123));
        let expanded = query_connections_with_targets_at(
            &rows,
            &ConnectionQuery {
                limit: 200,
                ..query.clone()
            },
            &[],
            Some(123),
        );
        let cursor_page = query_connections_with_targets_at(
            &rows,
            &ConnectionQuery {
                cursor: first.next_cursor.clone(),
                ..query.clone()
            },
            &[],
            Some(123),
        );

        assert_eq!(first.rows.len(), 1);
        assert_eq!(first.matched_count, 3);
        assert_eq!(first.sample_utc, Some(123));
        assert_eq!(
            first
                .summary
                .top_download
                .as_ref()
                .map(|item| item.identity.as_str()),
            Some("0:beta")
        );
        assert_eq!(
            first
                .summary
                .top_upload
                .as_ref()
                .map(|item| item.identity.as_str()),
            Some("0:beta")
        );
        assert_eq!(first.summary, expanded.summary);
        assert_eq!(first.summary, cursor_page.summary);
        let sorted = query_connections_with_targets_at(
            &rows,
            &ConnectionQuery {
                sort_field: "download".into(),
                descending: true,
                limit: 1,
                filter: query.filter.clone(),
                ..ConnectionQuery::default()
            },
            &[],
            Some(123),
        );
        assert_eq!(first.summary, sorted.summary);
    }

    #[test]
    fn hotspot_label_uses_safe_display_fallbacks() {
        let mut process = row("process", "");
        process.host = None;
        process.process_name = Some("worker.exe".into());
        process.process_path = Some("C:\\secret\\worker.exe".into());
        let mut destination = row("destination", "");
        destination.host = None;
        destination.destination_ip = Some("203.0.113.9".into());
        destination.destination_port = Some("443".into());
        let mut identity = row("identity", "");
        identity.host = None;

        assert_eq!(hotspot_label(&process), "worker.exe");
        assert_eq!(hotspot_label(&destination), "203.0.113.9:443");
        assert_eq!(hotspot_label(&identity), "0:identity");
        let item = hotspot(&[&process], |row| row.upload).expect("hotspot");
        assert_eq!(item.process.as_deref(), Some("worker.exe"));
        assert_ne!(item.label, "C:\\secret\\worker.exe");
        let json = serde_json::to_value(&item).expect("json");
        assert_eq!(json.get("processPath"), None);
        assert_eq!(json.get("rulePayload"), None);
        assert_eq!(
            json.get("process").and_then(|value| value.as_str()),
            Some("worker.exe")
        );
    }

    #[test]
    fn empty_match_has_null_hotspots_instead_of_zero() {
        let page = query_connections(
            &[row("a", "a.test")],
            &ConnectionQuery {
                filter: ConnectionFilter {
                    host: Some("missing".into()),
                    ..ConnectionFilter::default()
                },
                ..ConnectionQuery::default()
            },
        );
        assert_eq!(page.matched_count, 0);
        assert!(page.summary.top_download.is_none());
        assert!(page.summary.top_upload.is_none());
    }
}
