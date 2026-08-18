//! 命名 SQL corpus。值全部参数化；前端不传 SQL。

pub const TOTALS_RAW: &str = "
select coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0),
       count(distinct m.session_pk),
       count(distinct m.utc_minute) * 60
  from connection_minute m
  join connection_session s on s.session_pk = m.session_pk
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ?1 and m.utc_minute < ?2
   and (?3 = 0 or s.host = ?4)
   and (?5 = 0 or a.process_id = (select dimension_id from dimension_dict where dimension_kind = 'process' and value = ?6))
   and (?7 = 0 or a.rule_id = (select dimension_id from dimension_dict where dimension_kind = 'rule' and value = ?8))
   and (?9 = 0 or a.network_id = (select dimension_id from dimension_dict where dimension_kind = 'network' and value = ?10))
   and (?11 = 0 or a.chain_key = ?12)
";

pub const SERIES_RAW: &str = "
select (m.utc_minute / ?3) * ?3 as bucket,
       coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0),
       count(distinct m.session_pk),
       count(distinct m.utc_minute) * 60
  from connection_minute m
  join connection_session s on s.session_pk = m.session_pk
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ?1 and m.utc_minute < ?2
   and (?4 = 0 or s.host = ?5)
 group by bucket
 order by bucket
";

pub const RANK_RAW: &str = "
select coalesce(s.host, ''),
       coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0),
       count(distinct m.session_pk),
       count(distinct m.utc_minute) * 60
  from connection_minute m
  join connection_session s on s.session_pk = m.session_pk
 where m.utc_minute >= ?1 and m.utc_minute < ?2
 group by s.host
 order by sum(m.download) desc, s.host asc
 limit ?3
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

pub const TOTALS_HOURLY: &str = "
select coalesce(sum(upload), 0), coalesce(sum(download), 0),
       coalesce(sum(connection_count), 0), coalesce(sum(active_duration_sec), 0)
  from traffic_hourly_dimension
 where utc_hour >= ?1 and utc_hour < ?2
   and dimension_kind = ?3
";

pub const SERIES_HOURLY: &str = "
select utc_hour, sum(upload), sum(download), sum(connection_count), sum(active_duration_sec)
  from traffic_hourly_dimension
 where utc_hour >= ?1 and utc_hour < ?2
   and dimension_kind = ?3
 group by utc_hour
 order by utc_hour
";

pub const RANK_HOURLY: &str = "
select d.value, sum(h.upload), sum(h.download), sum(h.connection_count), sum(h.active_duration_sec)
  from traffic_hourly_dimension h
  join dimension_dict d
    on d.dimension_kind = h.dimension_kind and d.dimension_id = h.dimension_id
 where h.utc_hour >= ?1 and h.utc_hour < ?2
   and h.dimension_kind = ?3
 group by d.value
 order by sum(h.download) desc, d.value asc
 limit ?4
";

pub const TOTALS_DAILY_DIM: &str = "
select coalesce(sum(upload), 0), coalesce(sum(download), 0),
       coalesce(sum(connection_count), 0), coalesce(sum(active_duration_sec), 0)
  from traffic_daily_dimension
 where utc_day >= ?1 and utc_day < ?2
   and dimension_kind = ?3
";

pub const SERIES_DAILY_DIM: &str = "
select utc_day, sum(upload), sum(download), sum(connection_count), sum(active_duration_sec)
  from traffic_daily_dimension
 where utc_day >= ?1 and utc_day < ?2
   and dimension_kind = ?3
 group by utc_day
 order by utc_day
";

pub const RANK_DAILY_DIM: &str = "
select d.value, sum(h.upload), sum(h.download), sum(h.connection_count), sum(h.active_duration_sec)
  from traffic_daily_dimension h
  join dimension_dict d
    on d.dimension_kind = h.dimension_kind and d.dimension_id = h.dimension_id
 where h.utc_day >= ?1 and h.utc_day < ?2
   and h.dimension_kind = ?3
 group by d.value
 order by sum(h.download) desc, d.value asc
 limit ?4
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
        ("sessions_keyset", SESSIONS_KEYSET),
        ("coverage_raw", COVERAGE_RAW),
        ("totals_hourly_dimension", TOTALS_HOURLY),
        ("series_hourly_dimension", SERIES_HOURLY),
        ("rank_hourly_dimension", RANK_HOURLY),
        ("totals_daily_dimension", TOTALS_DAILY_DIM),
        ("series_daily_dimension", SERIES_DAILY_DIM),
        ("rank_daily_dimension", RANK_DAILY_DIM),
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

pub fn dimension_kind_sql(kind: crate::c3::query::DimensionKind) -> &'static str {
    match kind {
        crate::c3::query::DimensionKind::Category => "category",
        crate::c3::query::DimensionKind::Host => "host",
        crate::c3::query::DimensionKind::Process => "process",
        crate::c3::query::DimensionKind::Rule => "rule",
        crate::c3::query::DimensionKind::Chain => "chain",
        crate::c3::query::DimensionKind::Network => "network",
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
}
