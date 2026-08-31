//! C3 历史报告、导出、保留与备份恢复。
//!
//! 只通过 StorageCoordinator / RecoveryFacade 访问 SQLite，不另建 writer。

pub mod archive;
pub mod backup;
pub mod export;
pub mod query;
pub mod retention;
pub mod rule_name;
pub mod schema;
pub mod service;
pub mod share;
pub mod snapshot;
pub mod space;
pub mod sql;

pub use archive::ReportArchiveService;
pub use backup::BackupRestoreService;
pub use export::{ExportService, ExportSpec};
pub use query::{
    default_auto_report_query, local_day_bounds, local_hour_bounds, local_month_bounds,
    plan_capability, plan_capability_ex, timezone_offset_secs, validate_query, CapabilityPlan,
    DrilldownCapability, ReportError, ReportQuery, ReportResult, AUTO_DELETE_ENABLED,
    DIMENSION_RETAIN_DAYS, MAX_ACTIVE_TOKENS, MAX_SPOOL_BYTES, MAX_TOKEN_BYTES, PAGE_DEADLINE_MS,
    RAW_RETAIN_DAYS_DEFAULT, RAW_RETAIN_DAYS_MAX, REPORT_DEADLINE_MS, TOKEN_TTL_SECS,
};
pub use retention::RetentionService;
pub use service::{run_uncached, ReportService};
pub use share::{query_residential_share, query_residential_share_on, ResidentialShare};
pub use snapshot::ReportSnapshotStore;
pub use space::SpaceBudget;

#[cfg(test)]
pub fn c3_owners() -> &'static [&'static str] {
    &[
        "StorageCoordinator",
        "RecoveryFacade",
        "ReportService",
        "ReportArchiveService",
        "ReportSnapshotStore",
        "ExportService",
        "RetentionService",
        "BackupRestoreService",
    ]
}

#[cfg(test)]
mod c3_owner_tests {
    use super::c3_owners;

    #[test]
    fn c3_reuses_c1_storage_and_c2_recovery() {
        let owners = c3_owners();
        assert!(owners.contains(&"StorageCoordinator"));
        assert!(owners.contains(&"RecoveryFacade"));
        assert!(!owners.contains(&"AlertEngine"));
    }
}
