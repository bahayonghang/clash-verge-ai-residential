//! 在同一 writer 连接上写入规则、实例与事件。调用方必须已处于事务中。

use crate::c4::schema::ALERT_RETAIN_DAYS;
use crate::c4::types::{
    AlertEvent, AlertInstance, AlertRule, EventKind, InstanceStatus, SelectorKind,
};
use crate::storage::StorageError;
use rusqlite::{params, Connection, OptionalExtension};

pub fn persist_instances(
    connection: &Connection,
    instances: &[AlertInstance],
) -> Result<(), StorageError> {
    for item in instances {
        connection.execute(
            "insert into alert_instance(
                instance_id, rule_id, rule_version, selector_identity, status,
                started_utc, resolved_utc, last_eval_utc, last_observed, last_evidence_json
             ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             on conflict(instance_id) do update set
                status = excluded.status,
                started_utc = excluded.started_utc,
                resolved_utc = excluded.resolved_utc,
                last_eval_utc = excluded.last_eval_utc,
                last_observed = excluded.last_observed,
                last_evidence_json = excluded.last_evidence_json",
            params![
                item.instance_id,
                item.rule_id,
                item.rule_version,
                item.selector_identity,
                status_sql(item.status),
                item.started_utc,
                item.resolved_utc,
                item.last_eval_utc,
                item.last_observed,
                serde_json::to_string(&item.evidence).unwrap_or_else(|_| "{}".into())
            ],
        )?;
    }
    Ok(())
}

pub fn persist_events(connection: &Connection, events: &[AlertEvent]) -> Result<(), StorageError> {
    for item in events {
        connection.execute(
            "insert or ignore into alert_event(
                event_id, instance_id, bundle_id, kind, at_utc, evidence_json, idempotency_key
             ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                item.event_id,
                item.instance_id,
                item.bundle_id,
                event_sql(item.kind),
                item.at_utc,
                serde_json::to_string(&item.evidence).unwrap_or_else(|_| "{}".into()),
                item.idempotency_key
            ],
        )?;
    }
    Ok(())
}

pub fn upsert_rule(connection: &Connection, rule: &AlertRule) -> Result<(), StorageError> {
    connection.execute(
        "insert into alert_rule(
            rule_id, version, enabled, kind, selector_kind, selector_value, direction,
            threshold_value, recovery_threshold, period, timezone, cooldown_sec,
            quiet_start_min, quiet_end_min, created_utc, updated_utc
         ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
         on conflict(rule_id) do update set
            version = excluded.version,
            enabled = excluded.enabled,
            kind = excluded.kind,
            selector_kind = excluded.selector_kind,
            selector_value = excluded.selector_value,
            direction = excluded.direction,
            threshold_value = excluded.threshold_value,
            recovery_threshold = excluded.recovery_threshold,
            period = excluded.period,
            timezone = excluded.timezone,
            cooldown_sec = excluded.cooldown_sec,
            quiet_start_min = excluded.quiet_start_min,
            quiet_end_min = excluded.quiet_end_min,
            updated_utc = excluded.updated_utc",
        params![
            rule.rule_id,
            rule.version,
            i64::from(rule.enabled),
            kind_sql(rule.kind),
            selector_sql(rule.selector_kind),
            rule.selector_value,
            rule.direction.map(|item| match item {
                crate::c4::types::AlertDirection::Upload => "upload",
                crate::c4::types::AlertDirection::Download => "download",
                crate::c4::types::AlertDirection::Combined => "combined",
            }),
            rule.threshold_value,
            rule.recovery_threshold,
            rule.period.map(|item| match item {
                crate::c4::types::AlertPeriod::Rolling1h => "rolling_1h",
                crate::c4::types::AlertPeriod::LocalDay => "local_day",
                crate::c4::types::AlertPeriod::LocalMonth => "local_month",
            }),
            rule.timezone,
            rule.cooldown_sec,
            rule.quiet_start_min,
            rule.quiet_end_min,
            rule.created_utc,
            rule.updated_utc
        ],
    )?;
    Ok(())
}

