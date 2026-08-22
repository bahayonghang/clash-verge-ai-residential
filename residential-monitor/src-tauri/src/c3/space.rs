//! 低空间 fail closed。不得覆盖当前可用库。

use crate::c3::query::ReportError;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct SpaceBudget {
    pub available: Option<u64>,
}

impl SpaceBudget {
    pub fn unlimited() -> Self {
        Self {
            available: Some(u64::MAX / 4),
        }
    }

    pub fn exhausted() -> Self {
        Self { available: Some(0) }
    }

    pub fn check(&self, dest_dir: &Path, needed: u64) -> Result<(), ReportError> {
        let available = match self.available {
            Some(value) => value,
            None => probe_dir(dest_dir, needed)?,
        };
        if available < needed {
            return Err(ReportError::InsufficientSpace(
                "not enough free space for backup, restore, spool, or vacuum",
            ));
        }
        Ok(())
    }
}

fn probe_dir(dest_dir: &Path, needed: u64) -> Result<u64, ReportError> {
    std::fs::create_dir_all(dest_dir).map_err(|_| ReportError::Failed("create dest dir"))?;
    let probe = dest_dir.join(".space-check.partial");
    let result = (|| {
        let file = std::fs::File::create(&probe).map_err(|_| ReportError::Failed("space probe"))?;
        file.set_len(needed)
            .map_err(|_| ReportError::InsufficientSpace("set_len failed"))?;
        Ok(needed)
    })();
    let _ = std::fs::remove_file(&probe);
    result
}

#[cfg(test)]
mod space_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn exhausted_budget_fails_closed() {
        let dir = tempdir().expect("dir");
        let error = SpaceBudget::exhausted()
            .check(dir.path(), 1)
            .expect_err("space");
        assert_eq!(error.code(), "insufficient_space");
    }

    #[test]
    fn unlimited_budget_allows_small_artifact() {
        let dir = tempdir().expect("dir");
        SpaceBudget::unlimited()
            .check(dir.path(), 1024)
            .expect("ok");
    }
}
