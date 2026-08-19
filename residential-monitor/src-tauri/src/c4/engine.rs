//! AlertEngine：活动、恢复、冷却、静默、滞回与规则版本的唯一所有者。

use crate::accounting::{AccountingBatch, MinuteFact};
use crate::c2::hub::LiveConnectionView;
use crate::c4::schema::{MAX_ENABLED_RULES, MAX_RATE_SELECTORS, RATE_TRIGGER_HITS, RATE_WINDOW_MS};
use crate::c4::types::{
    selector_identity, validate_rule, AlertDirection, AlertError, AlertEvent, AlertEvidence,
    AlertInstance, AlertKind, AlertRule, AlertWriteSet, EventKind, InstanceStatus, OutboxIntent,
    OutboxStatus, SelectorKind,
};
use crate::controller::SessionStatus;
use crate::storage::StorageHealth;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Default)]
pub struct HealthSnapshot {
    pub session: Option<SessionStatus>,
    pub storage: Option<StorageHealth>,
    pub coverage_kinds: Vec<String>,
    pub migration_failed: bool,
    pub backup_failed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsageStatus {
    Ok,
    IncompleteCoverage,
    CapabilityUnsupported,
    Deadline,
    Interrupted,
    Unknown,
}

pub struct FrameInput<'a> {
    pub batch: &'a AccountingBatch,
    pub live: &'a [LiveConnectionView],
    pub health: &'a HealthSnapshot,
    pub usages: &'a [UsageObservation],
    pub now_utc: i64,
    pub now_mono: u64,
    pub data_version: u64,
    pub bundle_id: &'a str,
}

#[derive(Debug, Clone)]
pub struct UsageObservation {
    pub rule_id: String,
    pub observed: Option<i64>,
    pub status: UsageStatus,
    pub coverage_summary: String,
    pub data_version: Option<u64>,
    pub window_start_utc: i64,
    pub window_end_utc: i64,
    pub report_query: Option<crate::c3::query::ReportQuery>,
    pub policy_metadata: Option<String>,
}

#[derive(Debug, Clone)]
struct RateSample {
    mono_ms: u64,
    upload: u64,
    download: u64,
}

#[derive(Debug, Default)]
struct RateWindow {
    samples: VecDeque<RateSample>,
}

impl RateWindow {
    fn push(&mut self, sample: RateSample) {
        self.samples.push_back(sample);
    }

    fn prune(&mut self, now_mono: u64) {
        while let Some(front) = self.samples.front() {
            if now_mono.saturating_sub(front.mono_ms) > RATE_WINDOW_MS {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }

    fn rate_bps(&self, direction: AlertDirection, now_mono: u64) -> Option<i64> {
        if self.samples.len() < 2 {
            return None;
        }
        let first = self.samples.front()?;
        let span = now_mono.saturating_sub(first.mono_ms);
        if span < RATE_WINDOW_MS {
            return None;
        }
        let mut upload = 0_u64;
        let mut download = 0_u64;
        for sample in &self.samples {
            upload = upload.saturating_add(sample.upload);
            download = download.saturating_add(sample.download);
        }
        let bytes = match direction {
            AlertDirection::Upload => upload,
            AlertDirection::Download => download,
            AlertDirection::Combined => upload.saturating_add(download),
        };
        Some((bytes.saturating_mul(1000) / span.max(1)) as i64)
    }
}

#[derive(Debug, Clone)]
struct RuntimeState {
    instance: AlertInstance,
    consecutive_hits: u32,
    last_notify_utc: Option<i64>,
}

pub struct AlertEngine {
    rules: HashMap<String, AlertRule>,
    states: HashMap<String, RuntimeState>,
    windows: HashMap<String, RateWindow>,
    last_mono: Option<u64>,
    gap_open: bool,
    sql_in_hot_path: u32,
}

impl Default for AlertEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertEngine {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            states: HashMap::new(),
            windows: HashMap::new(),
            last_mono: None,
            gap_open: false,
            sql_in_hot_path: 0,
        }
    }

    pub fn sql_in_hot_path(&self) -> u32 {
        self.sql_in_hot_path
    }

    pub fn load_rules(&mut self, rules: Vec<AlertRule>) -> Result<(), AlertError> {
        self.rules.clear();
        for rule in rules {
            validate_rule(&rule)?;
            self.rules.insert(rule.rule_id.clone(), rule);
        }
        if self.rules.values().filter(|item| item.enabled).count() > MAX_ENABLED_RULES {
            return Err(AlertError::InvalidRule("too many rules"));
        }
        Ok(())
    }

