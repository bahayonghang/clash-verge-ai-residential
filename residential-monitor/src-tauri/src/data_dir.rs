//! 数据目录解析与 Temp 迁移。
//!
//! 默认目录是 exe 同级 `data` 子目录。历史版本（≤0.2.0）把数据写在
//! `%TEMP%\io.github.bahayonghang.residential-monitor`，Temp 清理会丢数据；
//! 首次以新默认启动时把整个旧目录搬过来。`RESIDENTIAL_MONITOR_DATA_DIR`
//! 覆盖优先级不变，覆盖生效时不做迁移。

use crate::identity::IDENTIFIER;
use serde_json::json;
use std::path::{Path, PathBuf};

pub const ENV_DATA_DIR: &str = "RESIDENTIAL_MONITOR_DATA_DIR";
const DB_FILE: &str = "monitor.sqlite3";
const DB_SIDECARS: [&str; 2] = ["monitor.sqlite3-wal", "monitor.sqlite3-shm"];
const DATA_DIRS: [&str; 2] = ["report-spool", "archive-tick"];

/// 迁移结果。`Failed` 时调用方沿用 legacy 目录，下次启动重试。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationOutcome {
    /// legacy 不存在或没有主库。
    NotNeeded,
    /// target 已有主库，不动 legacy。
    TargetAlreadyFresh,
    /// 整目录原子改名。
    Renamed,
    /// 逐项搬移（目标已存在时）。
    MovedItemWise,
    /// 迁移失败，沿用 legacy。
    Failed,
}

/// 解析数据目录并执行一次性迁移。永不 panic、永不阻塞启动。
pub fn prepare_data_dir() -> PathBuf {
    let explicit = std::env::var(ENV_DATA_DIR).ok().map(PathBuf::from);
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf));
    let legacy = std::env::temp_dir().join(IDENTIFIER);
    let (dir, outcome) = resolve_and_migrate(explicit.as_deref(), exe_dir.as_deref(), &legacy);
    log_outcome(&outcome);
    dir
}

/// 纯解析核心：`explicit` 命中时原样返回；`exe_dir` 不可得时沿用 legacy。
pub fn resolve_and_migrate(
    explicit: Option<&Path>,
    exe_dir: Option<&Path>,
    legacy: &Path,
) -> (PathBuf, MigrationOutcome) {
    if let Some(dir) = explicit {
        let _ = std::fs::create_dir_all(dir);
        return (dir.to_path_buf(), MigrationOutcome::NotNeeded);
    }
    let Some(exe_dir) = exe_dir else {
        let _ = std::fs::create_dir_all(legacy);
        return (legacy.to_path_buf(), MigrationOutcome::NotNeeded);
    };
    let target = exe_dir.join("data");
    let outcome = migrate_legacy(legacy, &target);
    let dir = if outcome == MigrationOutcome::Failed {
        legacy.to_path_buf()
    } else {
        target
    };
    let _ = std::fs::create_dir_all(&dir);
    (dir, outcome)
}

/// 把 legacy 数据搬进 target。`Failed` 时调用方沿用 legacy 目录，下次启动重试。
///
/// 主库三件套（db + wal + shm）按组搬移，中途失败回滚已移动项，避免
/// 「新目录半套 sidecar」与下次启动的陈旧 wal 配对。spool / archive-tick
/// 是可再生的快照缓存，搬移失败不视为致命，原项留在 legacy。
pub fn migrate_legacy(legacy: &Path, target: &Path) -> MigrationOutcome {
    if !legacy.join(DB_FILE).exists() {
        return MigrationOutcome::NotNeeded;
    }
    if target.join(DB_FILE).exists() {
        return MigrationOutcome::TargetAlreadyFresh;
    }
    if std::fs::rename(legacy, target).is_ok() {
        return MigrationOutcome::Renamed;
    }
    // 整目录 rename 失败（目标已存在、跨卷或占用）：逐项搬移。
    if std::fs::create_dir_all(target).is_err() || move_db_group(legacy, target).is_err() {
        return MigrationOutcome::Failed;
    }
    for dir in DATA_DIRS {
        let from = legacy.join(dir);
        if from.exists() {
            let to = target.join(dir);
            if std::fs::rename(&from, &to).is_err() {
                copy_dir_all(&from, &to).ok(); // 缓存目录，失败留在 legacy
            }
        }
    }
    if std::fs::remove_dir_all(legacy).is_err() {
        return MigrationOutcome::Failed;
    }
    MigrationOutcome::MovedItemWise
}

