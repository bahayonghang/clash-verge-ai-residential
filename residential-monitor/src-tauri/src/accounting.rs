//! 无 I/O 的核算状态机。

use crate::c0_contract::STRING_LIMIT;
use crate::controller::{ConnectionFact, ConnectionMeta, ControllerInput};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MinuteFact {
    pub session_key: String,
    pub utc_minute: i64,
    pub upload: u64,
    pub download: u64,
    pub primary: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CoverageChange {
    pub kind: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountingBatch {
    pub facts: Vec<MinuteFact>,
    pub coverage: Vec<CoverageChange>,
    pub attributed_upload: Option<u64>,
    pub attributed_download: Option<u64>,
    pub meter_upload: Option<u64>,
    pub meter_download: Option<u64>,
    pub gap_upload: Option<u64>,
    pub gap_download: Option<u64>,
    pub over_upload: Option<u64>,
    pub over_download: Option<u64>,
}

#[derive(Debug, Clone)]
struct SessionAcc {
    connection_id: String,
    last_upload: u64,
    last_download: u64,
    last_mono: u64,
    last_utc: i64,
    seen: bool,
    meta: ConnectionMeta,
    chains: Vec<String>,
    provider_chains: Vec<String>,
}

#[derive(Debug, Default)]
pub struct AccountingEngine {
    epoch: u64,
    sessions: HashMap<String, SessionAcc>,
    retired_ids: HashSet<String>,
    last_meter_up: Option<u64>,
    last_meter_down: Option<u64>,
    targets: Vec<String>,
    policy_version: u32,
}

impl AccountingEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_targets(&mut self, targets: Vec<String>) {
        self.targets = targets;
        self.policy_version = self.policy_version.saturating_add(1);
    }

    pub fn current_epoch(&self) -> u64 {
        self.epoch
    }

    pub fn reset_epoch(&mut self, epoch: u64) {
        self.epoch = epoch;
        self.sessions.clear();
        self.retired_ids.clear();
        self.last_meter_up = None;
        self.last_meter_down = None;
    }

    pub fn snapshot_requires_new_generation(
        &self,
        connections: &[ConnectionFact],
        upload_total: u64,
        download_total: u64,
    ) -> bool {
        if self.last_meter_up.is_some_and(|value| upload_total < value)
            || self
                .last_meter_down
                .is_some_and(|value| download_total < value)
        {
            return true;
        }
        connections.iter().any(|connection| {
            if self.retired_ids.contains(&connection.id) {
                return true;
            }
            let key = format!("{}:{}", self.epoch, connection.id);
            self.sessions.get(&key).is_some_and(|session| {
                connection.upload < session.last_upload
                    || connection.download < session.last_download
                    || start_changed(
                        session.meta.start.as_deref(),
                        connection.meta.start.as_deref(),
                    )
            })
        })
    }

    pub fn policy_version(&self) -> u32 {
        self.policy_version
    }

    pub fn targets(&self) -> &[String] {
        &self.targets
    }

    pub fn project_live(
        &self,
        connections: &[ConnectionFact],
    ) -> Vec<crate::c2::hub::LiveConnectionView> {
        connections
            .iter()
            .map(|connection| {
                let (tags, primary) = classify(&self.targets, &connection.chains);
                crate::c2::hub::LiveConnectionView {
                    identity: format!("{}:{}", self.epoch, connection.id),
                    connection_id: connection.id.clone(),
                    epoch: self.epoch,
                    upload: connection.upload,
                    download: connection.download,
                    rate_upload: None,
                    rate_download: None,
                    duration_ms: None,
                    primary,
                    tags,
                    host: crate::session_host::resolve_host_identity(
                        connection.meta.host.as_deref(),
                        connection.meta.sniff_host.as_deref(),
                        connection.meta.destination_ip.as_deref(),
                    ),
                    source_ip: connection.meta.source_ip.clone(),
                    destination_ip: connection.meta.destination_ip.clone(),
                    process_name: resolve_process_identity(
                        connection.meta.process_name.as_deref(),
                        connection.meta.process_path.as_deref(),
                    ),
                    process_path: connection.meta.process_path.clone(),
                    network: connection.meta.network.clone(),
                    inbound: connection.meta.inbound.clone(),
                    source_port: connection.meta.source_port.clone(),
                    destination_port: connection.meta.destination_port.clone(),
                    start: connection.meta.start.clone(),
                    rule: connection.meta.rule.clone(),
                    rule_payload: connection.meta.rule_payload.clone(),
                    chains: connection.chains.clone(),
                }
            })
            .collect()
    }

    pub fn apply_snapshot_and_project(
        &mut self,
        connections: Vec<ConnectionFact>,
        upload_total: u64,
        download_total: u64,
        monotonic_ms: u64,
        utc: i64,
    ) -> (AccountingBatch, Vec<crate::c2::hub::LiveConnectionView>) {
        let canonical = self.canonicalize(connections, monotonic_ms, utc);
        let live = self.project_live(&canonical);
        let batch = self.apply_snapshot(canonical, upload_total, download_total, monotonic_ms, utc);
        (batch, live)
    }

    pub fn apply(
        &mut self,
        input: ControllerInput,
        monotonic_ms: u64,
        utc: i64,
    ) -> AccountingBatch {
        match input {
            ControllerInput::Snapshot {
                upload_total,
                download_total,
                connections,
                ..
            } => {
                self.apply_snapshot_and_project(
                    connections,
                    upload_total,
                    download_total,
                    monotonic_ms,
                    utc,
                )
                .0
            }
            ControllerInput::Restarted { .. } => {
                self.reset_epoch(self.epoch.saturating_add(1));
                AccountingBatch {
                    facts: Vec::new(),
                    coverage: vec![CoverageChange {
                        kind: "epoch",
                        reason: "core_restart",
                    }],
                    attributed_upload: None,
                    attributed_download: None,
                    meter_upload: None,
                    meter_download: None,
                    gap_upload: None,
                    gap_download: None,
                    over_upload: None,
                    over_download: None,
                }
            }
            ControllerInput::Disconnected { .. } | ControllerInput::SleepGap { .. } => {
                AccountingBatch {
                    facts: Vec::new(),
                    coverage: vec![CoverageChange {
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
                }
            }
            ControllerInput::Shutdown | ControllerInput::Paused => AccountingBatch {
                facts: Vec::new(),
                coverage: vec![CoverageChange {
                    kind: "closed",
                    reason: "pause_or_shutdown",
                }],
                attributed_upload: None,
                attributed_download: None,
                meter_upload: None,
                meter_download: None,
                gap_upload: None,
                gap_download: None,
                over_upload: None,
                over_download: None,
            },
            _ => empty_known_zero(),
        }
    }

    fn canonicalize(
        &mut self,
        connections: Vec<ConnectionFact>,
        monotonic_ms: u64,
        utc: i64,
    ) -> Vec<ConnectionFact> {
        connections
            .into_iter()
            .map(|mut connection| {
                let key = format!("{}:{}", self.epoch, connection.id);
                let entry = self.sessions.entry(key).or_insert_with(|| SessionAcc {
                    connection_id: connection.id.clone(),
                    last_upload: connection.upload,
                    last_download: connection.download,
                    last_mono: monotonic_ms,
                    last_utc: utc,
                    seen: false,
                    meta: ConnectionMeta::default(),
                    chains: Vec::new(),
                    provider_chains: Vec::new(),
                });
                merge_meta(&mut entry.meta, &connection.meta);
                if !connection.chains.is_empty() {
                    entry.chains = connection.chains.clone();
                }
                if !connection.provider_chains.is_empty() {
                    entry.provider_chains = connection.provider_chains.clone();
                }
                connection.meta = entry.meta.clone();
                connection.chains = entry.chains.clone();
                connection.provider_chains = entry.provider_chains.clone();
                connection
            })
            .collect()
    }

    fn apply_snapshot(
        &mut self,
        connections: Vec<ConnectionFact>,
        upload_total: u64,
        download_total: u64,
        monotonic_ms: u64,
        utc: i64,
    ) -> AccountingBatch {
        let first_meter = self.last_meter_up.is_none();
        let mut attributed_up = 0_u64;
        let mut attributed_down = 0_u64;
        let mut facts = Vec::new();
        let utc_minute = utc.div_euclid(60);
        let seen: HashSet<String> = connections.iter().map(|item| item.id.clone()).collect();

        for connection in connections {
            let key = format!("{}:{}", self.epoch, connection.id);
            let (tags, primary) = classify(&self.targets, &connection.chains);
            let entry = self.sessions.entry(key.clone()).or_insert(SessionAcc {
                connection_id: connection.id.clone(),
                last_upload: connection.upload,
                last_download: connection.download,
                last_mono: monotonic_ms,
                last_utc: utc,
                seen: false,
                meta: connection.meta.clone(),
                chains: connection.chains.clone(),
                provider_chains: connection.provider_chains.clone(),
            });
            if !entry.seen {
                entry.seen = true;
                entry.last_upload = connection.upload;
                entry.last_download = connection.download;
                continue;
            }
            let delta_up = connection.upload.saturating_sub(entry.last_upload);
            let delta_down = connection.download.saturating_sub(entry.last_download);
            if connection.upload < entry.last_upload || connection.download < entry.last_download {
                entry.last_upload = connection.upload;
                entry.last_download = connection.download;
                continue;
            }
            entry.last_upload = connection.upload;
            entry.last_download = connection.download;
            entry.last_mono = monotonic_ms;
            entry.last_utc = utc;
            if delta_up == 0 && delta_down == 0 {
                continue;
            }
            attributed_up += delta_up;
            attributed_down += delta_down;
            facts.push(MinuteFact {
                session_key: key,
                utc_minute,
                upload: delta_up,
                download: delta_down,
                primary,
                tags,
            });
        }

        let retired: Vec<String> = self
            .sessions
            .values()
            .filter(|session| !seen.contains(&session.connection_id))
            .map(|session| session.connection_id.clone())
            .collect();
        self.retired_ids.extend(retired);
        self.sessions
            .retain(|_, session| seen.contains(&session.connection_id));

        let meter_up = if first_meter {
            None
        } else {
            Some(upload_total.saturating_sub(self.last_meter_up.unwrap_or(0)))
        };
        let meter_down = if first_meter {
            None
        } else {
            Some(download_total.saturating_sub(self.last_meter_down.unwrap_or(0)))
        };
        self.last_meter_up = Some(upload_total);
        self.last_meter_down = Some(download_total);

        let (gap_up, over_up) = diff_optional(meter_up, Some(attributed_up), first_meter);
        let (gap_down, over_down) = diff_optional(meter_down, Some(attributed_down), first_meter);
        AccountingBatch {
            facts,
            coverage: Vec::new(),
            attributed_upload: if first_meter {
                None
            } else {
                Some(attributed_up)
            },
            attributed_download: if first_meter {
                None
            } else {
                Some(attributed_down)
            },
            meter_upload: meter_up,
            meter_download: meter_down,
            gap_upload: gap_up,
            gap_download: gap_down,
            over_upload: over_up,
            over_download: over_down,
        }
    }
}

fn start_changed(stored: Option<&str>, incoming: Option<&str>) -> bool {
    matches!((stored, incoming), (Some(left), Some(right)) if left != right)
}

fn merge_optional(stored: &mut Option<String>, incoming: &Option<String>) {
    if let Some(value) = incoming
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        *stored = Some(value.to_string());
    }
}

fn merge_meta(stored: &mut ConnectionMeta, incoming: &ConnectionMeta) {
    merge_optional(&mut stored.host, &incoming.host);
    merge_optional(&mut stored.sniff_host, &incoming.sniff_host);
    merge_optional(&mut stored.source_ip, &incoming.source_ip);
    merge_optional(&mut stored.destination_ip, &incoming.destination_ip);
    merge_optional(&mut stored.source_port, &incoming.source_port);
    merge_optional(&mut stored.destination_port, &incoming.destination_port);
    merge_optional(&mut stored.process_name, &incoming.process_name);
    merge_optional(&mut stored.process_path, &incoming.process_path);
    merge_optional(&mut stored.network, &incoming.network);
    merge_optional(&mut stored.inbound, &incoming.inbound);
    merge_optional(&mut stored.start, &incoming.start);
    merge_optional(&mut stored.rule, &incoming.rule);
    merge_optional(&mut stored.rule_payload, &incoming.rule_payload);
}

pub fn process_basename(path: &str) -> Option<String> {
    let path = path.trim();
    if path.is_empty() || path.chars().count() > STRING_LIMIT || path.ends_with(['/', '\\']) {
        return None;
    }
    path.rsplit(['/', '\\'])
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

pub fn resolve_process_identity(process: Option<&str>, path: Option<&str>) -> Option<String> {
    process
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| path.and_then(process_basename))
}

fn classify(targets: &[String], chains: &[String]) -> (Vec<String>, Option<String>) {
    let tags = crate::residential::residential_tags(targets, chains);
    let primary = tags.first().cloned();
    (tags, primary)
}

fn diff_optional(
    meter: Option<u64>,
    attributed: Option<u64>,
    first: bool,
) -> (Option<u64>, Option<u64>) {
    if first {
        return (None, None);
    }
    match (meter, attributed) {
        (Some(meter), Some(attributed)) => (
            Some(meter.saturating_sub(attributed)),
            Some(attributed.saturating_sub(meter)),
        ),
        _ => (None, None),
    }
}

fn empty_known_zero() -> AccountingBatch {
    AccountingBatch {
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
    }
}

#[cfg(test)]
mod accounting_replay_tests {
    use super::*;
    use crate::controller::{ConnectionFact, ConnectionMeta};

    #[test]
    fn project_live_resolves_sniff_host_and_destination_ip() {
        let engine = AccountingEngine::new();
        let sniff = ConnectionFact {
            id: "s".into(),
            upload: 1,
            download: 1,
            chains: Vec::new(),
            provider_chains: Vec::new(),
            meta: ConnectionMeta {
                sniff_host: Some("sniff.example".into()),
                destination_ip: Some("1.1.1.1".into()),
                ..ConnectionMeta::default()
            },
        };
        let ip_only = ConnectionFact {
            id: "i".into(),
            upload: 1,
            download: 1,
            chains: Vec::new(),
            provider_chains: Vec::new(),
            meta: ConnectionMeta {
                destination_ip: Some("8.8.8.8".into()),
                ..ConnectionMeta::default()
            },
        };
        let rows = engine.project_live(&[sniff, ip_only]);
        assert_eq!(rows[0].host.as_deref(), Some("sniff.example"));
        assert_eq!(rows[1].host.as_deref(), Some("8.8.8.8"));
    }

    fn fact(id: &str, up: u64, down: u64, chains: &[&str]) -> ConnectionFact {
        ConnectionFact {
            id: id.to_string(),
            upload: up,
            download: down,
            chains: chains.iter().map(|item| (*item).to_string()).collect(),
            provider_chains: Vec::new(),
            meta: ConnectionMeta {
                host: Some(format!("{id}.test")),
                source_ip: None,
                destination_ip: None,
                process_name: None,
                process_path: None,
                network: Some("tcp".into()),
                rule: None,
                rule_payload: None,
                ..ConnectionMeta::default()
            },
        }
    }

    fn snap(up: u64, down: u64, connections: Vec<ConnectionFact>) -> ControllerInput {
        ControllerInput::Snapshot {
            received_monotonic_ms: 1,
            received_utc: 120,
            upload_total: up,
            download_total: down,
            connections,
        }
    }

    #[test]
    fn accounting_replay_first_frame_is_unknown_then_delta() {
        let mut engine = AccountingEngine::new();
        engine.set_targets(vec!["家宽".into(), "备用".into()]);
        let first = engine.apply(
            snap(100, 200, vec![fact("a", 10, 20, &["节点", "家宽"])]),
            10,
            120,
        );
        assert!(first.attributed_upload.is_none());
        assert!(first.facts.is_empty());
        let second = engine.apply(
            snap(130, 260, vec![fact("a", 15, 30, &["节点", "家宽"])]),
            20,
            120,
        );
        assert_eq!(second.attributed_upload, Some(5));
        assert_eq!(second.attributed_download, Some(10));
        assert_eq!(second.facts[0].primary.as_deref(), Some("家宽"));
        assert_eq!(second.facts[0].upload, 5);
    }

    #[test]
    fn accounting_replay_disappear_does_not_invent_tail() {
        let mut engine = AccountingEngine::new();
        engine.apply(snap(10, 10, vec![fact("a", 4, 4, &[])]), 1, 60);
        let gone = engine.apply(snap(10, 10, vec![]), 2, 60);
        assert!(gone.facts.is_empty());
        assert_eq!(gone.attributed_upload, Some(0));
    }

    #[test]
    fn canonical_metadata_enriches_and_does_not_downgrade_on_empty_frame() {
        let mut engine = AccountingEngine::new();
        engine.reset_epoch(7);
        let mut rich = fact("a", 10, 10, &["DIRECT"]);
        rich.meta.host = Some("explicit.test".into());
        rich.meta.sniff_host = Some("sniff.test".into());
        rich.meta.process_name = Some("browser.exe".into());
        rich.meta.process_path = Some("C:\\Apps\\browser.exe".into());
        rich.meta.rule = Some("IPCIDR".into());
        let (_, first) = engine.apply_snapshot_and_project(vec![rich], 10, 10, 1, 60);
        assert_eq!(first[0].host.as_deref(), Some("explicit.test"));
        assert_eq!(first[0].process_name.as_deref(), Some("browser.exe"));
        assert_eq!(first[0].chains, vec!["DIRECT"]);

        let empty = ConnectionFact {
            id: "a".into(),
            upload: 15,
            download: 12,
            chains: Vec::new(),
            provider_chains: Vec::new(),
            meta: ConnectionMeta::default(),
        };
        let (batch, second) = engine.apply_snapshot_and_project(vec![empty], 15, 12, 2, 61);
        assert_eq!(batch.facts[0].upload, 5);
        assert_eq!(second[0].host.as_deref(), Some("explicit.test"));
        assert_eq!(second[0].process_name.as_deref(), Some("browser.exe"));
        assert_eq!(second[0].chains, vec!["DIRECT"]);
        assert_eq!(second[0].rule.as_deref(), Some("IPCIDR"));
    }

    #[test]
    fn canonical_metadata_enriches_at_zero_delta_and_isolated_by_epoch() {
        let mut engine = AccountingEngine::new();
        engine.reset_epoch(7);
        let mut initial = fact("a", 10, 10, &[]);
        initial.meta.host = None;
        initial.meta.destination_ip = Some("203.0.113.7".into());
        initial.meta.process_path = Some("C:\\Apps\\fallback.exe".into());
        let (_, first) = engine.apply_snapshot_and_project(vec![initial], 10, 10, 1, 60);
        assert_eq!(first[0].host.as_deref(), Some("203.0.113.7"));
        assert_eq!(first[0].process_name.as_deref(), Some("fallback.exe"));

        let mut sniff = fact("a", 10, 10, &["DIRECT"]);
        sniff.meta.host = None;
        sniff.meta.sniff_host = Some("sniff.test".into());
        sniff.meta.process_name = Some("direct.exe".into());
        let (zero_delta, second) = engine.apply_snapshot_and_project(vec![sniff], 10, 10, 2, 61);
        assert!(zero_delta.facts.is_empty());
        assert_eq!(second[0].host.as_deref(), Some("sniff.test"));
        assert_eq!(second[0].process_name.as_deref(), Some("direct.exe"));
        assert_eq!(second[0].chains, vec!["DIRECT"]);

        let mut explicit = fact("a", 10, 10, &[]);
        explicit.meta.host = Some("explicit.test".into());
        let (_, third) = engine.apply_snapshot_and_project(vec![explicit], 10, 10, 3, 62);
        assert_eq!(third[0].host.as_deref(), Some("explicit.test"));
        assert_eq!(third[0].process_name.as_deref(), Some("direct.exe"));
        assert_eq!(third[0].chains, vec!["DIRECT"]);

        engine.reset_epoch(8);
        let mut new_generation = fact("a", 1, 1, &[]);
        new_generation.meta.host = None;
        let (_, isolated) = engine.apply_snapshot_and_project(vec![new_generation], 1, 1, 4, 63);
        assert_eq!(isolated[0].host, None);
        assert_eq!(isolated[0].process_name, None);
        assert!(isolated[0].chains.is_empty());
    }

    #[test]
    fn process_identity_uses_safe_cross_platform_basename() {
        assert_eq!(
            resolve_process_identity(None, Some("C:\\Program Files\\Browser\\browser.exe"))
                .as_deref(),
            Some("browser.exe")
        );
        assert_eq!(
            resolve_process_identity(None, Some("/usr/bin/curl")).as_deref(),
            Some("curl")
        );
        assert_eq!(resolve_process_identity(None, Some("C:\\Apps\\")), None);
        assert_eq!(resolve_process_identity(None, Some("/usr/bin/")), None);
        assert_eq!(
            resolve_process_identity(Some("direct.exe"), Some("C:\\path\\fallback.exe")).as_deref(),
            Some("direct.exe")
        );
        assert_eq!(
            resolve_process_identity(None, Some(&"x".repeat(STRING_LIMIT + 1))),
            None
        );
    }

    #[test]
    fn retired_id_and_counter_reset_require_new_generation() {
        let mut engine = AccountingEngine::new();
        engine.reset_epoch(1);
        engine.apply(snap(10, 10, vec![fact("a", 10, 10, &[])]), 1, 60);
        assert!(engine.snapshot_requires_new_generation(&[fact("a", 9, 10, &[])], 9, 10));
        engine.apply(snap(10, 10, vec![]), 2, 61);
        assert!(engine.snapshot_requires_new_generation(&[fact("a", 1, 1, &[])], 11, 11));
        engine.reset_epoch(2);
        assert!(!engine.snapshot_requires_new_generation(&[fact("a", 1, 1, &[])], 11, 11));
    }
}

#[cfg(test)]
mod accounting_properties_tests {
    use super::*;
    use crate::controller::{ConnectionFact, ConnectionMeta, ControllerInput};

    #[test]
    fn accounting_properties_categories_sum_to_attributed() {
        let mut engine = AccountingEngine::new();
        engine.set_targets(vec!["t1".into()]);
        let mk = |id: &str, up: u64, chains: Vec<&str>| ConnectionFact {
            id: id.into(),
            upload: up,
            download: 0,
            chains: chains.into_iter().map(str::to_string).collect(),
            provider_chains: Vec::new(),
            meta: ConnectionMeta {
                host: None,
                source_ip: None,
                destination_ip: None,
                process_name: None,
                process_path: None,
                network: None,
                rule: None,
                rule_payload: None,
                ..ConnectionMeta::default()
            },
        };
        engine.apply(
            ControllerInput::Snapshot {
                received_monotonic_ms: 1,
                received_utc: 0,
                upload_total: 3,
                download_total: 0,
                connections: vec![mk("a", 1, vec!["t1"]), mk("b", 2, vec!["other"])],
            },
            1,
            0,
        );
        let batch = engine.apply(
            ControllerInput::Snapshot {
                received_monotonic_ms: 2,
                received_utc: 0,
                upload_total: 8,
                download_total: 0,
                connections: vec![mk("a", 4, vec!["t1"]), mk("b", 4, vec!["other"])],
            },
            2,
            0,
        );
        let primary: u64 = batch
            .facts
            .iter()
            .filter(|fact| fact.primary.as_deref() == Some("t1"))
            .map(|fact| fact.upload)
            .sum();
        let other: u64 = batch
            .facts
            .iter()
            .filter(|fact| fact.primary.is_none())
            .map(|fact| fact.upload)
            .sum();
        assert_eq!(Some(primary + other), batch.attributed_upload);
        assert_eq!(batch.attributed_upload, Some(5));
    }
}

#[cfg(test)]
mod accounting_coverage_tests {
    use super::*;

    #[test]
    fn accounting_coverage_disconnect_is_gap_not_zero_fact() {
        let mut engine = AccountingEngine::new();
        let batch = engine.apply(
            ControllerInput::Disconnected {
                reason: crate::controller::SessionStatus::EndpointMissing,
            },
            1,
            1,
        );
        assert!(batch.facts.is_empty());
        assert_eq!(batch.coverage[0].kind, "gap");
        assert!(batch.attributed_upload.is_none());
    }
}

#[cfg(test)]
mod accounting_policy_tests {
    use super::*;
    use crate::controller::{ConnectionFact, ConnectionMeta, ControllerInput};

    #[test]
    fn accounting_policy_ignores_chain_order() {
        let mut left = AccountingEngine::new();
        let mut right = AccountingEngine::new();
        left.set_targets(vec!["家宽".into(), "备用".into()]);
        right.set_targets(vec!["家宽".into(), "备用".into()]);
        let mk = |up: u64, chains: Vec<&str>| ConnectionFact {
            id: "x".into(),
            upload: up,
            download: 0,
            chains: chains.into_iter().map(str::to_string).collect(),
            provider_chains: Vec::new(),
            meta: ConnectionMeta {
                host: None,
                source_ip: None,
                destination_ip: None,
                process_name: None,
                process_path: None,
                network: None,
                rule: None,
                rule_payload: None,
                ..ConnectionMeta::default()
            },
        };
        for engine in [&mut left, &mut right] {
            engine.apply(
                ControllerInput::Snapshot {
                    received_monotonic_ms: 1,
                    received_utc: 0,
                    upload_total: 9,
                    download_total: 0,
                    connections: vec![mk(9, vec!["家宽", "备用"])],
                },
                1,
                0,
            );
        }
        let a = left.apply(
            ControllerInput::Snapshot {
                received_monotonic_ms: 2,
                received_utc: 0,
                upload_total: 19,
                download_total: 0,
                connections: vec![mk(19, vec!["备用", "家宽"])],
            },
            2,
            0,
        );
        let b = right.apply(
            ControllerInput::Snapshot {
                received_monotonic_ms: 2,
                received_utc: 0,
                upload_total: 19,
                download_total: 0,
                connections: vec![mk(19, vec!["家宽", "备用"])],
            },
            2,
            0,
        );
        assert_eq!(a.facts[0].primary, b.facts[0].primary);
        assert_eq!(a.facts[0].primary.as_deref(), Some("家宽"));
    }
}
