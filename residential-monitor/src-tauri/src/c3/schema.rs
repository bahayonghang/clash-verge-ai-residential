//! C3 前向 schema。不改写 C1 已发布 migration 文本。

pub const C3_SCHEMA_VERSION: i32 = 2;
pub const C3_MIGRATION_CHECKSUM: &str = "c3-report-v2";
pub const C3_ARCHIVE_SCHEMA_VERSION: i32 = 4;
pub const C3_ARCHIVE_MIGRATION_CHECKSUM: &str = "c3-archive-v4";
pub const LEDGER_LIFECYCLE_SCHEMA_VERSION: i32 = 5;
pub const LEDGER_LIFECYCLE_MIGRATION_CHECKSUM: &str = "ledger-lifecycle-v5-layout3";

// 旧收据时间保持 NULL；retired_utc 是 owner 观察到退役的时间，不是提交时间。
pub const LEDGER_LIFECYCLE_DDL: &str = "
create table committed_bundle_v5 (
    data_version integer primary key,
    writer_epoch integer not null,
    bundle_seq integer not null,
    payload_hash text not null,
    committed_utc integer,
    unique(writer_epoch, bundle_seq)
) strict;
insert into committed_bundle_v5(data_version,writer_epoch,bundle_seq,payload_hash)
    select data_version,writer_epoch,bundle_seq,payload_hash from committed_bundle;
drop table committed_bundle;
alter table committed_bundle_v5 rename to committed_bundle;
alter table bundle_epoch add column expired_through_seq integer not null default 0;
alter table bundle_epoch add column expired_payload_digest text not null default '';
alter table bundle_epoch add column retired_utc integer;
alter table controller_epoch add column retired_utc integer;
drop index idx_connection_minute_utc;
create index idx_connection_minute_session on connection_minute(session_pk, utc_minute);
create index idx_coverage_sample_tail on coverage_interval(kind,reason,interval_id);
create index idx_retention_coverage_source on coverage_interval(started_utc,interval_id) where kind in ('covered','gap');
create index idx_session_attr_process on connection_session_attr(process_id);
create index idx_session_attr_rule on connection_session_attr(rule_id);
create index idx_session_attr_network on connection_session_attr(network_id);
create index idx_session_attr_category on connection_session_attr(primary_category_id);
create index idx_session_attr_chain_identity on connection_session_attr(chain_identity(chain_key));
create index idx_session_attr_rule_group on connection_session_attr(last_chain_hop(chain_key),rule_id);
create index idx_hourly_dimension_identity on traffic_hourly_dimension(dimension_kind,dimension_id);
create index idx_daily_dimension_identity on traffic_daily_dimension(dimension_kind,dimension_id);
create index idx_hourly_category on traffic_hourly_dimension(category_id);
create index idx_daily_dimension_category on traffic_daily_dimension(category_id);
create index idx_daily_core_category on traffic_daily_core(category_id);
update bundle_epoch set highest_contiguous_seq = 0;
create table retention_build (
    job_id integer primary key check(job_id = 1),
    day_utc integer not null,
    phase text not null,
    cursor_minute integer not null default 0,
    cursor_session integer not null default 0,
    build_hash text not null default '',
    check_hash text not null default '',
    output_cursor text not null default '',
    raw_rows integer not null default 0,
    output_started integer not null default 0,
    output_protected integer not null default 0,
    invalidated integer not null default 0
) strict;
create table retention_build_aggregate (
    granularity text not null,
    bucket_utc integer not null,
    category_id integer not null,
    dimension_kind text not null,
    dimension_id integer not null,
    upload integer not null default 0,
    download integer not null default 0,
    connection_count integer not null default 0,
    active_duration_sec integer not null default 0,
    check_upload integer not null default 0,
    check_download integer not null default 0,
    check_connection_count integer not null default 0,
    check_active_duration_sec integer not null default 0,
    primary key(granularity,bucket_utc,category_id,dimension_kind,dimension_id)
) strict;
create table retention_build_member (
    granularity text not null,
    bucket_utc integer not null,
    category_id integer not null,
    dimension_kind text not null,
    dimension_id integer not null,
    member_kind text not null,
    member_id integer not null,
    checked integer not null default 0,
    primary key(granularity,bucket_utc,category_id,dimension_kind,dimension_id,member_kind,member_id)
) strict;
create index idx_retention_build_member_session on retention_build_member(member_kind,member_id);
create trigger retention_build_attr_update after update on connection_session_attr
when (old.host_id is not new.host_id or old.process_id is not new.process_id
    or old.rule_id is not new.rule_id or old.network_id is not new.network_id
    or old.chain_key is not new.chain_key or old.primary_category_id is not new.primary_category_id)
    and exists(select 1 from retention_build_member where member_kind='session' and member_id=old.session_pk)
