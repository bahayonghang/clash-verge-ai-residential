//! 无 I/O 的核算状态机。

use crate::controller::{ConnectionFact, ControllerInput};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinuteFact {
    pub session_key: String,
    pub utc_minute: i64,
    pub upload: u64,
    pub download: u64,
    pub primary: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    last_upload: u64,
    last_download: u64,
    last_mono: u64,
    last_utc: i64,
    seen: bool,
}

#[derive(Debug, Default)]
pub struct AccountingEngine {
    epoch: u64,
    sessions: HashMap<String, SessionAcc>,
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
                    process_name: connection.meta.process_name.clone(),
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
            } => self.apply_snapshot(connections, upload_total, download_total, monotonic_ms, utc),
            ControllerInput::Restarted { .. } => {
                self.epoch += 1;
                self.sessions.clear();
                self.last_meter_up = None;
                self.last_meter_down = None;
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
        let seen: std::collections::HashSet<String> =
            connections.iter().map(|item| item.id.clone()).collect();

        for connection in connections {
            let key = format!("{}:{}", self.epoch, connection.id);
            let (tags, primary) = classify(&self.targets, &connection.chains);
            let entry = self.sessions.entry(key.clone()).or_insert(SessionAcc {
                last_upload: connection.upload,
                last_download: connection.download,
                last_mono: monotonic_ms,
                last_utc: utc,
                seen: false,
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

        self.sessions
            .retain(|key, _| seen.contains(key.rsplit_once(':').map(|(_, id)| id).unwrap_or(key)));

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
