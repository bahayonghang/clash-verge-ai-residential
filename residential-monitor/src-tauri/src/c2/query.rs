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
    row.chains.iter().any(|node| {
        targets.iter().any(|target| target == node) || node.contains("家宽")
    })
}

fn matches_clause(row: &LiveConnectionView, clause: &FilterClause) -> bool {
    let value = clause.value.trim();
    if value.is_empty() {
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
            if let (Some(rule), Some(payload)) = (row.rule.as_deref(), row.rule_payload.as_deref()) {
                items.push(format!("{rule}({payload})"));
            }
            items
        }
        "chain" => row.chains.clone(),
        "source" => joined_host(row.source_ip.as_deref(), row.source_port.as_deref()),
        "destination" => {
            joined_host(row.destination_ip.as_deref(), row.destination_port.as_deref())
        }
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

fn sort_key(row: &LiveConnectionView, field: &str) -> String {
    match field {
        "host" => row.host.clone().unwrap_or_default(),
        "process" => row.process_name.clone().unwrap_or_default(),
        "rule" => row.rule.clone().unwrap_or_default(),
        "network" => row.network.clone().unwrap_or_default(),
        "upload" => format!("{:020}", row.upload),
        "download" => format!("{:020}", row.download),
        _ => row.identity.clone(),
    }
}

pub fn query_connections(rows: &[LiveConnectionView], query: &ConnectionQuery) -> ConnectionPage {
    query_connections_with_targets(rows, query, &[])
}

pub fn query_connections_with_targets(
    rows: &[LiveConnectionView],
    query: &ConnectionQuery,
    targets: &[String],
) -> ConnectionPage {
    let limit = sanitize_limit(query.limit) as usize;
    let mut matched: Vec<&LiveConnectionView> = rows
        .iter()
        .filter(|row| matches_filter(row, &query.filter, targets))
        .collect();
    matched.sort_by(|left, right| {
        let left_key = (sort_key(left, &query.sort_field), left.identity.as_str());
        let right_key = (sort_key(right, &query.sort_field), right.identity.as_str());
        if query.descending {
            right_key.cmp(&left_key)
        } else {
            left_key.cmp(&right_key)
        }
    });
    let start = query.cursor.as_ref().map(|cursor| {
        matched
            .iter()
            .position(|row| {
                let key = sort_key(row, &query.sort_field);
                if query.descending {
                    (key.as_str(), row.identity.as_str())
                        < (cursor.sort_key.as_str(), cursor.identity.as_str())
                } else {
                    (key.as_str(), row.identity.as_str())
                        > (cursor.sort_key.as_str(), cursor.identity.as_str())
                }
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
    }
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
}
