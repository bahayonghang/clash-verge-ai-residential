//! 域名模式解析与字节归属匹配。
//!
//! 优先级 exact > 最长 suffix > regex > 输入顺序，只解释「这些字节可以归到哪条模式」，
//! 不模拟 Mihomo 的首个规则命中。

use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct RulesFile {
    pub schema_version: u32,
    pub group: String,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct SwitchesFile {
    pub schema_version: u32,
    pub supported: std::collections::BTreeMap<String, Vec<String>>,
    pub unsupported: Vec<String>,
}
#[derive(Debug, Clone)]
pub enum DomainKind {
    Exact(String),
    Suffix(String),
    Regex { source: String, compiled: Regex },
}

#[derive(Debug, Clone)]
pub struct DomainPattern {
    pub raw: String,
    pub kind: DomainKind,
}

#[derive(Debug, Clone)]
pub struct UnsupportedPattern {
    pub raw: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct ParsedRules {
    pub group: String,
    pub domain: Vec<DomainPattern>,
    pub unsupported: Vec<UnsupportedPattern>,
    pub fallback: Vec<String>,
    pub skipped_group: u32,
}

pub fn parse_rules(file: &RulesFile) -> ParsedRules {
    let mut parsed = ParsedRules {
        group: file.group.clone(),
        ..ParsedRules::default()
    };
    for raw in &file.rules {
        let parts: Vec<&str> = raw.split(',').collect();
        if parts.len() < 2 {
            parsed.unsupported.push(UnsupportedPattern {
                raw: raw.clone(),
                reason: "条目字段不足".into(),
            });
            continue;
        }
        let kind = parts[0].trim();
        let payload = parts[1].trim();
        let group = parts.get(2).map(|item| item.trim());
        if let Some(group) = group {
            if group != file.group {
                parsed.skipped_group += 1;
                continue;
            }
        }
        match kind {
            "DOMAIN" => parsed.domain.push(DomainPattern {
                raw: raw.clone(),
                kind: DomainKind::Exact(payload.to_ascii_lowercase()),
            }),
            "DOMAIN-SUFFIX" => parsed.domain.push(DomainPattern {
                raw: raw.clone(),
                kind: DomainKind::Suffix(payload.to_ascii_lowercase()),
            }),
            "DOMAIN-REGEX" => match Regex::new(payload) {
                Ok(compiled) => parsed.domain.push(DomainPattern {
                    raw: raw.clone(),
                    kind: DomainKind::Regex {
                        source: payload.to_string(),
                        compiled,
                    },
                }),
                Err(error) => parsed.unsupported.push(UnsupportedPattern {
                    raw: raw.clone(),
                    reason: format!("正则编译失败: {error}"),
                }),
            },
            other
                if other.starts_with("PROCESS-")
                    || other == "IP-CIDR"
                    || other == "IP-CIDR6"
                    || other == "DST-PORT"
                    || other == "AND" =>
            {
                parsed.fallback.push(raw.clone());
            }
            _ => parsed.unsupported.push(UnsupportedPattern {
                raw: raw.clone(),
                reason: format!("未知规则类型 {kind}"),
            }),
        }
    }
    parsed
}

/// 每个 host 至多归属一个模式。返回 `domain` 向量下标。
pub fn match_host(host: &str, patterns: &[DomainPattern]) -> Option<usize> {
    let host = host.to_ascii_lowercase();
    if host.is_empty() || host == "__unknown__" {
        return None;
    }
    for (index, pattern) in patterns.iter().enumerate() {
        if let DomainKind::Exact(expected) = &pattern.kind {
            if *expected == host {
                return Some(index);
            }
        }
    }
    let mut best: Option<(usize, usize)> = None;
    for (index, pattern) in patterns.iter().enumerate() {
        if let DomainKind::Suffix(suffix) = &pattern.kind {
            if host_matches_suffix(&host, suffix) {
                let len = suffix.len();
                if best.map(|(_, current)| len > current).unwrap_or(true) {
                    best = Some((index, len));
                }
            }
        }
    }
    if let Some((index, _)) = best {
        return Some(index);
    }
    for (index, pattern) in patterns.iter().enumerate() {
        if let DomainKind::Regex { compiled, .. } = &pattern.kind {
            if compiled.is_match(&host) {
                return Some(index);
            }
        }
    }
    None
}

fn host_matches_suffix(host: &str, suffix: &str) -> bool {
    host == suffix || host.ends_with(&format!(".{suffix}"))
}

pub fn pattern_key(pattern: &DomainPattern) -> String {
    match &pattern.kind {
        DomainKind::Exact(value) | DomainKind::Suffix(value) => value.clone(),
        DomainKind::Regex { source, .. } => source.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(items: &[&str]) -> ParsedRules {
        parse_rules(&RulesFile {
            schema_version: 1,
            group: "AI-家宽".into(),
            rules: items.iter().map(|item| (*item).to_string()).collect(),
        })
    }

    #[test]
    fn suffix_respects_label_boundary() {
        let parsed = rules(&["DOMAIN-SUFFIX,claude.ai,AI-家宽"]);
        assert_eq!(match_host("www.claude.ai", &parsed.domain), Some(0));
        assert_eq!(match_host("claude.ai", &parsed.domain), Some(0));
        assert_eq!(match_host("notclaude.ai", &parsed.domain), None);
    }

    #[test]
    fn vertex_regex_is_anchored_by_pattern() {
        let parsed = rules(&["DOMAIN-REGEX,^[a-z0-9-]+-aiplatform\\.googleapis\\.com$,AI-家宽"]);
        assert_eq!(
            match_host("us-central1-aiplatform.googleapis.com", &parsed.domain),
            Some(0)
        );
        assert_eq!(
            match_host("aiplatform.googleapis.com", &parsed.domain),
            None
        );
    }

    #[test]
    fn uncompilable_regex_is_unsupported() {
        let parsed = rules(&["DOMAIN-REGEX,[invalid,AI-家宽"]);
        assert!(parsed.domain.is_empty());
        assert_eq!(parsed.unsupported.len(), 1);
    }

    #[test]
    fn exact_beats_suffix() {
        let parsed = rules(&[
            "DOMAIN-SUFFIX,example.com,AI-家宽",
            "DOMAIN,api.example.com,AI-家宽",
        ]);
        assert_eq!(match_host("api.example.com", &parsed.domain), Some(1));
    }
}
