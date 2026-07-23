# Configuration

## Residential SOCKS5

Use the ignored `clash-verge-ai-residential.local.toml` to store local endpoint and credential values, then run `just render-local` to produce `clash-verge-ai-residential.local.js`. Do not edit the public `clash-verge-ai-residential.js` template with real values.

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

The generated local script and local TOML are both excluded by `.gitignore`; `clash-verge-ai-residential.local.toml.example` is the safe tracked starting point. See [Local TOML configuration and sync](local-configuration.md) for the complete setup, validation rules, and platform-specific copy command.

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

The final `HOME_PROXY_TEMPLATE["dialer-proxy"]` value remains the preferred cross-Profile default.

## Scope switches

The defaults intentionally minimize residential traffic. Keep shared dependencies and process-wide fallbacks disabled unless a captured connection proves they are required:

```javascript
const ROUTE_OPENAI_SHARED_DEPENDENCIES = false;
const ROUTE_CLAUDE_SHARED_DEPENDENCIES = false;
const ROUTE_ANTIGRAVITY_GOOGLE_AUTH = false;
const ROUTE_ANTIGRAVITY_PROJECT_APIS = false;
const ROUTE_ANTIGRAVITY_UPDATE_AND_TELEMETRY = false;
const ROUTE_CURSOR_PROCESS_FALLBACK = false;
const ROUTE_SHARED_REALTIME_INFRASTRUCTURE = false;
const ROUTE_GLOBAL_REALTIME_PORTS = false;
const ROUTE_PUBLIC_ENCRYPTED_DNS = false;
```

Enabling a switch changes the privacy and cost boundary. Add a regression test for every switch change.

## Clash Verge Rev settings

Recommended runtime settings:

- Rule mode.
- TUN enabled when system-wide interception or process rules are required.
- DNS hijack enabled; the script supplements `any:53` and `tcp://any:53` only when TUN is already enabled.
- Browser private/secure DNS disabled when it bypasses the system resolver.
- The selected upstream group must not resolve to `DIRECT`, `REJECT`, or the residential proxy itself.
- Both the selected airport path and the residential SOCKS5 service must support UDP when the target feature needs UDP.
