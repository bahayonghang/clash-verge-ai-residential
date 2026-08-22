//! C4 前向 schema。不改写 C1 / C3 已发布 migration 文本。

pub const C4_SCHEMA_VERSION: i32 = 3;
pub const C4_MIGRATION_CHECKSUM: &str = "c4-alert-v3";
pub const ALERT_RETAIN_DAYS: i64 = 180;
pub const OUTBOX_SCAN_LIMIT: i64 = 32;
pub const RATE_WINDOW_MS: u64 = 60_000;
pub const RATE_TRIGGER_HITS: u32 = 3;
pub const MAX_RATE_SELECTORS: usize = 4_096;
pub const MAX_ENABLED_RULES: usize = 256;

pub const C4_TABLES: &[&str] = &[
    "alert_rule",
    "alert_instance",
    "alert_event",
    "notification_outbox",
];

pub const C4_DDL: &str = "
create table if not exists alert_rule (
    rule_id text primary key,
    version integer not null,
    enabled integer not null,
    kind text not null,
    selector_kind text not null,
    selector_value text,
    direction text,
    threshold_value integer not null,
    recovery_threshold integer,
    period text,
    timezone text,
    cooldown_sec integer not null,
    quiet_start_min integer,
    quiet_end_min integer,
    created_utc integer not null,
    updated_utc integer not null
) strict;
create table if not exists alert_instance (
    instance_id text primary key,
    rule_id text not null,
    rule_version integer not null,
    selector_identity text not null,
    status text not null,
    started_utc integer,
    resolved_utc integer,
    last_eval_utc integer not null,
    last_observed integer,
    last_evidence_json text not null
) strict;
create unique index if not exists alert_instance_active_uniq
    on alert_instance(rule_id, rule_version, selector_identity)
    where status = 'active';
create unique index if not exists alert_instance_identity
    on alert_instance(rule_id, rule_version, selector_identity, instance_id);
create index if not exists idx_alert_instance_center
    on alert_instance(status, last_eval_utc, instance_id);
create table if not exists alert_event (
    event_id text primary key,
    instance_id text not null,
    bundle_id text not null,
    kind text not null,
    at_utc integer not null,
    evidence_json text not null,
    idempotency_key text not null unique
) strict;
create index if not exists idx_alert_event_retention
    on alert_event(at_utc, event_id);
create table if not exists notification_outbox (
    outbox_id text primary key,
    event_id text not null,
    bundle_id text not null,
    status text not null,
    attempt integer not null,
    next_attempt_at integer not null,
    lease_until integer,
    lease_token text,
    error_class text,
    error_summary text,
    idempotency_key text not null unique,
    created_utc integer not null
) strict;
create index if not exists idx_outbox_due
    on notification_outbox(status, next_attempt_at, outbox_id);
create index if not exists idx_outbox_lease
    on notification_outbox(status, lease_until, outbox_id);
insert or ignore into alert_rule(
    rule_id, version, enabled, kind, selector_kind, selector_value, direction,
    threshold_value, recovery_threshold, period, timezone, cooldown_sec,
    quiet_start_min, quiet_end_min, created_utc, updated_utc
) values
    ('health-disconnect', 1, 1, 'health', 'health_kind', 'disconnect', null, 1, null, null, 'UTC', 300, null, null, 0, 0),
    ('health-tcp-auth', 1, 1, 'health', 'health_kind', 'tcp_auth', null, 1, null, null, 'UTC', 300, null, null, 0, 0),
    ('health-protocol', 1, 1, 'health', 'health_kind', 'protocol', null, 1, null, null, 'UTC', 300, null, null, 0, 0),
    ('health-collection-gap', 1, 1, 'health', 'health_kind', 'collection_gap', null, 1, null, null, 'UTC', 300, null, null, 0, 0),
    ('health-storage', 1, 1, 'health', 'health_kind', 'storage', null, 1, null, null, 'UTC', 300, null, null, 0, 0),
    ('health-migration', 1, 1, 'health', 'health_kind', 'migration', null, 1, null, null, 'UTC', 300, null, null, 0, 0),
    ('health-backup', 1, 1, 'health', 'health_kind', 'backup', null, 1, null, null, 'UTC', 300, null, null, 0, 0);
";

pub fn c4_table_allowlist() -> &'static [&'static str] {
    C4_TABLES
}

#[cfg(test)]
mod c4_schema_contract_tests {
    use super::*;

    #[test]
    fn c4_tables_are_alert_outbox_only() {
        assert!(C4_TABLES.contains(&"alert_rule"));
        assert!(C4_TABLES.contains(&"notification_outbox"));
        assert!(!C4_DDL.contains("create table") || C4_DDL.contains("alert_rule"));
        assert!(C4_DDL.contains("notification_outbox"));
        assert!(!C4_DDL.contains("traffic_hourly"));
    }
}
