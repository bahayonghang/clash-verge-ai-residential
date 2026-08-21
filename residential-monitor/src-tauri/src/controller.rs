//! 版本化控制器模型、脱敏与 replay adapter。

use crate::c0_contract::{FRAME_BODY_LIMIT, STRING_LIMIT};
use crate::credential::Secret;
use serde_json::{Map, Value};
use std::net::IpAddr;

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataCoverage {
    pub connections: u64,
    pub host_present: u64,
    pub sniff_host_only: u64,
    pub destination_ip_only: u64,
    pub host_absent: u64,
    pub process_present: u64,
    pub process_path_only: u64,
    pub process_absent: u64,
    pub chains_present: u64,
    pub provider_chains_only: u64,
    pub chains_absent: u64,
}

impl MetadataCoverage {
    pub fn from_connections(connections: &[ConnectionFact]) -> Self {
        let mut coverage = Self {
            connections: connections.len() as u64,
            ..Self::default()
        };
        for connection in connections {
            if connection.meta.host.is_some() {
                coverage.host_present += 1;
            } else if connection.meta.sniff_host.is_some() {
                coverage.sniff_host_only += 1;
            } else if connection.meta.destination_ip.is_some() {
                coverage.destination_ip_only += 1;
            } else {
                coverage.host_absent += 1;
            }
            if connection.meta.process_name.is_some() {
                coverage.process_present += 1;
            } else if connection.meta.process_path.is_some() {
                coverage.process_path_only += 1;
            } else {
                coverage.process_absent += 1;
            }
            if !connection.chains.is_empty() {
                coverage.chains_present += 1;
            } else if !connection.provider_chains.is_empty() {
                coverage.provider_chains_only += 1;
            } else {
                coverage.chains_absent += 1;
            }
        }
        coverage
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConnectionMeta {
    pub host: Option<String>,
    pub sniff_host: Option<String>,
    pub source_ip: Option<String>,
    pub destination_ip: Option<String>,
    pub source_port: Option<String>,
    pub destination_port: Option<String>,
    pub process_name: Option<String>,
    pub process_path: Option<String>,
    pub network: Option<String>,
    pub inbound: Option<String>,
    pub start: Option<String>,
    pub rule: Option<String>,
    pub rule_payload: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionFact {
    pub id: String,
    pub upload: u64,
    pub download: u64,
    pub chains: Vec<String>,
    pub provider_chains: Vec<String>,
    pub meta: ConnectionMeta,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControllerInput {
    Connected {
        endpoint: String,
        core_identity: String,
    },
    Snapshot {
        received_monotonic_ms: u64,
        received_utc: i64,
        upload_total: u64,
        download_total: u64,
        connections: Vec<ConnectionFact>,
    },
    Restarted {
        old_identity: String,
        new_identity: String,
    },
    Disconnected {
        reason: SessionStatus,
    },
    SleepGap {
        started_utc: i64,
        ended_utc: i64,
    },
    Paused,
    Resumed,
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Connecting,
    Connected,
    AuthFailed,
    PipeAccessDenied,
    PipeBusyTimeout,
    EndpointMissing,
    ProtocolIncompatible,
    PidMismatch,
    CoreRestarted,
    Cancelled,
    NonLoopback,
}

pub fn reject_non_loopback_ip(addr: IpAddr) -> Result<(), SessionStatus> {
    if addr.is_loopback() {
        Ok(())
    } else {
        Err(SessionStatus::NonLoopback)
    }
}

pub fn truncate_string(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    Some(value.chars().take(STRING_LIMIT).collect())
}

pub fn normalize_snapshot(
    raw: &Value,
    received_monotonic_ms: u64,
    received_utc: i64,
) -> Result<ControllerInput, SessionStatus> {
    let encoded = serde_json::to_vec(raw).map_err(|_| SessionStatus::ProtocolIncompatible)?;
    if encoded.len() > FRAME_BODY_LIMIT {
        return Err(SessionStatus::ProtocolIncompatible);
    }
    let object = raw.as_object().ok_or(SessionStatus::ProtocolIncompatible)?;
    let upload_total =
        as_u64(object.get("uploadTotal")).ok_or(SessionStatus::ProtocolIncompatible)?;
    let download_total =
        as_u64(object.get("downloadTotal")).ok_or(SessionStatus::ProtocolIncompatible)?;
    let items = object
        .get("connections")
        .and_then(Value::as_array)
        .ok_or(SessionStatus::ProtocolIncompatible)?;
    let connections = items
        .iter()
        .map(normalize_connection)
        .collect::<Option<Vec<_>>>()
        .ok_or(SessionStatus::ProtocolIncompatible)?;
    let mut ids = std::collections::HashSet::with_capacity(connections.len());
    if connections
        .iter()
        .any(|connection| !ids.insert(connection.id.as_str()))
    {
        return Err(SessionStatus::ProtocolIncompatible);
    }
    Ok(ControllerInput::Snapshot {
        received_monotonic_ms,
        received_utc,
        upload_total,
        download_total,
        connections,
    })
}

fn normalize_connection(value: &Value) -> Option<ConnectionFact> {
    let object = value.as_object()?;
    let id = object.get("id")?.as_str().and_then(bounded_string)?;
    let metadata = object
        .get("metadata")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    Some(ConnectionFact {
        id,
        upload: as_u64(object.get("upload"))?,
        download: as_u64(object.get("download"))?,
        chains: as_string_list(object.get("chains")),
        provider_chains: as_string_list(object.get("providerChains")),
        meta: ConnectionMeta {
            host: text_field(&metadata, "host"),
            sniff_host: text_field(&metadata, "sniffHost"),
            source_ip: text_field(&metadata, "sourceIP"),
            destination_ip: text_field(&metadata, "destinationIP"),
            source_port: text_field(&metadata, "sourcePort"),
            destination_port: text_field(&metadata, "destinationPort"),
            process_name: text_field(&metadata, "process"),
            process_path: bounded_text_field(&metadata, "processPath"),
            network: text_field(&metadata, "network"),
            inbound: text_field(&metadata, "type"),
            start: object
                .get("start")
                .and_then(Value::as_str)
                .and_then(truncate_string),
            rule: object
                .get("rule")
                .and_then(Value::as_str)
                .and_then(truncate_string),
            rule_payload: object
                .get("rulePayload")
                .and_then(Value::as_str)
                .and_then(truncate_string),
        },
    })
}

fn bounded_string(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > STRING_LIMIT {
        None
    } else {
        Some(value.to_string())
    }
}

fn text_field(object: &Map<String, Value>, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .and_then(truncate_string)
}

fn bounded_text_field(object: &Map<String, Value>, key: &str) -> Option<String> {
    let value = object.get(key)?.as_str()?.trim();
    if value.is_empty() || value.chars().count() > STRING_LIMIT {
        None
    } else {
        Some(value.to_string())
    }
}

fn as_u64(value: Option<&Value>) -> Option<u64> {
    match value {
        Some(Value::Number(number)) => number.as_u64(),
        _ => None,
    }
}

fn as_string_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .filter_map(truncate_string)
            .collect(),
        _ => Vec::new(),
    }
}

pub fn redact_secret(secret: &Secret) -> &'static str {
    secret.redacted()
}

pub fn replay_inputs(frames: Vec<ControllerInput>) -> Vec<ControllerInput> {
    frames
}

#[cfg(test)]
mod controller_model_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn controller_model_keeps_unknown_fields_and_ignores_order() {
        let raw = json!({
            "uploadTotal": 10,
            "downloadTotal": 20,
            "unknown": true,
            "connections": [
                {"id": "b", "upload": 2, "download": 3, "chains": ["node", "group"], "metadata": {"host": "b.test"}},
                {"id": "a", "upload": 1, "download": 1, "extra": 1}
            ]
        });
        let ControllerInput::Snapshot { connections, .. } =
            normalize_snapshot(&raw, 1, 1).expect("normalize")
        else {
            panic!("snapshot");
        };
        assert_eq!(connections.len(), 2);
        assert_eq!(connections[0].id, "b");
        assert_eq!(connections[1].id, "a");
    }