begin update retention_build set invalidated=1 where job_id=1 and phase not in ('delete','clear'); end;
create trigger retention_build_attr_delete after delete on connection_session_attr
when exists(select 1 from retention_build_member where member_kind='session' and member_id=old.session_pk)
begin update retention_build set invalidated=1 where job_id=1 and phase not in ('delete','clear'); end;
create trigger retention_hourly_insert after insert on traffic_hourly_dimension
begin update retention_build set invalidated=1 where output_protected=1 and phase not in ('_publish','_prune','discard','clear') and new.utc_hour>=day_utc and new.utc_hour<day_utc+86400; end;
create trigger retention_hourly_update after update on traffic_hourly_dimension
begin update retention_build set invalidated=1 where output_protected=1 and phase not in ('_publish','_prune','discard','clear') and
    ((old.utc_hour>=day_utc and old.utc_hour<day_utc+86400) or (new.utc_hour>=day_utc and new.utc_hour<day_utc+86400)); end;
create trigger retention_hourly_delete after delete on traffic_hourly_dimension
begin update retention_build set invalidated=1 where output_protected=1 and phase not in ('_publish','_prune','discard','clear') and old.utc_hour>=day_utc and old.utc_hour<day_utc+86400; end;
create trigger retention_daily_insert after insert on traffic_daily_dimension
begin update retention_build set invalidated=1 where output_protected=1 and phase not in ('_publish','_prune','discard','clear') and new.utc_day=day_utc; end;
create trigger retention_daily_update after update on traffic_daily_dimension
begin update retention_build set invalidated=1 where output_protected=1 and phase not in ('_publish','_prune','discard','clear') and (old.utc_day=day_utc or new.utc_day=day_utc); end;
create trigger retention_daily_delete after delete on traffic_daily_dimension
begin update retention_build set invalidated=1 where output_protected=1 and phase not in ('_publish','_prune','discard','clear') and old.utc_day=day_utc; end;
create trigger retention_core_insert after insert on traffic_daily_core
begin update retention_build set invalidated=1 where output_protected=1 and phase not in ('_publish','discard','clear') and new.utc_day=day_utc; end;
create trigger retention_core_update after update on traffic_daily_core
begin update retention_build set invalidated=1 where output_protected=1 and phase not in ('_publish','discard','clear') and (old.utc_day=day_utc or new.utc_day=day_utc); end;
create trigger retention_core_delete after delete on traffic_daily_core
begin update retention_build set invalidated=1 where output_protected=1 and phase not in ('_publish','discard','clear') and old.utc_day=day_utc; end;
create trigger retention_coverage_insert after insert on coverage_daily
begin update retention_build set invalidated=1 where output_protected=1 and phase not in ('_confirm','discard','clear') and new.utc_day=day_utc; end;
create trigger retention_coverage_update after update on coverage_daily
begin update retention_build set invalidated=1 where output_protected=1 and phase not in ('_confirm','discard','clear') and (old.utc_day=day_utc or new.utc_day=day_utc); end;
create trigger retention_coverage_delete after delete on coverage_daily
begin update retention_build set invalidated=1 where output_protected=1 and phase not in ('_confirm','discard','clear') and old.utc_day=day_utc; end;
create trigger retention_raw_insert after insert on connection_minute
begin update retention_build set invalidated=1 where new.utc_minute>=day_utc/60 and new.utc_minute<(day_utc+86400)/60; end;
create trigger retention_raw_update after update on connection_minute
begin update retention_build set invalidated=1 where
    (old.utc_minute>=day_utc/60 and old.utc_minute<(day_utc+86400)/60)
    or (new.utc_minute>=day_utc/60 and new.utc_minute<(day_utc+86400)/60); end;
create trigger retention_raw_delete after delete on connection_minute
begin update retention_build set invalidated=1 where phase not in ('delete','clear','discard','prune_check','prune_hour','prune_day','prune_finish')
    and old.utc_minute>=day_utc/60 and old.utc_minute<(day_utc+86400)/60; end;
create trigger retention_interval_insert after insert on coverage_interval
begin update retention_build set invalidated=1 where phase not in ('delete','clear')
    and new.started_utc<day_utc+86400 and coalesce(new.ended_utc,day_utc+86400)>day_utc; end;
create trigger retention_interval_update after update on coverage_interval
begin update retention_build set invalidated=1 where phase not in ('delete','clear') and
    ((old.started_utc<day_utc+86400 and coalesce(old.ended_utc,day_utc+86400)>day_utc)
    or (new.started_utc<day_utc+86400 and coalesce(new.ended_utc,day_utc+86400)>day_utc)); end;
create trigger retention_interval_delete after delete on coverage_interval
begin update retention_build set invalidated=1 where phase not in ('delete','clear')
    and old.started_utc<day_utc+86400 and coalesce(old.ended_utc,day_utc+86400)>day_utc; end;
";

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
    "retention_build",
    "retention_build_aggregate",
    "retention_build_member",
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
