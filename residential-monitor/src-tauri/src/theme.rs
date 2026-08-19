//! 界面外观。不写入控制器 JSON。

use serde::{Deserialize, Serialize};

pub const THEME_SETTING_KEY: &str = "ui_theme";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum UiTheme {
    Latte,
    Frappe,
    Macchiato,
    #[default]
    Mocha,
}

impl UiTheme {
    pub fn parse(raw: Option<&str>) -> Self {
        match raw {
            Some("latte") => Self::Latte,
            Some("frappe") => Self::Frappe,
            Some("macchiato") => Self::Macchiato,
            Some("mocha") => Self::Mocha,
            _ => Self::Mocha,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Latte => "latte",
            Self::Frappe => "frappe",
            Self::Macchiato => "macchiato",
            Self::Mocha => "mocha",
        }
    }
}

#[cfg(test)]
mod theme_tests {
    use super::*;

    #[test]
    fn parse_falls_back_to_mocha() {
        assert_eq!(UiTheme::parse(None), UiTheme::Mocha);
        assert_eq!(UiTheme::parse(Some("")), UiTheme::Mocha);
        assert_eq!(UiTheme::parse(Some("dark")), UiTheme::Mocha);
        assert_eq!(UiTheme::parse(Some(" mocha ")), UiTheme::Mocha);
        assert_eq!(UiTheme::parse(Some("latte")), UiTheme::Latte);
        assert_eq!(UiTheme::parse(Some("frappe")), UiTheme::Frappe);
        assert_eq!(UiTheme::parse(Some("macchiato")), UiTheme::Macchiato);
        assert_eq!(UiTheme::parse(Some("mocha")), UiTheme::Mocha);
    }
}
