//! 库路径解析。不触发数据目录迁移，不创建空库。

use super::DbCliError;
use crate::data_dir::ENV_DATA_DIR;
use crate::identity::PRODUCT_NAME;
use std::path::{Path, PathBuf};

const DB_FILE: &str = "monitor.sqlite3";

pub fn resolve_existing_db(explicit: Option<&Path>) -> Result<PathBuf, DbCliError> {
    resolve_existing_db_with(
        explicit,
        std::env::var_os(ENV_DATA_DIR).map(PathBuf::from),
        default_install_db(),
    )
}

pub(crate) fn default_install_db() -> PathBuf {
    let local = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    local.join(PRODUCT_NAME).join("data").join(DB_FILE)
}

pub(crate) fn resolve_existing_db_with(
    explicit: Option<&Path>,
    env_data_dir: Option<PathBuf>,
    default_db: PathBuf,
) -> Result<PathBuf, DbCliError> {
    if let Some(path) = explicit {
        return require_existing(path);
    }
    if let Some(dir) = env_data_dir {
        return require_existing(&dir.join(DB_FILE));
    }
    require_existing(&default_db)
}

fn require_existing(path: &Path) -> Result<PathBuf, DbCliError> {
    if path.is_file() {
        Ok(path.to_path_buf())
    } else {
        Err(DbCliError::DatabaseMissing(path.to_path_buf()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("dir");
        }
        fs::write(path, b"x").expect("touch");
    }

    #[test]
    fn missing_db_does_not_create_files() {
        let dir = tempdir().expect("dir");
        let path = dir.path().join(DB_FILE);
        let err = resolve_existing_db_with(Some(&path), None, dir.path().join("default.sqlite3"))
            .expect_err("missing");
        assert!(matches!(err, DbCliError::DatabaseMissing(_)));
        assert!(!path.exists());
        assert_eq!(fs::read_dir(dir.path()).expect("read").count(), 0);
    }

    #[test]
    fn explicit_beats_env_and_default() {
        let dir = tempdir().expect("dir");
        let explicit = dir.path().join("explicit.sqlite3");
        let env_dir = dir.path().join("env");
        let env_db = env_dir.join(DB_FILE);
        let default_db = dir.path().join("default").join(DB_FILE);
        touch(&explicit);
        touch(&env_db);
        touch(&default_db);
        let resolved =
            resolve_existing_db_with(Some(&explicit), Some(env_dir), default_db).expect("ok");
        assert_eq!(resolved, explicit);
    }

    #[test]
    fn env_beats_default_when_no_explicit() {
        let dir = tempdir().expect("dir");
        let env_dir = dir.path().join("env");
        let env_db = env_dir.join(DB_FILE);
        let default_db = dir.path().join("default").join(DB_FILE);
        touch(&env_db);
        touch(&default_db);
        let resolved = resolve_existing_db_with(None, Some(env_dir), default_db).expect("ok");
        assert_eq!(resolved, env_db);
    }

    #[test]
    fn default_used_when_no_explicit_or_env() {
        let dir = tempdir().expect("dir");
        let default_db = dir.path().join("default").join(DB_FILE);
        touch(&default_db);
        let resolved = resolve_existing_db_with(None, None, default_db.clone()).expect("ok");
        assert_eq!(resolved, default_db);
    }

    #[test]
    fn explicit_missing_does_not_fall_through() {
        let dir = tempdir().expect("dir");
        let missing = dir.path().join("missing.sqlite3");
        let env_dir = dir.path().join("env");
        let env_db = env_dir.join(DB_FILE);
        touch(&env_db);
        let err = resolve_existing_db_with(Some(&missing), Some(env_dir), env_db)
            .expect_err("no fallthrough");
        assert!(matches!(err, DbCliError::DatabaseMissing(_)));
    }
}