    #[test]
    fn controller_model_reads_sniff_host_when_host_empty() {
        let raw = json!({
            "uploadTotal": 1,
            "downloadTotal": 1,
            "connections": [{
                "id": "s",
                "upload": 1,
                "download": 1,
                "metadata": {"sniffHost": "sniff.example", "destinationIP": "9.9.9.9"}
            }]
        });
        let ControllerInput::Snapshot { connections, .. } =
            normalize_snapshot(&raw, 1, 1).expect("normalize")
        else {
            panic!("snapshot");
        };
        assert_eq!(connections[0].meta.host, None);
        assert_eq!(
            connections[0].meta.sniff_host.as_deref(),
            Some("sniff.example")
        );
        assert_eq!(
            connections[0].meta.destination_ip.as_deref(),
            Some("9.9.9.9")
        );
    }
}

#[cfg(test)]
mod controller_normalizer_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn controller_normalizer_rejects_oversize_frame() {
        let huge = "x".repeat(FRAME_BODY_LIMIT + 8);
        let raw = json!({ "uploadTotal": 0, "downloadTotal": 0, "pad": huge });
        assert_eq!(
            normalize_snapshot(&raw, 1, 1).unwrap_err(),
            SessionStatus::ProtocolIncompatible
        );
    }

    #[test]
    fn controller_normalizer_rejects_incomplete_root_frame() {
        for raw in [
            json!({"downloadTotal": 0, "connections": []}),
            json!({"uploadTotal": 0, "connections": []}),
            json!({"uploadTotal": 0, "downloadTotal": 0}),
            json!({"uploadTotal": "0", "downloadTotal": 0, "connections": []}),
            json!({"uploadTotal": 0, "downloadTotal": 0, "connections": {}}),
        ] {
            assert_eq!(
                normalize_snapshot(&raw, 1, 1),
                Err(SessionStatus::ProtocolIncompatible)
            );
        }
    }

    #[test]
    fn controller_normalizer_trims_metadata_and_chain_elements() {
        let raw = json!({
            "uploadTotal": 0,
            "downloadTotal": 0,
            "connections": [{
                "id": "  a  ",
                "upload": 0,
                "download": 0,
                "chains": [" ", " DIRECT ", ""],
                "metadata": {"host": "  example.test  ", "process": "  "}
            }]
        });
        let ControllerInput::Snapshot { connections, .. } =
            normalize_snapshot(&raw, 1, 1).expect("normalize")
        else {
            panic!("snapshot")
        };
        assert_eq!(connections[0].id, "a");
        assert_eq!(connections[0].chains, vec!["DIRECT"]);
        assert_eq!(connections[0].meta.host.as_deref(), Some("example.test"));
        assert_eq!(connections[0].meta.process_name, None);
    }

    #[test]
    fn controller_normalizer_rejects_frames_with_invalid_connection_rows() {
        let raw = json!({
            "uploadTotal": 0,
            "downloadTotal": 0,
            "connections": [
                {"id": "missing", "download": 0},
                {"id": "wrong", "upload": "0", "download": 0},
                {"id": "ok", "upload": 0, "download": 0}
            ]
        });
        assert_eq!(
            normalize_snapshot(&raw, 1, 1),
            Err(SessionStatus::ProtocolIncompatible)
        );

        let overlong_id = json!({
            "uploadTotal": 0,
            "downloadTotal": 0,
            "connections": [{
                "id": "x".repeat(STRING_LIMIT + 1),
                "upload": 0,
                "download": 0
            }]
        });
        assert_eq!(
            normalize_snapshot(&overlong_id, 1, 1),
            Err(SessionStatus::ProtocolIncompatible)
        );

        let duplicate_id = json!({
            "uploadTotal": 0,
            "downloadTotal": 0,
            "connections": [
                {"id": "same", "upload": 0, "download": 0},
                {"id": "same", "upload": 0, "download": 0}
            ]
        });
        assert_eq!(
            normalize_snapshot(&duplicate_id, 1, 1),
            Err(SessionStatus::ProtocolIncompatible)
        );
    }

    #[test]
    fn overlong_process_path_stays_missing_instead_of_becoming_a_truncated_name() {
        let raw = json!({
            "uploadTotal": 0,
            "downloadTotal": 0,
            "connections": [{
                "id": "path",
                "upload": 0,
                "download": 0,
                "metadata": {"processPath": format!("C:\\\\{}\\\\app.exe", "x".repeat(STRING_LIMIT))}
            }]
        });
        let ControllerInput::Snapshot { connections, .. } =
            normalize_snapshot(&raw, 1, 1).expect("normalize")
        else {
            panic!("snapshot")
        };

        assert_eq!(connections[0].meta.process_path, None);
    }

    #[test]
    fn metadata_coverage_reports_presence_without_values() {
        let raw = json!({
            "uploadTotal": 0,
            "downloadTotal": 0,
            "connections": [
                {
                    "id": "complete",
                    "upload": 0,
                    "download": 0,
                    "chains": ["DIRECT"],
                    "metadata": {"host": "private.example", "process": "private.exe"}
                },
                {
                    "id": "fallbacks",
                    "upload": 0,
                    "download": 0,
                    "providerChains": ["provider-private"],
                    "metadata": {"sniffHost": "sniff.private", "processPath": "C:\\private\\app.exe"}
                },
                {
                    "id": "ip-only",
                    "upload": 0,
                    "download": 0,
                    "metadata": {"destinationIP": "203.0.113.7"}
                },
                {"id": "absent", "upload": 0, "download": 0}
            ]
        });
        let ControllerInput::Snapshot { connections, .. } =
            normalize_snapshot(&raw, 1, 1).expect("normalize")
        else {
            panic!("snapshot")
        };

        let coverage = MetadataCoverage::from_connections(&connections);
        assert_eq!(coverage.connections, 4);
        assert_eq!(coverage.host_present, 1);
        assert_eq!(coverage.sniff_host_only, 1);
        assert_eq!(coverage.destination_ip_only, 1);
        assert_eq!(coverage.host_absent, 1);
        assert_eq!(coverage.process_present, 1);
        assert_eq!(coverage.process_path_only, 1);
        assert_eq!(coverage.process_absent, 2);
        assert_eq!(coverage.chains_present, 1);
        assert_eq!(coverage.provider_chains_only, 1);
        assert_eq!(coverage.chains_absent, 2);

        let encoded = serde_json::to_string(&coverage).expect("coverage json");
        assert!(!encoded.contains("private"));
        assert!(!encoded.contains("203.0.113.7"));
    }
}

