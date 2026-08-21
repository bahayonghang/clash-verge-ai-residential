//! 命名 SQL corpus。值全部参数化；前端不传 SQL。
//!
//! 带 `{filters}` 的模板由 `render_sql` 替换一次。片段只由枚举驱动，
//! 用户值只进绑定参数。

use crate::c3::query::{DimensionKind, ReportFilters};
use rusqlite::types::Value;

/// 维度值缺失时排名 identity 的哨兵。真实 `dimension_dict.value` 不得等于该字面量。
pub const UNKNOWN_IDENTITY: &str = "__unknown__";

/// SQL 侧规则聚合键：多跳取顶层策略组，单跳取原始 rule，皆空取 DIRECT。
/// 与 `build_rule_name` 在「单跳且有 payload」时不同；聚合路径只用本定义。
pub const RULE_KEY_SQL: &str = "coalesce(last_chain_hop(a.chain_key), (select value from dimension_dict where dimension_kind = 'rule' and dimension_id = a.rule_id), 'DIRECT')";

pub const TOTALS_RAW: &str = "
select coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0),
       count(distinct m.session_pk),
       count(distinct m.utc_minute) * 60
  from connection_minute m
  join connection_session s on s.session_pk = m.session_pk
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ? and m.utc_minute < ?
 {filters}
";

pub const SERIES_RAW: &str = "
select (m.utc_minute / ?) * ? as bucket,
       coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0),
       count(distinct m.session_pk),
       count(distinct m.utc_minute) * 60
  from connection_minute m
  join connection_session s on s.session_pk = m.session_pk
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ? and m.utc_minute < ?
 {filters}
 group by bucket
 order by bucket
";

pub const RANK_RAW: &str = "
select case when coalesce(s.host, '') = '' then '__unknown__' else s.host end,
       coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0),
       count(distinct m.session_pk),
       count(distinct m.utc_minute) * 60
  from connection_minute m
  join connection_session s on s.session_pk = m.session_pk
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ? and m.utc_minute < ?
 {filters}
 group by 1
 order by sum(m.download) desc, 1 asc
 limit ?
";

pub const RANK_RAW_ATTR: &str = "
select case when coalesce(d.value, '') = '' then '__unknown__' else d.value end,
       coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0),
       count(distinct m.session_pk), count(distinct m.utc_minute) * 60
  from connection_minute m
  join connection_session s on s.session_pk = m.session_pk
  left join connection_session_attr a on a.session_pk = m.session_pk
  left join dimension_dict d on d.dimension_kind = ? and d.dimension_id = case ?
    when 'process' then a.process_id
    when 'network' then a.network_id
    when 'category' then a.primary_category_id
    else a.host_id end
 where m.utc_minute >= ? and m.utc_minute < ?
 {filters}
 group by 1
 order by sum(m.download) desc, 1 asc
 limit ?
";

pub const RANK_RAW_RULE: &str = "
select coalesce(last_chain_hop(a.chain_key), (select value from dimension_dict where dimension_kind = 'rule' and dimension_id = a.rule_id), 'DIRECT'),
       coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0),
       count(distinct m.session_pk), count(distinct m.utc_minute) * 60
  from connection_minute m
  join connection_session s on s.session_pk = m.session_pk
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ? and m.utc_minute < ?
 {filters}
 group by 1
 order by sum(m.download) desc, 1 asc
 limit ?
";

pub const RANK_RAW_CHAIN: &str = "
select coalesce(chain_identity(a.chain_key), '__unknown__'),
       coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0),
       count(distinct m.session_pk), count(distinct m.utc_minute) * 60
  from connection_minute m
  join connection_session s on s.session_pk = m.session_pk
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ? and m.utc_minute < ?
 {filters}
 group by chain_identity(a.chain_key)
 order by sum(m.download) desc, 1 asc
 limit ?
";

