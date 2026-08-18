//! ControllerSession：TCP loopback、pipe 错误分类与重连。

use crate::controller::{
    normalize_snapshot, reject_non_loopback_ip, ControllerInput, SessionStatus,
};
use crate::credential::{CredentialStore, FakeCredentialStore, Secret};
use crate::transport::{fetch_connections, fetch_version, map_os_error, TransportErrorKind};
use std::net::SocketAddr;
use std::time::Instant;

pub struct ControllerSession {
    pub endpoint: String,
    pub core_identity: Option<String>,
    store: FakeCredentialStore,
}

impl ControllerSession {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            core_identity: None,
            store: FakeCredentialStore::new(),
        }
    }

    pub fn pipe_authorization_header() -> Option<String> {
        None
    }

    pub fn remember_secret(&self, target: &str, secret: &Secret) -> Result<(), SessionStatus> {
        self.store
            .put(target, secret)
            .map_err(|_| SessionStatus::AuthFailed)
    }

    pub fn resolve_secret(&self, target: &str) -> Result<Secret, SessionStatus> {
        self.store
            .get(target)
            .map_err(|_| SessionStatus::AuthFailed)
    }

    pub fn map_transport(kind: TransportErrorKind) -> SessionStatus {
        match kind {
            TransportErrorKind::AuthFailed => SessionStatus::AuthFailed,
            TransportErrorKind::PipeAccessDenied => SessionStatus::PipeAccessDenied,
            TransportErrorKind::PipeBusyTimeout => SessionStatus::PipeBusyTimeout,
            TransportErrorKind::EndpointMissing => SessionStatus::EndpointMissing,
            TransportErrorKind::ProtocolIncompatible => SessionStatus::ProtocolIncompatible,
            TransportErrorKind::PidMismatch => SessionStatus::PidMismatch,
            TransportErrorKind::Cancelled => SessionStatus::Cancelled,
            TransportErrorKind::NonLoopback => SessionStatus::NonLoopback,
        }
    }

    pub fn classify_pipe_os_error(code: i32) -> SessionStatus {
        Self::map_transport(map_os_error(code))
    }

    pub fn detect_restart(&self, new_identity: &str) -> Option<ControllerInput> {
        match &self.core_identity {
            Some(old) if old != new_identity => Some(ControllerInput::Restarted {
                old_identity: old.clone(),
                new_identity: new_identity.to_string(),
            }),
            _ => None,
        }
    }

    pub async fn connect_tcp(
        &mut self,
        addr: SocketAddr,
        secret: Option<&str>,
    ) -> Result<Vec<ControllerInput>, SessionStatus> {
        reject_non_loopback_ip(addr.ip())?;
        let (status, version_body) = fetch_version(addr, secret)
            .await
            .map_err(|_| SessionStatus::EndpointMissing)?;
        if status.as_u16() == 401 {
            return Err(SessionStatus::AuthFailed);
        }
        if !status.is_success() {
            return Err(SessionStatus::ProtocolIncompatible);
        }
        let identity = version_body;
        let mut inputs = Vec::new();
        if let Some(restarted) = self.detect_restart(&identity) {
            inputs.push(restarted);
        }
        self.core_identity = Some(identity.clone());
        inputs.push(ControllerInput::Connected {
            endpoint: addr.to_string(),
            core_identity: identity,
        });
        let (conn_status, body) = fetch_connections(addr, secret)
            .await
            .map_err(|_| SessionStatus::EndpointMissing)?;
        if !conn_status.is_success() {
            return Err(SessionStatus::ProtocolIncompatible);
        }
        let raw = serde_json::from_str(&body).map_err(|_| SessionStatus::ProtocolIncompatible)?;
        let now = Instant::now();
        inputs.push(normalize_snapshot(
            &raw,
            now.elapsed().as_millis() as u64,
            chrono::Utc::now().timestamp(),
        )?);
        Ok(inputs)
    }

    pub fn probe_missing_pipe() -> SessionStatus {
        Self::classify_pipe_os_error(2)
    }
}

#[cfg(test)]
mod controller_session_tests {
    use super::*;
    use crate::identity::CREDENTIAL_SPIKE_TARGET;
    use crate::transport::spawn_fixture_server;

    #[tokio::test]
    async fn controller_session_tcp_secret_states() {
        let (addr, stop) = spawn_fixture_server(Some("fixture-secret")).await;
        let mut session = ControllerSession::new(addr.to_string());
        session
            .remember_secret(
                CREDENTIAL_SPIKE_TARGET,
                &Secret::from_plain("fixture-secret"),
            )
            .expect("store");
        let resolved = session
            .resolve_secret(CREDENTIAL_SPIKE_TARGET)
            .expect("resolve");
        let ok = session
            .connect_tcp(
                addr,
                Some(std::str::from_utf8(resolved.as_header_bytes()).unwrap()),
            )
            .await
            .expect("ok");
        assert!(matches!(ok[0], ControllerInput::Connected { .. }));
        assert!(matches!(ok[1], ControllerInput::Snapshot { .. }));
        assert_eq!(
            session.connect_tcp(addr, Some("wrong")).await.unwrap_err(),
            SessionStatus::AuthFailed
        );
        let _ = stop.send(());
    }
}

#[cfg(test)]
mod controller_pipe_tests {
    use super::*;

    #[test]
    fn controller_pipe_does_not_send_secret() {
        assert!(ControllerSession::pipe_authorization_header().is_none());
        assert_eq!(
            ControllerSession::classify_pipe_os_error(5),
            SessionStatus::PipeAccessDenied
        );
        assert_eq!(
            ControllerSession::classify_pipe_os_error(231),
            SessionStatus::PipeBusyTimeout
        );
        assert_eq!(
            ControllerSession::probe_missing_pipe(),
            SessionStatus::EndpointMissing
        );
    }
}

#[cfg(test)]
mod controller_reconnect_tests {
    use super::*;

    #[test]
    fn controller_reconnect_emits_restart_on_identity_change() {
        let mut session = ControllerSession::new("127.0.0.1:9");
        session.core_identity = Some("core-a".into());
        let restarted = session.detect_restart("core-b").expect("restart");
        assert!(matches!(
            restarted,
            ControllerInput::Restarted { old_identity, new_identity }
                if old_identity == "core-a" && new_identity == "core-b"
        ));
    }
}
