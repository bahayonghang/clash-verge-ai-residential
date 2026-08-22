//! 统一证据包与 adopt/reject/fallback 决策模板。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::path::Path;

use crate::identity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Adopt,
    Reject,
    Fallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineInfo {
    pub os: String,
    pub arch: String,
    pub cpu_cores: usize,
    pub hostname_redacted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolVersions {
    pub rustc: String,
    pub sqlite: Option<String>,
    pub binding: String,
    pub node: Option<String>,
    pub tauri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundle {
    pub protocol_version: u32,
    pub generated_at_utc: String,
    pub subject: String,
    pub machine: MachineInfo,
    pub tool_versions: ToolVersions,
    pub inputs: serde_json::Value,
    pub observations: serde_json::Value,
    pub threshold_source: String,
    pub decision: Decision,
    pub fallback_trigger: Option<String>,
    pub constraints_for_c1: Vec<String>,
    pub evidence_paths: Vec<String>,
    pub artifact_sha256: BTreeMap<String, String>,
    pub redacted: bool,
    pub approved_by: Option<String>,
}

impl EvidenceBundle {
    pub fn draft(subject: &str, decision: Decision) -> Self {
        Self {
            protocol_version: crate::workload::PROTOCOL_VERSION,
            generated_at_utc: chrono::Utc::now().to_rfc3339(),
            subject: subject.to_string(),
            machine: MachineInfo {
                os: env::consts::OS.to_string(),
                arch: env::consts::ARCH.to_string(),
                cpu_cores: std::thread::available_parallelism()
                    .map(|value| value.get())
                    .unwrap_or(1),
                hostname_redacted: true,
            },
            tool_versions: ToolVersions {
                rustc: env::var("RUSTC_VERSION").unwrap_or_else(|_| "rustc-host".to_string()),
                sqlite: None,
                binding: "rusqlite".to_string(),
                node: None,
                tauri: "2".to_string(),
            },
            inputs: serde_json::json!({
                "identifier": identity::IDENTIFIER
            }),
            observations: serde_json::json!({}),
            threshold_source: "parent-prd-ac15-and-c0-implement".to_string(),
            decision,
            fallback_trigger: None,
            constraints_for_c1: Vec::new(),
            evidence_paths: Vec::new(),
            artifact_sha256: BTreeMap::new(),
            redacted: true,
            approved_by: None,
        }
    }

    pub fn write_json(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let payload = serde_json::to_vec_pretty(self)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        std::fs::write(path, payload)
    }
}

#[cfg(test)]
mod evidence_tests {
    use super::*;

    #[test]
    fn evidence_template_has_decision_and_machine_fields() {
        let bundle = EvidenceBundle::draft("sqlite-binding", Decision::Adopt);
        assert_eq!(bundle.decision, Decision::Adopt);
        assert!(bundle.redacted);
        assert_eq!(bundle.machine.os, env::consts::OS);
        let json = serde_json::to_value(&bundle).expect("serialize");
        assert!(json.get("approved_by").is_some());
        assert_eq!(json["identifier"], serde_json::Value::Null);
        assert_eq!(json["inputs"]["identifier"], identity::IDENTIFIER);
    }
}
