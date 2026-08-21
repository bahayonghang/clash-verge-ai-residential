//! 规则 / 链路派生键。判定只在本模块。
//!
//! SQL 侧规则键（见 `RULE_KEY_SQL`）与 `build_rule_name` 在「单跳且有 payload」
//! 时不同：Rust 函数给出 `rule(payload)`，SQL 只给原始 `rule`（`connection_session_attr`
//! 不存 payload）。聚合路径必须用 SQL 定义，不得调用 `build_rule_name`。

use rusqlite::functions::FunctionFlags;
use rusqlite::Connection;

/// 顶层策略组：`chain_key` 形如 `a>b>c`，取最后一段并 trim。
/// 无 `>` 视为单跳，返回 None（调用方回退到 rule(payload)）。
pub fn last_chain_hop(chain_key: Option<&str>) -> Option<String> {
    let raw = chain_key?.trim();
    if raw.is_empty() || !raw.contains('>') {
        return None;
    }
    let last = raw.rsplit('>').next()?.trim();
    if last.is_empty() {
        None
    } else {
        Some(last.to_string())
    }
}

/// 与 neko `buildRuleName` 等价，仅用于单测与展示。
///
/// - `chains.len() > 1` 且最后一跳非空 → 最后一跳
/// - `rule` 非空 → `rule(payload)` 或 `rule`
/// - 否则 → 单跳 hop 或 `"DIRECT"`
///
/// 聚合路径不得调用本函数。
pub fn build_rule_name(
    rule: Option<&str>,
    payload: Option<&str>,
    chain_key: Option<&str>,
) -> String {
    if let Some(hop) = last_chain_hop(chain_key) {
        return hop;
    }
    let rule = rule.map(str::trim).unwrap_or("");
    if !rule.is_empty() {
        let payload = payload.map(str::trim).unwrap_or("");
        return if payload.is_empty() {
            rule.to_string()
        } else {
            format!("{rule}({payload})")
        };
    }
    single_hop_or_direct(chain_key)
}

fn single_hop_or_direct(chain_key: Option<&str>) -> String {
    let Some(raw) = chain_key.map(str::trim).filter(|item| !item.is_empty()) else {
        return "DIRECT".into();
    };
    if !raw.contains('>') {
        return raw.to_string();
    }
    "DIRECT".into()
}

/// 在 `apply_required_pragmas` 之后注册。每个连接都要调用。
pub fn register_last_chain_hop(connection: &Connection) -> rusqlite::Result<()> {
    connection.create_scalar_function(
        "last_chain_hop",
        1,
        FunctionFlags::SQLITE_UTF8
            | FunctionFlags::SQLITE_DETERMINISTIC
            | FunctionFlags::SQLITE_INNOCUOUS,
        |ctx| {
            let input: Option<String> = ctx.get(0)?;
            Ok(last_chain_hop(input.as_deref()))
        },
    )
}

#[cfg(test)]
mod rule_name_tests {
    use super::*;
    use crate::storage::{open_interruptible_reader, StorageCoordinator};
    use tempfile::tempdir;

    #[test]
    fn build_rule_name_covers_required_samples() {
        assert_eq!(
            build_rule_name(Some("RuleSet"), Some("ai"), Some("DIRECT>PROXY>家宽")),
            "家宽"
        );
        assert_eq!(
            build_rule_name(Some("Match"), Some("1.2.3.4"), Some("DIRECT")),
            "Match(1.2.3.4)"
        );
        assert_eq!(
            build_rule_name(Some("IPCIDR"), Some("10.0.0.0/8"), None),
            "IPCIDR(10.0.0.0/8)"
        );
        assert_eq!(build_rule_name(None, None, None), "DIRECT");
        assert_eq!(
            build_rule_name(Some("RuleSet"), None, Some("  PROXY>家宽  ")),
            "家宽"
        );
    }

    #[test]
    fn last_chain_hop_trims_and_requires_separator() {
        assert_eq!(last_chain_hop(None), None);
        assert_eq!(last_chain_hop(Some("DIRECT")), None);
        assert_eq!(last_chain_hop(Some("a>b>c")).as_deref(), Some("c"));
        assert_eq!(
            last_chain_hop(Some("  PROXY>家宽  ")).as_deref(),
            Some("家宽")
        );
        assert_eq!(last_chain_hop(Some("a>")), None);
    }

    #[test]
    fn storage_coordinator_connection_can_execute_last_chain_hop() {
        let dir = tempdir().expect("dir");
        let coordinator = StorageCoordinator::open(&dir.path().join("fn.sqlite3")).expect("open");
        let hop: Option<String> = coordinator
            .connection()
            .query_row("select last_chain_hop('a>b>c')", [], |row| row.get(0))
            .expect("scalar");
        assert_eq!(hop.as_deref(), Some("c"));
        let reader = open_interruptible_reader(coordinator.path()).expect("reader");
        let hop: Option<String> = reader
            .query_row("select last_chain_hop(?1)", ["PROXY>家宽"], |row| {
                row.get(0)
            })
            .expect("reader scalar");
        assert_eq!(hop.as_deref(), Some("家宽"));
    }
}