pub const SESSIONS_KEYSET: &str = "
select s.session_pk, s.epoch_id, s.connection_id, s.host, s.started_utc,
       coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0)
  from connection_minute m
  join connection_session s on s.session_pk = m.session_pk
 where m.utc_minute >= ?1 and m.utc_minute < ?2
   and (?3 = '' or (coalesce(sum(m.download), 0) < ?4) or (coalesce(sum(m.download), 0) = ?4 and (s.epoch_id || ':' || s.connection_id) > ?3))
 group by s.session_pk
 order by sum(m.download) desc, (s.epoch_id || ':' || s.connection_id) asc
 limit ?5
";

pub const COVERAGE_RAW: &str = "
select kind, reason, started_utc, ended_utc
  from coverage_interval
 where started_utc < ?2 and (ended_utc is null or ended_utc > ?1)
 order by started_utc, interval_id
";

/// 一次扫描同时得到家宽分子与可归因观测分母。家宽子集 = `primary_category_id IS NOT NULL`。
pub const SHARE_RESIDENTIAL_RAW: &str = "
select
  coalesce(sum(case when a.primary_category_id is not null then m.upload end), 0),
  coalesce(sum(case when a.primary_category_id is not null then m.download end), 0),
  coalesce(sum(m.upload), 0),
  coalesce(sum(m.download), 0)
from connection_minute m
left join connection_session_attr a on a.session_pk = m.session_pk
where m.utc_minute >= ?1 and m.utc_minute < ?2
";

pub const TOTALS_HOURLY: &str = "
select coalesce(sum(upload), 0), coalesce(sum(download), 0),
       coalesce(sum(connection_count), 0), coalesce(sum(active_duration_sec), 0)
  from traffic_hourly_dimension h
 where h.utc_hour >= ? and h.utc_hour < ?
   and h.dimension_kind = ?
 {filters}
";

pub const SERIES_HOURLY: &str = "
select utc_hour, sum(upload), sum(download), sum(connection_count), sum(active_duration_sec)
  from traffic_hourly_dimension h
 where h.utc_hour >= ? and h.utc_hour < ?
   and h.dimension_kind = ?
 {filters}
 group by utc_hour
 order by utc_hour
";

pub const RANK_HOURLY: &str = "
select case when coalesce(d.value, '') = '' then '__unknown__' else d.value end,
       sum(h.upload), sum(h.download), sum(h.connection_count), sum(h.active_duration_sec)
  from traffic_hourly_dimension h
  left join dimension_dict d
    on d.dimension_kind = h.dimension_kind and d.dimension_id = h.dimension_id
 where h.utc_hour >= ? and h.utc_hour < ?
   and h.dimension_kind = ?
 {filters}
 group by 1
 order by sum(h.download) desc, 1 asc
 limit ?
";

/// 五维物化后同一份流量有五行；固定 `dimension_kind = 'host'` 以免总量放大五倍。
pub const RANK_HOURLY_CATEGORY: &str = "
-- 五维物化后同一份流量有五行；固定 dimension_kind='host' 以免总量放大五倍。
select case when coalesce(d.value, '') = '' then '__unknown__' else d.value end,
       sum(h.upload), sum(h.download), sum(h.connection_count), sum(h.active_duration_sec)
  from traffic_hourly_dimension h
  left join dimension_dict d
    on d.dimension_kind = 'category' and d.dimension_id = h.category_id
 where h.utc_hour >= ? and h.utc_hour < ?
   and h.dimension_kind = 'host'
 {filters}
 group by h.category_id
 order by sum(h.download) desc, 1 asc
 limit ?
";

pub const TOTALS_DAILY_DIM: &str = "
select coalesce(sum(upload), 0), coalesce(sum(download), 0),
       coalesce(sum(connection_count), 0), coalesce(sum(active_duration_sec), 0)
  from traffic_daily_dimension h
 where h.utc_day >= ? and h.utc_day < ?
   and h.dimension_kind = ?
 {filters}
";

pub const SERIES_DAILY_DIM: &str = "
select utc_day, sum(upload), sum(download), sum(connection_count), sum(active_duration_sec)
  from traffic_daily_dimension h
 where h.utc_day >= ? and h.utc_day < ?
   and h.dimension_kind = ?
 {filters}
 group by utc_day
 order by utc_day
";

