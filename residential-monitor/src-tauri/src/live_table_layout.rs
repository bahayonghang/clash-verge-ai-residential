//! 实时表列宽与显隐。不写入控制器 JSON。

use crate::c2::contract::SETTING_VALUE_MAX;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

pub const LAYOUT_SETTING_KEY: &str = "live_table_layout";
pub const WIDTH_MAX: u32 = 640;
pub const ACTION_WIDTH: u32 = 76;
pub const ACTION_COLUMN: &str = "action";

pub const DATA_COLUMNS: [&str; 12] = [
    "host",
    "download",
    "upload",
    "rateDownload",
    "rateUpload",
    "chain",
    "rule",
    "process",
    "duration",
    "source",
    "destination",
    "type",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveTableLayout {
    #[serde(default)]
    pub widths: BTreeMap<String, u32>,
    #[serde(default)]
    pub hidden: Vec<String>,
}

impl Default for LiveTableLayout {
    fn default() -> Self {
        sanitize_layout(LiveTableLayout {
            widths: BTreeMap::new(),
            hidden: Vec::new(),
        })
    }
}

pub fn default_width(column: &str) -> u32 {
    match column {
        "host" => 180,
        "download" | "upload" => 88,
        "rateDownload" | "rateUpload" => 104,
        "chain" => 280,
        "rule" => 220,
        "process" => 180,
        "duration" => 108,
        "source" | "destination" => 160,
        "type" => 120,
        ACTION_COLUMN => ACTION_WIDTH,
        _ => 120,
    }
}

pub fn min_width(column: &str) -> u32 {
    match column {
        "host" => 140,
        "download" | "upload" => 72,
        "rateDownload" | "rateUpload" => 80,
        "chain" | "rule" => 160,
        "process" => 100,
        "duration" => 88,
        "source" | "destination" => 120,
        "type" => 80,
        ACTION_COLUMN => ACTION_WIDTH,
        _ => 72,
    }
}

fn is_data_column(column: &str) -> bool {
    DATA_COLUMNS.contains(&column)
}

pub fn sanitize_layout(input: LiveTableLayout) -> LiveTableLayout {
    let mut widths = BTreeMap::new();
    for column in DATA_COLUMNS {
        let raw = input
            .widths
            .get(column)
            .copied()
            .unwrap_or_else(|| default_width(column));
        let min = min_width(column);
        widths.insert(column.to_string(), raw.clamp(min, WIDTH_MAX));
    }
    let mut hidden = Vec::new();
    let mut seen = HashSet::new();
    for column in input.hidden {
        if !is_data_column(&column) || !seen.insert(column.clone()) {
            continue;
        }
        if seen.len() >= DATA_COLUMNS.len() {
            seen.remove(&column);
            continue;
        }
        hidden.push(column);
    }
    LiveTableLayout { widths, hidden }
}

pub fn parse_setting(raw: Option<&str>) -> LiveTableLayout {
    let Some(text) = raw else {
        return LiveTableLayout::default();
    };
    if text.len() > SETTING_VALUE_MAX {
        return LiveTableLayout::default();
    }
    let Ok(parsed) = serde_json::from_str::<LiveTableLayout>(text) else {
        return LiveTableLayout::default();
    };
    sanitize_layout(parsed)
}

pub fn encode_setting(layout: &LiveTableLayout) -> Option<String> {
    let encoded = serde_json::to_string(layout).ok()?;
    if encoded.len() > SETTING_VALUE_MAX {
        return None;
    }
    Some(encoded)
}

#[cfg(test)]
mod live_table_layout_tests {
    use super::*;

    #[test]
    fn default_shows_all_data_columns() {
        let layout = LiveTableLayout::default();
        assert_eq!(layout.widths.len(), DATA_COLUMNS.len());
        assert!(layout.hidden.is_empty());
        assert_eq!(layout.widths.get("host"), Some(&180));
        assert_eq!(layout.widths.get("rateDownload"), Some(&104));
        assert!(!layout.widths.contains_key(ACTION_COLUMN));
    }

    #[test]
    fn drops_unknown_and_action_keys() {
        let mut widths = BTreeMap::new();
        widths.insert("action".into(), 40);
        widths.insert("nope".into(), 200);
        widths.insert("host".into(), 200);
        let layout = sanitize_layout(LiveTableLayout {
            widths,
            hidden: vec!["action".into(), "nope".into(), "host".into()],
        });
        assert_eq!(layout.widths.get("host"), Some(&200));
        assert!(!layout.widths.contains_key("nope"));
        assert!(!layout.widths.contains_key(ACTION_COLUMN));
        assert_eq!(layout.hidden, vec!["host".to_string()]);
    }

    #[test]
    fn clamps_width() {
        let mut widths = BTreeMap::new();
        widths.insert("host".into(), 10);
        widths.insert("download".into(), 9000);
        let layout = sanitize_layout(LiveTableLayout {
            widths,
            hidden: Vec::new(),
        });
        assert_eq!(layout.widths.get("host"), Some(&140));
        assert_eq!(layout.widths.get("download"), Some(&640));
    }

    #[test]
    fn keeps_one_data_column_visible() {
        let layout = sanitize_layout(LiveTableLayout {
            widths: BTreeMap::new(),
            hidden: DATA_COLUMNS
                .iter()
                .map(|item| (*item).to_string())
                .collect(),
        });
        assert_eq!(layout.hidden.len(), DATA_COLUMNS.len() - 1);
        let visible: Vec<_> = DATA_COLUMNS
            .iter()
            .filter(|column| !layout.hidden.iter().any(|hidden| hidden == *column))
            .collect();
        assert_eq!(visible.len(), 1);
    }

    #[test]
    fn invalid_or_overlong_setting_falls_back() {
        assert_eq!(parse_setting(None), LiveTableLayout::default());
        assert_eq!(parse_setting(Some("not-json")), LiveTableLayout::default());
        let huge = "x".repeat(SETTING_VALUE_MAX + 1);
        assert_eq!(parse_setting(Some(&huge)), LiveTableLayout::default());
    }

    #[test]
    fn encode_fits_setting_limit() {
        let encoded = encode_setting(&LiveTableLayout::default()).expect("encode");
        assert!(encoded.len() < SETTING_VALUE_MAX);
    }
}
