//! 凭据补偿与首次引导。测试只使用 Fake / 进程内 secret。

use crate::c2::contract::{SETTING_VALUE_MAX, TARGET_COUNT_MAX, TARGET_NAME_MAX};
use crate::c2::desktop::{AutostartError, AutostartPort};
use crate::credential::{CredentialError, CredentialStore, ProcessLocalStore, Secret};
use crate::identity::CREDENTIAL_TARGET;
use crate::transport::reject_non_loopback;
use serde::{Deserialize, Serialize};
use std::net::ToSocketAddrs;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControllerSettings {
    pub transport: String,
    pub address: String,
    pub credential_target: String,
    pub has_secret: bool,
    pub secret_mode: String,
}

impl Default for ControllerSettings {
    fn default() -> Self {
        Self {
            transport: "tcp".into(),
            address: String::new(),
            credential_target: CREDENTIAL_TARGET.into(),
            has_secret: false,
            secret_mode: "none".into(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SettingsError {
    InvalidAddress,
    NonLoopback,
    FieldTooLong,
    TooManyTargets,
    EmptyTarget,
    Credential(CredentialError),
    ProbeFailed,
    Unavailable,
}

impl SettingsError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidAddress => "invalid_address",
            Self::NonLoopback => "non_loopback",
            Self::FieldTooLong => "field_too_long",
            Self::TooManyTargets => "too_many_targets",
            Self::EmptyTarget => "empty_target",
            Self::Credential(CredentialError::Unavailable) => "credential_unavailable",
            Self::Credential(CredentialError::NotFound) => "credential_missing",
            Self::Credential(_) => "credential_invalid",
            Self::ProbeFailed => "probe_failed",
            Self::Unavailable => "settings_unavailable",
        }
    }

    pub fn message_key(&self) -> &'static str {
        match self {
            Self::InvalidAddress => "settings.invalid_address",
            Self::NonLoopback => "settings.non_loopback",
            Self::FieldTooLong => "settings.field_too_long",
            Self::TooManyTargets => "settings.too_many_targets",
            Self::EmptyTarget => "settings.empty_target",
            Self::Credential(CredentialError::Unavailable) => "settings.credential_unavailable",
            Self::Credential(_) => "settings.credential_failed",
            Self::ProbeFailed => "settings.probe_failed",
            Self::Unavailable => "settings.unavailable",
        }
    }

    pub fn message(&self, locale: crate::i18n::UiLocale) -> &'static str {
        crate::i18n::t(locale, self.message_key())
    }

    pub fn message_zh(&self) -> &'static str {
        self.message(crate::i18n::UiLocale::Zh)
    }
}

pub fn validate_address(address: &str) -> Result<(), SettingsError> {
    if address.len() > SETTING_VALUE_MAX {
        return Err(SettingsError::FieldTooLong);
    }
    let Some((host, _port)) = address.rsplit_once(':') else {
        return Err(SettingsError::InvalidAddress);
    };
    let host = host.trim_matches(|c| c == '[' || c == ']');
    reject_non_loopback(host).map_err(|_| SettingsError::NonLoopback)?;
    address
        .to_socket_addrs()
        .map_err(|_| SettingsError::InvalidAddress)?
        .next()
        .ok_or(SettingsError::InvalidAddress)?;
    Ok(())
}

pub fn validate_targets(targets: &[String]) -> Result<(), SettingsError> {
    if targets.len() > TARGET_COUNT_MAX {
        return Err(SettingsError::TooManyTargets);
    }
    for name in targets {
        if name.is_empty() {
            return Err(SettingsError::EmptyTarget);
        }
        if name.len() > TARGET_NAME_MAX {
            return Err(SettingsError::FieldTooLong);
        }
    }
    Ok(())
}

pub struct SettingsWorkflow {
    store: Box<dyn CredentialStore>,
    session: ProcessLocalStore,
    persistent_available: bool,
}

impl SettingsWorkflow {
    pub fn new(store: impl CredentialStore + 'static, persistent_available: bool) -> Self {
        Self {
            store: Box::new(store),
            session: ProcessLocalStore::new(),
            persistent_available,
        }
    }

    pub fn persistent_available(&self) -> bool {
        self.persistent_available
    }

    pub fn clear_session(&self) {
        self.session.clear();
    }

    pub fn delete_stored_target(&self, target: &str) -> Result<(), SettingsError> {
        self.session.clear();
        match self.store.delete(target) {
            Ok(()) | Err(CredentialError::NotFound) => Ok(()),
            Err(error) => Err(SettingsError::Credential(error)),
        }
    }

    pub fn resolve(&self, target: &str, mode: &str) -> Result<Secret, SettingsError> {
        if mode == "session" {
            self.session.get(target).map_err(SettingsError::Credential)
        } else {
            self.store.get(target).map_err(SettingsError::Credential)
        }
    }