pub const RANK_DAILY_DIM: &str = "
select case when coalesce(d.value, '') = '' then '__unknown__' else d.value end,
       sum(h.upload), sum(h.download), sum(h.connection_count), sum(h.active_duration_sec)
  from traffic_daily_dimension h
  left join dimension_dict d
    on d.dimension_kind = h.dimension_kind and d.dimension_id = h.dimension_id
 where h.utc_day >= ? and h.utc_day < ?
   and h.dimension_kind = ?
 {filters}
 group by 1
 order by sum(h.download) desc, 1 asc
 limit ?
";

/// 与 `RANK_HOURLY_CATEGORY` 相同：按 `category_id` 分组并固定 host kind 去重。
pub const RANK_DAILY_CATEGORY: &str = "
-- 五维物化后同一份流量有五行；固定 dimension_kind='host' 以免总量放大五倍。
select case when coalesce(d.value, '') = '' then '__unknown__' else d.value end,
       sum(h.upload), sum(h.download), sum(h.connection_count), sum(h.active_duration_sec)
  from traffic_daily_dimension h
  left join dimension_dict d
    on d.dimension_kind = 'category' and d.dimension_id = h.category_id
 where h.utc_day >= ? and h.utc_day < ?
   and h.dimension_kind = 'host'
 {filters}
 group by h.category_id
 order by sum(h.download) desc, 1 asc
 limit ?
";

pub const TOTALS_DAILY_CORE: &str = "
select coalesce(sum(upload), 0), coalesce(sum(download), 0),
       coalesce(sum(connection_count), 0), coalesce(sum(active_duration_sec), 0)
  from traffic_daily_core
 where utc_day >= ?1 and utc_day < ?2
";

pub const SERIES_DAILY_CORE: &str = "
select utc_day, upload, download, connection_count, active_duration_sec
  from traffic_daily_core
 where utc_day >= ?1 and utc_day < ?2
   and category_id = 0
 order by utc_day
";

pub const COVERAGE_DAILY: &str = "
select utc_day, covered_sec, gap_sec, reasons_json
  from coverage_daily
 where utc_day >= ?1 and utc_day < ?2
 order by utc_day
";

pub fn corpus() -> &'static [(&'static str, &'static str)] {
    &[
        ("totals_raw", TOTALS_RAW),
        ("series_raw", SERIES_RAW),
        ("rank_raw", RANK_RAW),
        ("rank_raw_attr", RANK_RAW_ATTR),
        ("rank_raw_rule", RANK_RAW_RULE),
        ("rank_raw_chain", RANK_RAW_CHAIN),
        ("sessions_keyset", SESSIONS_KEYSET),
        ("coverage_raw", COVERAGE_RAW),
        ("share_residential_raw", SHARE_RESIDENTIAL_RAW),
        ("totals_hourly_dimension", TOTALS_HOURLY),
        ("series_hourly_dimension", SERIES_HOURLY),
        ("rank_hourly_dimension", RANK_HOURLY),
        ("rank_hourly_category", RANK_HOURLY_CATEGORY),
        ("totals_daily_dimension", TOTALS_DAILY_DIM),
        ("series_daily_dimension", SERIES_DAILY_DIM),
        ("rank_daily_dimension", RANK_DAILY_DIM),
        ("rank_daily_category", RANK_DAILY_CATEGORY),
        ("totals_daily_core", TOTALS_DAILY_CORE),
        ("series_daily_core", SERIES_DAILY_CORE),
        ("coverage_daily", COVERAGE_DAILY),
    ]
}

pub fn lookup(name: &str) -> Option<&'static str> {
    corpus()
        .iter()
        .find(|(key, _)| *key == name)
        .map(|item| item.1)
}

pub fn render_sql(sql: &str, filters_sql: &str) -> String {
    sql.replace("{filters}", filters_sql)
}

/// raw 层 `dimension_dict.dimension_kind`。
pub fn dimension_kind_sql(kind: DimensionKind) -> &'static str {
    match kind {
        DimensionKind::Category => "category",
        DimensionKind::Host => "host",
        DimensionKind::Process => "process",
        DimensionKind::Rule => "rule",
        DimensionKind::Chain => "chain",
        DimensionKind::Network => "network",
    }
}

