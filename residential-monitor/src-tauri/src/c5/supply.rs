//! SHA-256 与依赖清单。不把未签名资产标成 signed。

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplyInventory {
    pub schema_version: u32,
    pub cargo_packages: usize,
    pub npm_packages: usize,
    pub cargo_sample: Vec<String>,
    pub npm_sample: Vec<String>,
    pub secret_hits: Vec<String>,
    pub installer_sha256: Option<String>,
    pub signed: bool,
    pub note_zh: String,
}

pub fn file_sha256(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

pub fn inventory_from_locks(
    cargo_lock: &Path,
    package_lock: &Path,
    installer: Option<&Path>,
) -> Result<SupplyInventory, String> {
    let cargo = std::fs::read_to_string(cargo_lock).map_err(|error| error.to_string())?;
    let npm = std::fs::read_to_string(package_lock).map_err(|error| error.to_string())?;
    let cargo_names = cargo_names(&cargo);
    let npm_names = npm_names(&npm);
    let secret_hits = scan_secrets(&cargo)
        .into_iter()
        .chain(scan_secrets(&npm))
        .collect::<Vec<_>>();
    let installer_sha256 = match installer {
        Some(path) if path.exists() => Some(file_sha256(path)?),
        _ => None,
    };
    Ok(SupplyInventory {
        schema_version: 1,
        cargo_packages: cargo_names.len(),
        npm_packages: npm_names.len(),
        cargo_sample: cargo_names.into_iter().take(8).collect(),
        npm_sample: npm_names.into_iter().take(8).collect(),
        secret_hits,
        installer_sha256,
        signed: false,
        note_zh: "依赖清单来自 lockfile。installer 未签名。不得把未签名构建写成已验证签名。".into(),
    })
}

fn cargo_names(raw: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut current = None;
    for line in raw.lines() {
        if let Some(name) = line.strip_prefix("name = \"") {
            current = Some(name.trim_end_matches('"').to_string());
        } else if line.starts_with("version = \"") {
            if let Some(name) = current.take() {
                names.push(name);
            }
        }
    }
    names.sort();
    names.dedup();
    names
}

fn npm_names(raw: &str) -> Vec<String> {
    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("\"node_modules/")
                .and_then(|rest| rest.split('"').next())
                .map(str::to_string)
        })
        .collect()
}

fn scan_secrets(raw: &str) -> Vec<String> {
    let mut hits = Vec::new();
    for needle in ["bearer ", "password=", "secret="] {
        if raw.to_ascii_lowercase().contains(needle) {
            hits.push(needle.trim().to_string());
        }
    }
    hits
}

#[cfg(test)]
mod supply_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn inventory_reads_locks_and_stays_unsigned() {
        let dir = tempdir().expect("dir");
        let cargo = dir.path().join("Cargo.lock");
        let npm = dir.path().join("package-lock.json");
        std::fs::write(
            &cargo,
            "[[package]]\nname = \"rusqlite\"\nversion = \"0.40.0\"\n",
        )
        .expect("cargo");
        std::fs::write(&npm, "{\n  \"node_modules/vite\": {}\n}\n").expect("npm");
        let installer = dir.path().join("setup.exe");
        std::fs::write(&installer, b"nsis").expect("exe");
        let inventory = inventory_from_locks(&cargo, &npm, Some(&installer)).expect("inv");
        assert_eq!(inventory.cargo_packages, 1);
        assert_eq!(inventory.npm_packages, 1);
        assert!(!inventory.signed);
        assert!(inventory.installer_sha256.is_some());
        assert!(inventory.secret_hits.is_empty());
    }
}
