//! 周期用量：只复用 C3 ReportService / 时区边界，不建第二套聚合。

use crate::c3::query::{
    local_day_bounds, local_month_bounds, DimensionKind, ReportFilters, ReportQuery, ReportResult,
    TargetPolicy,
};
use crate::c3::snapshot::ReportSnapshotStore;
use crate::c3::ReportService;
use crate::c4::engine::{UsageObservation, UsageStatus};
use crate::c4::types::{
    map_report_to_not_evaluable, AlertDirection, AlertKind, AlertPeriod, AlertRule, SelectorKind,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct QueryShape {
    start: i64,
    end: i64,
    timezone: String,
    host: Option<String>,
    process: Option<String>,
    category: Option<String>,
}

pub fn usage_query_for_rule(rule: &AlertRule, now_utc: i64) -> Result<ReportQuery, UsageStatus> {
    if rule.kind != AlertKind::PeriodUsage {
        return Err(UsageStatus::Unknown);
    }
    let period = rule.period.ok_or(UsageStatus::Unknown)?;
    let (start, end) = match period {
        AlertPeriod::Rolling1h => (now_utc.saturating_sub(3_600), now_utc),
        AlertPeriod::LocalDay => {
            local_day_bounds(&rule.timezone, now_utc).map_err(|_| UsageStatus::Unknown)?
        }
        AlertPeriod::LocalMonth => {
            local_month_bounds(&rule.timezone, now_utc).map_err(|_| UsageStatus::Unknown)?
        }
    };
    if end <= start {
        return Err(UsageStatus::Unknown);
    }
    let mut filters = ReportFilters::default();
    match rule.selector_kind {
        SelectorKind::PrimaryCategory => filters.category = rule.selector_value.clone(),
        SelectorKind::Domain => filters.host = rule.selector_value.clone(),
        SelectorKind::Process => filters.process = rule.selector_value.clone(),
        SelectorKind::HealthKind => return Err(UsageStatus::Unknown),
    }
    let grouping = match rule.selector_kind {
        SelectorKind::PrimaryCategory => DimensionKind::Category,
        SelectorKind::Domain => DimensionKind::Host,
        SelectorKind::Process => DimensionKind::Process,
        SelectorKind::HealthKind => DimensionKind::Category,
    };
    Ok(ReportQuery {
        range_start_utc: start,
        range_end_utc: end,
        display_timezone: rule.timezone.clone(),
        granularity: match period {
            AlertPeriod::Rolling1h => crate::c3::query::Granularity::Hour,
            AlertPeriod::LocalDay => crate::c3::query::Granularity::Day,
            AlertPeriod::LocalMonth => crate::c3::query::Granularity::Month,
        },
        filters,
        grouping,
        target_policy: TargetPolicy::Historical,
        comparison: None,
        sort: crate::c3::query::SortSpec::default(),
        page: crate::c3::query::PageSpec {
            limit: 20,
            after: None,
        },
        top_n: 20,
        include_sessions: false,
    })
}

pub fn evaluate_period_rules(
    db_path: &Path,
    store: &mut ReportSnapshotStore,
    rules: &[AlertRule],
    now_utc: i64,
    raw_retain_days: i64,
    cancel: &Arc<AtomicBool>,
) -> Vec<UsageObservation> {
    let mut groups: HashMap<QueryShape, (ReportQuery, Vec<AlertRule>)> = HashMap::new();
    let mut out = Vec::new();
    for rule in rules
        .iter()
        .filter(|item| item.enabled && item.kind == AlertKind::PeriodUsage)
    {
        match usage_query_for_rule(rule, now_utc) {
            Ok(query) => {
                let shape = QueryShape {
                    start: query.range_start_utc,
                    end: query.range_end_utc,
                    timezone: query.display_timezone.clone(),
                    host: query.filters.host.clone(),
                    process: query.filters.process.clone(),
                    category: query.filters.category.clone(),
                };
                groups
                    .entry(shape)
                    .or_insert_with(|| (query, Vec::new()))
                    .1
                    .push(rule.clone());
            }
            Err(status) => {
                return_placeholder(&mut out, rule, now_utc, status);
            }
        }
    }
    for (_, (query, grouped)) in groups {
        let result = ReportService::run(
            db_path,
            store,
            query.clone(),
            now_utc,
            raw_retain_days,
            cancel,
            Some(Duration::from_millis(crate::c3::query::REPORT_DEADLINE_MS)),
        );
        match result {
            Ok(report) => {
                let _ = store.release(&report.report_snapshot_token);
                for rule in grouped {
                    out.push(observation_from_report(rule, &query, &report));
                }
            }
            Err(error) => {
                let status = match error {
                    crate::c3::query::ReportError::CapabilityUnsupported(_) => {
                        UsageStatus::CapabilityUnsupported
                    }
                    crate::c3::query::ReportError::DeadlineExceeded(_) => UsageStatus::Deadline,
                    crate::c3::query::ReportError::Cancelled(_) => UsageStatus::Interrupted,
                    _ => UsageStatus::Unknown,
                };
                for rule in grouped {
                    out.push(UsageObservation {
                        rule_id: rule.rule_id,
                        observed: None,
                        status,
                        coverage_summary: map_report_to_not_evaluable(&error).into(),
                        data_version: None,
                        window_start_utc: query.range_start_utc,
                        window_end_utc: query.range_end_utc,
                        report_query: Some(query.clone()),
                        policy_metadata: None,
                    });
                }
            }
        }
    }
    out
}

fn return_placeholder(
    out: &mut Vec<UsageObservation>,
    rule: &AlertRule,
    now_utc: i64,
    status: UsageStatus,
) {
    out.push(UsageObservation {
        rule_id: rule.rule_id.clone(),
        observed: None,
        status,
        coverage_summary: "unknown".into(),
        data_version: None,
        window_start_utc: now_utc,
        window_end_utc: now_utc,
        report_query: None,
        policy_metadata: None,
    });
}

fn observation_from_report(
    rule: AlertRule,
    query: &ReportQuery,
    report: &ReportResult,
) -> UsageObservation {
    let incomplete = report.coverage.status == "partial"
        || report.coverage.status == "unknown"
        || report.coverage.gap_sec > 0;
    if incomplete {
        return UsageObservation {
            rule_id: rule.rule_id,
            observed: None,
            status: UsageStatus::IncompleteCoverage,
            coverage_summary: report.coverage.status.clone(),
            data_version: Some(report.data_version),
            window_start_utc: query.range_start_utc,
            window_end_utc: query.range_end_utc,
            report_query: Some(query.clone()),
            policy_metadata: Some(report.policy_metadata.note_zh.clone()),
        };
    }
    let observed = match rule.direction.unwrap_or(AlertDirection::Combined) {
        AlertDirection::Upload => report.totals.upload,
        AlertDirection::Download => report.totals.download,
        AlertDirection::Combined => report.totals.upload.saturating_add(report.totals.download),
    };
    UsageObservation {
        rule_id: rule.rule_id,
        observed: Some(observed),
        status: UsageStatus::Ok,
        coverage_summary: report.coverage.status.clone(),
        data_version: Some(report.data_version),
        window_start_utc: query.range_start_utc,
        window_end_utc: query.range_end_utc,
        report_query: Some(query.clone()),
        policy_metadata: Some(report.policy_metadata.note_zh.clone()),
    }
}

#[cfg(test)]
mod period_reuse_tests {
    use super::*;
    use crate::c4::types::AlertRule;
    use crate::storage::StorageCoordinator;
    use tempfile::tempdir;

    fn period_rule(period: AlertPeriod, tz: &str) -> AlertRule {
        AlertRule {
            rule_id: format!("p-{period:?}"),
            version: 1,
            enabled: true,
            kind: AlertKind::PeriodUsage,
            selector_kind: SelectorKind::Domain,
            selector_value: Some("a.example".into()),
            direction: Some(AlertDirection::Combined),
            threshold_value: 1,
            recovery_threshold: Some(0),
            period: Some(period),
            timezone: tz.into(),
            cooldown_sec: 60,
            quiet_start_min: None,
            quiet_end_min: None,
            created_utc: 0,
            updated_utc: 0,
        }
    }

    #[test]
    fn period_observation_matches_c3_report() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("p.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        let mut store = ReportSnapshotStore::open(dir.path());
        let rule = period_rule(AlertPeriod::Rolling1h, "UTC");
        let now = 3_600;
        let query = usage_query_for_rule(&rule, now).expect("query");
        let cancel = Arc::new(AtomicBool::new(false));
        let report = ReportService::run(
            coordinator.path(),
            &mut store,
            query.clone(),
            now,
            30,
            &cancel,
            None,
        )
        .expect("report");
        let usages =
            evaluate_period_rules(coordinator.path(), &mut store, &[rule], now, 30, &cancel);
        assert_eq!(usages.len(), 1);
        let expected = report.totals.upload + report.totals.download;
        match usages[0].status {
            UsageStatus::Ok => assert_eq!(usages[0].observed, Some(expected)),
            UsageStatus::IncompleteCoverage => {
                assert!(usages[0].observed.is_none());
            }
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(usages[0].report_query.as_ref(), Some(&query));
    }

    #[test]
    fn capability_unsupported_is_not_evaluable() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("p.sqlite3")).expect("open");
        coordinator.seed_report_fixture().expect("seed");
        let mut store = ReportSnapshotStore::open(dir.path());
        let mut rule = period_rule(AlertPeriod::Rolling1h, "Not/AZone");
        rule.selector_kind = SelectorKind::Domain;
        rule.selector_value = Some("a.example".into());
        let cancel = Arc::new(AtomicBool::new(false));
        let usages =
            evaluate_period_rules(coordinator.path(), &mut store, &[rule], 3_600, 30, &cancel);
        assert_eq!(usages.len(), 1);
        assert_ne!(usages[0].status, UsageStatus::Ok);
        assert!(usages[0].observed.is_none());
        let expired = crate::c3::query::ReportQuery {
            include_sessions: true,
            range_start_utc: 0,
            range_end_utc: 3_600,
            ..crate::c3::query::ReportQuery::default()
        };
        let error = crate::c3::query::plan_capability(&expired, 90 * 86_400, 30).expect_err("cap");
        assert_eq!(
            map_report_to_not_evaluable(&error),
            "capability_unsupported"
        );
    }
}