/// 搬 sidecar 与主库；任一项失败则把已移动项搬回 legacy。
fn move_db_group(legacy: &Path, target: &Path) -> Result<(), ()> {
    let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
    for name in DB_SIDECARS {
        let from = legacy.join(name);
        if from.exists() {
            let to = target.join(name);
            move_item(&from, &to).map_err(|_| rollback(&moved))?;
            moved.push((from, to));
        }
    }
    let from = legacy.join(DB_FILE);
    let to = target.join(DB_FILE);
    if move_item(&from, &to).is_err() || !to.exists() {
        rollback(&moved);
        return Err(());
    }
    Ok(())
}

fn rollback(moved: &[(PathBuf, PathBuf)]) {
    for (from, to) in moved {
        let _ = std::fs::rename(to, from);
    }
}

/// rename 优先；跨卷时 copy + size 校验 + 删源。
fn move_item(from: &Path, to: &Path) -> Result<(), ()> {
    if std::fs::rename(from, to).is_ok() {
        return Ok(());
    }
    let len = std::fs::copy(from, to).map_err(|_| ())?;
    if len != std::fs::metadata(from).map_err(|_| ())?.len() {
        return Err(());
    }
    std::fs::remove_file(from).map_err(|_| ())
}

fn copy_dir_all(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let dst = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dst)?;
        } else {
            std::fs::copy(entry.path(), dst)?;
        }
    }
    Ok(())
}

fn log_outcome(outcome: &MigrationOutcome) {
    use MigrationOutcome as O;
    let (level, event, fields) = match outcome {
        O::Renamed | O::MovedItemWise => (
            crate::app_log::Level::Info,
            "data_dir_migrated",
            json!({ "mode": if matches!(outcome, O::Renamed) { "rename" } else { "item_wise" } }),
        ),
        O::TargetAlreadyFresh => (
            crate::app_log::Level::Info,
            "data_dir_skip",
            json!({ "reason": "target_has_db" }),
        ),
        O::Failed => (
            crate::app_log::Level::Warn,
            "data_dir_migration_failed",
            json!({}),
        ),
        O::NotNeeded => return,
    };
    crate::app_log::emit(level, event, fields);
}

#[cfg(test)]
mod data_dir_tests {
    use super::*;
    use tempfile::tempdir;

    fn seed_legacy(dir: &Path, with_db: bool) {
        std::fs::create_dir_all(dir).unwrap();
        if with_db {
            std::fs::write(dir.join(DB_FILE), "db-bytes").unwrap();
            std::fs::write(dir.join(DB_SIDECARS[0]), "wal-bytes").unwrap();
        }
        std::fs::create_dir_all(dir.join(DATA_DIRS[0])).unwrap();
        std::fs::write(dir.join(DATA_DIRS[0]).join("token.bin"), "spool").unwrap();
    }

    #[test]
    fn explicit_override_wins_without_migration() {
        let legacy = tempdir().unwrap();
        let target_base = tempdir().unwrap();
        seed_legacy(legacy.path(), true);
        let explicit = target_base.path().join("custom");
        let (dir, outcome) =
            resolve_and_migrate(Some(&explicit), Some(target_base.path()), legacy.path());
        assert_eq!(dir, explicit);
        assert_eq!(outcome, MigrationOutcome::NotNeeded);
        assert!(legacy.path().join(DB_FILE).exists(), "覆盖模式不迁移");
    }

