//! C2 常量。不改写 C0/C1 已冻结值。

pub const SCHEMA_VERSION: u32 = 1;
pub const LIST_PAGE_DEFAULT: u32 = 200;
pub const LIST_PAGE_MAX: u32 = 1_000;
pub const COALESCE_MAX_KEYS: usize = 20_000;
pub const COALESCE_MAX_BYTES: usize = 1_048_576;
pub const CONNECTION_ID_MAX: usize = 128;
pub const SETTING_VALUE_MAX: usize = 4_096;
pub const TARGET_NAME_MAX: usize = 128;
pub const TARGET_COUNT_MAX: usize = 64;
pub const CLOSE_UNCONFIRMED_MS: u64 = 15_000;

pub fn c2_consumes_c1_modules() -> &'static [&'static str] {
    &[
        "ControllerSession",
        "AccountingEngine",
        "StorageCoordinator",
        "LiveProjection",
        "RecoveryFacade",
    ]
}