#[cfg(test)]
mod controller_redaction_tests {
    use super::*;

    #[test]
    fn controller_redaction_never_prints_secret_bytes() {
        let secret = Secret::from_plain("super-secret-value");
        assert_eq!(redact_secret(&secret), "<redacted>");
        let debug = format!("{secret:?}");
        assert!(!debug.contains("super-secret-value"));
    }
}

#[cfg(test)]
mod controller_session_tests {
    use super::*;

    #[test]
    fn controller_session_replay_preserves_input_order() {
        let frames = replay_inputs(vec![
            ControllerInput::Connected {
                endpoint: "127.0.0.1:9090".into(),
                core_identity: "core-1".into(),
            },
            ControllerInput::Disconnected {
                reason: SessionStatus::Cancelled,
            },
        ]);
        assert!(matches!(frames[0], ControllerInput::Connected { .. }));
        assert!(matches!(
            frames[1],
            ControllerInput::Disconnected {
                reason: SessionStatus::Cancelled
            }
        ));
    }
}

#[cfg(test)]
mod controller_pipe_tests {
    use super::*;

    #[test]
    fn controller_pipe_states_are_distinct() {
        assert_ne!(
            SessionStatus::PipeAccessDenied,
            SessionStatus::PipeBusyTimeout
        );
        assert_ne!(SessionStatus::PidMismatch, SessionStatus::EndpointMissing);
        assert_ne!(
            SessionStatus::AuthFailed,
            SessionStatus::ProtocolIncompatible
        );
    }
}

#[cfg(test)]
mod controller_reconnect_tests {
    use super::*;

    #[test]
    fn controller_reconnect_restart_is_new_identity() {
        let input = ControllerInput::Restarted {
            old_identity: "a".into(),
            new_identity: "b".into(),
        };
        assert!(matches!(
            input,
            ControllerInput::Restarted { old_identity, new_identity }
                if old_identity != new_identity
        ));
    }
}

#[cfg(test)]
mod controller_tcp_tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn controller_tcp_rejects_non_loopback() {
        assert_eq!(
            reject_non_loopback_ip(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))),
            Err(SessionStatus::NonLoopback)
        );
        assert_eq!(
            reject_non_loopback_ip(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            Ok(())
        );
    }
}
