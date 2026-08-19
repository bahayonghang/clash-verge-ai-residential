//! 日志与诊断共用的禁止子串扫描。

pub const FORBIDDEN: &[&str] = &[
    "bearer ",
    "password=",
    "secret=",
    "authorization:",
    "credential",
];

pub fn scan_text_for_secrets(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    FORBIDDEN.iter().any(|item| lower.contains(item))
}

#[cfg(test)]
mod redact_tests {
    use super::*;

    #[test]
    fn detects_forbidden_substrings() {
        assert!(scan_text_for_secrets("Authorization: Bearer abc"));
        assert!(scan_text_for_secrets("password=secret"));
        assert!(scan_text_for_secrets("secret=1"));
        assert!(!scan_text_for_secrets("storage_open class=sqlite"));
    }
}
