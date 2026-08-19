//! 脱敏诊断。secret 与完整敏感连接字段不得进入导出。

use crate::c0_contract::{SCHEMA_VERSION, SYNCHRONOUS};
use crate::c4::outbox;
use crate::c4::schema::C4_MIGRATION_CHECKSUM;
use crate::controller::SessionStatus;
use crate::storage::{StorageCoordinator, StorageError};
use serde::{Deserialize, Serialize};
use std::path::Path;

const FORBIDDEN: &[&str] = &[
    "bearer ",
    "password=",
    "secret=",
    "authorization:",
    "credential",
];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsSnapshot {
    pub schema_version: u32,
    pub app_version: String,
    pub sqlite_user_version: i32,
    pub supported_schema: i32,
    pub c4_checksum: String,
    pub journal_mode: String,
    pub synchronous: String,
    pub controller_transport_status: String,
    pub coverage_summary: String,
    pub writer_watermark: u64,
    pub writer_receipts: u64,
    pub last_frame_utc: Option<i64>,
    pub reconnect_hint_zh: String,
    pub database_ok: bool,
    pub wal_checkpoint_ok: bool,
    pub backup_retention_note_zh: String,
    pub alert_active: u32,
    pub outbox_backlog: u32,
    pub recent_redacted_error_classes: Vec<String>,
}

impl DiagnosticsSnapshot {
    pub fn contains_secret(&self) -> bool {
        let encoded = serde_json::to_string(self).unwrap_or_default();
        let lower = encoded.to_ascii_lowercase();
        FORBIDDEN.iter().any(|item| lower.contains(item))
    }
}

pub fn collect(
    coordinator: &StorageCoordinator,
    session: SessionStatus,
    last_frame_utc: Option<i64>,
    coverage_summary: &str,
) -> Result<DiagnosticsSnapshot, StorageError> {
    let user_version: i32 =
        coordinator
            .connection()
            .query_row("pragma user_version", [], |row| row.get(0))?;
    let journal: String = coordinator
        .connection()
        .query_row("pragma journal_mode", [], |row| row.get(0))?;
    let active: i64 = coordinator.connection().query_row(
        "select count(*) from alert_instance where status = 'active'",
        [],
        |row| row.get(0),
    )?;
    let errors = recent_error_classes(coordinator)?;
    Ok(DiagnosticsSnapshot {
        schema_version: 1,
        app_version: env!("CARGO_PKG_VERSION").into(),
        sqlite_user_version: user_version,
        supported_schema: SCHEMA_VERSION,
        c4_checksum: C4_MIGRATION_CHECKSUM.into(),
        journal_mode: journal,
        synchronous: SYNCHRONOUS.into(),
        controller_transport_status: crate::c2::hub::session_status_name(session),
        coverage_summary: coverage_summary.to_string(),
        writer_watermark: coordinator.watermark()?,
        writer_receipts: coordinator.receipt_count()?,
        last_frame_utc,
        reconnect_hint_zh: crate::c2::facade::status_action_zh(session).into(),
        database_ok: coordinator.health()?.ok,
        wal_checkpoint_ok: true,
        backup_retention_note_zh: "备份与保留仍走 C3。自动 DELETE 关闭。".into(),
        alert_active: active as u32,
        outbox_backlog: outbox::backlog(coordinator)?,
        recent_redacted_error_classes: errors,
    })
}

pub fn export_atomic(snapshot: &DiagnosticsSnapshot, dest: &Path) -> Result<String, StorageError> {
    if snapshot.contains_secret() {
        return Err(StorageError::Closed("diagnostics leaked secret".into()));
    }
    let parent = dest.parent().unwrap_or(dest);
    let tmp = parent.join(format!(
        ".{}.partial",
        dest.file_name()
            .and_then(|item| item.to_str())
            .unwrap_or("diag.json")
    ));
    let encoded = serde_json::to_vec_pretty(snapshot)
        .map_err(|error| StorageError::Closed(error.to_string()))?;
    let lower = String::from_utf8_lossy(&encoded).to_ascii_lowercase();
    if FORBIDDEN.iter().any(|item| lower.contains(item)) {
        return Err(StorageError::Closed("diagnostics leaked secret".into()));
    }
    std::fs::write(&tmp, encoded).map_err(|error| StorageError::Closed(error.to_string()))?;
    std::fs::rename(&tmp, dest).map_err(|error| StorageError::Closed(error.to_string()))?;
    Ok(dest.to_string_lossy().into_owned())
}

fn recent_error_classes(coordinator: &StorageCoordinator) -> Result<Vec<String>, StorageError> {
    let mut statement = coordinator.connection().prepare(
        "select distinct error_class from notification_outbox
          where error_class is not null
          order by created_utc desc
          limit 8",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    Ok(rows.filter_map(Result::ok).collect())
}

pub fn scan_text_for_secrets(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    FORBIDDEN.iter().any(|item| lower.contains(item))
}

#[cfg(test)]
mod diagnose_tests {
    use super::*;
    use crate::storage::StorageCoordinator;
    use tempfile::tempdir;

    #[test]
    fn diagnostics_omit_secret_and_full_host() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("d.sqlite3")).expect("open");
        let snap =
            collect(&coordinator, SessionStatus::Connected, Some(10), "covered").expect("collect");
        assert_eq!(snap.sqlite_user_version, crate::c0_contract::SCHEMA_VERSION);
        assert!(!snap.contains_secret());
        let encoded = serde_json::to_string(&snap).expect("json");
        assert!(!encoded.contains("127.0.0.1"));
        assert!(!scan_text_for_secrets(&encoded));
        let path = dir.path().join("diag.json");
        export_atomic(&snap, &path).expect("export");
        let text = std::fs::read_to_string(path).expect("read");
        assert!(text.contains("c4-alert-v3"));
        assert!(!scan_text_for_secrets(&text));
    }
}
