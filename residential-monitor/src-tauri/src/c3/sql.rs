//! 命名 SQL corpus。值全部参数化；前端不传 SQL。
//!
//! 带 `{filters}` 的模板由 `render_sql` 替换一次；排名模板再由
//! `render_rank_sql` 用枚举白名单替换 `{order_by}`。用户值只进绑定参数。

use crate::c3::query::{DimensionKind, ReportFilters, SortField, SortSpec};
use rusqlite::types::Value;

/// 维度值缺失时排名 identity 的哨兵。真实 `dimension_dict.value` 不得等于该字面量。
pub const UNKNOWN_IDENTITY: &str = "__unknown__";

/// 家宽子集过滤哨兵。不是字典值。
pub const RESIDENTIAL_ACCOUNTING_FILTER: &str = "__residential__";

/// raw 层家宽归属的唯一 SQL 谓词。
///
/// 已写入的历史 category 保持权威；只有 category 为空的 legacy raw 行才以当前 target
/// 与已保存链路恢复。`EXISTS` 保证多个 target / 链路节点不会倍增流量。内置 `家宽`
/// target 与 [`crate::residential::RESIDENTIAL_SELECTOR`] 一样做包含匹配，其它 target 精确匹配。
pub const RESIDENTIAL_RAW_MEMBERSHIP_SQL: &str = "(a.primary_category_id is not null or (a.primary_category_id is null and exists (select 1 from connection_chain rc join target_item rt on rt.set_id = 1 where rc.session_pk = m.session_pk and (rc.node = rt.name or (rt.name = '家宽' and instr(rc.node, '家宽') > 0)))))";

/// 进程 identity 缺失。与字段归因、未知下钻共用。
pub const PROCESS_MISSING_SQL: &str = "a.process_id is null or not exists (select 1 from dimension_dict q where q.dimension_kind='process' and q.dimension_id=a.process_id)";

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
 {order_by}
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
 {order_by}
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
 {order_by}
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
 {order_by}
 limit ?
";

/// Top N identity × 非空 chain_key 的 download 聚合。identity 表达式与 RANK_RAW 第 1 列相同。
pub const RANK_RAW_EXITS: &str = "
select case when coalesce(s.host, '') = '' then '__unknown__' else s.host end,
       a.chain_key,
       coalesce(sum(m.download), 0)
  from connection_minute m
  join connection_session s on s.session_pk = m.session_pk
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ? and m.utc_minute < ?
 {filters}
   and trim(coalesce(a.chain_key, '')) != ''
   and case when coalesce(s.host, '') = '' then '__unknown__' else s.host end in ({identities})
 group by 1, 2
";

/// Process 出口聚合。identity 表达式与 RANK_RAW_ATTR 第 1 列相同。
pub const RANK_RAW_ATTR_EXITS: &str = "
select case when coalesce(d.value, '') = '' then '__unknown__' else d.value end,
       a.chain_key,
       coalesce(sum(m.download), 0)
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
   and trim(coalesce(a.chain_key, '')) != ''
   and case when coalesce(d.value, '') = '' then '__unknown__' else d.value end in ({identities})
 group by 1, 2
";

/// Rule 出口聚合。identity 表达式与 RANK_RAW_RULE 第 1 列相同。
pub const RANK_RAW_RULE_EXITS: &str = "
select coalesce(last_chain_hop(a.chain_key), (select value from dimension_dict where dimension_kind = 'rule' and dimension_id = a.rule_id), 'DIRECT'),
       a.chain_key,
       coalesce(sum(m.download), 0)
  from connection_minute m
  join connection_session s on s.session_pk = m.session_pk
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ? and m.utc_minute < ?
 {filters}
   and trim(coalesce(a.chain_key, '')) != ''
   and coalesce(last_chain_hop(a.chain_key), (select value from dimension_dict where dimension_kind = 'rule' and dimension_id = a.rule_id), 'DIRECT') in ({identities})
 group by 1, 2
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

/// 一次扫描同时得到家宽分子与可归因观测分母。
pub const SHARE_RESIDENTIAL_RAW: &str = "
select
  coalesce(sum(case when {residential_membership} then m.upload end), 0),
  coalesce(sum(case when {residential_membership} then m.download end), 0),
  coalesce(sum(m.upload), 0),
  coalesce(sum(m.download), 0)
