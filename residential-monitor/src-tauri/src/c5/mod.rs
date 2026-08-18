//! C5 发布硬化：集成验证、删除、VACUUM、故障矩阵、并发、供应链。
//! 不重新定义 C1 核算、C3 报告 / retention / backup、C4 告警语义。

pub mod about;
pub mod baseline;
pub mod concurrent;
pub mod fault;
pub mod purge;
pub mod soak;
pub mod supply;
pub mod vacuum;

pub use about::{about, AboutDto};
pub use baseline::{verify_c0_baseline, BaselineStatus};
pub use concurrent::{run_overlap, ConcurrentReport};
pub use fault::{run_fault_matrix, FaultResult};
pub use purge::{confirm_delete, preview_delete, DeletePreview, DeleteReport};
pub use soak::{soak_schedule, soak_smoke, SoakSchedule, SoakSmokeReport};
pub use supply::{file_sha256, inventory_from_locks, SupplyInventory};
pub use vacuum::run_user_vacuum;