    pub fn upsert_rule(
        &mut self,
        mut rule: AlertRule,
        now_utc: i64,
    ) -> Result<AlertWriteSet, AlertError> {
        validate_rule(&rule)?;
        let mut writes = AlertWriteSet::default();
        if let Some(old) = self.rules.get(&rule.rule_id).cloned() {
            if old.version >= rule.version {
                rule.version = old.version + 1;
            }
            writes.merge(self.supersede_rule(&old, now_utc));
        }
        rule.updated_utc = now_utc;
        if rule.created_utc == 0 {
            rule.created_utc = now_utc;
        }
        self.rules.insert(rule.rule_id.clone(), rule);
        Ok(writes)
    }

    pub fn restore_instance(&mut self, instance: AlertInstance) {
        let key = runtime_key(
            &instance.rule_id,
            instance.rule_version,
            &instance.selector_identity,
        );
        self.states.insert(
            key,
            RuntimeState {
                instance,
                consecutive_hits: 0,
                last_notify_utc: None,
            },
        );
    }

    pub fn evaluate_frame(&mut self, input: FrameInput<'_>) -> AlertWriteSet {
        let batch = input.batch;
        let live = input.live;
        let health = input.health;
        let usages = input.usages;
        let now_utc = input.now_utc;
        let now_mono = input.now_mono;
        let data_version = input.data_version;
        let bundle_id = input.bundle_id;
        self.sql_in_hot_path = 0;
        let gap = is_gap_batch(batch, health);
        if gap {
            self.windows.clear();
            self.last_mono = None;
            self.gap_open = true;
        } else {
            self.ingest_rate_samples(batch, live, now_mono);
            self.gap_open = false;
        }
        self.last_mono = Some(now_mono);

        let mut writes = AlertWriteSet::default();
        let rules: Vec<AlertRule> = self
            .rules
            .values()
            .filter(|item| item.enabled)
            .cloned()
            .collect();
        for rule in rules {
            match rule.kind {
                AlertKind::Health => {
                    writes.merge(self.eval_health(&rule, health, now_utc, data_version, bundle_id));
                }
                AlertKind::Rate => {
                    writes.merge(self.eval_rate(
                        &rule,
                        gap,
                        now_utc,
                        now_mono,
                        data_version,
                        bundle_id,
                    ));
                }
                AlertKind::PeriodUsage => {
                    if let Some(usage) = usages.iter().find(|item| item.rule_id == rule.rule_id) {
                        writes.merge(self.eval_period(
                            &rule,
                            usage,
                            now_utc,
                            data_version,
                            bundle_id,
                        ));
                    }
                }
            }
        }
        writes
    }

    fn ingest_rate_samples(
        &mut self,
        batch: &AccountingBatch,
        live: &[LiveConnectionView],
        now_mono: u64,
    ) {
        if self.windows.len() > MAX_RATE_SELECTORS {
            self.windows.clear();
        }
        let mut by_key: HashMap<String, (u64, u64)> = HashMap::new();
        for fact in &batch.facts {
            accumulate_fact(&mut by_key, fact, live);
        }
        for (key, (upload, download)) in by_key {
            let window = self.windows.entry(key).or_default();
            window.push(RateSample {
                mono_ms: now_mono,
                upload,
                download,
            });
            window.prune(now_mono);
        }
        for window in self.windows.values_mut() {
            window.prune(now_mono);
        }
    }

    fn eval_health(
        &mut self,
        rule: &AlertRule,
        health: &HealthSnapshot,
        now_utc: i64,
        data_version: u64,
        bundle_id: &str,
    ) -> AlertWriteSet {
        let Some(kind) = rule.selector_value.as_deref() else {
            return AlertWriteSet::default();
        };
        let firing = health_kind_active(kind, health);
        let observed = if firing { Some(1) } else { Some(0) };
        self.transition(
            rule,
            &selector_identity(SelectorKind::HealthKind, kind),
            EvalInput {
                known: true,
                firing,
                recovering: !firing,
                observed,
                coverage: if firing { "unhealthy" } else { "healthy" },
                window: None,
                usage: None,
            },
            now_utc,
            data_version,
            bundle_id,
        )
    }

