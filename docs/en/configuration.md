# Configuration

Store local endpoint, credential, and switch values in the ignored `clash-verge-ai-residential.local.toml`, then run `just render-local` or `node scripts/sync-local-config.js` to produce `clash-verge-ai-residential.local.js`. Do not edit the public template or the generated script; change the TOML and render again.

```toml
[home_proxy]
name = "家宽-SOCKS5"
type = "socks5"
server = "xxx"
port = 443
username = "xxx"
password = "xxx"
udp = true
dialer-proxy = "🚀节点选择"
```

The generated local script and local TOML are both excluded by `.gitignore`. The tracked starting point is `clash-verge-ai-residential.local.toml.example`. Full setup, validation, and the switch table are in [Local configuration](local-configuration.md).

Two modes are supported:

- Enter the actual endpoint and credentials in the ignored local TOML, then paste the generated local script into Clash Verge Rev.
- Define an existing `家宽-SOCKS5` proxy in each Profile and leave `server`, `username`, and `password` as `xxx`; the script reuses the existing endpoint and credentials.

For an unauthenticated SOCKS5 service, set both `username` and `password` to empty strings. Leaving either value as `xxx` fails closed unless a same-name Profile node provides the missing values.

## Profile-specific upstream candidates

`dialer-proxy` accepts one proxy or group name. Candidate arrays are resolution order, not Mihomo configuration values:

```javascript
const PROFILE_UPSTREAM_OVERRIDES = {
  "Profile A": ["🚀节点选择", "Proxy", "自动选择"],
  "Profile B": ["Proxy", "🚀节点选择", "自动选择"]
};
```

The final `HOME_PROXY_TEMPLATE["dialer-proxy"]` value remains the preferred cross-Profile default. Resolution order is in [Multi-profile](multi-profile.md).

## Switches

The optional `[routing]` and `[runtime]` tables accept partial overrides. During sync, missing switch keys (including a missing table) are auto-completed into the local TOML from the example defaults; existing values, comments, and line endings are preserved. Defaults intentionally minimize residential traffic; enabling shared-dependency or process-wide fallbacks changes privacy, cost, and scope.

The 28 TOML-to-JavaScript rows live only in [Local configuration — switches](local-configuration.md#switches). Do not guess key names from the `ROUTE_*` / `ENABLE_*` prefixes. Keep shared dependencies and process-wide fallbacks off unless sanitized Connections evidence requires them.

## Clash Verge Rev settings

Recommended runtime settings:

- Rule mode.
- Put `find-process-mode: always` at the Mihomo YAML top level when Clash Verge Merge still nests it under `profile:`. The kernel does not read `profile.find-process-mode`.
- Enable TUN when system-wide interception or process rules are required.
- Enable DNS hijack in Clash Verge Rev TUN settings. When TUN is already on, the script also adds `any:53` and `tcp://any:53`. The current host restores `tun` and `ipv6` from the settings page after the global script runs; treat those fields as owned by the settings page and turn IPv6 off there.
- Disable browser private/secure DNS when it bypasses the system resolver.
- The selected upstream group must not resolve to `DIRECT`, `REJECT`, or the residential proxy itself.
- When the target feature needs UDP, both the selected airport path and the residential SOCKS5 service must support UDP. Airport subscription nodes that omit `udp` are treated as UDP-disabled by Mihomo.
