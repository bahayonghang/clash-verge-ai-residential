//! C4 告警、通知 outbox 与脱敏诊断。
//!
//! 周期用量只调用 C3 ReportService。写入只经 StorageCoordinator 单 writer。

pub mod diagnose;
pub mod engine;
pub mod notify;
pub mod outbox;
pub mod period;
pub mod schema;
pub mod store;
pub mod types;

pub use engine::AlertEngine;
pub use notify::{FakeNotificationSink, NotificationSink, WindowsNotificationSink};
pub use schema::{c4_table_allowlist, C4_MIGRATION_CHECKSUM, C4_SCHEMA_VERSION, C4_TABLES};
pub use types::{AlertRule, AlertWriteSet};

#[cfg(test)]
pub fn c4_owners() -> &'static [&'static str] {
    &[
        "StorageCoordinator",
        "AlertEngine",
        "ReportService",
        "NotificationSink",
    ]
}

#[cfg(test)]
mod c4_owner_tests {
    use super::c4_owners;

    #[test]
    fn c4_reuses_c1_writer_and_c3_report() {
        let owners = c4_owners();
        assert!(owners.contains(&"StorageCoordinator"));
        assert!(owners.contains(&"ReportService"));
        assert!(owners.contains(&"AlertEngine"));
    }
}
