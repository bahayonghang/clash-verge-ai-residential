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

    pub fn message_zh(&self) -> &'static str {
        match self {
            Self::InvalidAddress => "控制器地址无效。",
            Self::NonLoopback => "TCP 只接受本机回环地址。",
            Self::FieldTooLong => "字段超过长度上限。",
            Self::TooManyTargets => "重点目标数量超过上限。",
            Self::EmptyTarget => "目标名称不能为空。",
            Self::Credential(CredentialError::Unavailable) => {
                "凭据存储不可用，只能使用当前进程临时 secret。"
            }
            Self::Credential(_) => "凭据操作失败。",
            Self::ProbeFailed => "控制器探测失败。",
            Self::Unavailable => "设置无法保存。",
        }
    }
}

pub fn validate_address(address: &str) -> Result<(), SettingsError> {
    if address.len() > SETTING_VALUE_MAX {
        return Err(SettingsError::FieldTooLong);
    }
    let host = address
        .rsplit_once(':')
        .map(|(host, _)| host.trim_matches(|c| c == '[' || c == ']'))
        .unwrap_or(address);
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

pub fn apply_autostart<A: AutostartPort>(port: &A, enabled: bool) -> Result<bool, AutostartError> {
    port.set_enabled(enabled)?;
    port.is_enabled()
}

#[cfg(test)]
mod settings_workflow_tests {
    use super::*;
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
}