/// 维度层 `traffic_*_dimension.dimension_kind`。`Rule` 映射到 `rule_group`，不覆写 raw 的 `rule`。
pub fn dimension_kind_sql_layer(kind: DimensionKind) -> &'static str {
    match kind {
        DimensionKind::Rule => "rule_group",
        other => dimension_kind_sql(other),
    }
}

/// 返回 (sql_fragment, params)。无过滤时 fragment 为空串。用户值只进绑定参数。
pub fn filter_clause(filters: &ReportFilters) -> (String, Vec<String>) {
    let mut fragment = String::new();
    let mut params = Vec::new();
    if let Some(value) = &filters.host {
        if value == UNKNOWN_IDENTITY {
            fragment.push_str(" and coalesce(s.host, '') = ''");
        } else {
            fragment.push_str(" and s.host = ?");
            params.push(value.clone());
        }
    }
    if let Some(value) = &filters.process {
        fragment.push_str(" and a.process_id = (select dimension_id from dimension_dict where dimension_kind = 'process' and value = ?)");
        params.push(value.clone());
    }
    if let Some(value) = &filters.rule {
        fragment.push_str(" and ");
        fragment.push_str(RULE_KEY_SQL);
        fragment.push_str(" = ?");
        params.push(value.clone());
    }
    if let Some(value) = &filters.network {
        fragment.push_str(" and a.network_id = (select dimension_id from dimension_dict where dimension_kind = 'network' and value = ?)");
        params.push(value.clone());
    }
    if let Some(value) = &filters.chain {
        fragment.push_str(" and chain_identity(a.chain_key) = ?");
        params.push(value.clone());
    }
    if let Some(value) = &filters.category {
        fragment.push_str(" and a.primary_category_id = (select dimension_id from dimension_dict where dimension_kind = 'category' and value = ?)");
        params.push(value.clone());
    }
    (fragment, params)
}

/// 维度层过滤。category 走 `category_id`；其余只在 grouping 与过滤字段一致时生效。
pub fn dimension_filter_clause(
    filters: &ReportFilters,
    grouping: DimensionKind,
) -> (String, Vec<String>) {
    let mut fragment = String::new();
    let mut params = Vec::new();
    if let Some(value) = &filters.category {
        fragment.push_str(" and h.category_id = (select dimension_id from dimension_dict where dimension_kind = 'category' and value = ?)");
        params.push(value.clone());
    }
    match grouping {
        DimensionKind::Host => {
            append_dim_identity(&mut fragment, &mut params, "host", filters.host.as_ref())
        }
        DimensionKind::Process => append_dim_identity(
            &mut fragment,
            &mut params,
            "process",
            filters.process.as_ref(),
        ),
        DimensionKind::Rule => append_dim_identity(
            &mut fragment,
            &mut params,
            "rule_group",
            filters.rule.as_ref(),
        ),
        DimensionKind::Chain => {
            append_dim_identity(&mut fragment, &mut params, "chain", filters.chain.as_ref())
        }
        DimensionKind::Network => append_dim_identity(
            &mut fragment,
            &mut params,
            "network",
            filters.network.as_ref(),
        ),
        DimensionKind::Category => {}
    }
    (fragment, params)
}

fn append_dim_identity(
    fragment: &mut String,
    params: &mut Vec<String>,
    kind: &'static str,
    value: Option<&String>,
) {
    let Some(value) = value else {
        return;
    };
    if value == UNKNOWN_IDENTITY && kind == "host" {
        fragment.push_str(" and h.dimension_id = 0");
        return;
    }
    match kind {
        "host" => fragment.push_str(" and h.dimension_id = (select dimension_id from dimension_dict where dimension_kind = 'host' and value = ?)"),
        "process" => fragment.push_str(" and h.dimension_id = (select dimension_id from dimension_dict where dimension_kind = 'process' and value = ?)"),
        "rule_group" => fragment.push_str(" and h.dimension_id = (select dimension_id from dimension_dict where dimension_kind = 'rule_group' and value = ?)"),
        "chain" => fragment.push_str(" and h.dimension_id = (select dimension_id from dimension_dict where dimension_kind = 'chain' and value = ?)"),
        "network" => fragment.push_str(" and h.dimension_id = (select dimension_id from dimension_dict where dimension_kind = 'network' and value = ?)"),
        _ => return,
    }
    params.push(value.clone());
}

