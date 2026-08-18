//! 发布后不得修改的稳定标识。

pub const IDENTIFIER: &str = "io.github.bahayonghang.residential-monitor";
pub const PRODUCT_NAME: &str = "家宽流量监控";
pub const BINARY_NAME: &str = "residential-monitor";
pub const CREDENTIAL_TARGET: &str = "io.github.bahayonghang.residential-monitor/controller";
pub const CREDENTIAL_SPIKE_TARGET: &str =
    "io.github.bahayonghang.residential-monitor/c0-spike-test";

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
        assert!(CREDENTIAL_TARGET.starts_with(IDENTIFIER));
        assert_eq!(local_app_data_leaf(), IDENTIFIER);
        assert_eq!(PRODUCT_NAME, "家宽流量监控");
    }
}
