//! CredentialStore port。C0 冻结接口；C1 使用 Fake；Windows adapter 留给 C2。

use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CredentialError {
    #[error("target 不存在")]
    NotFound,
    #[error("凭据存储不可用")]
    Unavailable,
    #[error("目标为空")]
    InvalidTarget,
}

#[derive(Clone)]
pub struct Secret(Vec<u8>);

impl std::fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Secret(<redacted>)")
    }
}

impl Secret {
    pub fn from_plain(value: impl AsRef<[u8]>) -> Self {
        Self(value.as_ref().to_vec())
    }

    pub fn as_header_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn redacted(&self) -> &'static str {
        "<redacted>"
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        self.0.fill(0);
    }
}

pub trait CredentialStore: Send + Sync {
    fn put(&self, target: &str, secret: &Secret) -> Result<(), CredentialError>;
    fn get(&self, target: &str) -> Result<Secret, CredentialError>;
    fn replace(&self, target: &str, secret: &Secret) -> Result<(), CredentialError>;
    fn delete(&self, target: &str) -> Result<(), CredentialError>;
}

#[derive(Default)]
pub struct FakeCredentialStore {
    inner: Mutex<HashMap<String, String>>,
}

impl FakeCredentialStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CredentialStore for FakeCredentialStore {
    fn put(&self, target: &str, secret: &Secret) -> Result<(), CredentialError> {
        if target.is_empty() {
            return Err(CredentialError::InvalidTarget);
        }
        self.inner.lock().expect("credential mutex").insert(
            target.to_string(),
            String::from_utf8_lossy(secret.as_header_bytes()).into_owned(),
        );
        Ok(())
    }

    fn get(&self, target: &str) -> Result<Secret, CredentialError> {
        self.inner
            .lock()
            .expect("credential mutex")
            .get(target)
            .cloned()
            .map(Secret::from_plain)
            .ok_or(CredentialError::NotFound)
    }

    fn replace(&self, target: &str, secret: &Secret) -> Result<(), CredentialError> {
        let mut guard = self.inner.lock().expect("credential mutex");
        if !guard.contains_key(target) {
            return Err(CredentialError::NotFound);
        }
        guard.insert(
            target.to_string(),
            String::from_utf8_lossy(secret.as_header_bytes()).into_owned(),
        );
        Ok(())
    }

    fn delete(&self, target: &str) -> Result<(), CredentialError> {
        self.inner
            .lock()
            .expect("credential mutex")
            .remove(target)
            .map(|_| ())
            .ok_or(CredentialError::NotFound)
    }
}

pub struct ProcessLocalStore {
    inner: Mutex<Option<(String, String)>>,
}

impl ProcessLocalStore {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub fn clear(&self) {
        *self.inner.lock().expect("credential mutex") = None;
    }
}

impl Default for ProcessLocalStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore for ProcessLocalStore {
    fn put(&self, target: &str, secret: &Secret) -> Result<(), CredentialError> {
        if target.is_empty() {
            return Err(CredentialError::InvalidTarget);
        }
        *self.inner.lock().expect("credential mutex") = Some((
            target.to_string(),
            String::from_utf8_lossy(secret.as_header_bytes()).into_owned(),
        ));
        Ok(())
    }

    fn get(&self, target: &str) -> Result<Secret, CredentialError> {
        match self.inner.lock().expect("credential mutex").as_ref() {
            Some((stored, secret)) if stored == target => Ok(Secret::from_plain(secret.clone())),
            _ => Err(CredentialError::NotFound),
        }
    }

    fn replace(&self, target: &str, secret: &Secret) -> Result<(), CredentialError> {
        self.get(target)?;
        self.put(target, secret)
    }

    fn delete(&self, target: &str) -> Result<(), CredentialError> {
        let mut guard = self.inner.lock().expect("credential mutex");
        match guard.as_ref() {
            Some((stored, _)) if stored == target => {
                *guard = None;
                Ok(())
            }
            _ => Err(CredentialError::NotFound),
        }
    }
}