    fn eval_rate(
        &mut self,
        rule: &AlertRule,
        gap: bool,
        now_utc: i64,
        now_mono: u64,
        data_version: u64,
        bundle_id: &str,
    ) -> AlertWriteSet {
        let selector = rule
            .selector_value
            .as_deref()
            .map(|value| selector_identity(rule.selector_kind, value))
            .unwrap_or_else(|| selector_identity(rule.selector_kind, "*"));
        let direction = rule.direction.unwrap_or(AlertDirection::Combined);
        let observed = if gap {
            None
        } else {
            self.windows
                .get(&selector)
                .and_then(|window| window.rate_bps(direction, now_mono))
        };
        let known = observed.is_some();
        let firing = observed.is_some_and(|value| value >= rule.threshold_value);
        let recovering = observed
            .is_some_and(|value| value <= rule.recovery_threshold.unwrap_or(rule.threshold_value));
        self.transition(
            rule,
            &selector,
            EvalInput {
                known,
                firing,
                recovering,
                observed,
                coverage: if gap {
                    "gap"
                } else if known {
                    "covered"
                } else {
                    "insufficient_window"
                },
                window: Some((
                    now_utc.saturating_sub(i64::try_from(RATE_WINDOW_MS / 1000).unwrap_or(60)),
                    now_utc,
                )),
                usage: None,
            },
            now_utc,
            data_version,
            bundle_id,
        )
    }

    fn eval_period(
        &mut self,
        rule: &AlertRule,
        usage: &UsageObservation,
        now_utc: i64,
        data_version: u64,
        bundle_id: &str,
    ) -> AlertWriteSet {
        let selector = rule
            .selector_value
            .as_deref()
            .map(|value| selector_identity(rule.selector_kind, value))
            .unwrap_or_else(|| selector_identity(rule.selector_kind, "*"));
        let known = usage.status == UsageStatus::Ok && usage.observed.is_some();
        let observed = if known { usage.observed } else { None };
        let firing = observed.is_some_and(|value| value >= rule.threshold_value);
        let recovering = observed
            .is_some_and(|value| value <= rule.recovery_threshold.unwrap_or(rule.threshold_value));
        let coverage = match usage.status {
            UsageStatus::Ok => usage.coverage_summary.as_str(),
            UsageStatus::IncompleteCoverage => "partial",
            UsageStatus::CapabilityUnsupported => "capability_unsupported",
            UsageStatus::Deadline => "deadline_exceeded",
            UsageStatus::Interrupted => "cancelled",
            UsageStatus::Unknown => "unknown",
        };
        self.transition(
            rule,
            &selector,
            EvalInput {
                known,
                firing,
                recovering,
                observed,
                coverage,
                window: Some((usage.window_start_utc, usage.window_end_utc)),
                usage: Some(usage),
            },
            now_utc,
            usage.data_version.unwrap_or(data_version),
            bundle_id,
        )
    }

