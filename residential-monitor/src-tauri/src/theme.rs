//! 界面外观。不写入控制器 JSON。

use serde::{Deserialize, Serialize};

pub const THEME_SETTING_KEY: &str = "ui_theme";
pub const FONT_SETTING_KEY: &str = "ui_font";
pub const FONT_SIZE_SETTING_KEY: &str = "ui_font_size";
pub const DENSITY_SETTING_KEY: &str = "ui_density";

const FONT_FAMILY_MAX_UNITS: usize = 31;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiFont(String);

impl Default for UiFont {
    fn default() -> Self {
        Self::system()
    }
}

impl UiFont {
    pub fn system() -> Self {
        Self("system".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parse(raw: Option<&str>) -> Self {
        let Some(name) = raw.map(str::trim).filter(|name| !name.is_empty()) else {
            return Self::system();
        };
        match name {
            "system" | "yahei" | "serif" | "mono" => Self(name.to_string()),
            _ if is_family_name(name) => Self(name.to_string()),
            _ => Self::system(),
        }
    }
}

pub fn is_family_name(name: &str) -> bool {
    if name.is_empty() || name.starts_with('@') {
        return false;
    }
    if name.encode_utf16().count() > FONT_FAMILY_MAX_UNITS {
        return false;
    }
    !name
        .chars()
        .any(|ch| ch.is_control() || matches!(ch, '"' | '\'' | ';' | '{' | '}' | '<' | '>' | '\\'))
}

pub fn list_installed_families() -> Result<Vec<String>, &'static str> {
    #[cfg(windows)]
    {
        list_installed_families_windows()
    }
    #[cfg(not(windows))]
    {
        Ok(Vec::new())
    }
}

#[cfg(windows)]
fn list_installed_families_windows() -> Result<Vec<String>, &'static str> {
    use std::os::windows::ffi::OsStringExt;
    use std::ptr;
    use windows_sys::Win32::Graphics::Gdi::{
        CreateDCW, DeleteDC, EnumFontFamiliesExW, DEFAULT_CHARSET, LOGFONTW, TEXTMETRICW,
    };

    unsafe extern "system" fn enum_font_proc(
        lplf: *const LOGFONTW,
        _metrics: *const TEXTMETRICW,
        _font_type: u32,
        lparam: windows_sys::Win32::Foundation::LPARAM,
    ) -> i32 {
        if lplf.is_null() || lparam == 0 {
            return 1;
        }
        let face = unsafe { (*lplf).lfFaceName };
        let end = face
            .iter()
            .position(|&unit| unit == 0)
            .unwrap_or(face.len());
        if end == 0 || face[0] == u16::from(b'@') {
            return 1;
        }
        let name = std::ffi::OsString::from_wide(&face[..end]);
        let Some(name) = name.to_str() else {
            return 1;
        };
        let names = unsafe { &mut *(lparam as *mut Vec<String>) };
        names.push(name.to_string());
        1
    }

    let driver: Vec<u16> = "DISPLAY".encode_utf16().chain(std::iter::once(0)).collect();
    let hdc = unsafe { CreateDCW(driver.as_ptr(), ptr::null(), ptr::null(), ptr::null()) };
    if hdc.is_null() {
        return Err("display-dc");
    }
    let logfont = LOGFONTW {
        lfCharSet: DEFAULT_CHARSET,
        ..LOGFONTW::default()
    };
    let mut names: Vec<String> = Vec::new();
    let enumerated = unsafe {
        EnumFontFamiliesExW(
            hdc,
            &logfont,
            Some(enum_font_proc),
            &mut names as *mut Vec<String> as isize,
            0,
        )
    };
    unsafe {
        DeleteDC(hdc);
    }
    if enumerated == 0 && names.is_empty() {
        return Err("enum-fonts");
    }
    names.retain(|name| is_family_name(name));
    names.sort_by_key(|left| left.to_lowercase());
    names.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    if names.is_empty() {
        return Err("enum-fonts");
    }
    Ok(names)
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
    fn font_accepts_legacy_aliases_and_safe_families() {
        assert_eq!(UiFont::parse(None), UiFont::system());
        assert_eq!(UiFont::parse(Some("yahei")).as_str(), "yahei");
        assert_eq!(UiFont::parse(Some("serif")).as_str(), "serif");
        assert_eq!(UiFont::parse(Some("mono")).as_str(), "mono");
        assert_eq!(
            UiFont::parse(Some("Microsoft YaHei")).as_str(),
            "Microsoft YaHei"
        );
        assert_eq!(UiFont::parse(Some("Comic Sans")).as_str(), "Comic Sans");
        assert_eq!(UiFont::parse(Some("nope;")).as_str(), "system");
        assert_eq!(UiFont::parse(Some("@SimSun")).as_str(), "system");
        assert_eq!(UiFont::parse(Some(&"a".repeat(32))).as_str(), "system");
        assert_eq!(UiFont::parse(Some("")).as_str(), "system");
    }

    #[test]
    fn font_size_and_density_fall_back_to_defaults() {
        assert_eq!(UiFontSize::parse(None), UiFontSize::Md);
        assert_eq!(UiFontSize::parse(Some("20")), UiFontSize::Md);
        assert_eq!(UiFontSize::parse(Some("sm")), UiFontSize::Sm);
        assert_eq!(UiDensity::parse(None), UiDensity::Comfortable);
        assert_eq!(UiDensity::parse(Some("tight")), UiDensity::Comfortable);
        assert_eq!(UiDensity::parse(Some("compact")), UiDensity::Compact);
    }

    #[cfg(windows)]
    #[test]
    fn lists_installed_families_without_vertical_faces() {
        let names = list_installed_families().expect("list");
        assert!(!names.is_empty());
        assert!(names.iter().all(|name| is_family_name(name)));
        let mut lower: Vec<String> = names.iter().map(|name| name.to_lowercase()).collect();
        let original = lower.clone();
        lower.sort();
        lower.dedup();
        assert_eq!(lower, original);
        assert!(names
            .iter()
            .any(|name| name.eq_ignore_ascii_case("Segoe UI")
                || name.contains("YaHei")
                || name.contains("雅黑")));
    }
}
