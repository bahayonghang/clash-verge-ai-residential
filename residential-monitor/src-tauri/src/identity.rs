//! 发布后不得修改的稳定标识。

pub const IDENTIFIER: &str = "io.github.bahayonghang.residential-monitor";
pub const PRODUCT_NAME: &str = "家宽流量监控";
pub const BINARY_NAME: &str = "residential-monitor";
pub const AUMID: &str = IDENTIFIER;
pub const AUTOSTART_ARGUMENT: &str = "--background";
pub const RELEASES_URL: &str =
    "https://github.com/bahayonghang/clash-verge-ai-residential/releases";
pub const CREDENTIAL_TARGET: &str = "io.github.bahayonghang.residential-monitor/controller";
pub const CREDENTIAL_SPIKE_TARGET: &str =
    "io.github.bahayonghang.residential-monitor/c0-spike-test";
pub const DELETE_CONFIRM_PHRASE: &str = "删除全部本地数据";

pub fn local_app_data_leaf() -> &'static str {
    IDENTIFIER
}

#[cfg(test)]
mod windows_identity_tests {
    use super::*;

    #[test]
    fn windows_identity_is_stable() {
        assert_eq!(IDENTIFIER, "io.github.bahayonghang.residential-monitor");
        assert_eq!(BINARY_NAME, "residential-monitor");
        assert_eq!(AUMID, IDENTIFIER);
        assert_eq!(AUTOSTART_ARGUMENT, "--background");
        assert!(RELEASES_URL.starts_with("https://github.com/"));
        assert!(!RELEASES_URL.contains("secret"));
        assert!(CREDENTIAL_TARGET.starts_with(IDENTIFIER));
        assert_eq!(local_app_data_leaf(), IDENTIFIER);
        assert_eq!(PRODUCT_NAME, "家宽流量监控");
    }
}