    fn transition(
        &mut self,
        rule: &AlertRule,
        selector: &str,
        input: EvalInput<'_>,
        now_utc: i64,
        data_version: u64,
        bundle_id: &str,
    ) -> AlertWriteSet {
        let key = runtime_key(&rule.rule_id, rule.version, selector);
        let mut writes = AlertWriteSet::default();
        let evidence = build_evidence(rule, selector, &input, now_utc, data_version);
        let mut state = self.states.remove(&key).unwrap_or_else(|| RuntimeState {
            instance: skeleton_instance(rule, selector, now_utc, evidence.clone()),
            consecutive_hits: 0,
            last_notify_utc: None,
        });

        if input.known && input.firing {
            state.consecutive_hits = state.consecutive_hits.saturating_add(1);
        } else if input.known {
            state.consecutive_hits = 0;
        }

        let previous = state.instance.status;
        if !input.known {
            if previous == InstanceStatus::Active {
                writes.merge(emit(
                    &mut state,
                    EventKind::EvaluationGap,
                    evidence.clone(),
                    now_utc,
                    bundle_id,
                    false,
                    rule,
                ));
            } else if previous != InstanceStatus::NotEvaluable {
                state.instance.status = InstanceStatus::NotEvaluable;
                state.instance.last_observed = None;
                writes.merge(emit(
                    &mut state,
                    EventKind::NotEvaluable,
                    evidence.clone(),
                    now_utc,
                    bundle_id,
                    false,
                    rule,
                ));
            }
        } else if previous == InstanceStatus::Active {
            if input.recovering {
                state.instance.status = InstanceStatus::Resolved;
                state.instance.resolved_utc = Some(now_utc);
                state.instance.last_observed = input.observed;
                writes.merge(emit(
                    &mut state,
                    EventKind::Recovered,
                    evidence.clone(),
                    now_utc,
                    bundle_id,
                    true,
                    rule,
                ));
            } else {
                state.instance.last_observed = input.observed;
                state.instance.last_eval_utc = now_utc;
                state.instance.evidence = evidence;
            }
        } else if input.firing && state.consecutive_hits >= RATE_TRIGGER_HITS
            || (rule.kind == AlertKind::Health && input.firing && state.consecutive_hits >= 1)
            || (rule.kind == AlertKind::PeriodUsage && input.firing && state.consecutive_hits >= 1)
        {
            state.instance.status = InstanceStatus::Active;
            state.instance.started_utc = Some(now_utc);
            state.instance.resolved_utc = None;
            state.instance.last_observed = input.observed;
            writes.merge(emit(
                &mut state,
                EventKind::Activated,
                evidence,
                now_utc,
                bundle_id,
                true,
                rule,
            ));
        } else if previous == InstanceStatus::NotEvaluable && input.known {
            state.instance.status = InstanceStatus::Inactive;
            state.instance.last_observed = input.observed;
            state.instance.last_eval_utc = now_utc;
            state.instance.evidence = evidence;
        } else {
            state.instance.last_eval_utc = now_utc;
            state.instance.last_observed = input.observed;
            state.instance.evidence = evidence;
            if previous == InstanceStatus::Resolved {
                state.instance.status = InstanceStatus::Inactive;
            }
        }

        writes.instances.push(state.instance.clone());
        self.states.insert(key, state);
        writes
    }

    #[cfg(test)]
    fn set_window_bytes(&mut self, key: &str, now_mono: u64, download: u64) {
        let mut window = RateWindow::default();
        window.push(RateSample {
            mono_ms: now_mono.saturating_sub(RATE_WINDOW_MS),
            upload: 0,
            download: 0,
        });
        window.push(RateSample {
            mono_ms: now_mono,
            upload: 0,
            download,
        });
        self.windows.insert(key.to_string(), window);
    }

    fn supersede_rule(&mut self, old: &AlertRule, now_utc: i64) -> AlertWriteSet {
        let mut writes = AlertWriteSet::default();
        let keys: Vec<String> = self
            .states
            .keys()
            .filter(|key| key.starts_with(&format!("{}:{}:", old.rule_id, old.version)))
            .cloned()
            .collect();
        for key in keys {
            if let Some(mut state) = self.states.remove(&key) {
                state.instance.status = InstanceStatus::Superseded;
                state.instance.resolved_utc = Some(now_utc);
                let evidence = state.instance.evidence.clone();
                writes.merge(emit(
                    &mut state,
                    EventKind::Superseded,
                    evidence,
                    now_utc,
                    "rule-edit",
                    false,
                    old,
                ));
                writes.instances.push(state.instance.clone());
            }
        }
        writes
    }
}

struct EvalInput<'a> {
    known: bool,
    firing: bool,
    recovering: bool,
    observed: Option<i64>,
    coverage: &'a str,
    window: Option<(i64, i64)>,
    usage: Option<&'a UsageObservation>,
}

impl AlertWriteSet {
    fn merge(&mut self, other: AlertWriteSet) {
        self.instances.extend(other.instances);
        self.events.extend(other.events);
        self.outbox.extend(other.outbox);
    }
}