    pub fn save_secret(
        &self,
        previous: &ControllerSettings,
        address: &str,
        secret: Option<&str>,
        prefer_session: bool,
        probe_ok: bool,
    ) -> Result<ControllerSettings, SettingsError> {
        validate_address(address)?;
        let mut next = previous.clone();
        next.address = address.to_string();
        next.transport = "tcp".into();
        next.credential_target = CREDENTIAL_TARGET.into();
        if secret.is_none() {
            return Ok(next);
        }
        let secret = Secret::from_plain(secret.unwrap());
        let use_session = prefer_session || !self.persistent_available;
        if use_session {
            self.session
                .put(CREDENTIAL_TARGET, &secret)
                .map_err(SettingsError::Credential)?;
            next.has_secret = true;
            next.secret_mode = "session".into();
            return Ok(next);
        }
        let pending = format!("{CREDENTIAL_TARGET}/pending");
        if let Err(error) = self.store.put(&pending, &secret) {
            if error == CredentialError::Unavailable {
                self.session
                    .put(CREDENTIAL_TARGET, &secret)
                    .map_err(SettingsError::Credential)?;
                next.has_secret = true;
                next.secret_mode = "session".into();
                return Ok(next);
            }
            return Err(SettingsError::Credential(error));
        }
        if self.store.get(&pending).is_err() {
            let _ = self.store.delete(&pending);
            return Err(SettingsError::Credential(CredentialError::NotFound));
        }
        if !probe_ok {
            let _ = self.store.delete(&pending);
            return Err(SettingsError::ProbeFailed);
        }
        if let Err(error) = self.store.put(CREDENTIAL_TARGET, &secret) {
            let _ = self.store.delete(&pending);
            return Err(SettingsError::Credential(error));
        }
        let _ = self.store.delete(&pending);
        if previous.has_secret
            && previous.credential_target != CREDENTIAL_TARGET
            && previous.secret_mode == "persistent"
        {
            let _ = self.store.delete(&previous.credential_target);
        }
        next.has_secret = true;
        next.secret_mode = "persistent".into();
        Ok(next)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WizardStep {
    Controller,
    Targets,
    Autostart,
    RetentionPrivacy,
    Notification,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WizardState {
    pub step: WizardStep,
    pub cancelled: bool,
}

impl Default for WizardState {
    fn default() -> Self {
        Self {
            step: WizardStep::Controller,
            cancelled: false,
        }
    }
}

impl WizardState {
    pub fn next(&mut self) {
        self.step = match self.step {
            WizardStep::Controller => WizardStep::Targets,
            WizardStep::Targets => WizardStep::Autostart,
            WizardStep::Autostart => WizardStep::RetentionPrivacy,
            WizardStep::RetentionPrivacy => WizardStep::Notification,
            WizardStep::Notification => WizardStep::Notification,
        };
    }

    pub fn back(&mut self) {
        self.step = match self.step {
            WizardStep::Controller => WizardStep::Controller,
            WizardStep::Targets => WizardStep::Controller,
            WizardStep::Autostart => WizardStep::Targets,
            WizardStep::RetentionPrivacy => WizardStep::Autostart,
            WizardStep::Notification => WizardStep::RetentionPrivacy,
        };
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
        self.step = WizardStep::Controller;
    }
}

pub fn apply_autostart<A: AutostartPort + ?Sized>(
    port: &A,
    enabled: bool,
) -> Result<bool, AutostartError> {
    port.set_enabled(enabled)?;
    port.is_enabled()
}

#[cfg(test)]
mod settings_workflow_tests {
    use super::*;
    use crate::c2::desktop::FakeAutostart;
    use crate::credential::FakeCredentialStore;

    #[test]
    fn rejects_non_loopback() {
        assert!(matches!(
            validate_address("8.8.8.8:9090"),
            Err(SettingsError::NonLoopback)
        ));
        validate_address("127.0.0.1:9090").expect("loopback");
    }

    #[test]
    fn validate_address_accepts_loopback_forms() {
        validate_address("127.0.0.1:9097").expect("ipv4");
        validate_address("localhost:9097").expect("localhost");
        validate_address("[::1]:9097").expect("ipv6");
    }

    #[test]
    fn validate_address_rejects_non_loopback_hosts() {
        for address in [
            "8.8.8.8:9097",
            "0.0.0.0:9097",
            "example.com:9097",
            "127.0.0.2:9097",
        ] {
            assert!(
                matches!(validate_address(address), Err(SettingsError::NonLoopback)),
                "{address}"
            );
        }
    }

    #[test]
    fn validate_address_rejects_invalid_and_overlong() {
        assert!(matches!(
            validate_address("not-an-addr"),
            Err(SettingsError::InvalidAddress)
        ));
        assert!(matches!(
            validate_address("127.0.0.1"),
            Err(SettingsError::InvalidAddress)
        ));
        let too_long = "x".repeat(SETTING_VALUE_MAX + 1);
        assert!(matches!(
            validate_address(&too_long),
            Err(SettingsError::FieldTooLong)
        ));
    }

    #[test]
    fn validate_targets_rejects_empty_overlong_and_too_many() {
        assert!(matches!(
            validate_targets(&[String::new()]),
            Err(SettingsError::EmptyTarget)
        ));
        let long_name = "n".repeat(TARGET_NAME_MAX + 1);
        assert!(matches!(
            validate_targets(&[long_name]),
            Err(SettingsError::FieldTooLong)
        ));
        let many: Vec<String> = (0..=TARGET_COUNT_MAX).map(|i| format!("t{i}")).collect();
        assert!(matches!(
            validate_targets(&many),
            Err(SettingsError::TooManyTargets)
        ));
    }

    #[test]
    fn probe_failure_deletes_pending_and_keeps_old() {
        let store = FakeCredentialStore::new();
        store
            .put(CREDENTIAL_TARGET, &Secret::from_plain("old-secret"))
            .expect("old");
        let workflow = SettingsWorkflow::new(store, true);
        let previous = ControllerSettings {
            has_secret: true,
            secret_mode: "persistent".into(),
            ..ControllerSettings::default()
        };
        let error = workflow
            .save_secret(
                &previous,
                "127.0.0.1:9090",
                Some("new-secret"),
                false,
                false,
            )
            .expect_err("probe");
        assert_eq!(error, SettingsError::ProbeFailed);
        let old = workflow
            .resolve(CREDENTIAL_TARGET, "persistent")
            .expect("old");
        assert_eq!(old.as_header_bytes(), b"old-secret");
        assert!(workflow
            .resolve(&format!("{CREDENTIAL_TARGET}/pending"), "persistent")
            .is_err());
    }

    #[test]
    fn persistent_save_can_be_resolved() {
        let workflow = SettingsWorkflow::new(FakeCredentialStore::new(), true);
        let next = workflow
            .save_secret(
                &ControllerSettings::default(),
                "127.0.0.1:9097",
                Some("echo-secret"),
                false,
                true,
            )
            .expect("save");
        assert_eq!(next.secret_mode, "persistent");
        assert!(next.has_secret);
        let saved = workflow
            .resolve(CREDENTIAL_TARGET, "persistent")
            .expect("resolve");
        assert_eq!(saved.as_header_bytes(), b"echo-secret");
    }

    #[test]
    fn unavailable_store_uses_session_only() {
        let workflow = SettingsWorkflow::new(FakeCredentialStore::new(), false);
        let next = workflow
            .save_secret(
                &ControllerSettings::default(),
                "127.0.0.1:9090",
                Some("temp-secret"),
                false,
                true,
            )
            .expect("session");
        assert_eq!(next.secret_mode, "session");
        workflow.clear_session();
        assert!(workflow.resolve(CREDENTIAL_TARGET, "session").is_err());
    }

    #[test]
    fn wizard_can_advance_back_cancel() {
        let mut wizard = WizardState::default();
        wizard.next();
        wizard.next();
        assert_eq!(wizard.step, WizardStep::Autostart);
        wizard.back();
        assert_eq!(wizard.step, WizardStep::Targets);
        wizard.cancel();
        assert!(wizard.cancelled);
        assert_eq!(wizard.step, WizardStep::Controller);
    }

    #[test]
    fn autostart_enable_disable_returns_os_readback() {
        let port = FakeAutostart::new();
        assert!(apply_autostart(&port, true).expect("enable readback"));
        assert_eq!(*port.requested.lock().expect("requested"), Some(true));
        assert!(!apply_autostart(&port, false).expect("disable readback"));
        assert_eq!(*port.requested.lock().expect("requested"), Some(false));
    }

    #[test]
    fn autostart_write_and_readback_failures_are_distinct_steps() {
        let write_failure = FakeAutostart::new();
        *write_failure.fail_write.lock().expect("fail write") = true;
        assert!(apply_autostart(&write_failure, true).is_err());
        assert_eq!(
            *write_failure.requested.lock().expect("requested"),
            Some(true)
        );
        assert!(!*write_failure.os_state.lock().expect("os state"));

        let read_failure = FakeAutostart::new();
        *read_failure.fail_read.lock().expect("fail read") = true;
        assert!(apply_autostart(&read_failure, true).is_err());
        assert_eq!(
            *read_failure.requested.lock().expect("requested"),
            Some(true)
        );
        assert!(*read_failure.os_state.lock().expect("os state"));
    }
}
