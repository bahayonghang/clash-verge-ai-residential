//! C3 前向 schema。不改写 C1 已发布 migration 文本。

pub const C3_SCHEMA_VERSION: i32 = 2;
pub const C3_MIGRATION_CHECKSUM: &str = "c3-report-v2";
pub const C3_ARCHIVE_SCHEMA_VERSION: i32 = 4;
pub const C3_ARCHIVE_MIGRATION_CHECKSUM: &str = "c3-archive-v4";

pub const C3_TABLES: &[&str] = &[
    "dimension_dict",
    "connection_session_attr",
    "traffic_hourly_dimension",
    "traffic_daily_dimension",
    "traffic_daily_core",
    "coverage_daily",
    "retention_state",
    "retention_watermark",
    "report_snapshot_meta",
    "report_archive",
];

pub const C3_DDL: &str = "
create table if not exists dimension_dict (
    dimension_kind text not null,
    dimension_id integer not null,
    value text not null,
    primary key (dimension_kind, dimension_id)
) strict;
create unique index if not exists dimension_dict_value
    on dimension_dict(dimension_kind, value);
create unique index if not exists connection_session_identity
    on connection_session(epoch_id, connection_id);
create table if not exists connection_session_attr (
    session_pk integer primary key,
    host_id integer,
    process_id integer,
    rule_id integer,
    network_id integer,
    chain_key text,
    policy_version integer,
    primary_category_id integer,
    started_utc integer not null,
    ended_utc integer
) strict;
create table if not exists traffic_hourly_dimension (
    utc_hour integer not null,
    category_id integer not null,
    dimension_kind text not null,
    dimension_id integer not null,
    upload integer not null,
    download integer not null,
    connection_count integer not null,
    active_duration_sec integer not null,
    primary key (utc_hour, category_id, dimension_kind, dimension_id)
) strict;
create table if not exists traffic_daily_dimension (
    utc_day integer not null,
    category_id integer not null,
    dimension_kind text not null,
    dimension_id integer not null,
    upload integer not null,
    download integer not null,
    connection_count integer not null,
    active_duration_sec integer not null,
    primary key (utc_day, category_id, dimension_kind, dimension_id)
) strict;
create table if not exists traffic_daily_core (
    utc_day integer not null,
    category_id integer not null,
    upload integer not null,
    download integer not null,
    connection_count integer not null,
    active_duration_sec integer not null,
    primary key (utc_day, category_id)
) strict;
create table if not exists coverage_daily (
    utc_day integer not null primary key,
    covered_sec integer not null,
    gap_sec integer not null,
    reasons_json text not null
) strict;
create table if not exists retention_state (
    layer text not null,
    chunk_utc integer not null,
    status text not null,
    checksum text not null,
    updated_utc integer not null,
    primary key (layer, chunk_utc)
) strict;
create table if not exists retention_watermark (
    layer text primary key,
    watermark_utc integer not null,
    delete_watermark_utc integer not null
) strict;
create table if not exists report_snapshot_meta (
    token text primary key,
    query_fingerprint text not null,
    schema_version integer not null,
    data_version integer not null,
    created_utc integer not null,
    expires_utc integer not null,
    bytes integer not null,
    checksum text not null,
    spool_name text
) strict;
create index if not exists idx_connection_minute_utc
    on connection_minute(utc_minute, session_pk);
create index if not exists idx_connection_session_started
    on connection_session(started_utc);
create index if not exists idx_hourly_dim_lookup
    on traffic_hourly_dimension(utc_hour, dimension_kind, dimension_id);
create index if not exists idx_daily_dim_lookup
    on traffic_daily_dimension(utc_day, dimension_kind, dimension_id);
create index if not exists idx_daily_core_day
    on traffic_daily_core(utc_day);
create index if not exists idx_session_attr_host
    on connection_session_attr(host_id);
insert or ignore into retention_watermark(layer, watermark_utc, delete_watermark_utc)
    values ('hourly', 0, 0), ('daily', 0, 0), ('core', 0, 0), ('raw_delete', 0, 0);
";

pub const C3_ARCHIVE_DDL: &str = "
create table if not exists report_archive (
    archive_id text primary key,
    kind text not null,
    range_start_utc integer not null,
    range_end_utc integer not null,
    display_timezone text not null,
    grouping text not null,
    query_fingerprint text not null,
    status text not null,
    generated_utc integer not null,
    data_version integer,
    coverage_status text,
    totals_upload integer,
    totals_download integer,
    connection_count integer,
    result_json text,
    error_code text,
    note_zh text
) strict;
create unique index if not exists report_archive_period_uniq
    on report_archive(kind, range_start_utc, query_fingerprint);
create index if not exists idx_report_archive_kind_start
    on report_archive(kind, range_start_utc desc);
";

pub fn c3_table_allowlist() -> &'static [&'static str] {
    C3_TABLES
}

#[cfg(test)]
mod c3_schema_contract_tests {
    use super::*;

    #[test]
    fn c3_tables_do_not_include_c4_alert_schema() {
        for name in C3_TABLES {
            assert!(!name.contains("alert"));
            assert!(!name.contains("notification"));
            assert!(!name.contains("outbox"));
        }
        assert!(C3_DDL.contains("traffic_hourly_dimension"));
        assert!(!C3_DDL.contains("create table") || !C3_DDL.contains("alert_"));
        assert!(!C3_DDL.contains("notification_outbox"));
        assert!(!C3_DDL.contains("report_archive"));
        assert!(C3_TABLES.contains(&"report_archive"));
        assert!(C3_ARCHIVE_DDL.contains("report_archive"));
        assert_eq!(C3_MIGRATION_CHECKSUM, "c3-report-v2");
        assert_eq!(C3_ARCHIVE_MIGRATION_CHECKSUM, "c3-archive-v4");
        assert_eq!(C3_ARCHIVE_SCHEMA_VERSION, 4);
    }
}