fn emit(
    state: &mut RuntimeState,
    kind: EventKind,
    evidence: AlertEvidence,
    now_utc: i64,
    bundle_id: &str,
    may_notify: bool,
    rule: &AlertRule,
) -> AlertWriteSet {
    state.instance.last_eval_utc = now_utc;
    state.instance.evidence = evidence.clone();
    let event_id = format!("{bundle_id}:{}:{kind:?}", state.instance.instance_id);
    let idempotency = format!(
        "{bundle_id}:{}:{}:{kind:?}",
        state.instance.instance_id, state.instance.rule_version
    );
    let event = AlertEvent {
        event_id: event_id.clone(),
        instance_id: state.instance.instance_id.clone(),
        bundle_id: bundle_id.to_string(),
        kind,
        at_utc: now_utc,
        evidence,
        idempotency_key: idempotency.clone(),
    };
    let mut writes = AlertWriteSet {
        events: vec![event],
        ..AlertWriteSet::default()
    };
    if may_notify {
        let quiet = in_quiet(rule, now_utc);
        let cooling = state
            .last_notify_utc
            .is_some_and(|last| now_utc.saturating_sub(last) < rule.cooldown_sec);
        let status = if quiet || cooling {
            OutboxStatus::Suppressed
        } else {
            OutboxStatus::Pending
        };
        if status == OutboxStatus::Pending {
            state.last_notify_utc = Some(now_utc);
        }
        writes.outbox.push(OutboxIntent {
            outbox_id: format!("outbox:{idempotency}"),
            event_id,
            bundle_id: bundle_id.to_string(),
            status,
            attempt: 0,
            next_attempt_at: now_utc,
            lease_until: None,
            lease_token: None,
            error_class: if quiet {
                Some("quiet".into())
            } else if cooling {
                Some("cooldown".into())
            } else {
                None
            },
            error_summary: None,
            idempotency_key: format!("notify:{idempotency}"),
            created_utc: now_utc,
        });
    }
    writes
}

fn build_evidence(
    rule: &AlertRule,
    selector: &str,
    input: &EvalInput<'_>,
    now_utc: i64,
    data_version: u64,
) -> AlertEvidence {
    AlertEvidence {
        rule_id: rule.rule_id.clone(),
        rule_version: rule.version,
        data_version: Some(data_version),
        evaluated_at_utc: now_utc,
        window_start_utc: input.window.map(|item| item.0),
        window_end_utc: input.window.map(|item| item.1),
        display_timezone: rule.timezone.clone(),
        selector: selector.to_string(),
        direction: rule.direction,
        observed_value: input.observed,
        trigger_threshold: rule.threshold_value,
        recovery_threshold: rule.recovery_threshold,
        coverage_summary: input.coverage.to_string(),
        policy_metadata: input.usage.and_then(|item| item.policy_metadata.clone()),
        report_query: input.usage.and_then(|item| item.report_query.clone()),
        not_evaluable_reason: if input.known {
            None
        } else {
            Some(input.coverage.to_string())
        },
    }
}

fn skeleton_instance(
    rule: &AlertRule,
    selector: &str,
    now_utc: i64,
    evidence: AlertEvidence,
) -> AlertInstance {
    AlertInstance {
        instance_id: format!("{}:{}:{selector}", rule.rule_id, rule.version),
        rule_id: rule.rule_id.clone(),
        rule_version: rule.version,
        selector_identity: selector.to_string(),
        status: InstanceStatus::Inactive,
        started_utc: None,
        resolved_utc: None,
        last_eval_utc: now_utc,
        last_observed: None,
        evidence,
    }
}

fn runtime_key(rule_id: &str, version: i64, selector: &str) -> String {
    format!("{rule_id}:{version}:{selector}")
}

fn is_gap_batch(batch: &AccountingBatch, health: &HealthSnapshot) -> bool {
    batch
        .coverage
        .iter()
        .any(|item| item.kind == "gap" || item.kind == "epoch")
        || health
            .coverage_kinds
            .iter()
            .any(|item| item == "gap" || item == "epoch")
        || health.storage.as_ref().is_some_and(|item| !item.ok)
        || batch.attributed_upload.is_none() && !batch.facts.is_empty()
}

fn health_kind_active(kind: &str, health: &HealthSnapshot) -> bool {
    match kind {
        "disconnect" => matches!(
            health.session,
            Some(
                SessionStatus::EndpointMissing
                    | SessionStatus::PipeBusyTimeout
                    | SessionStatus::PipeAccessDenied
                    | SessionStatus::Connecting
            )
        ),
        "tcp_auth" => matches!(health.session, Some(SessionStatus::AuthFailed)),
        "protocol" => matches!(health.session, Some(SessionStatus::ProtocolIncompatible)),
        "collection_gap" => health
            .coverage_kinds
            .iter()
            .any(|item| item == "gap" || item == "epoch"),
        "storage" => health.storage.as_ref().is_some_and(|item| !item.ok),
        "migration" => health.migration_failed,
        "backup" => health.backup_failed,
        _ => false,
    }
}

