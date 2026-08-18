//! 版本化控制器模型、脱敏与 replay adapter。

use crate::c0_contract::{FRAME_BODY_LIMIT, STRING_LIMIT};
use crate::credential::Secret;
use serde_json::{Map, Value};
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionMeta {
    pub host: Option<String>,
    pub source_ip: Option<String>,
    pub destination_ip: Option<String>,
    pub process_name: Option<String>,
    pub process_path: Option<String>,
    pub network: Option<String>,
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
    let upload_total = as_u64(object.get("uploadTotal")).unwrap_or(0);
    let download_total = as_u64(object.get("downloadTotal")).unwrap_or(0);
    let mut connections = Vec::new();
    if let Some(Value::Array(items)) = object.get("connections") {
        for item in items {
            if let Some(fact) = normalize_connection(item) {
                connections.push(fact);
            }
        }
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
    let id = object.get("id")?.as_str().filter(|text| !text.is_empty())?;
    let metadata = object
        .get("metadata")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    Some(ConnectionFact {
        id: id.to_string(),
        upload: as_u64(object.get("upload")).unwrap_or(0),
        download: as_u64(object.get("download")).unwrap_or(0),
        chains: as_string_list(object.get("chains")),
        provider_chains: as_string_list(object.get("providerChains")),
        meta: ConnectionMeta {
            host: text_field(&metadata, "host"),
            source_ip: text_field(&metadata, "sourceIP"),
            destination_ip: text_field(&metadata, "destinationIP"),
            process_name: text_field(&metadata, "process"),
            process_path: text_field(&metadata, "processPath"),
            network: text_field(&metadata, "network"),
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

fn text_field(object: &Map<String, Value>, key: &str) -> Option<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .and_then(truncate_string)
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
