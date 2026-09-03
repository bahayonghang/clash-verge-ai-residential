//! 表格是同一 envelope 的渲染，不另算数值。

use super::envelope::Envelope;
use sha2::{Digest, Sha256};

pub fn print_envelope(envelope: &Envelope, format: super::OutputFormat, redact: bool) {
    let mut value = serde_json::to_value(envelope).unwrap_or(serde_json::Value::Null);
    if redact {
        redact_value(&mut value);
    }
    match format {
        super::OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into())
            );
        }
        super::OutputFormat::Table => print_table(&value),
    }
}

fn print_table(value: &serde_json::Value) {
    println!("command\t{}", value["command"].as_str().unwrap_or(""));
    println!(
        "window\t{} .. {}\t{}",
        value["window"]["startUtc"], value["window"]["endUtc"], value["window"]["timezone"]
    );
    println!(
        "capability\tsupported={}\t{}",
        value["capability"]["supported"], value["capability"]["reason"]
    );
    println!(
        "coverage\t{}\tobserved={}\tgap={}",
        value["coverage"]["status"], value["coverage"]["observedSec"], value["coverage"]["gapSec"]
    );
    println!(
        "truncation\t{}\trows={}/{}",
        value["truncation"]["status"], value["truncation"]["rows"], value["truncation"]["rowCap"]
    );
    println!("dataVersion\t{}", value["dataVersion"]);
    if let Some(notes) = value["notes"].as_array() {
        for note in notes {
            println!("note\t{}", note.as_str().unwrap_or(""));
        }
    }
    print_result(&value["result"]);
}

fn print_result(result: &serde_json::Value) {
    if result.is_null() {
        println!("result\tnull");
        return;
    }
    if let Some(rows) = result["rankings"].as_array() {
        println!("identity\tunknown\tupload\tdownload\tconnections\tzeroFlow");
        for row in rows {
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}",
                cell(&row["identity"]),
                cell(&row["unknown"]),
                cell(&row["upload"]),
                cell(&row["download"]),
                cell(&row["connectionCount"]),
                cell(&row["zeroFlow"])
            );
        }
    }
    if let Some(share) = result.get("residentialUpload") {
        println!(
            "share\tresidentialUpload={}\tresidentialDownload={}\tattributedUpload={}\tattributedDownload={}",
            share,
            result["residentialDownload"],
            result["attributedUpload"],
            result["attributedDownload"]
        );
    }
    for key in [
        "covered",
        "dead",
        "unsupportedPattern",
        "uncovered",
        "mapped",
        "shared",
        "unmapped",
        "unsupportedSwitch",
        "outbound",
        "items",
    ] {
        if let Some(rows) = result[key].as_array() {
            println!("{key}\tcount={}", rows.len());
            for row in rows {
                println!(
                    "{key}\t{}",
                    serde_json::to_string(row).unwrap_or_else(|_| "{}".into())
                );
            }
        }
    }
    if let Some(object) = result.as_object() {
        for (key, value) in object {
            if (value.is_object() || value.is_number() || value.is_boolean() || value.is_string())
                && !matches!(
                    key.as_str(),
                    "rankings"
                        | "covered"
                        | "dead"
                        | "unsupportedPattern"
                        | "uncovered"
                        | "mapped"
                        | "shared"
                        | "unmapped"
                        | "unsupportedSwitch"
                        | "outbound"
                        | "items"
                        | "residentialUpload"
                        | "residentialDownload"
                        | "attributedUpload"
                        | "attributedDownload"
                )
            {
                println!("{key}\t{value}");
            }
        }
    }
}

fn cell(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".into(),
        other => other.to_string().trim_matches('"').to_string(),
    }
}

fn redact_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if matches!(
                    key.as_str(),
                    "identity" | "host" | "process" | "pattern" | "label"
                ) {
                    if let Some(text) = map.get(&key).and_then(|item| item.as_str()) {
                        if !text.is_empty() && text != "__unknown__" {
                            map.insert(
                                key.clone(),
                                serde_json::Value::String(redact_identity(text)),
                            );
                        }
                    }
                } else if let Some(child) = map.get_mut(&key) {
                    redact_value(child);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                redact_value(item);
            }
        }
        _ => {}
    }
}

pub fn redact_identity(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    let prefix = hex::encode(&digest[..4]);
    format!("{prefix}#{}", value.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_uses_sha256_prefix_and_length() {
        let redacted = redact_identity("claude.ai");
        assert!(redacted.ends_with("#9"));
        assert_eq!(redacted.len(), 8 + 1 + 1);
        assert!(!redacted.contains("claude"));
    }
}