from connection_minute m
left join connection_session_attr a on a.session_pk = m.session_pk
where m.utc_minute >= ?1 and m.utc_minute < ?2
";

/// audit 全量投影行数上限。返回行数等于该值时视为截断，守恒字段写 null。
pub const AUDIT_MAX_ROWS: i64 = 200_000;

pub const AUDIT_RESIDENTIAL_HOST_RULE_PROCESS: &str = "
select
  case when coalesce(h.value,'') = '' then '__unknown__' else h.value end,
  coalesce(r.value, '__unknown__'),
  case when coalesce(p.value,'') = '' then '__unknown__' else p.value end,
  coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0),
  count(distinct m.session_pk)
from connection_minute m
join connection_session s on s.session_pk = m.session_pk
left join connection_session_attr a on a.session_pk = m.session_pk
left join dimension_dict h on h.dimension_kind = 'host'    and h.dimension_id = a.host_id
left join dimension_dict r on r.dimension_kind = 'rule'    and r.dimension_id = a.rule_id
left join dimension_dict p on p.dimension_kind = 'process' and p.dimension_id = a.process_id
where m.utc_minute >= ?1 and m.utc_minute < ?2 and {residential_membership}
group by 1, 2, 3
order by 4 + 5 desc
limit ?3
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
 {order_by}
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
 {order_by}
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
 {order_by}
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
 {order_by}
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
        ("rank_raw_exits", RANK_RAW_EXITS),
        ("rank_raw_attr_exits", RANK_RAW_ATTR_EXITS),
        ("rank_raw_rule_exits", RANK_RAW_RULE_EXITS),
        ("sessions_keyset", SESSIONS_KEYSET),
        ("coverage_raw", COVERAGE_RAW),
        ("share_residential_raw", SHARE_RESIDENTIAL_RAW),
        (
            "audit_residential_host_rule_process",
            AUDIT_RESIDENTIAL_HOST_RULE_PROCESS,
        ),
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

/// 将 Top N identity 列表绑成 `IN (?, …)`。调用方必须保证 `identity_count > 0`。
pub fn render_exit_sql(sql: &str, filters_sql: &str, identity_count: usize) -> String {
    let placeholders = vec!["?"; identity_count].join(", ");
    let rendered = render_sql(sql, filters_sql).replace("{identities}", &placeholders);
    debug_assert!(!rendered.contains("{filters}"));
    debug_assert!(!rendered.contains("{identities}"));
    rendered
}

pub fn raw_exit_sql(kind: DimensionKind) -> &'static str {
    match kind {
        DimensionKind::Rule => RANK_RAW_RULE_EXITS,
        DimensionKind::Process => RANK_RAW_ATTR_EXITS,
        _ => RANK_RAW_EXITS,
    }
}

pub fn raw_exit_sql_name(kind: DimensionKind) -> &'static str {
    match kind {
        DimensionKind::Rule => "rank_raw_rule_exits",
        DimensionKind::Process => "rank_raw_attr_exits",
        _ => "rank_raw_exits",
    }
}

