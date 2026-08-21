//! 在 C1 LiveProjection 之上提供原子 bootstrap / 增量。

use crate::accounting::AccountingBatch;
use crate::c2::contract::{COALESCE_MAX_BYTES, COALESCE_MAX_KEYS, SCHEMA_VERSION};
use crate::controller::SessionStatus;
use crate::live::{LiveProjection, MonitorMessage};
use crate::storage::StorageHealth;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveConnectionView {
    pub identity: String,
    pub connection_id: String,
    pub epoch: u64,
    pub upload: u64,
    pub download: u64,
    pub rate_upload: Option<u64>,
    pub rate_download: Option<u64>,
    pub duration_ms: Option<u64>,
    pub primary: Option<String>,
    pub tags: Vec<String>,
    pub host: Option<String>,
    pub source_ip: Option<String>,
    pub destination_ip: Option<String>,
    pub process_name: Option<String>,
    pub process_path: Option<String>,
    pub network: Option<String>,
    pub inbound: Option<String>,
    pub source_port: Option<String>,
    pub destination_port: Option<String>,
    pub start: Option<String>,
    pub rule: Option<String>,
    pub rule_payload: Option<String>,
    pub chains: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthView {
    pub session: String,
    pub storage_ok: bool,
    pub storage_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveOverview {
    pub schema_version: u32,
    pub observation_phase: ObservationPhase,
    pub meter_upload: Option<u64>,
    pub meter_download: Option<u64>,
    pub attributed_upload: Option<u64>,
    pub attributed_download: Option<u64>,
    pub category_upload: BTreeMap<String, u64>,
    pub category_download: BTreeMap<String, u64>,
    pub other_upload: Option<u64>,
    pub other_download: Option<u64>,
    pub gap_upload: Option<u64>,
    pub gap_download: Option<u64>,
    pub over_upload: Option<u64>,
    pub over_download: Option<u64>,
    pub active_count: u32,
    pub last_sample_utc: Option<i64>,
    pub coverage_kind: Option<String>,
    pub coverage_reason: Option<String>,
    pub health: HealthView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ObservationPhase {
    Unconfigured,
    Connecting,
    BaselinePending,
    Current,
    Paused,
    Disconnected,
    ResyncRequired,
    DecodeFailed,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MonitorStreamMessage {
    #[serde(rename = "bootstrap")]
    Bootstrap {
        #[serde(rename = "schemaVersion")]
        schema_version: u32,
        #[serde(rename = "subscriptionId")]
        subscription_id: u64,
        snapshot: LiveOverview,
        #[serde(rename = "baseSeq")]
        base_seq: u64,
        #[serde(rename = "backendTime")]
        backend_time: i64,
    },
    #[serde(rename = "connectionDelta")]
    ConnectionDelta {
        #[serde(rename = "schemaVersion")]
        schema_version: u32,
        #[serde(rename = "subscriptionId")]
        subscription_id: u64,
        seq: u64,
        snapshot: LiveOverview,
        upserts: Vec<LiveConnectionView>,
        removes: Vec<String>,
        #[serde(rename = "backendTime")]
        backend_time: i64,
    },
    #[serde(rename = "healthChanged")]
    HealthChanged {
        #[serde(rename = "schemaVersion")]
        schema_version: u32,
        #[serde(rename = "subscriptionId")]
        subscription_id: u64,
        seq: u64,
        health: HealthView,
        #[serde(rename = "backendTime")]
        backend_time: i64,
    },
    #[serde(rename = "summaryChanged")]
    SummaryChanged {
        #[serde(rename = "schemaVersion")]
        schema_version: u32,
        #[serde(rename = "subscriptionId")]
        subscription_id: u64,
        seq: u64,
        snapshot: LiveOverview,
        #[serde(rename = "backendTime")]
        backend_time: i64,
    },
    #[serde(rename = "alertChanged")]
    AlertChanged {
        #[serde(rename = "schemaVersion")]
        schema_version: u32,
        #[serde(rename = "subscriptionId")]
        subscription_id: u64,
        seq: u64,
        summary: crate::c4::types::AlertSummary,
        #[serde(rename = "backendTime")]
        backend_time: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PendingOp {
    Upsert(Box<LiveConnectionView>),
    Remove(String),
}

#[derive(Debug)]
pub struct CoalesceOverflow;

#[derive(Debug, Default)]
struct Coalescer {
    pending: BTreeMap<String, PendingOp>,
    bytes: usize,
}

impl Coalescer {
    fn push(&mut self, op: PendingOp) -> Result<(), CoalesceOverflow> {
        let key = match &op {
            PendingOp::Upsert(row) => row.identity.clone(),
            PendingOp::Remove(id) => id.clone(),
        };
        if let Some(existing) = self.pending.get(&key) {
            if matches!(existing, PendingOp::Remove(_)) && matches!(op, PendingOp::Upsert(_)) {
                return Ok(());
            }
        }
        let size = key.len() + 64;
        if !self.pending.contains_key(&key)
            && (self.pending.len() + 1 > COALESCE_MAX_KEYS
                || self.bytes + size > COALESCE_MAX_BYTES)
        {
            return Err(CoalesceOverflow);
        }
        if !self.pending.contains_key(&key) {
            self.bytes += size;
        }
        self.pending.insert(key, op);
        Ok(())
    }

    fn take(&mut self) -> (Vec<LiveConnectionView>, Vec<String>) {
        let pending = std::mem::take(&mut self.pending);
        self.bytes = 0;
        let mut upserts = Vec::new();
        let mut removes = Vec::new();
        for op in pending.into_values() {
            match op {
                PendingOp::Upsert(row) => upserts.push(*row),
                PendingOp::Remove(id) => removes.push(id),
            }
        }
        (upserts, removes)
    }
}

#[derive(Debug)]
struct Inner {
    snapshot: LiveOverview,
    rows: BTreeMap<String, LiveConnectionView>,
    prev_sample: BTreeMap<String, (u64, u64, i64)>,
    coalescer: Coalescer,
    next_subscription: u64,
    active: BTreeMap<u64, bool>,
    serialize_ui: bool,
    overflow: bool,
}

pub struct MonitorHub {
    live: LiveProjection,
    inner: Mutex<Inner>,
}

impl Default for MonitorHub {
    fn default() -> Self {
        Self::new()
    }
}

impl MonitorHub {
    pub fn new() -> Self {
        Self {
            live: LiveProjection::new(),
            inner: Mutex::new(Inner {
                snapshot: empty_overview(SessionStatus::Connecting),
                rows: BTreeMap::new(),
                prev_sample: BTreeMap::new(),
                coalescer: Coalescer::default(),
                next_subscription: 1,
                active: BTreeMap::new(),
                serialize_ui: true,
                overflow: false,
            }),
        }
    }

    pub fn set_serialize_ui(&self, enabled: bool) {
        self.inner.lock().expect("hub").serialize_ui = enabled;
    }

    pub fn set_observation_phase(&self, phase: ObservationPhase) {
        self.inner.lock().expect("hub").snapshot.observation_phase = phase;
    }

    pub fn subscribe(&self) -> MonitorStreamMessage {
        let c1 = self.live.subscribe();
        let base_seq = match c1 {
            MonitorMessage::Bootstrap { base_seq } => base_seq,
            MonitorMessage::Delta { seq } => seq,
        };
        let mut guard = self.inner.lock().expect("hub");
        let subscription_id = guard.next_subscription;
        guard.next_subscription += 1;
        guard.active.insert(subscription_id, true);
        guard.overflow = false;
        MonitorStreamMessage::Bootstrap {
            schema_version: SCHEMA_VERSION,
            subscription_id,
            snapshot: guard.snapshot.clone(),
            base_seq,
            backend_time: guard.snapshot.last_sample_utc.unwrap_or(0),
        }
    }

    pub fn resync(&self, old_subscription: u64) -> MonitorStreamMessage {
        {
            let mut guard = self.inner.lock().expect("hub");
            guard.active.remove(&old_subscription);
        }
        self.subscribe()
    }

    pub fn drop_subscription(&self, subscription_id: u64) {
        self.inner
            .lock()
            .expect("hub")
            .active
            .remove(&subscription_id);
    }

    pub fn is_active(&self, subscription_id: u64) -> bool {
        self.inner
            .lock()
            .expect("hub")
            .active
            .contains_key(&subscription_id)
    }

    pub fn publish(
        &self,
        batch: &AccountingBatch,
        live_rows: Vec<LiveConnectionView>,
        health: HealthView,
        utc: i64,
    ) -> Result<Option<MonitorStreamMessage>, CoalesceOverflow> {
        let seq = self.live.apply_receipt(utc as u64);
        let mut guard = self.inner.lock().expect("hub");
        let previous: Vec<String> = guard.rows.keys().cloned().collect();
        let mut next = BTreeMap::new();
        let mut next_sample = BTreeMap::new();
        for mut row in live_rows {
            row.duration_ms = duration_from_start(row.start.as_deref(), utc);
            if let Some((prev_up, prev_down, prev_utc)) = guard.prev_sample.get(&row.connection_id)
            {
                if utc > *prev_utc {
                    let dt = (utc - *prev_utc) as u64;
                    row.rate_upload =
                        Some(row.upload.saturating_sub(*prev_up).saturating_mul(1000) / dt.max(1));
                    row.rate_download = Some(
                        row.download.saturating_sub(*prev_down).saturating_mul(1000) / dt.max(1),
                    );
                }
            }
            next_sample.insert(row.connection_id.clone(), (row.upload, row.download, utc));
            next.insert(row.identity.clone(), row);
        }
        guard.prev_sample = next_sample;
        for gone in previous {
            if !next.contains_key(&gone) {
                guard
                    .coalescer
                    .push(PendingOp::Remove(gone))
                    .map_err(|_| CoalesceOverflow)?;
            }
        }
        for (id, row) in &next {
            guard
                .coalescer
                .push(PendingOp::Upsert(Box::new(row.clone())))
                .map_err(|_| CoalesceOverflow)?;
            let _ = id;
        }
        guard.rows = next;
        guard.snapshot = overview_from(batch, &guard.rows, health, Some(utc));
        if !guard.serialize_ui || guard.active.is_empty() {
            let _ = guard.coalescer.take();
            return Ok(None);
        }
        let (upserts, removes) = guard.coalescer.take();
        let subscription_id = *guard.active.keys().next().unwrap_or(&0);
        Ok(Some(MonitorStreamMessage::ConnectionDelta {
            schema_version: SCHEMA_VERSION,
            subscription_id,
            seq,
            snapshot: guard.snapshot.clone(),
            upserts,
            removes,
            backend_time: utc,
        }))
    }

    /// Publish a lifecycle/health transition without pretending it was a new
    /// controller sample. Retained rows keep their previous rates and sample
    /// time; terminal transitions may explicitly clear them.
    pub fn publish_lifecycle(
        &self,
        batch: &AccountingBatch,
        retain_rows: bool,
        health: HealthView,
        utc: i64,
    ) -> Result<Option<MonitorStreamMessage>, CoalesceOverflow> {
        let seq = self.live.apply_receipt(utc as u64);
        let mut guard = self.inner.lock().expect("hub");
        if !retain_rows {
            let removed: Vec<String> = guard.rows.keys().cloned().collect();
            for identity in removed {
                guard
                    .coalescer
                    .push(PendingOp::Remove(identity))
                    .map_err(|_| CoalesceOverflow)?;
            }
            guard.rows.clear();
            guard.prev_sample.clear();
        }
        let last_sample_utc = guard.snapshot.last_sample_utc;
        guard.snapshot = overview_from(batch, &guard.rows, health, last_sample_utc);
        if !guard.serialize_ui || guard.active.is_empty() {
            let _ = guard.coalescer.take();
            return Ok(None);
        }
        let (upserts, removes) = guard.coalescer.take();
        let subscription_id = *guard.active.keys().next().unwrap_or(&0);
        Ok(Some(MonitorStreamMessage::ConnectionDelta {
            schema_version: SCHEMA_VERSION,
            subscription_id,
            seq,
            snapshot: guard.snapshot.clone(),
            upserts,
            removes,
            backend_time: utc,
        }))
    }

    pub fn publish_alert(
        &self,
        summary: crate::c4::types::AlertSummary,
        utc: i64,
    ) -> MonitorStreamMessage {
        let seq = self.live.apply_receipt(utc as u64);
        let guard = self.inner.lock().expect("hub");
        let subscription_id = *guard.active.keys().next().unwrap_or(&0);
        MonitorStreamMessage::AlertChanged {
            schema_version: SCHEMA_VERSION,
            subscription_id,
            seq,
            summary,
            backend_time: utc,
        }
    }

    pub fn publish_health(&self, health: HealthView, utc: i64) -> MonitorStreamMessage {
        let seq = self.live.apply_receipt(utc as u64);
        let mut guard = self.inner.lock().expect("hub");
        guard.snapshot.health = health.clone();
        let subscription_id = *guard.active.keys().next().unwrap_or(&0);
        MonitorStreamMessage::HealthChanged {
            schema_version: SCHEMA_VERSION,
            subscription_id,
            seq,
            health,
            backend_time: utc,
        }
    }

    pub fn overview(&self) -> LiveOverview {
        self.inner.lock().expect("hub").snapshot.clone()
    }

    pub fn rows(&self) -> Vec<LiveConnectionView> {
        self.inner
            .lock()
            .expect("hub")
            .rows
            .values()
            .cloned()
            .collect()
    }

    /// 在同一把 hub 锁下取出列表与概览，避免 rows 与 sampleUtc 来自不同采集 tick。
    pub fn query_snapshot(&self) -> (Vec<LiveConnectionView>, LiveOverview) {
        let guard = self.inner.lock().expect("hub");
        (
            guard.rows.values().cloned().collect(),
            guard.snapshot.clone(),
        )
    }

    pub fn row(&self, identity: &str) -> Option<LiveConnectionView> {
        self.inner.lock().expect("hub").rows.get(identity).cloned()
    }

    pub fn overflowed(&self) -> bool {
        self.inner.lock().expect("hub").overflow
    }

    pub fn row_count(&self) -> usize {
        self.inner.lock().expect("hub").rows.len()
    }
}

fn duration_from_start(start: Option<&str>, utc: i64) -> Option<u64> {
    let start = start?;
    let parsed = chrono::DateTime::parse_from_rfc3339(start)
        .or_else(|_| chrono::DateTime::parse_from_rfc3339(&format!("{start}Z")))
        .ok()?;
    let delta = utc
        .saturating_mul(1000)
        .saturating_sub(parsed.timestamp_millis());
    Some(delta.max(0) as u64)
}

pub fn session_status_name(status: SessionStatus) -> String {
    match status {
        SessionStatus::Connecting => "connecting",
        SessionStatus::Connected => "connected",
        SessionStatus::AuthFailed => "tcp_unauthorized",
        SessionStatus::PipeAccessDenied => "pipe_access_denied",
        SessionStatus::PipeBusyTimeout => "pipe_busy_timeout",
        SessionStatus::EndpointMissing => "endpoint_missing",
        SessionStatus::ProtocolIncompatible => "protocol_incompatible",
        SessionStatus::PidMismatch => "pid_mismatch",
        SessionStatus::CoreRestarted => "core_restarted",
        SessionStatus::Cancelled => "cancelled",
        SessionStatus::NonLoopback => "non_loopback",
    }
    .into()
}

pub fn health_from(status: SessionStatus, storage: Option<&StorageHealth>) -> HealthView {
    HealthView {
        session: session_status_name(status),
        storage_ok: storage.map(|item| item.ok).unwrap_or(true),
        storage_reason: storage.and_then(|item| item.reason.map(str::to_string)),
    }
}

fn empty_overview(status: SessionStatus) -> LiveOverview {
    LiveOverview {
        schema_version: SCHEMA_VERSION,
        observation_phase: phase_from_status(status),
        meter_upload: None,
        meter_download: None,
        attributed_upload: None,
        attributed_download: None,
        category_upload: BTreeMap::new(),
        category_download: BTreeMap::new(),
        other_upload: None,
        other_download: None,
        gap_upload: None,
        gap_download: None,
        over_upload: None,
        over_download: None,
        active_count: 0,
        last_sample_utc: None,
        coverage_kind: None,
        coverage_reason: None,
        health: health_from(status, None),
    }
}

fn overview_from(
    batch: &AccountingBatch,
    rows: &BTreeMap<String, LiveConnectionView>,
    health: HealthView,
    last_sample_utc: Option<i64>,
) -> LiveOverview {
    let mut category_upload = BTreeMap::new();
    let mut category_download = BTreeMap::new();
    let mut other_up = 0_u64;
    let mut other_down = 0_u64;
    for fact in &batch.facts {
        match &fact.primary {
            Some(name) => {
                *category_upload.entry(name.clone()).or_insert(0) += fact.upload;
                *category_download.entry(name.clone()).or_insert(0) += fact.download;
            }
            None => {
                other_up += fact.upload;
                other_down += fact.download;
            }
        }
    }
    let coverage = batch.coverage.first();
    LiveOverview {
        schema_version: SCHEMA_VERSION,
        observation_phase: phase_from_batch(batch, &health),
        meter_upload: batch.meter_upload,
        meter_download: batch.meter_download,
        attributed_upload: batch.attributed_upload,
        attributed_download: batch.attributed_download,
        category_upload,
        category_download,
        other_upload: batch.attributed_upload.map(|_| other_up),
        other_download: batch.attributed_download.map(|_| other_down),
        gap_upload: batch.gap_upload,
        gap_download: batch.gap_download,
        over_upload: batch.over_upload,
        over_download: batch.over_download,
        active_count: rows.len() as u32,
        last_sample_utc,
        coverage_kind: coverage.map(|item| item.kind.to_string()),
        coverage_reason: coverage.map(|item| item.reason.to_string()),
        health,
    }
}

fn phase_from_status(status: SessionStatus) -> ObservationPhase {
    match status {
        SessionStatus::Connecting => ObservationPhase::Connecting,
        SessionStatus::Connected => ObservationPhase::BaselinePending,
        SessionStatus::ProtocolIncompatible => ObservationPhase::DecodeFailed,
        SessionStatus::CoreRestarted => ObservationPhase::ResyncRequired,
        SessionStatus::AuthFailed
        | SessionStatus::PipeAccessDenied
        | SessionStatus::PipeBusyTimeout
        | SessionStatus::EndpointMissing
        | SessionStatus::PidMismatch
        | SessionStatus::Cancelled
        | SessionStatus::NonLoopback => ObservationPhase::Disconnected,
    }
}

fn phase_from_batch(batch: &AccountingBatch, health: &HealthView) -> ObservationPhase {
    if batch
        .coverage
        .iter()
        .any(|item| item.kind == "epoch" && item.reason == "core_restart")
    {
        return ObservationPhase::ResyncRequired;
    }
    if batch
        .coverage
        .iter()
        .any(|item| item.kind == "closed" && item.reason == "pause_or_shutdown")
    {
        return ObservationPhase::Paused;
    }
    match health.session.as_str() {
        "connected" => {
            if batch.meter_upload.is_some()
                && batch.meter_download.is_some()
                && batch.attributed_upload.is_some()
                && batch.attributed_download.is_some()
            {
                ObservationPhase::Current
            } else {
                ObservationPhase::BaselinePending
            }
        }
        "connecting" => ObservationPhase::Connecting,
        "protocol_incompatible" => ObservationPhase::DecodeFailed,
        "core_restarted" => ObservationPhase::ResyncRequired,
        _ => ObservationPhase::Disconnected,
    }
}

#[cfg(test)]
mod channel_contract_tests {
    use super::*;

    #[test]
    fn subscribe_first_message_is_atomic_bootstrap() {
        let hub = MonitorHub::new();
        let first = hub.subscribe();
        assert!(matches!(
            first,
            MonitorStreamMessage::Bootstrap {
                schema_version: SCHEMA_VERSION,
                base_seq: 0,
                ..
            }
        ));
    }

    #[test]
    fn resync_issues_new_subscription_identity() {
        let hub = MonitorHub::new();
        let MonitorStreamMessage::Bootstrap {
            subscription_id: first,
            ..
        } = hub.subscribe()
        else {
            panic!("bootstrap");
        };
        let MonitorStreamMessage::Bootstrap {
            subscription_id: second,
            ..
        } = hub.resync(first)
        else {
            panic!("resync");
        };
        assert_ne!(first, second);
        assert!(!hub.is_active(first));
        assert!(hub.is_active(second));
    }

    #[test]
    fn observation_phase_distinguishes_baseline_current_and_failures() {
        let mut batch = AccountingBatch {
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
        assert_eq!(
            phase_from_batch(&batch, &health_from(SessionStatus::Connected, None)),
            ObservationPhase::BaselinePending
        );
        batch.attributed_upload = Some(0);
        batch.attributed_download = Some(0);
        batch.meter_upload = Some(0);
        batch.meter_download = Some(0);
        assert_eq!(
            phase_from_batch(&batch, &health_from(SessionStatus::Connected, None)),
            ObservationPhase::Current
        );
        assert_eq!(
            phase_from_status(SessionStatus::ProtocolIncompatible),
            ObservationPhase::DecodeFailed
        );
        assert_eq!(
            phase_from_status(SessionStatus::CoreRestarted),
            ObservationPhase::ResyncRequired
        );
    }

    #[test]
    fn connection_delta_carries_the_same_current_overview() {
        let hub = MonitorHub::new();
        let MonitorStreamMessage::Bootstrap {
            subscription_id, ..
        } = hub.subscribe()
        else {
            panic!("bootstrap");
        };
        let batch = AccountingBatch {
            facts: Vec::new(),
            coverage: Vec::new(),
            attributed_upload: Some(0),
            attributed_download: Some(0),
            meter_upload: Some(0),
            meter_download: Some(0),
            gap_upload: Some(0),
            gap_download: Some(0),
            over_upload: Some(0),
            over_download: Some(0),
        };
        let message = hub
            .publish(
                &batch,
                Vec::new(),
                health_from(SessionStatus::Connected, None),
                100,
            )
            .expect("publish")
            .expect("message");
        let MonitorStreamMessage::ConnectionDelta {
            subscription_id: actual,
            snapshot,
            ..
        } = message
        else {
            panic!("delta");
        };
        assert_eq!(actual, subscription_id);
        assert_eq!(snapshot.observation_phase, ObservationPhase::Current);
        assert_eq!(snapshot.last_sample_utc, Some(100));
    }

    #[test]
    fn lifecycle_publish_preserves_last_controller_sample() {
        let hub = MonitorHub::new();
        let current = AccountingBatch {
            facts: Vec::new(),
            coverage: Vec::new(),
            attributed_upload: Some(0),
            attributed_download: Some(0),
            meter_upload: Some(0),
            meter_download: Some(0),
            gap_upload: Some(0),
            gap_download: Some(0),
            over_upload: Some(0),
            over_download: Some(0),
        };
        hub.publish(
            &current,
            Vec::new(),
            health_from(SessionStatus::Connected, None),
            100,
        )
        .expect("sample");
        let gap = AccountingBatch {
            facts: Vec::new(),
            coverage: vec![crate::accounting::CoverageChange {
                kind: "gap",
                reason: "disconnect_or_sleep",
            }],
            attributed_upload: None,
            attributed_download: None,
            meter_upload: None,
            meter_download: None,
            gap_upload: None,
            gap_download: None,
            over_upload: None,
            over_download: None,
        };
        hub.publish_lifecycle(
            &gap,
            true,
            health_from(SessionStatus::ProtocolIncompatible, None),
            200,
        )
        .expect("lifecycle");

        let overview = hub.overview();
        assert_eq!(overview.last_sample_utc, Some(100));
        assert_eq!(overview.observation_phase, ObservationPhase::DecodeFailed);
    }
}

#[cfg(test)]
mod coalesce_tests {
    use super::*;

    #[test]
    fn remove_wins_same_window() {
        let mut coalescer = Coalescer::default();
        let row = LiveConnectionView {
            identity: "0:a".into(),
            connection_id: "a".into(),
            epoch: 0,
            upload: 1,
            download: 1,
            rate_upload: None,
            rate_download: None,
            duration_ms: None,
            primary: None,
            tags: Vec::new(),
            host: None,
            source_ip: None,
            destination_ip: None,
            process_name: None,
            process_path: None,
            network: None,
            rule: None,
            rule_payload: None,
            chains: Vec::new(),
            ..LiveConnectionView::default()
        };
        coalescer
            .push(PendingOp::Upsert(Box::new(row.clone())))
            .expect("up");
        coalescer.push(PendingOp::Remove("0:a".into())).expect("rm");
        coalescer
            .push(PendingOp::Upsert(Box::new(row)))
            .expect("late");
        let (upserts, removes) = coalescer.take();
        assert!(upserts.is_empty());
        assert_eq!(removes, vec!["0:a".to_string()]);
    }
}