#[cfg(windows)]
pub mod windows_cm {
    use super::{CredentialError, CredentialStore, Secret};
    use std::ptr;
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_NOT_FOUND};
    use windows_sys::Win32::Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE,
        CRED_TYPE_GENERIC,
    };

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub struct WindowsCredentialManager;

    impl CredentialStore for WindowsCredentialManager {
        fn put(&self, target: &str, secret: &Secret) -> Result<(), CredentialError> {
            if target.is_empty() {
                return Err(CredentialError::InvalidTarget);
            }
            let mut target_w = wide(target);
            let mut blob = secret.as_header_bytes().to_vec();
            let cred = CREDENTIALW {
                Flags: 0,
                Type: CRED_TYPE_GENERIC,
                TargetName: target_w.as_mut_ptr(),
                Comment: ptr::null_mut(),
                LastWritten: unsafe { std::mem::zeroed() },
                CredentialBlobSize: blob.len() as u32,
                CredentialBlob: blob.as_mut_ptr(),
                Persist: CRED_PERSIST_LOCAL_MACHINE,
                AttributeCount: 0,
                Attributes: ptr::null_mut(),
                TargetAlias: ptr::null_mut(),
                UserName: ptr::null_mut(),
            };
            let ok = unsafe { CredWriteW(&cred, 0) };
            if ok == 0 {
                Err(CredentialError::Unavailable)
            } else {
                Ok(())
            }
        }

        fn get(&self, target: &str) -> Result<Secret, CredentialError> {
            let mut target_w = wide(target);
            let mut cred = ptr::null_mut();
            let ok = unsafe { CredReadW(target_w.as_mut_ptr(), CRED_TYPE_GENERIC, 0, &mut cred) };
            if ok == 0 {
                let code = unsafe { GetLastError() };
                return if code == ERROR_NOT_FOUND {
                    Err(CredentialError::NotFound)
                } else {
                    Err(CredentialError::Unavailable)
                };
            }
            let result = unsafe {
                let slice = std::slice::from_raw_parts(
                    (*cred).CredentialBlob,
                    (*cred).CredentialBlobSize as usize,
                );
                Secret::from_plain(String::from_utf8_lossy(slice).into_owned())
            };
            unsafe { CredFree(cred.cast()) };
            Ok(result)
        }

        fn replace(&self, target: &str, secret: &Secret) -> Result<(), CredentialError> {
            self.get(target)?;
            self.put(target, secret)
        }

        fn delete(&self, target: &str) -> Result<(), CredentialError> {
            let mut target_w = wide(target);
            let ok = unsafe { CredDeleteW(target_w.as_mut_ptr(), CRED_TYPE_GENERIC, 0) };
            if ok == 0 {
                let code = unsafe { GetLastError() };
                if code == ERROR_NOT_FOUND {
                    Err(CredentialError::NotFound)
                } else {
                    Err(CredentialError::Unavailable)
                }
            } else {
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod credential_port_tests {
    use super::*;
    use crate::identity::CREDENTIAL_SPIKE_TARGET;

    #[test]
    fn credential_port_fake_crud_and_redaction() {
        let store = FakeCredentialStore::new();
        let secret = Secret::from_plain("not-a-real-secret-value");
        assert_eq!(secret.redacted(), "<redacted>");
        store.put(CREDENTIAL_SPIKE_TARGET, &secret).expect("put");
        let loaded = store.get(CREDENTIAL_SPIKE_TARGET).expect("get");
        assert_eq!(loaded.as_header_bytes(), b"not-a-real-secret-value");
        store
            .replace(CREDENTIAL_SPIKE_TARGET, &Secret::from_plain("rotated"))
            .expect("replace");
        store.delete(CREDENTIAL_SPIKE_TARGET).expect("delete");
        assert_eq!(
            store.get(CREDENTIAL_SPIKE_TARGET).unwrap_err(),
            CredentialError::NotFound
        );
    }

    #[test]
    fn credential_port_process_local_clears() {
        let store = ProcessLocalStore::new();
        store
            .put("temp", &Secret::from_plain("session-only"))
            .expect("put");
        store.clear();
        assert_eq!(store.get("temp").unwrap_err(), CredentialError::NotFound);
    }
}

#[cfg(all(test, windows))]
mod credential_windows_tests {
    use super::*;

    #[test]
    #[ignore = "会写入本机 Credential Manager，需人工授权后单独运行"]
    fn credential_windows_generic_crud() {
        let store = windows_cm::WindowsCredentialManager;
        let target = crate::identity::CREDENTIAL_SPIKE_TARGET;
        let _ = store.delete(target);
        store
            .put(target, &Secret::from_plain("spike-only"))
            .expect("put");
        let loaded = store.get(target).expect("get");
        assert_eq!(loaded.as_header_bytes(), b"spike-only");
        store.delete(target).expect("delete");
    }
}