fn accumulate_fact(
    by_key: &mut HashMap<String, (u64, u64)>,
    fact: &MinuteFact,
    live: &[LiveConnectionView],
) {
    if let Some(primary) = &fact.primary {
        add_bytes(
            by_key,
            &selector_identity(SelectorKind::PrimaryCategory, primary),
            fact.upload,
            fact.download,
        );
    }
    let row = live.iter().find(|item| item.identity == fact.session_key);
    if let Some(host) = row.and_then(|item| item.host.as_deref()) {
        add_bytes(
            by_key,
            &selector_identity(SelectorKind::Domain, host),
            fact.upload,
            fact.download,
        );
    }
    if let Some(process) = row.and_then(|item| item.process_name.as_deref()) {
        add_bytes(
            by_key,
            &selector_identity(SelectorKind::Process, process),
            fact.upload,
            fact.download,
        );
    }
}

fn add_bytes(map: &mut HashMap<String, (u64, u64)>, key: &str, upload: u64, download: u64) {
    let entry = map.entry(key.to_string()).or_insert((0, 0));
    entry.0 = entry.0.saturating_add(upload);
    entry.1 = entry.1.saturating_add(download);
}

fn in_quiet(rule: &AlertRule, now_utc: i64) -> bool {
    let (Some(start), Some(end)) = (rule.quiet_start_min, rule.quiet_end_min) else {
        return false;
    };
    let offset = crate::c3::query::timezone_offset_secs(&rule.timezone, now_utc).unwrap_or(0);
    let local = now_utc + i64::from(offset);
    let minute = local.rem_euclid(86_400) / 60;
    if start <= end {
        minute >= start && minute < end
    } else {
        minute >= start || minute < end
    }
}

#[cfg(test)]
mod alert_engine_tests {
    use super::*;
    use crate::accounting::CoverageChange;

    fn rate_rule() -> AlertRule {
        AlertRule {
            rule_id: "rate-cat".into(),
            version: 1,
            enabled: true,
            kind: AlertKind::Rate,
            selector_kind: SelectorKind::PrimaryCategory,
            selector_value: Some("家宽".into()),
            direction: Some(AlertDirection::Download),
            threshold_value: 100,
            recovery_threshold: Some(40),
            period: None,
            timezone: "UTC".into(),
            cooldown_sec: 300,
            quiet_start_min: None,
            quiet_end_min: None,
            created_utc: 0,
            updated_utc: 0,
        }
    }

    fn fact(bytes: u64) -> MinuteFact {
        MinuteFact {
            session_key: "1:a".into(),
            utc_minute: 1,
            upload: 0,
            download: bytes,
            primary: Some("家宽".into()),
            tags: vec!["家宽".into()],
        }
    }

    fn batch(bytes: u64) -> AccountingBatch {
        AccountingBatch {
            facts: vec![fact(bytes)],
            coverage: Vec::new(),
            attributed_upload: Some(0),
            attributed_download: Some(bytes),
            meter_upload: Some(0),
            meter_download: Some(bytes),
            gap_upload: Some(0),
            gap_download: Some(0),
            over_upload: Some(0),
            over_download: Some(0),
        }
    }

    fn live() -> Vec<LiveConnectionView> {
        vec![LiveConnectionView {
            identity: "1:a".into(),
            connection_id: "a".into(),
            epoch: 1,
            upload: 0,
            download: 1,
            rate_upload: None,
            rate_download: None,
            duration_ms: None,
            primary: Some("家宽".into()),
            tags: vec!["家宽".into()],
            host: Some("a.example".into()),
            source_ip: None,
            destination_ip: None,
            process_name: Some("app.exe".into()),
            process_path: None,
            network: Some("tcp".into()),
            rule: None,
            rule_payload: None,
            chains: vec!["家宽".into()],
            ..LiveConnectionView::default()
        }]
    }

    fn eval_frame(
        engine: &mut AlertEngine,
        batch: &AccountingBatch,
        live_rows: &[LiveConnectionView],
        health: &HealthSnapshot,
        utc: i64,
        mono: u64,
        bundle: &str,
    ) -> AlertWriteSet {
        engine.evaluate_frame(FrameInput {
            batch,
            live: live_rows,
            health,
            usages: &[],
            now_utc: utc,
            now_mono: mono,
            data_version: 1,
            bundle_id: bundle,
        })
    }

