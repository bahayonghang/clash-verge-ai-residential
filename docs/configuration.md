# Configuration

## Residential SOCKS5

Use the ignored `clash-verge-ai-residential.local.toml` to store local endpoint, credential, and switch values, then run `just render-local` or `node scripts/sync-local-config.js` to produce `clash-verge-ai-residential.local.js`. Do not edit the public template or the generated script; change the TOML and render again.

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

The optional `[routing]` and `[runtime]` TOML tables accept partial overrides. During sync, missing switch keys (including a missing table) are auto-completed into the local TOML from the example defaults; existing values, comments, and line endings are preserved verbatim. The defaults intentionally minimize residential traffic; enabling permissive or shared-infrastructure switches can change privacy, cost, compatibility, and traffic scope.

### Routing table

| TOML key | JavaScript constant | Default | Effect | Dependency or risk |
| --- | --- | --- | --- | --- |
| `routing.openai_shared_dependencies` | `ROUTE_OPENAI_SHARED_DEPENDENCIES` | `false` | Routes OpenAI shared identity, support, telemetry, and payment dependencies. | Expands beyond model traffic. |
| `routing.openai_core` | `ROUTE_OPENAI_CORE` | `true` | Routes the ChatGPT product, the OpenAI model API, and uploaded/generated user content. | When disabled, GPT traffic falls through to the airport upstream. |
| `routing.claude_shared_dependencies` | `ROUTE_CLAUDE_SHARED_DEPENDENCIES` | `false` | Routes Claude analytics, support, risk-control, and other shared dependencies. | Expands beyond model traffic. |
| `routing.antigravity_google_auth` | `ROUTE_ANTIGRAVITY_GOOGLE_AUTH` | `false` | Routes the shared Google authentication entry used by Antigravity. | Can affect authentication for other Google products. |
| `routing.antigravity_project_apis` | `ROUTE_ANTIGRAVITY_PROJECT_APIS` | `false` | Routes project-management APIs such as Service Usage, Resource Manager, IAM, and API Hub. | These are project configuration rather than inference. |
| `routing.antigravity_update_and_telemetry` | `ROUTE_ANTIGRAVITY_UPDATE_AND_TELEMETRY` | `false` | Routes Antigravity updates, extension marketplace, and telemetry. | Expands into update and analytics traffic. |
| `routing.gemini_web_core` | `ROUTE_GEMINI_WEB_CORE` | `true` | Routes Gemini Web and Google AI Studio product endpoints. | None. |
| `routing.cursor_core` | `ROUTE_CURSOR_CORE` | `true` | Routes Cursor AI APIs, Tab, Agent, the authorize/SSO admin portal, Cloud Agent VMs, and product-specific authentication. | Set to `false` when Cursor core traffic should stay on the airport upstream. `api2.cursor.sh` stays under this switch. |
| `routing.cursor_repository_indexing` | `ROUTE_CURSOR_REPOSITORY_INDEXING` | `false` | Routes Cursor repository-indexing hosts `repo[0-9]+.cursor.sh`. | Independent of `routing.cursor_core`. The default sends repo indexing back to the original Profile/airport upstream. A missing local TOML field is completed as `false`. Set the field to `true` to restore v5.8.1 residential routing for those hosts. Official docs and local 2026-08-17 logs jointly confirm `repo42.cursor.sh`. The `repo[0-9]+.cursor.sh` pattern is this project's forward-compat policy, not an official Cursor wildcard contract. Privacy Mode does not stop indexing uploads. `disableHttp2` or a server-forced HTTP/1.1 fallback can put RepositoryService on shared `api2.cursor.sh`; Clash domain rules cannot isolate that path, so this switch does not exclude all repository uploads from the residential link. |
| `routing.grok_core` | `ROUTE_GROK_CORE` | `true` | Routes Grok Build (xAI grok CLI) inference API (`cli-chat-proxy.grok.com`) and the Grok product domain. | Set to `false` when Grok should stay on the airport upstream; Grok web and telemetry share the host and cannot be split at the domain layer. |
| `routing.cursor_process_fallback` | `ROUTE_CURSOR_PROCESS_FALLBACK` | `false` | Adds Cursor process-level fallback rules. | Requires `routing.ai_process_fallback = true` and can capture non-AI requests. |
| `routing.claude_code_auxiliary` | `ROUTE_CLAUDE_CODE_AUXILIARY` | `false` | Routes Claude Code installation, update, documentation, and package endpoints. | These are auxiliary rather than inference traffic. |
| `routing.ai_process_fallback` | `ENABLE_AI_PROCESS_FALLBACK` | `false` | Adds process-level fallbacks for known AI applications. | Can capture non-AI requests made by those processes. |
| `routing.anthropic_ip_fallback` | `ENABLE_ANTHROPIC_IP_FALLBACK` | `true` | Routes Anthropic's official inbound IP ranges when domain matching is unavailable. | None. |
| `routing.shared_realtime_infrastructure` | `ROUTE_SHARED_REALTIME_INFRASTRUCTURE` | `false` | Routes shared STUN/TURN infrastructure. | Can capture realtime traffic from unrelated applications. |
| `routing.global_realtime_ports` | `ROUTE_GLOBAL_REALTIME_PORTS` | `false` | Adds broad realtime UDP-port rules. | Requires `routing.shared_realtime_infrastructure = true`; scope is intentionally broad. |
| `routing.public_encrypted_dns` | `ROUTE_PUBLIC_ENCRYPTED_DNS` | `false` | Routes public DoH/DoT services. | Affects shared DNS traffic. |

