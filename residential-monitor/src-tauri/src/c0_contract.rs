//! C1 只能消费已批准的 C0 决策。缺项则 fail closed。

use crate::identity::{BINARY_NAME, CREDENTIAL_TARGET, IDENTIFIER, PRODUCT_NAME};

pub const SQLITE_BINDING: &str = "rusqlite";
pub const SQLITE_VERSION_MIN: &str = "3.51.3";
pub const JOURNAL_MODE: &str = "WAL";
pub const SYNCHRONOUS: &str = "FULL";
pub const BUSY_TIMEOUT_MS: u32 = 5_000;
pub const WRITER_BATCH_MS: u32 = 1_000;
pub const PREPARED_BATCH_ROWS: u32 = 10_000;
pub const QUEUE_MAX_BATCHES: u32 = 8;
pub const FRAME_BODY_LIMIT: usize = 8 * 1024 * 1024;
pub const STRING_LIMIT: usize = 4_096;
pub const RETRY_WINDOW_RECEIPTS: u32 = 100_000;
pub const RETRY_WINDOW_HOURS: u32 = 24;
pub const DESIGN_AVERAGE_ACTIVE: u32 = 250;
pub const REGRESSION_AVERAGE_ACTIVE: u32 = 50;
pub const STRESS_AVERAGE_ACTIVE: u32 = 1_000;
pub const PEAK_ACTIVE: u32 = 10_000;
pub const PEAK_HZ: u32 = 1;
pub const PEAK_MINUTES: u32 = 30;
pub const SCHEMA_VERSION: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractError {
    MissingApproval,
    DurabilityNotFull,
}

pub fn assert_runtime_contract() -> Result<(), ContractError> {
    if SYNCHRONOUS != "FULL" || JOURNAL_MODE != "WAL" {
        return Err(ContractError::DurabilityNotFull);
    }
    if IDENTIFIER.is_empty() || PRODUCT_NAME.is_empty() || BINARY_NAME.is_empty() {
        return Err(ContractError::MissingApproval);
    }
    let _ = CREDENTIAL_TARGET;
    Ok(())
}

pub fn core_table_allowlist() -> &'static [&'static str] {
    &[
        "schema_migration",
        "data_version",
        "bundle_epoch",
        "committed_bundle",
        "machine_setting",
        "controller_epoch",
        "target_set",
        "target_item",
        "coverage_interval",
        "connection_session",
        "connection_chain",
        "connection_minute",
        "backup_manifest",
    ]
}

pub fn forbidden_table_fragments() -> &'static [&'static str] {
    &[
        "report",
        "hourly",
        "daily",
        "retention",
        "alert",
        "notification",
        "outbox",
    ]
}

#[cfg(test)]
mod c0_contract_tests {
    use super::*;

    #[test]
    fn c0_contract_runtime_uses_approved_full_wal() {
        assert_eq!(assert_runtime_contract(), Ok(()));
        assert_eq!(SYNCHRONOUS, "FULL");
        assert_eq!(JOURNAL_MODE, "WAL");
        assert_eq!(DESIGN_AVERAGE_ACTIVE, 250);
        assert_eq!(PEAK_ACTIVE, 10_000);
    }
}

#[cfg(test)]
mod schema_allowlist_tests {
    use super::*;

    #[test]
    fn schema_allowlist_excludes_report_and_alert_tables() {
        for name in core_table_allowlist() {
            for fragment in forbidden_table_fragments() {
                assert!(!name.contains(fragment), "{name} 不得包含 {fragment}");
            }
        }
    }
}