pub fn load_rules(connection: &Connection) -> Result<Vec<AlertRule>, StorageError> {
    let mut statement = connection.prepare(
        "select rule_id, version, enabled, kind, selector_kind, selector_value, direction,
                threshold_value, recovery_threshold, period, timezone, cooldown_sec,
                quiet_start_min, quiet_end_min, created_utc, updated_utc
           from alert_rule order by rule_id",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(AlertRule {
            rule_id: row.get(0)?,
            version: row.get(1)?,
            enabled: row.get::<_, i64>(2)? == 1,
            kind: parse_kind(&row.get::<_, String>(3)?),
            selector_kind: parse_selector(&row.get::<_, String>(4)?),
            selector_value: row.get(5)?,
            direction: row
                .get::<_, Option<String>>(6)?
                .as_deref()
                .and_then(parse_direction),
            threshold_value: row.get(7)?,
            recovery_threshold: row.get(8)?,
            period: row
                .get::<_, Option<String>>(9)?
                .as_deref()
                .and_then(parse_period),
            timezone: row.get(10)?,
            cooldown_sec: row.get(11)?,
            quiet_start_min: row.get(12)?,
            quiet_end_min: row.get(13)?,
            created_utc: row.get(14)?,
            updated_utc: row.get(15)?,
        })
    })?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn load_instances(connection: &Connection) -> Result<Vec<AlertInstance>, StorageError> {
    let mut statement = connection.prepare(
        "select instance_id, rule_id, rule_version, selector_identity, status,
                started_utc, resolved_utc, last_eval_utc, last_observed, last_evidence_json
           from alert_instance",
    )?;
    let rows = statement.query_map([], |row| {
        let evidence_json: String = row.get(9)?;
        Ok(AlertInstance {
            instance_id: row.get(0)?,
            rule_id: row.get(1)?,
            rule_version: row.get(2)?,
            selector_identity: row.get(3)?,
            status: parse_status(&row.get::<_, String>(4)?),
            started_utc: row.get(5)?,
            resolved_utc: row.get(6)?,
            last_eval_utc: row.get(7)?,
            last_observed: row.get(8)?,
            evidence: serde_json::from_str(&evidence_json).unwrap_or(
                crate::c4::types::AlertEvidence {
                    rule_id: String::new(),
                    rule_version: 0,
                    data_version: None,
                    evaluated_at_utc: 0,
                    window_start_utc: None,
                    window_end_utc: None,
                    display_timezone: "UTC".into(),
                    selector: String::new(),
                    direction: None,
                    observed_value: None,
                    trigger_threshold: 0,
                    recovery_threshold: None,
                    coverage_summary: "unknown".into(),
                    policy_metadata: None,
                    report_query: None,
                    not_evaluable_reason: None,
                },
            ),
        })
    })?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn list_instances(
    connection: &Connection,
    status: Option<&str>,
    after_utc: Option<i64>,
    after_id: Option<&str>,
    limit: i64,
) -> Result<Vec<AlertInstance>, StorageError> {
    let mut statement = connection.prepare(
        "select instance_id, rule_id, rule_version, selector_identity, status,
                started_utc, resolved_utc, last_eval_utc, last_observed, last_evidence_json
           from alert_instance
          where (?1 = 0 or status = ?2)
            and (?3 = 0 or last_eval_utc < ?4 or (last_eval_utc = ?4 and instance_id < ?5))
          order by last_eval_utc desc, instance_id desc
          limit ?6",
    )?;
    let status_on = i64::from(status.is_some());
    let cursor_on = i64::from(after_utc.is_some());
    let rows = statement.query_map(
        params![
            status_on,
            status.unwrap_or(""),
            cursor_on,
            after_utc.unwrap_or(0),
            after_id.unwrap_or(""),
            limit
        ],
        |row| {
            let evidence_json: String = row.get(9)?;
            Ok(AlertInstance {
                instance_id: row.get(0)?,
                rule_id: row.get(1)?,
                rule_version: row.get(2)?,
                selector_identity: row.get(3)?,
                status: parse_status(&row.get::<_, String>(4)?),
                started_utc: row.get(5)?,
                resolved_utc: row.get(6)?,
                last_eval_utc: row.get(7)?,
                last_observed: row.get(8)?,
                evidence: serde_json::from_str(&evidence_json).unwrap_or(
                    crate::c4::types::AlertEvidence {
                        rule_id: String::new(),
                        rule_version: 0,
                        data_version: None,
                        evaluated_at_utc: 0,
                        window_start_utc: None,
                        window_end_utc: None,
                        display_timezone: "UTC".into(),
                        selector: String::new(),
                        direction: None,
                        observed_value: None,
                        trigger_threshold: 0,
                        recovery_threshold: None,
                        coverage_summary: "unknown".into(),
                        policy_metadata: None,
                        report_query: None,
                        not_evaluable_reason: None,
                    },
                ),
            })
        },
    )?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn retain_alerts(connection: &Connection, now_utc: i64) -> Result<u32, StorageError> {
    let cutoff = now_utc - ALERT_RETAIN_DAYS * 86_400;
    let deleted = connection.execute(
        "delete from alert_event
          where at_utc < ?1
            and instance_id not in (
                select instance_id from alert_instance where status = 'active'
            )
            and event_id not in (
                select event_id from notification_outbox
                 where status in ('pending', 'retry', 'leased')
            )",
        [cutoff],
    )?;
    connection.execute(
        "delete from alert_instance
          where status in ('resolved', 'superseded')
            and coalesce(resolved_utc, last_eval_utc) < ?1
            and instance_id not in (
                select instance_id from notification_outbox
                 where status in ('pending', 'retry', 'leased')
            )",
        [cutoff],
    )?;
    Ok(deleted as u32)
}

pub fn get_rule(connection: &Connection, rule_id: &str) -> Result<Option<AlertRule>, StorageError> {
    load_rules(connection).map(|rules| rules.into_iter().find(|item| item.rule_id == rule_id))
}

pub fn count_status(connection: &Connection, status: &str) -> Result<u32, StorageError> {
    let value: i64 = connection.query_row(
        "select count(*) from alert_instance where status = ?1",
        [status],
        |row| row.get(0),
    )?;
    Ok(value as u32)
}

pub fn last_event_utc(connection: &Connection) -> Result<Option<i64>, StorageError> {
    connection
        .query_row("select max(at_utc) from alert_event", [], |row| row.get(0))
        .optional()
        .map_err(StorageError::from)
}

fn kind_sql(kind: crate::c4::types::AlertKind) -> &'static str {
    match kind {
        crate::c4::types::AlertKind::Health => "health",
        crate::c4::types::AlertKind::Rate => "rate",
        crate::c4::types::AlertKind::PeriodUsage => "period_usage",
    }
}

fn selector_sql(kind: SelectorKind) -> &'static str {
    match kind {
        SelectorKind::HealthKind => "health_kind",
        SelectorKind::PrimaryCategory => "primary_category",
        SelectorKind::Domain => "domain",
        SelectorKind::Process => "process",
    }
}

