//! 核验 C0 冻结的升级基线。缺失必须显式失败，不得用当前构建冒充旧版本。

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const C0_BASELINE_DIR: &str =
    ".trellis/tasks/archive/2026-08/08-18-monitor-foundation-spike/research/evidence/c0-upgrade-baseline";
pub const INSTALLER_NAME: &str = "c0-nsis-current-user-setup.exe";
pub const SCHEMA_FIXTURE_NAME: &str = "c0-schema-fixture.sqlite3";
pub const CHECKSUMS_NAME: &str = "checksums.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineStatus {
    pub schema_version: u32,
    pub dir: String,
    pub installer_present: bool,
    pub schema_fixture_present: bool,
    pub checksums_present: bool,
    pub installer_sha256: Option<String>,
    pub fixture_sha256: Option<String>,
    pub checksum_match: Option<bool>,
    pub usable_for_upgrade: bool,
    pub note_zh: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
struct ChecksumsFile {
    installer_sha256: String,
    schema_fixture_sha256: String,
}

pub fn verify_c0_baseline(repo_root: &Path) -> BaselineStatus {
    let dir = repo_root.join(C0_BASELINE_DIR);
    let installer = dir.join(INSTALLER_NAME);
    let fixture = dir.join(SCHEMA_FIXTURE_NAME);
    let checksums = dir.join(CHECKSUMS_NAME);
    let installer_present = installer.exists();
    let schema_fixture_present = fixture.exists();
    let checksums_present = checksums.exists();
    let installer_sha256 = installer_present.then(|| sha256(&installer));
    let fixture_sha256 = schema_fixture_present.then(|| sha256(&fixture));
    let checksum_match = if checksums_present {
        std::fs::read_to_string(&checksums)
            .ok()
            .and_then(|raw| serde_json::from_str::<ChecksumsFile>(&raw).ok())
            .map(|expected| {
                installer_sha256.as_deref() == Some(expected.installer_sha256.as_str())
                    && fixture_sha256.as_deref() == Some(expected.schema_fixture_sha256.as_str())
            })
    } else {
        None
    };
    let usable_for_upgrade =
        installer_present && schema_fixture_present && checksum_match == Some(true);
    let note_zh = if usable_for_upgrade {
        "C0 升级基线可用。".into()
    } else {
        "C0 冻结 NSIS 安装包或 schema fixture 缺失，或 checksum 对不上。不得用当前代码重做旧版本。C5-AC5 未通过。".into()
    };
    BaselineStatus {
        schema_version: 1,
        dir: dir.to_string_lossy().into_owned(),
        installer_present,
        schema_fixture_present,
        checksums_present,
        installer_sha256,
        fixture_sha256,
        checksum_match,
        usable_for_upgrade,
        note_zh,
    }
}

fn sha256(path: &PathBuf) -> String {
    let bytes = std::fs::read(path).unwrap_or_default();
    hex::encode(Sha256::digest(bytes))
}

#[cfg(test)]
mod baseline_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn missing_baseline_is_not_usable() {
        let dir = tempdir().expect("dir");
        let status = verify_c0_baseline(dir.path());
        assert!(!status.usable_for_upgrade);
        assert!(!status.installer_present);
        assert!(status.note_zh.contains("C5-AC5"));
    }

    #[test]
    fn matching_checksums_are_usable() {
        let root = tempdir().expect("dir");
        let dir = root.path().join(C0_BASELINE_DIR);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let installer = dir.join(INSTALLER_NAME);
        let fixture = dir.join(SCHEMA_FIXTURE_NAME);
        std::fs::write(&installer, b"nsis-fixture").expect("exe");
        std::fs::write(&fixture, b"schema-fixture").expect("db");
        let payload = serde_json::json!({
            "installer_sha256": hex::encode(Sha256::digest(b"nsis-fixture")),
            "schema_fixture_sha256": hex::encode(Sha256::digest(b"schema-fixture"))
        });
        std::fs::write(dir.join(CHECKSUMS_NAME), payload.to_string()).expect("sum");
        let status = verify_c0_baseline(root.path());
        assert!(status.usable_for_upgrade);
        assert_eq!(status.checksum_match, Some(true));
    }
}
