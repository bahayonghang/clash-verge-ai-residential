//! 关于页与发布身份。不注册 updater，不声称已签名。

use crate::identity::{AUMID, BINARY_NAME, IDENTIFIER, PRODUCT_NAME, RELEASES_URL};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AboutDto {
    pub schema_version: u32,
    pub product_name: &'static str,
    pub binary_name: &'static str,
    pub identifier: &'static str,
    pub aumid: &'static str,
    pub version: &'static str,
    pub releases_url: &'static str,
    pub signed: bool,
    pub updater_plugin: bool,
    pub windows_service: bool,
    pub signature_note_zh: &'static str,
}

pub fn about() -> AboutDto {
    AboutDto {
        schema_version: 1,
        product_name: PRODUCT_NAME,
        binary_name: BINARY_NAME,
        identifier: IDENTIFIER,
        aumid: AUMID,
        version: env!("CARGO_PKG_VERSION"),
        releases_url: RELEASES_URL,
        signed: false,
        updater_plugin: false,
        windows_service: false,
        signature_note_zh:
            "本候选未做 Authenticode 签名。发布前须签名或由发布负责人对具体资产哈希批准未签名例外。",
    }
}

#[cfg(test)]
mod about_tests {
    use super::*;

    #[test]
    fn about_is_unsigned_and_has_stable_url() {
        let dto = about();
        assert_eq!(dto.schema_version, 1);
        assert!(!dto.signed);
        assert!(!dto.updater_plugin);
        assert!(!dto.windows_service);
        assert_eq!(dto.aumid, IDENTIFIER);
        assert_eq!(dto.releases_url, RELEASES_URL);
        assert!(!dto.signature_note_zh.contains("已签名"));
        assert!(!dto.releases_url.contains("secret"));
    }
}
