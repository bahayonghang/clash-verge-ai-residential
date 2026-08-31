//! 家宽目标匹配的唯一产品语义。
//!
//! - target 精确为 [`RESIDENTIAL_SELECTOR`] 时，匹配包含该语义词的链路节点；
//! - 其它自定义 target 只做节点全值精确匹配；
//! - 未配置 target 时不匹配任何连接。
//!
//! 实时筛选、未来核算写入与 raw 历史恢复都必须遵守这套语义。tags 保持 target
//! 配置顺序，首个命中 target 因而稳定地成为 primary category。

/// 产品内置的语义 target。SQL raw 恢复谓词必须与这个常量保持行为一致。
pub const RESIDENTIAL_SELECTOR: &str = "家宽";

fn target_matches_node(target: &str, node: &str) -> bool {
    if target == RESIDENTIAL_SELECTOR {
        node.contains(RESIDENTIAL_SELECTOR)
    } else {
        node == target
    }
}

/// 按 target 配置顺序收集命中的 target 名；同一 target 最多返回一次。
pub fn residential_tags(targets: &[String], chains: &[String]) -> Vec<String> {
    targets
        .iter()
        .filter(|target| chains.iter().any(|node| target_matches_node(target, node)))
        .cloned()
        .collect()
}

/// 是否命中任一已配置 target。
pub fn is_residential_target(targets: &[String], chains: &[String]) -> bool {
    targets
        .iter()
        .any(|target| chains.iter().any(|node| target_matches_node(target, node)))
}

/// C2 实时筛选沿用共享产品语义，不再维护更宽的隐式口径。
pub fn is_residential_filter(targets: &[String], chains: &[String]) -> bool {
    is_residential_target(targets, chains)
}

#[cfg(test)]
mod residential_matcher_tests {
    use super::*;

    fn s(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn semantic_selector_matches_containing_nodes() {
        let targets = s(&[RESIDENTIAL_SELECTOR]);
        for chains in [s(&["家宽"]), s(&["AI-家宽"]), s(&["PROXY>家宽节点"])] {
            assert_eq!(residential_tags(&targets, &chains), targets);
            assert!(is_residential_target(&targets, &chains));
            assert!(is_residential_filter(&targets, &chains));
        }
    }

    #[test]
    fn custom_targets_remain_exact() {
        let targets = s(&["家宽-SOCKS5", "Residential-US #1"]);
        assert!(residential_tags(&targets, &s(&["AI-家宽-SOCKS5"])).is_empty());
        assert_eq!(
            residential_tags(&targets, &s(&["Residential-US #1"])),
            s(&["Residential-US #1"])
        );
    }

    #[test]
    fn target_order_and_duplicate_chain_hits_are_stable() {
        let targets = s(&["备用", RESIDENTIAL_SELECTOR, "DIRECT"]);
        let chains = s(&["AI-家宽", "家宽-SOCKS5", "备用", "家宽"]);
        assert_eq!(
            residential_tags(&targets, &chains),
            s(&["备用", RESIDENTIAL_SELECTOR])
        );
    }

    #[test]
    fn empty_targets_match_nothing() {
        let chains = s(&["AI-家宽", "家宽-SOCKS5"]);
        assert!(residential_tags(&[], &chains).is_empty());
        assert!(!is_residential_target(&[], &chains));
        assert!(!is_residential_filter(&[], &chains));
    }

    #[test]
    fn unrelated_and_special_character_targets_are_not_guessed() {
        let targets = s(&["节点[甲] / 東京", "DIRECT"]);
        assert!(!is_residential_target(
            &targets,
            &s(&["节点[甲]", "REJECT"])
        ));
        assert!(is_residential_target(&targets, &s(&["节点[甲] / 東京"])));
    }
}