    fn eval_with_rate(
        engine: &mut AlertEngine,
        download_bytes_in_window: u64,
        utc: i64,
        mono: u64,
        bundle: &str,
    ) -> AlertWriteSet {
        engine.set_window_bytes("primary_category:家宽", mono, download_bytes_in_window);
        engine.evaluate_frame(FrameInput {
            batch: &batch(1),
            live: &live(),
            health: &HealthSnapshot::default(),
            usages: &[],
            now_utc: utc,
            now_mono: mono,
            data_version: 1,
            bundle_id: bundle,
        })
    }

    #[test]
    fn rate_needs_three_consecutive_hits() {
        let mut engine = AlertEngine::new();
        engine.load_rules(vec![rate_rule()]).expect("rules");
        // 100 bps * 60s = 6000 字节。7200 字节 ≈ 120 bps。
        let first = eval_with_rate(&mut engine, 7_200, 1_000, 60_000, "h1");
        let second = eval_with_rate(&mut engine, 7_200, 1_001, 61_000, "h2");
        let third = eval_with_rate(&mut engine, 7_200, 1_002, 62_000, "h3");
        assert!(!first
            .events
            .iter()
            .any(|item| item.kind == EventKind::Activated));
        assert!(!second
            .events
            .iter()
            .any(|item| item.kind == EventKind::Activated));
        assert!(third
            .events
            .iter()
            .any(|item| item.kind == EventKind::Activated));
        assert_eq!(engine.sql_in_hot_path(), 0);
    }

    #[test]
    fn hysteresis_keeps_active_inside_band() {
        let mut engine = AlertEngine::new();
        engine.load_rules(vec![rate_rule()]).expect("rules");
        for step in 1..=3 {
            let _ = eval_with_rate(
                &mut engine,
                7_200,
                1_000 + step,
                60_000 + (step as u64) * 1_000,
                &format!("h{step}"),
            );
        }
        // 70 bps * 60s = 4200 字节，位于 40–100 滞回带。
        let mid = eval_with_rate(&mut engine, 4_200, 1_010, 70_000, "mid");
        assert!(!mid
            .events
            .iter()
            .any(|item| item.kind == EventKind::Recovered));
        let low = eval_with_rate(&mut engine, 600, 1_011, 71_000, "low");
        assert!(low
            .events
            .iter()
            .any(|item| item.kind == EventKind::Recovered));
    }

    #[test]
    fn gap_is_not_zero_rate() {
        let mut engine = AlertEngine::new();
        engine.load_rules(vec![rate_rule()]).expect("rules");
        let _ = eval_with_rate(&mut engine, 7_200, 1_000, 60_000, "prep");
        let mut gap_batch = batch(0);
        gap_batch.facts.clear();
        gap_batch.coverage = vec![CoverageChange {
            kind: "gap",
            reason: "disconnect_or_sleep",
        }];
        gap_batch.attributed_download = None;
        let out = eval_frame(
            &mut engine,
            &gap_batch,
            &[],
            &HealthSnapshot {
                coverage_kinds: vec!["gap".into()],
                ..HealthSnapshot::default()
            },
            2_000,
            80_000,
            "gap",
        );
        assert!(!out
            .events
            .iter()
            .any(|item| item.kind == EventKind::Activated));
        assert!(out.instances.iter().any(|item| {
            item.status == InstanceStatus::NotEvaluable
                || item.evidence.not_evaluable_reason.is_some()
        }));
    }

