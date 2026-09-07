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

For an unauthenticated SOCKS5 service, set both `username` and `password` to empty strings. Leaving either value as `xxx` rejects configuration generation unless a same-name Profile node supplies it. The host may continue with the original Profile after an error; rejection does not prove traffic is blocked.

## Reserved residential group

**Unreleased**: an existing `AI-家宽` group must be `select` with only `家宽-SOCKS5` in its explicit members. Allowed fields are `name`, `type`, `proxies`, `disable-udp`, `icon` and `hidden`. Output enables UDP and preserves display metadata. Extra fields such as `use`, `include-all*`, filters, default selection or empty-group fallback are rejected. Rename a custom group or remove the extra configuration; do not add airport nodes to this reserved group.

This constrains the successfully generated group to one residential exit; it does not prove that the SOCKS service offers a static public IP. Clash Verge Rev may keep the original Profile after a script error. Fix the configuration and confirm that it reloads; an exception is not runtime fail-closed enforcement.

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

The tables below cover 24 routing and 7 runtime switches. Setup details are in [Local configuration — switches](local-configuration.md#switches). Do not guess key names from `ROUTE_*` / `ENABLE_*` prefixes. Keep shared dependencies and process-wide fallbacks off unless sanitized Connections evidence requires them.

The three new core switches and dedicated-fallback gating are **Unreleased**. All three default to `true`, so they do not automatically reduce residential usage. Disabling a core switch removes its managed domain/DNS entries and dedicated fallbacks, leaving the original Profile to decide the route. Existing rules may select an airport, DIRECT, another group or a user-owned residential rule; an airport exit is not guaranteed. Authentication, auxiliary, static-asset and global realtime/DNS switches remain independent.

To return all Google core traffic to the original Profile, disable `gemini_web_core`, `gemini_api_core`, `vertex_ai_endpoints` and `antigravity_core` together. Optional authentication, project and update switches are separate. See [DNS and leak model](dns-and-leak-model.md) for the regex-domain DNS exception.

### Routing

| TOML key | JavaScript constant | Default | Effect | Dependency or risk |
| --- | --- | --- | --- | --- |
| `routing.anthropic_core` | `ROUTE_ANTHROPIC_CORE` | `true` | Routes Claude products, model API, MCP and session content. | Disabling also removes dedicated process and Anthropic IP fallbacks; auxiliary and shared-dependency switches remain independent. |
| `routing.gemini_api_core` | `ROUTE_GEMINI_API_CORE` | `true` | Routes the Gemini Developer API at `generativelanguage.googleapis.com`. | Independent of Gemini Web, Vertex AI and Google authentication. |
| `routing.antigravity_core` | `ROUTE_ANTIGRAVITY_CORE` | `true` | Routes the Antigravity product host and core Code Assist hosts. | Disabling also removes Antigravity process fallbacks; Vertex, authentication, project APIs, updates and telemetry remain independent. |
| `routing.openai_shared_dependencies` | `ROUTE_OPENAI_SHARED_DEPENDENCIES` | `false` | Routes OpenAI WorkOS, support, telemetry, payment, and other shared dependencies. | Expands beyond model traffic. |
| `routing.openai_core` | `ROUTE_OPENAI_CORE` | `true` | Routes the ChatGPT product, the OpenAI model API, and uploaded/generated user content. | Disabling removes dedicated OpenAI process fallbacks and returns the traffic to the original Profile. |
| `routing.openai_auth` | `ROUTE_OPENAI_AUTH` | `false` | Routes first-party login hosts `auth.openai.com` (including children) and exact `auth0.openai.com`. | Independent from core, web assets, and shared third-party dependencies; does not match all of `openai.com`. |
| `routing.openai_web_assets` | `ROUTE_OPENAI_WEB_ASSETS` | `false` | Routes the `oaistatic.com` web-asset suffix. | Independent from first-party login and shared dependencies; enable only when page assets need the same exit. |
| `routing.claude_shared_dependencies` | `ROUTE_CLAUDE_SHARED_DEPENDENCIES` | `false` | Routes Claude analytics, support, risk-control, and other shared dependencies. | Expands beyond model traffic. |
| `routing.antigravity_google_auth` | `ROUTE_ANTIGRAVITY_GOOGLE_AUTH` | `false` | Routes the shared Google login entry used by Antigravity. | Affects authentication for other Google products. |
| `routing.antigravity_project_apis` | `ROUTE_ANTIGRAVITY_PROJECT_APIS` | `false` | Routes Service Usage, Resource Manager, IAM, API Hub, and other project APIs. | Project configuration, not inference. |
| `routing.antigravity_update_and_telemetry` | `ROUTE_ANTIGRAVITY_UPDATE_AND_TELEMETRY` | `false` | Routes Antigravity updates, extension marketplace, and telemetry. | Expands into update and analytics traffic. |
| `routing.gemini_web_core` | `ROUTE_GEMINI_WEB_CORE` | `true` | Routes Gemini Web and Google AI Studio product entry points. | None. |
| `routing.vertex_ai_endpoints` | `ROUTE_VERTEX_AI_ENDPOINTS` | `true` | Routes four Vertex AI / Agent Platform rules: `aiplatform.googleapis.com`, `aiplatform.us.rep.googleapis.com`, `aiplatform.eu.rep.googleapis.com`, and the regional regex `^[a-z0-9-]+-aiplatform\.googleapis\.com$`. | Set to `false` when Antigravity enterprise inference and other Vertex AI traffic should return to the original Profile. |
| `routing.cursor_core` | `ROUTE_CURSOR_CORE` | `true` | Routes Cursor AI API, Tab, Agent, authorize/SSO portal, Cloud Agent VMs, and product-specific authentication. | Set to `false` when Cursor core traffic should return to the original Profile. `api2.cursor.sh` stays under this switch. |
| `routing.cursor_repository_indexing` | `ROUTE_CURSOR_REPOSITORY_INDEXING` | `false` | Routes Cursor repository-indexing hosts `repo[0-9]+.cursor.sh`. | Independent of `routing.cursor_core`. Default falls back to the original Profile. A missing field is completed as `false`. Set `true` to restore v5.8.1 residential routing for those hosts. Official docs and local 2026-08-17 logs jointly confirm `repo42.cursor.sh`. The numeric wildcard is this project's forward-compat policy, not an official Cursor wildcard contract. Privacy Mode does not stop indexing uploads. `disableHttp2` or a server-forced HTTP/1.1 fallback can put RepositoryService on shared `api2.cursor.sh`; domain rules cannot isolate that path while keeping most APIs, so default-off cannot claim all repo uploads are excluded. |
| `routing.grok_core` | `ROUTE_GROK_CORE` | `true` | Routes Grok Build (xAI grok CLI) inference API (`cli-chat-proxy.grok.com`), the Grok product domain, `auth.x.ai`, and `api.x.ai`. | Set to `false` when Grok should return to the original Profile. |
| `routing.grok_web_assets` | `ROUTE_GROK_WEB_ASSETS` | `true` | When `true`, injects `DOMAIN-SUFFIX,grok.com`. When `false`, replaces that suffix with exact hosts `grok.com`, `cli-chat-proxy.grok.com`, and `code.grok.com`. `DOMAIN-SUFFIX,api.x.ai` stays under `routing.grok_core`. | Requires `routing.grok_core = true`. `false` returns `assets.grok.com` to the original Profile. |
| `routing.cursor_process_fallback` | `ROUTE_CURSOR_PROCESS_FALLBACK` | `false` | Adds Cursor process-level fallback rules. | Requires both `routing.ai_process_fallback = true` and `routing.cursor_core = true`; it can still capture non-AI requests. |
| `routing.claude_code_auxiliary` | `ROUTE_CLAUDE_CODE_AUXILIARY` | `false` | Routes Claude Code install, update, docs, and package endpoints. | Auxiliary traffic, not inference. |
| `routing.ai_process_fallback` | `ENABLE_AI_PROCESS_FALLBACK` | `false` | Adds process fallbacks for enabled Claude, OpenAI and Antigravity cores; Cursor also needs its process switch. | Captures non-AI requests from those processes. Process lookup is separate: the script writes top-level `find-process-mode: always`. A value nested under `profile:` is ignored by the kernel. |
| `routing.anthropic_ip_fallback` | `ENABLE_ANTHROPIC_IP_FALLBACK` | `true` | Uses Anthropic official inbound ranges for IP-only connections. | Also requires `routing.anthropic_core = true`; non-core hosts in the same ranges can match. |
| `routing.shared_realtime_infrastructure` | `ROUTE_SHARED_REALTIME_INFRASTRUCTURE` | `false` | Routes generic STUN/TURN realtime infrastructure. | Can capture realtime traffic from other apps. |
| `routing.global_realtime_ports` | `ROUTE_GLOBAL_REALTIME_PORTS` | `false` | Adds broad realtime UDP-port rules. | Requires `routing.shared_realtime_infrastructure = true`; scope is wide. |
| `routing.public_encrypted_dns` | `ROUTE_PUBLIC_ENCRYPTED_DNS` | `false` | Routes public DoH/DoT services. | Affects shared DNS traffic. |

### Runtime

| TOML key | JavaScript constant | Default | Effect | Dependency or risk |
| --- | --- | --- | --- | --- |
| `runtime.allow_final_rule_upstream_fallback` | `ALLOW_FINAL_RULE_UPSTREAM_FALLBACK` | `true` | Tries the current Profile's last `MATCH` / `FINAL` target when named candidates miss. | The target still passes structural and recursion checks. |
| `runtime.allow_heuristic_upstream_fallback` | `ALLOW_HEURISTIC_UPSTREAM_FALLBACK` | `false` | Guesses an upstream from group-name semantics. | Used only after earlier candidates fail; can pick the wrong exit. |
| `runtime.preserve_unmanaged_nameserver_policy` | `PRESERVE_UNMANAGED_NAMESERVER_POLICY` | `false` | Keeps subscription `nameserver-policy` entries the script does not manage. | Relaxes the strict DNS-rebuild boundary. |
| `runtime.enable_domain_sniffer` | `ENABLE_DOMAIN_SNIFFER` | `true` | Hardens domain sniffing for IP-only connections and missing DNS mappings. | Does not globally rewrite destinations. |
| `runtime.harden_existing_tun_dns_hijack` | `HARDEN_EXISTING_TUN_DNS_HIJACK` | `true` | Completes DNS-hijack entries for an already enabled TUN. | Effective only when the Profile already has TUN on. |
| `runtime.enable_tun_strict_route` | `ENABLE_TUN_STRICT_ROUTE` | `false` | Enables `strict-route` on the existing TUN. | Requires TUN on and `runtime.harden_existing_tun_dns_hijack = true`; may affect VMs or special routes. |
| `runtime.warn_on_reachable_udp_disabled` | `WARN_ON_REACHABLE_UDP_DISABLED` | `true` | Emits one summary warning when reachable leaves explicitly disable UDP (at most 8 samples). | A top-level upstream with UDP disabled still fails validation. |

## Clash Verge Rev settings

Recommended runtime settings:

- Rule mode.
- Put `find-process-mode: always` at the Mihomo YAML top level when Clash Verge Merge still nests it under `profile:`. The kernel does not read `profile.find-process-mode`.
- Enable TUN when system-wide interception or process rules are required.
- Enable DNS hijack in Clash Verge Rev TUN settings. When TUN is already on, the script also adds `any:53` and `tcp://any:53`. The current host restores `tun` and `ipv6` from the settings page after the global script runs; treat those fields as owned by the settings page and turn IPv6 off there.
- Disable browser private/secure DNS when it bypasses the system resolver.
- The selected upstream group must not resolve to `DIRECT`, `REJECT`, or the residential proxy itself.
- When the target feature needs UDP, both the selected airport path and the residential SOCKS5 service must support UDP. Airport subscription nodes that omit `udp` are treated as UDP-disabled by Mihomo.
