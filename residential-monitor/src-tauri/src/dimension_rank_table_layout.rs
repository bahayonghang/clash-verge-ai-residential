//! 维度排名表列宽。不写入控制器 JSON，不复用 `live_table_layout`。

use crate::c2::contract::SETTING_VALUE_MAX;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const LAYOUT_SETTING_KEY: &str = "dimension_rank_table_layout";
pub const WIDTH_MIN: u32 = 48;
pub const WIDTH_MAX: u32 = 640;

pub const DATA_COLUMNS: [&str; 6] = [
    "name",
    "upload",
    "download",
    "connections",
    "share",
    "attribution",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionRankTableLayout {
    #[serde(default)]
    pub widths: BTreeMap<String, u32>,
}

impl Default for DimensionRankTableLayout {
    fn default() -> Self {
        sanitize_layout(DimensionRankTableLayout {
            widths: BTreeMap::new(),
        })
    }
}

pub fn default_width(column: &str) -> u32 {
    match column {
        "name" => 280,
        "upload" | "download" => 88,
        "connections" => 72,
        "share" => 64,
        "attribution" => 160,
        _ => 88,
    }
}

pub fn sanitize_layout(input: DimensionRankTableLayout) -> DimensionRankTableLayout {
    let mut widths = BTreeMap::new();
    for column in DATA_COLUMNS {
        let raw = input
            .widths
            .get(column)
            .copied()
            .unwrap_or_else(|| default_width(column));
        widths.insert(column.to_string(), raw.clamp(WIDTH_MIN, WIDTH_MAX));
    }
    DimensionRankTableLayout { widths }
}

pub fn parse_setting(raw: Option<&str>) -> DimensionRankTableLayout {
    let Some(text) = raw else {
        return DimensionRankTableLayout::default();
    };
    if text.len() > SETTING_VALUE_MAX {
        return DimensionRankTableLayout::default();
    }
    let Ok(parsed) = serde_json::from_str::<DimensionRankTableLayout>(text) else {
        return DimensionRankTableLayout::default();
    };
    sanitize_layout(parsed)
}

pub fn encode_setting(layout: &DimensionRankTableLayout) -> Option<String> {
    let encoded = serde_json::to_string(layout).ok()?;
    if encoded.len() > SETTING_VALUE_MAX {
        return None;
    }
    Some(encoded)
}

#[cfg(test)]
mod dimension_rank_table_layout_tests {
    use super::*;

    #[test]
    fn default_fills_data_columns_only() {
        let layout = DimensionRankTableLayout::default();
        assert_eq!(layout.widths.len(), DATA_COLUMNS.len());
        assert_eq!(layout.widths.get("name"), Some(&280));
        assert_eq!(layout.widths.get("upload"), Some(&88));
        assert_eq!(layout.widths.get("download"), Some(&88));
        assert_eq!(layout.widths.get("connections"), Some(&72));
        assert_eq!(layout.widths.get("share"), Some(&64));
        assert_eq!(layout.widths.get("attribution"), Some(&160));
        assert!(!layout.widths.contains_key("rank"));
        assert!(!layout.widths.contains_key("drill"));
    }

    #[test]
    fn drops_unknown_rank_and_drill_keys() {
        let mut widths = BTreeMap::new();
        widths.insert("rank".into(), 40);
        widths.insert("drill".into(), 90);
        widths.insert("nope".into(), 200);
        widths.insert("name".into(), 300);
        let layout = sanitize_layout(DimensionRankTableLayout { widths });
        assert_eq!(layout.widths.get("name"), Some(&300));
        assert!(!layout.widths.contains_key("rank"));
        assert!(!layout.widths.contains_key("drill"));
        assert!(!layout.widths.contains_key("nope"));
    }

    #[test]
    fn clamps_width() {
        let mut widths = BTreeMap::new();
        widths.insert("name".into(), 10);
        widths.insert("download".into(), 9000);
        let layout = sanitize_layout(DimensionRankTableLayout { widths });
        assert_eq!(layout.widths.get("name"), Some(&WIDTH_MIN));
        assert_eq!(layout.widths.get("download"), Some(&WIDTH_MAX));
    }

    #[test]
    fn missing_columns_get_defaults() {
        let mut widths = BTreeMap::new();
        widths.insert("name".into(), 200);
        let layout = sanitize_layout(DimensionRankTableLayout { widths });
        assert_eq!(layout.widths.get("name"), Some(&200));
        assert_eq!(layout.widths.get("attribution"), Some(&160));
        assert_eq!(layout.widths.get("share"), Some(&64));
    }

    #[test]
    fn invalid_or_overlong_setting_falls_back() {
        assert_eq!(parse_setting(None), DimensionRankTableLayout::default());
        assert_eq!(
            parse_setting(Some("not-json")),
            DimensionRankTableLayout::default()
        );
        let huge = "x".repeat(SETTING_VALUE_MAX + 1);
        assert_eq!(
            parse_setting(Some(&huge)),
            DimensionRankTableLayout::default()
        );
    }

    #[test]
    fn encode_fits_setting_limit() {
        let encoded = encode_setting(&DimensionRankTableLayout::default()).expect("encode");
        assert!(encoded.len() < SETTING_VALUE_MAX);
    }
}