### Runtime table

| TOML key | JavaScript constant | Default | Effect | Dependency or risk |
| --- | --- | --- | --- | --- |
| `runtime.allow_final_rule_upstream_fallback` | `ALLOW_FINAL_RULE_UPSTREAM_FALLBACK` | `true` | Tries the final `MATCH` / `FINAL` target when named candidates do not match. | The target still passes structural and recursion validation. |
| `runtime.allow_heuristic_upstream_fallback` | `ALLOW_HEURISTIC_UPSTREAM_FALLBACK` | `false` | Guesses an upstream from group-name semantics. | Used only after earlier candidates fail and can choose the wrong exit. |
| `runtime.preserve_unmanaged_nameserver_policy` | `PRESERVE_UNMANAGED_NAMESERVER_POLICY` | `false` | Preserves subscription `nameserver-policy` entries not managed by the script. | Relaxes the strict DNS-rebuild boundary. |
| `runtime.enable_domain_sniffer` | `ENABLE_DOMAIN_SNIFFER` | `true` | Hardens domain sniffing for IP-only connections and missing DNS mappings. | Does not globally override destinations. |
| `runtime.harden_existing_tun_dns_hijack` | `HARDEN_EXISTING_TUN_DNS_HIJACK` | `true` | Completes DNS-hijack entries for an already enabled TUN. | Effective only when the Profile has TUN enabled. |
| `runtime.enable_tun_strict_route` | `ENABLE_TUN_STRICT_ROUTE` | `false` | Enables `strict-route` on the existing TUN. | Requires enabled TUN and `runtime.harden_existing_tun_dns_hijack = true`; may affect VMs or special routes. |
| `runtime.warn_on_reachable_udp_disabled` | `WARN_ON_REACHABLE_UDP_DISABLED` | `true` | Emits one summary warning when reachable leaves explicitly disable UDP (at most 8 samples). | A top-level upstream with UDP disabled still fails validation. |

The detailed Chinese-language setup guide in [Local TOML configuration and sync](local-configuration.md) includes both `just` and direct Node workflows. Keep shared dependencies and process-wide fallbacks disabled unless sanitized connection evidence proves they are required.

## Clash Verge Rev settings

Recommended runtime settings:

- Rule mode.
- TUN enabled when system-wide interception or process rules are required.
- 在 Clash Verge Rev 的 TUN 设置中启用 DNS 劫持。TUN 已启用时，脚本还会补充 `any:53` 和 `tcp://any:53`。但当前版本会在全局脚本运行后，从设置页面恢复 `tun` 和 `ipv6`；这两个字段应以设置页面为准，并在该页面关闭 IPv6。
- Browser private/secure DNS disabled when it bypasses the system resolver.
- The selected upstream group must not resolve to `DIRECT`, `REJECT`, or the residential proxy itself.
- 目标功能需要 UDP 时，选中的机场线路和住宅 SOCKS5 服务都必须支持 UDP。机场订阅节点若省略 `udp` 字段，Mihomo 默认禁用 UDP。
