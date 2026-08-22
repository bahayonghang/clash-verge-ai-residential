//! 24 小时 soak 日程与短 smoke。smoke 不得冒充 24 小时。

use crate::c5::concurrent::run_overlap;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoakSchedule {
    pub schema_version: u32,
    pub design_average_active: u32,
    pub warmup_hours: u32,
    pub duration_hours: u32,
    pub report_every_secs: u32,
    pub export_every_secs: u32,
    pub retention_every_secs: u32,
    pub checkpoint_every_secs: u32,
    pub backup_min_count: u32,
    pub alert_inject_every_secs: u32,
    pub note_zh: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoakSmokeReport {
    pub schema_version: u32,
    pub schedule: SoakSchedule,
    pub elapsed_ms: u128,
    pub overlap_ok: bool,
    pub full_24h: bool,
    pub note_zh: String,
}

pub fn soak_schedule() -> SoakSchedule {
    SoakSchedule {
        schema_version: 1,
        design_average_active: 250,
        warmup_hours: 1,
        duration_hours: 24,
        report_every_secs: 300,
        export_every_secs: 3600,
        retention_every_secs: 3600,
        checkpoint_every_secs: 3600,
        backup_min_count: 2,
        alert_inject_every_secs: 3600,
        note_zh: "运行前冻结的发布设计点日程。A=250 完整 tuple。不得用未声明轻载替代。".into(),
    }
}

pub fn soak_smoke(dir: &Path) -> Result<SoakSmokeReport, String> {
    let started = std::time::Instant::now();
    let overlap = run_overlap(dir)?;
    Ok(SoakSmokeReport {
        schema_version: 1,
        schedule: soak_schedule(),
        elapsed_ms: started.elapsed().as_millis(),
        overlap_ok: overlap.report_ok && overlap.backup_ok,
        full_24h: false,
        note_zh: "仅 soak 入口 smoke。24 小时 soak 未执行，C5-AC11 未通过。".into(),
    })
}

#[cfg(test)]
mod soak_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn schedule_is_frozen_and_smoke_is_not_24h() {
        let schedule = soak_schedule();
        assert_eq!(schedule.duration_hours, 24);
        assert_eq!(schedule.report_every_secs, 300);
        assert_eq!(schedule.design_average_active, 250);
        let dir = tempdir().expect("dir");
        let smoke = soak_smoke(dir.path()).expect("smoke");
        assert!(!smoke.full_24h);
        assert!(smoke.overlap_ok);
        assert!(smoke.note_zh.contains("未执行"));
    }
}
