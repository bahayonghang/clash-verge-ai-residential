//! 界面外观。不写入控制器 JSON。

use serde::{Deserialize, Serialize};

pub const THEME_SETTING_KEY: &str = "ui_theme";
pub const FONT_SETTING_KEY: &str = "ui_font";
pub const FONT_SIZE_SETTING_KEY: &str = "ui_font_size";
pub const DENSITY_SETTING_KEY: &str = "ui_density";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum UiFont {
    #[default]
    System,
    Yahei,
    Serif,
    Mono,
}

impl UiFont {
    pub fn parse(raw: Option<&str>) -> Self {
        match raw {
            Some("yahei") => Self::Yahei,
            Some("serif") => Self::Serif,
            Some("mono") => Self::Mono,
            Some("system") => Self::System,
            _ => Self::System,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Yahei => "yahei",
            Self::Serif => "serif",
            Self::Mono => "mono",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum UiFontSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl UiFontSize {
    pub fn parse(raw: Option<&str>) -> Self {
        match raw {
            Some("sm") => Self::Sm,
            Some("lg") => Self::Lg,
            Some("md") => Self::Md,
            _ => Self::Md,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum UiDensity {
    #[default]
    Comfortable,
    Compact,
}

impl UiDensity {
    pub fn parse(raw: Option<&str>) -> Self {
        match raw {
            Some("compact") => Self::Compact,
            Some("comfortable") => Self::Comfortable,
            _ => Self::Comfortable,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Comfortable => "comfortable",
            Self::Compact => "compact",
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

    #[test]
    fn font_size_and_density_fall_back_to_defaults() {
        assert_eq!(UiFont::parse(None), UiFont::System);
        assert_eq!(UiFont::parse(Some("Comic Sans")), UiFont::System);
        assert_eq!(UiFont::parse(Some("yahei")), UiFont::Yahei);
        assert_eq!(UiFontSize::parse(None), UiFontSize::Md);
        assert_eq!(UiFontSize::parse(Some("20")), UiFontSize::Md);
        assert_eq!(UiFontSize::parse(Some("sm")), UiFontSize::Sm);
        assert_eq!(UiDensity::parse(None), UiDensity::Comfortable);
        assert_eq!(UiDensity::parse(Some("tight")), UiDensity::Comfortable);
        assert_eq!(UiDensity::parse(Some("compact")), UiDensity::Compact);
    }
}