fn status_sql(status: InstanceStatus) -> &'static str {
    match status {
        InstanceStatus::Inactive => "inactive",
        InstanceStatus::Active => "active",
        InstanceStatus::NotEvaluable => "not_evaluable",
        InstanceStatus::Resolved => "resolved",
        InstanceStatus::Superseded => "superseded",
    }
}

fn event_sql(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Activated => "activated",
        EventKind::Recovered => "recovered",
        EventKind::EvaluationGap => "evaluation_gap",
        EventKind::Superseded => "superseded",
        EventKind::NotEvaluable => "not_evaluable",
    }
}

fn parse_kind(value: &str) -> crate::c4::types::AlertKind {
    match value {
        "rate" => crate::c4::types::AlertKind::Rate,
        "period_usage" => crate::c4::types::AlertKind::PeriodUsage,
        _ => crate::c4::types::AlertKind::Health,
    }
}

fn parse_selector(value: &str) -> SelectorKind {
    match value {
        "primary_category" => SelectorKind::PrimaryCategory,
        "domain" => SelectorKind::Domain,
        "process" => SelectorKind::Process,
        _ => SelectorKind::HealthKind,
    }
}

fn parse_direction(value: &str) -> Option<crate::c4::types::AlertDirection> {
    match value {
        "upload" => Some(crate::c4::types::AlertDirection::Upload),
        "download" => Some(crate::c4::types::AlertDirection::Download),
        "combined" => Some(crate::c4::types::AlertDirection::Combined),
        _ => None,
    }
}

fn parse_period(value: &str) -> Option<crate::c4::types::AlertPeriod> {
    match value {
        "rolling_1h" => Some(crate::c4::types::AlertPeriod::Rolling1h),
        "local_day" => Some(crate::c4::types::AlertPeriod::LocalDay),
        "local_month" => Some(crate::c4::types::AlertPeriod::LocalMonth),
        _ => None,
    }
}

fn parse_status(value: &str) -> InstanceStatus {
    match value {
        "active" => InstanceStatus::Active,
        "not_evaluable" => InstanceStatus::NotEvaluable,
        "resolved" => InstanceStatus::Resolved,
        "superseded" => InstanceStatus::Superseded,
        _ => InstanceStatus::Inactive,
    }
}