    #[test]
    fn each_health_kind_activates_and_recovers() {
        let kinds = [
            "disconnect",
            "tcp_auth",
            "protocol",
            "collection_gap",
            "storage",
            "migration",
            "backup",
        ];
        for kind in kinds {
            let mut engine = AlertEngine::new();
            let mut rule = rate_rule();
            rule.rule_id = format!("health-{kind}");
            rule.kind = AlertKind::Health;
            rule.selector_kind = SelectorKind::HealthKind;
            rule.selector_value = Some(kind.into());
            rule.direction = None;
            rule.recovery_threshold = None;
            engine.load_rules(vec![rule]).expect("rules");
            let session = match kind {
                "disconnect" => SessionStatus::EndpointMissing,
                "tcp_auth" => SessionStatus::AuthFailed,
                "protocol" => SessionStatus::ProtocolIncompatible,
                _ => SessionStatus::Connected,
            };
            let bad = HealthSnapshot {
                session: Some(session),
                coverage_kinds: if kind == "collection_gap" {
                    vec!["gap".into()]
                } else {
                    Vec::new()
                },
                storage: if kind == "storage" {
                    Some(crate::storage::StorageHealth {
                        ok: false,
                        watermark: 0,
                        reason: Some("io"),
                    })
                } else {
                    None
                },
                migration_failed: kind == "migration",
                backup_failed: kind == "backup",
            };
            let empty = AccountingBatch {
                facts: Vec::new(),
                coverage: Vec::new(),
                attributed_upload: None,
                attributed_download: None,
                meter_upload: None,
                meter_download: None,
                gap_upload: None,
                gap_download: None,
                over_upload: None,
                over_download: None,
            };
            let on = eval_frame(&mut engine, &empty, &[], &bad, 10, 10, "on");
            assert!(
                on.events
                    .iter()
                    .any(|item| item.kind == EventKind::Activated),
                "{kind} 应激活"
            );
            let good = HealthSnapshot {
                session: Some(SessionStatus::Connected),
                ..HealthSnapshot::default()
            };
            let off = eval_frame(&mut engine, &empty, &[], &good, 11, 11, "off");
            assert!(
                off.events
                    .iter()
                    .any(|item| item.kind == EventKind::Recovered),
                "{kind} 应恢复"
            );
        }
    }

    #[test]
    fn health_dedups_same_root_cause() {
        let mut engine = AlertEngine::new();
        let mut rule = rate_rule();
        rule.rule_id = "health-tcp-auth".into();
        rule.kind = AlertKind::Health;
        rule.selector_kind = SelectorKind::HealthKind;
        rule.selector_value = Some("tcp_auth".into());
        rule.direction = None;
        rule.recovery_threshold = None;
        engine.load_rules(vec![rule]).expect("rules");
        let health = HealthSnapshot {
            session: Some(SessionStatus::AuthFailed),
            ..HealthSnapshot::default()
        };
        let empty = AccountingBatch {
            facts: Vec::new(),
            coverage: Vec::new(),
            attributed_upload: None,
            attributed_download: None,
            meter_upload: None,
            meter_download: None,
            gap_upload: None,
            gap_download: None,
            over_upload: None,
            over_download: None,
        };
        let first = eval_frame(&mut engine, &empty, &[], &health, 10, 10, "h1");
        let second = eval_frame(&mut engine, &empty, &[], &health, 11, 11, "h2");
        assert_eq!(
            first
                .events
                .iter()
                .filter(|item| item.kind == EventKind::Activated)
                .count(),
            1
        );
        assert!(!second
            .events
            .iter()
            .any(|item| item.kind == EventKind::Activated));
    }

    #[test]
    fn rule_edit_resets_consecutive_hits() {
        let mut engine = AlertEngine::new();
        engine.load_rules(vec![rate_rule()]).expect("rules");
        let _ = eval_with_rate(&mut engine, 7_200, 1_000, 60_000, "a1");
        let mut edited = rate_rule();
        edited.threshold_value = 10_000;
        edited.recovery_threshold = Some(1_000);
        let writes = engine.upsert_rule(edited, 2_000).expect("edit");
        assert!(writes
            .events
            .iter()
            .any(|item| item.kind == EventKind::Superseded));
        let after = eval_frame(
            &mut engine,
            &batch(200),
            &live(),
            &HealthSnapshot::default(),
            2_001,
            62_000,
            "a2",
        );
        assert!(!after
            .events
            .iter()
            .any(|item| item.kind == EventKind::Activated));
    }

    #[test]
    fn many_rules_do_not_issue_sql_on_hot_path() {
        let mut engine = AlertEngine::new();
        let rules = (0..80)
            .map(|index| {
                let mut rule = rate_rule();
                rule.rule_id = format!("r{index}");
                rule.selector_value = Some(format!("cat{index}"));
                rule
            })
            .collect();
        engine.load_rules(rules).expect("rules");
        let started = std::time::Instant::now();
        for step in 0..30 {
            let _ = eval_frame(
                &mut engine,
                &batch(10),
                &live(),
                &HealthSnapshot::default(),
                1_000 + step,
                60_000 + step as u64 * 1_000,
                &format!("p{step}"),
            );
        }
        assert_eq!(engine.sql_in_hot_path(), 0);
        assert!(started.elapsed().as_millis() < 500);
    }
}
