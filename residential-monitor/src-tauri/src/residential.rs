//! 家宽判定：一个模块，两个口径。
//!
//! - **核算口径** [`residential_tags`] / [`is_residential_target`]：链路节点精确等于某个已配置
//!   target。这是写入 `connection_session_attr.primary_category_id` 的判据，也决定
//!   `LiveOverview.categoryUpload/Download` 的键与 `DimensionKind::Category` 的分类值。
//! - **实时筛选口径** [`is_residential_filter`]：精确 target 匹配，**或**节点名包含「家宽」。
//!   比核算口径宽。
//!
//! 两者不能合并。合并到精确匹配会让实时页「只看家宽」少选中以前靠子串命中的连接；
//! 合并到含启发式会把中文子串写进 `primary_category_id` 并改变历史分类归属。
//! 收敛到本模块是为了停止两处各自漂移，不改已发布行为。

/// 核算口径：按 target 列表顺序，收集链路中精确命中的 target 名。
pub fn residential_tags(targets: &[String], chains: &[String]) -> Vec<String> {
    targets
        .iter()
        .filter(|target| chains.iter().any(|node| node == *target))
        .cloned()
        .collect()
}

/// 核算口径：是否命中任一已配置 target。
pub fn is_residential_target(targets: &[String], chains: &[String]) -> bool {
    targets
        .iter()
        .any(|target| chains.iter().any(|node| node == target))
}

/// 实时筛选口径：精确 target 匹配，或节点名包含「家宽」。
pub fn is_residential_filter(targets: &[String], chains: &[String]) -> bool {
    chains
        .iter()
        .any(|node| targets.iter().any(|target| target == node) || node.contains("家宽"))
}

#[cfg(test)]
mod residential_caliber_tests {
    use super::*;

    fn s(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    /// 改造前 `c2/query.rs` 的 `is_residential` 原文。对比测试用，不作为产品路径。
    fn legacy_is_residential_filter(targets: &[String], chains: &[String]) -> bool {
        chains
            .iter()
            .any(|node| targets.iter().any(|target| target == node) || node.contains("家宽"))
    }

    #[test]
    fn exact_hit() {
        let targets = s(&["家宽-SOCKS5"]);
        let chains = s(&["AI-家宽", "家宽-SOCKS5"]);
        assert_eq!(residential_tags(&targets, &chains), s(&["家宽-SOCKS5"]));
        assert!(is_residential_target(&targets, &chains));
        assert!(is_residential_filter(&targets, &chains));
    }

    #[test]
    fn substring_home_but_not_target() {
        let targets = s(&["节点A"]);
        let chains = s(&["AI-家宽"]);
        assert!(residential_tags(&targets, &chains).is_empty());
        assert!(!is_residential_target(&targets, &chains));
        assert!(is_residential_filter(&targets, &chains));
    }

    #[test]
    fn no_hit() {
        let targets = s(&["家宽-SOCKS5"]);
        let chains = s(&["DIRECT"]);
        assert!(residential_tags(&targets, &chains).is_empty());
        assert!(!is_residential_target(&targets, &chains));
        assert!(!is_residential_filter(&targets, &chains));
    }

    #[test]
    fn empty_targets() {
        let targets = s(&[]);
        let chains = s(&["AI-家宽", "家宽-SOCKS5"]);
        assert!(residential_tags(&targets, &chains).is_empty());
        assert!(!is_residential_target(&targets, &chains));
        assert!(is_residential_filter(&targets, &chains));
    }

    #[test]
    fn filter_selected_set_matches_legacy() {
        let samples = [
            (s(&["家宽-SOCKS5"]), s(&["AI-家宽", "家宽-SOCKS5"])),
            (s(&["节点A"]), s(&["AI-家宽"])),
            (s(&["家宽-SOCKS5"]), s(&["DIRECT"])),
            (s(&[]), s(&["AI-家宽", "家宽-SOCKS5"])),
            (s(&["家宽"]), s(&["PROXY", "家宽"])),
            (s(&["家宽"]), s(&["PROXY>家宽节点"])),
            (s(&["A", "B"]), s(&["B", "C"])),
            (s(&["A"]), s(&[])),
        ];
        for (targets, chains) in &samples {
            assert_eq!(
                is_residential_filter(targets, chains),
                legacy_is_residential_filter(targets, chains),
                "targets={targets:?} chains={chains:?}"
            );
        }
    }
}