/// 将 share 模板中的家宽归属槽位替换为与报告过滤相同的 raw 谓词。
pub fn render_residential_membership_sql(sql: &str) -> String {
    let rendered = sql.replace("{residential_membership}", RESIDENTIAL_RAW_MEMBERSHIP_SQL);
    debug_assert!(!rendered.contains("{residential_membership}"));
    rendered
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankLayer {
    Raw,
    Dimension,
}

/// 排名字段和方向只来自反序列化后的枚举；调用方不能传入 SQL 片段。
pub fn render_rank_sql(sql: &str, filters_sql: &str, sort: &SortSpec, layer: RankLayer) -> String {
    let direction = if sort.descending { "desc" } else { "asc" };
    let aggregate_alias = match layer {
        RankLayer::Raw => "m",
        RankLayer::Dimension => "h",
    };
    let order_by = match sort.field {
        SortField::Upload => format!("order by sum({aggregate_alias}.upload) {direction}, 1 asc"),
        SortField::Download => {
            format!("order by sum({aggregate_alias}.download) {direction}, 1 asc")
        }
        SortField::Name | SortField::Identity => format!("order by 1 {direction}"),
    };
    let rendered = render_sql(sql, filters_sql).replace("{order_by}", &order_by);
    debug_assert!(!rendered.contains("{filters}"));
    debug_assert!(!rendered.contains("{order_by}"));
    rendered
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
        if value == UNKNOWN_IDENTITY {
            fragment.push_str(" and (");
            fragment.push_str(PROCESS_MISSING_SQL);
            fragment.push(')');
        } else {
            fragment.push_str(" and a.process_id = (select dimension_id from dimension_dict where dimension_kind = 'process' and value = ?)");
            params.push(value.clone());
        }
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
        if value == RESIDENTIAL_ACCOUNTING_FILTER {
            fragment.push_str(" and ");
            fragment.push_str(RESIDENTIAL_RAW_MEMBERSHIP_SQL);
        } else {
            fragment.push_str(" and a.primary_category_id = (select dimension_id from dimension_dict where dimension_kind = 'category' and value = ?)");
            params.push(value.clone());
        }
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
        if value == RESIDENTIAL_ACCOUNTING_FILTER {
            fragment.push_str(" and h.category_id != 0");
        } else {
            fragment.push_str(" and h.category_id = (select dimension_id from dimension_dict where dimension_kind = 'category' and value = ?)");
            params.push(value.clone());
        }
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
    if value == UNKNOWN_IDENTITY && matches!(kind, "host" | "process") {
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
    fn rank_order_is_enum_rendered_for_every_field_and_direction() {
        for layer in [RankLayer::Raw, RankLayer::Dimension] {
            let template = match layer {
                RankLayer::Raw => RANK_RAW,
                RankLayer::Dimension => RANK_HOURLY,
            };
            for field in [
                SortField::Upload,
                SortField::Download,
                SortField::Name,
                SortField::Identity,
            ] {
                for descending in [false, true] {
                    let rendered = render_rank_sql(
                        template,
                        " and 1 = 1",
                        &SortSpec { field, descending },
                        layer,
                    );
                    assert!(!rendered.contains("{filters}"));
                    assert!(!rendered.contains("{order_by}"));
                    assert!(rendered.contains("and 1 = 1"));
                    let direction = if descending { "desc" } else { "asc" };
                    match field {
                        SortField::Upload | SortField::Download => {
                            let column = if field == SortField::Upload {
                                "upload"
                            } else {
                                "download"
                            };
                            let alias = if layer == RankLayer::Raw { "m" } else { "h" };
                            assert!(rendered.contains(&format!(
                                "order by sum({alias}.{column}) {direction}, 1 asc"
                            )));
                        }
                        SortField::Name | SortField::Identity => {
                            assert!(rendered.contains(&format!("order by 1 {direction}")));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn every_rank_template_resolves_internal_slots() {
        for (name, template, layer) in [
            ("rank_raw", RANK_RAW, RankLayer::Raw),
            ("rank_raw_attr", RANK_RAW_ATTR, RankLayer::Raw),
            ("rank_raw_rule", RANK_RAW_RULE, RankLayer::Raw),
            ("rank_raw_chain", RANK_RAW_CHAIN, RankLayer::Raw),
            ("rank_hourly", RANK_HOURLY, RankLayer::Dimension),
            (
                "rank_hourly_category",
                RANK_HOURLY_CATEGORY,
                RankLayer::Dimension,
            ),
            ("rank_daily", RANK_DAILY_DIM, RankLayer::Dimension),
            (
                "rank_daily_category",
                RANK_DAILY_CATEGORY,
                RankLayer::Dimension,
            ),
        ] {
            let rendered = render_rank_sql(template, "", &SortSpec::default(), layer);
            assert!(!rendered.contains('{'), "{name} has an unresolved slot");
            assert!(rendered.contains("order by"), "{name} has no order");
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
    fn unknown_process_filter_matches_missing_process_without_binding_sentinel() {
        let filters = ReportFilters {
            process: Some(UNKNOWN_IDENTITY.into()),
            ..ReportFilters::default()
        };
        let (fragment, params) = filter_clause(&filters);
        assert!(fragment.contains(PROCESS_MISSING_SQL));
        assert!(!fragment.contains("a.process_id = (select"));
        assert!(params.is_empty());
        let mut dim = String::new();
        let mut dim_params = Vec::new();
        append_dim_identity(
            &mut dim,
            &mut dim_params,
            "process",
            Some(&UNKNOWN_IDENTITY.to_string()),
        );
        assert_eq!(dim, " and h.dimension_id = 0");
        assert!(dim_params.is_empty());
    }

    #[test]
    fn residential_accounting_filter_uses_legacy_safe_raw_membership() {
        let filters = ReportFilters {
            category: Some(RESIDENTIAL_ACCOUNTING_FILTER.into()),
            ..ReportFilters::default()
        };
        let (fragment, params) = filter_clause(&filters);
        assert!(fragment.contains("a.primary_category_id is not null"));
        assert!(fragment.contains("a.primary_category_id is null and exists"));
        assert!(fragment.contains("connection_chain"));
        assert!(fragment.contains("target_item"));
        assert!(fragment.contains("instr(rc.node, '家宽')"));
        assert!(params.is_empty());
        let (dim, dim_params) = dimension_filter_clause(&filters, DimensionKind::Process);
        assert!(dim.contains("h.category_id != 0"));
        assert!(dim_params.is_empty());
    }

    #[test]
    fn share_and_filter_use_the_same_residential_membership_predicate() {
        let template = lookup("share_residential_raw").expect("share_residential_raw");
        let sql = render_residential_membership_sql(template);
        assert_eq!(sql.matches(RESIDENTIAL_RAW_MEMBERSHIP_SQL).count(), 2);
        assert!(!sql.contains("{residential_membership}"));
        assert!(sql.contains('?'));
        assert!(RESIDENTIAL_RAW_MEMBERSHIP_SQL.contains(crate::residential::RESIDENTIAL_SELECTOR));
    }

    #[test]
    fn residential_membership_plan_uses_session_key_lookup() {
        let connection = rusqlite::Connection::open_in_memory().expect("open");
        connection
            .execute_batch(
                "create table connection_minute(utc_minute integer, session_pk integer, upload integer, download integer);
                 create table connection_session(session_pk integer primary key, host text);
                 create table connection_session_attr(session_pk integer primary key, primary_category_id integer);
                 create table connection_chain(session_pk integer, position integer, node text, primary key(session_pk, position));
                 create table target_item(set_id integer, position integer, name text, primary key(set_id, position));",
            )
            .expect("schema");
        let filters = ReportFilters {
            category: Some(RESIDENTIAL_ACCOUNTING_FILTER.into()),
            ..ReportFilters::default()
        };
        let (fragment, _) = filter_clause(&filters);
        let sql = format!("explain query plan {}", render_sql(TOTALS_RAW, &fragment));
        let mut statement = connection.prepare(&sql).expect("prepare");
        let details: Vec<String> = statement
            .query_map(rusqlite::params![0, 1], |row| row.get(3))
            .expect("query plan")
            .collect::<Result<_, _>>()
            .expect("plan rows");
        assert!(
            details.iter().any(|detail| {
                detail.contains("connection_chain")
                    && detail.contains("session_pk")
                    && (detail.contains("INDEX") || detail.contains("PRIMARY KEY"))
            }),
            "plan={details:?}"
        );
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

    #[test]
    fn rank_exit_templates_bind_identities() {
        for (name, template) in [
            ("rank_raw_exits", RANK_RAW_EXITS),
            ("rank_raw_attr_exits", RANK_RAW_ATTR_EXITS),
            ("rank_raw_rule_exits", RANK_RAW_RULE_EXITS),
        ] {
            let rendered = render_exit_sql(template, " and 1 = 1", 2);
            assert!(!rendered.contains('{'), "{name} has an unresolved slot");
            assert!(
                rendered.contains("in (?, ?)"),
                "{name} missing identity bind"
            );
            assert!(rendered.contains("trim(coalesce(a.chain_key, '')) != ''"));
        }
        assert_eq!(raw_exit_sql_name(DimensionKind::Host), "rank_raw_exits");
        assert_eq!(
            raw_exit_sql_name(DimensionKind::Process),
            "rank_raw_attr_exits"
        );
        assert_eq!(
            raw_exit_sql_name(DimensionKind::Rule),
            "rank_raw_rule_exits"
        );
    }
}