    #[test]
    fn missing_exe_dir_keeps_legacy() {
        let legacy = tempdir().unwrap();
        seed_legacy(legacy.path(), true);
        let (dir, outcome) = resolve_and_migrate(None, None, legacy.path());
        assert_eq!(dir, legacy.path());
        assert_eq!(outcome, MigrationOutcome::NotNeeded);
    }

    #[test]
    fn legacy_without_db_just_prepares_target() {
        let base = tempdir().unwrap();
        let legacy = base.path().join("legacy");
        let exe_dir = base.path().join("app");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::create_dir_all(&exe_dir).unwrap();
        let (dir, outcome) = resolve_and_migrate(None, Some(&exe_dir), &legacy);
        assert_eq!(dir, exe_dir.join("data"));
        assert_eq!(outcome, MigrationOutcome::NotNeeded);
        assert!(exe_dir.join("data").exists());
    }

    #[test]
    fn rename_moves_whole_dir() {
        let base = tempdir().unwrap();
        let legacy = base.path().join("legacy");
        let exe_dir = base.path().join("app");
        seed_legacy(&legacy, true);
        std::fs::create_dir_all(&exe_dir).unwrap();
        let (dir, outcome) = resolve_and_migrate(None, Some(&exe_dir), &legacy);
        assert_eq!(outcome, MigrationOutcome::Renamed);
        assert_eq!(dir, exe_dir.join("data"));
        assert!(!legacy.exists());
        assert_eq!(std::fs::read(dir.join(DB_FILE)).unwrap(), b"db-bytes");
        assert!(dir.join(DATA_DIRS[0]).join("token.bin").exists());
    }

    #[test]
    fn item_wise_when_target_exists() {
        let base = tempdir().unwrap();
        let legacy = base.path().join("legacy");
        let exe_dir = base.path().join("app");
        seed_legacy(&legacy, true);
        std::fs::create_dir_all(&exe_dir).unwrap();
        std::fs::create_dir_all(exe_dir.join("data")).unwrap();
        std::fs::write(exe_dir.join("data").join("marker.txt"), "x").unwrap();
        let (dir, outcome) = resolve_and_migrate(None, Some(&exe_dir), &legacy);
        assert_eq!(outcome, MigrationOutcome::MovedItemWise);
        assert_eq!(dir, exe_dir.join("data"));
        assert!(!legacy.exists());
        assert!(dir.join(DB_FILE).exists());
        assert!(dir.join(DB_SIDECARS[0]).exists());
        assert!(dir.join("marker.txt").exists(), "既有目标内容不丢");
    }

    #[test]
    fn target_with_db_keeps_legacy() {
        let base = tempdir().unwrap();
        let legacy = base.path().join("legacy");
        let exe_dir = base.path().join("app");
        seed_legacy(&legacy, true);
        std::fs::create_dir_all(&exe_dir).unwrap();
        std::fs::create_dir_all(exe_dir.join("data")).unwrap();
        std::fs::write(exe_dir.join("data").join(DB_FILE), "fresh").unwrap();
        let (dir, outcome) = resolve_and_migrate(None, Some(&exe_dir), &legacy);
        assert_eq!(outcome, MigrationOutcome::TargetAlreadyFresh);
        assert_eq!(dir, exe_dir.join("data"));
        assert!(legacy.join(DB_FILE).exists(), "legacy 原样保留");
    }

    #[test]
    fn failure_keeps_legacy() {
        let base = tempdir().unwrap();
        let legacy = base.path().join("legacy");
        seed_legacy(&legacy, true);
        // exe_dir 指向一个文件，target 无法创建 → 迁移失败。
        let blocked = base.path().join("blocked");
        std::fs::write(&blocked, "not a dir").unwrap();
        let (dir, outcome) = resolve_and_migrate(None, Some(&blocked), &legacy);
        assert_eq!(outcome, MigrationOutcome::Failed);
        assert_eq!(dir, legacy);
        assert!(legacy.join(DB_FILE).exists(), "失败时数据原样保留");
    }
}
