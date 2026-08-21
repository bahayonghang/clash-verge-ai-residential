//! 主机 identity：`host` → `sniffHost` → 目的 IP。

use std::net::IpAddr;

pub fn looks_like_ip(value: &str) -> bool {
    let trimmed = value.trim();
    let trimmed = trimmed
        .strip_prefix('[')
        .and_then(|item| item.strip_suffix(']'))
        .unwrap_or(trimmed);
    trimmed.parse::<IpAddr>().is_ok()
}

fn nonempty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|item| !item.is_empty())
}

pub fn resolve_host_identity(
    host: Option<&str>,
    sniff_host: Option<&str>,
    destination_ip: Option<&str>,
) -> Option<String> {
    nonempty(host)
        .or_else(|| nonempty(sniff_host))
        .or_else(|| nonempty(destination_ip))
        .map(str::to_string)
}

pub fn prefer_host_identity(stored: Option<&str>, incoming: Option<&str>) -> Option<String> {
    let stored = nonempty(stored);
    let incoming = nonempty(incoming);
    match (stored, incoming) {
        (None, None) => None,
        (Some(value), None) => Some(value.to_string()),
        (None, Some(value)) => Some(value.to_string()),
        (Some(stored), Some(incoming)) if stored == incoming => Some(stored.to_string()),
        (Some(stored), Some(incoming)) => {
            if looks_like_ip(incoming) && !looks_like_ip(stored) {
                Some(stored.to_string())
            } else {
                Some(incoming.to_string())
            }
        }
    }
}

#[cfg(test)]
mod session_host_tests {
    use super::*;

    #[test]
    fn resolve_prefers_host_then_sniff_then_ip() {
        assert_eq!(
            resolve_host_identity(Some("a.test"), Some("sniff.test"), Some("1.1.1.1")).as_deref(),
            Some("a.test")
        );
        assert_eq!(
            resolve_host_identity(None, Some("sniff.test"), Some("1.1.1.1")).as_deref(),
            Some("sniff.test")
        );
        assert_eq!(
            resolve_host_identity(Some(""), Some("  "), Some("8.8.8.8")).as_deref(),
            Some("8.8.8.8")
        );
        assert_eq!(resolve_host_identity(None, None, None), None);
    }

    #[test]
    fn looks_like_ip_accepts_v4_v6() {
        assert!(looks_like_ip("1.2.3.4"));
        assert!(looks_like_ip("::1"));
        assert!(looks_like_ip("[2001:db8::1]"));
        assert!(!looks_like_ip("a.test"));
        assert!(!looks_like_ip("1.2.3"));
    }

    #[test]
    fn prefer_upgrades_empty_and_ip_but_not_domain() {
        assert_eq!(
            prefer_host_identity(None, Some("1.1.1.1")).as_deref(),
            Some("1.1.1.1")
        );
        assert_eq!(
            prefer_host_identity(Some("1.1.1.1"), Some("a.test")).as_deref(),
            Some("a.test")
        );
        assert_eq!(
            prefer_host_identity(Some("a.test"), Some("1.1.1.1")).as_deref(),
            Some("a.test")
        );
        assert_eq!(
            prefer_host_identity(Some("a.test"), None).as_deref(),
            Some("a.test")
        );
        assert_eq!(
            prefer_host_identity(Some("old.test"), Some("new.test")).as_deref(),
            Some("new.test")
        );
    }
}