pub fn merge_sql_params(
    prefix: impl IntoIterator<Item = Value>,
    filters: &[String],
    suffix: impl IntoIterator<Item = Value>,
) -> Vec<Value> {
    let mut out: Vec<Value> = prefix.into_iter().collect();
    out.extend(filters.iter().cloned().map(Value::Text));
    out.extend(suffix);
    out
}

pub fn raw_rank_sql(kind: DimensionKind) -> &'static str {
    match kind {
        DimensionKind::Host => "rank_raw",
        DimensionKind::Chain => "rank_raw_chain",
        DimensionKind::Rule => "rank_raw_rule",
        DimensionKind::Process | DimensionKind::Network | DimensionKind::Category => {
            "rank_raw_attr"
        }
    }
}

pub fn dim_rank_sql(kind: DimensionKind, hourly: bool) -> &'static str {
    match (kind, hourly) {
        (DimensionKind::Category, true) => "rank_hourly_category",
        (DimensionKind::Category, false) => "rank_daily_category",
        (_, true) => "rank_hourly_dimension",
        (_, false) => "rank_daily_dimension",
    }
}

#[cfg(test)]
mod sql_corpus_tests {
    use super::*;

    #[test]
    fn every_public_query_is_named_and_parameterized() {
        for (name, sql) in corpus() {
            assert!(!name.is_empty());
            assert!(sql.contains('?'), "{name} 必须参数化");
            assert!(!sql.to_ascii_lowercase().contains("offset "));
        }
    }

    #[test]
    fn filter_clause_binds_user_values() {
        let filters = ReportFilters {
            host: Some("'; drop table x --".into()),
            ..ReportFilters::default()
        };
        let (fragment, params) = filter_clause(&filters);
        assert!(fragment.contains("s.host = ?"));
        assert!(!fragment.contains("drop table"));
        assert_eq!(params, vec!["'; drop table x --"]);
    }

    #[test]
    fn unknown_host_filter_matches_empty_host_without_binding_sentinel() {
        let filters = ReportFilters {
            host: Some(UNKNOWN_IDENTITY.into()),
            ..ReportFilters::default()
        };
        let (fragment, params) = filter_clause(&filters);
        assert!(fragment.contains("coalesce(s.host, '') = ''"));
        assert!(!fragment.contains("s.host = ?"));
        assert!(params.is_empty());
        let mut dim = String::new();
        let mut dim_params = Vec::new();
        append_dim_identity(
            &mut dim,
            &mut dim_params,
            "host",
            Some(&UNKNOWN_IDENTITY.to_string()),
        );
        assert_eq!(dim, " and h.dimension_id = 0");
        assert!(dim_params.is_empty());
    }

    #[test]
    fn share_residential_raw_is_named_and_uses_category_null() {
        let sql = lookup("share_residential_raw").expect("share_residential_raw");
        assert!(sql.contains("primary_category_id is not null"));
        assert!(sql.contains('?'));
    }

    #[test]
    fn unknown_sentinel_is_stable() {
        assert_eq!(UNKNOWN_IDENTITY, "__unknown__");
        assert!(RANK_HOURLY.contains("'__unknown__'"));
        assert!(RANK_DAILY_DIM.contains("'__unknown__'"));
        assert!(RANK_RAW_CHAIN.contains("'__unknown__'"));
    }

    #[test]
    fn dimension_layer_rule_kind_is_rule_group() {
        assert_eq!(dimension_kind_sql(DimensionKind::Rule), "rule");
        assert_eq!(dimension_kind_sql_layer(DimensionKind::Rule), "rule_group");
        assert_eq!(dimension_kind_sql_layer(DimensionKind::Host), "host");
    }
}
