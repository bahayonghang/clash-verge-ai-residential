# Local TOML configuration and sync

The public `clash-verge-ai-residential.js` must always keep `xxx` placeholders. Local credentials and script switches live in `clash-verge-ai-residential.local.toml`, then render one-way into `clash-verge-ai-residential.local.js`. Both local files are excluded by `.gitignore`. Only the example `clash-verge-ai-residential.local.toml.example` is tracked.

## Prerequisites

- Node.js 18 or later for the extension renderer.
- Optional [just](https://github.com/casey/just). Without `just`, run the Node renderer directly.

Run the following commands from the repository root. The justfile uses Windows PowerShell on Windows and `sh` on macOS/Linux, so Windows does not need Git Bash.

The docs site preview (`just docs-dev`) is a separate Node.js 22+ toolchain in `docs/package.json`. It is not required to render the local script.

## First-time setup

Prefer `just render-local` the first time. If `clash-verge-ai-residential.local.toml` is missing, the command copies it from `clash-verge-ai-residential.local.toml.example` and asks you to fill it in, then exits. Edit the local TOML:

```toml
[home_proxy]
name = "家宽-SOCKS5"
type = "socks5"
server = "residential.example.com"
port = 1080
username = "your-username"
password = "your-password"
udp = true
dialer-proxy = "🚀节点选择"

[routing]
cursor_core = true
cursor_repository_indexing = false
grok_core = true
```

Field meanings:

| Field | Meaning |
| --- | --- |
| `name` | Must match the public template `HOME_PROXY_NAME`, currently `家宽-SOCKS5`. |
| `type` | Only `socks5` is allowed. |
| `server` / `port` | Residential SOCKS5 host and a port in `1-65535`. |
| `username` / `password` | Credentials. An unauthenticated service must set both to `""`. |
| `udp` | `true` when the SOCKS5 service supports UDP. |
| `dialer-proxy` | An upstream proxy or group name that actually exists in the local airport Profile. |

TOML strings use double quotes. Escape double quotes and backslashes in usernames or passwords. `#` inside quotes is part of the value; outside quotes it starts a comment.

Without `just`, copy the example only when the file does not exist. Windows PowerShell:

```powershell
if (-not (Test-Path clash-verge-ai-residential.local.toml)) {
  Copy-Item clash-verge-ai-residential.local.toml.example clash-verge-ai-residential.local.toml
}
```

macOS/Linux:

```bash
test -e clash-verge-ai-residential.local.toml || \
  cp clash-verge-ai-residential.local.toml.example clash-verge-ai-residential.local.toml
```

Then edit the same local TOML. Do not write real values into the public JavaScript template.

## Switches

`[routing]` and `[runtime]` are optional tables. You may write only the keys you need to override. On sync, missing switch keys (including a missing table) are completed from the example defaults and written back; existing values, comments, and line endings stay verbatim. Missing keys under `[home_proxy]` still fail and must be filled by hand. An old local TOML that only has `[home_proxy]` can still render; the renderer fills in missing switches. Each TOML key maps to exactly one JavaScript constant. Do not guess names from the `ROUTE_*` / `ENABLE_*` prefixes.

### Routing

| TOML key | JavaScript constant | Default | Effect | Dependency or risk |
| --- | --- | --- | --- | --- |
| `routing.openai_shared_dependencies` | `ROUTE_OPENAI_SHARED_DEPENDENCIES` | `false` | Routes OpenAI WorkOS, support, telemetry, payment, and other shared dependencies. | Expands beyond model traffic. |
| `routing.openai_core` | `ROUTE_OPENAI_CORE` | `true` | Routes the ChatGPT product, the OpenAI model API, and uploaded/generated user content. | When disabled, GPT traffic falls through to the airport upstream. |
| `routing.openai_auth` | `ROUTE_OPENAI_AUTH` | `false` | Routes first-party login hosts `auth.openai.com` (including children) and exact `auth0.openai.com`. | Independent from core, web assets, and shared third-party dependencies; does not match all of `openai.com`. |
| `routing.openai_web_assets` | `ROUTE_OPENAI_WEB_ASSETS` | `false` | Routes the `oaistatic.com` web-asset suffix. | Independent from first-party login and shared dependencies; enable only when page assets need the same exit. |
| `routing.claude_shared_dependencies` | `ROUTE_CLAUDE_SHARED_DEPENDENCIES` | `false` | Routes Claude analytics, support, risk-control, and other shared dependencies. | Expands beyond model traffic. |
| `routing.antigravity_google_auth` | `ROUTE_ANTIGRAVITY_GOOGLE_AUTH` | `false` | Routes the shared Google login entry used by Antigravity. | Affects authentication for other Google products. |
| `routing.antigravity_project_apis` | `ROUTE_ANTIGRAVITY_PROJECT_APIS` | `false` | Routes Service Usage, Resource Manager, IAM, API Hub, and other project APIs. | Project configuration, not inference. |
| `routing.antigravity_update_and_telemetry` | `ROUTE_ANTIGRAVITY_UPDATE_AND_TELEMETRY` | `false` | Routes Antigravity updates, extension marketplace, and telemetry. | Expands into update and analytics traffic. |
| `routing.gemini_web_core` | `ROUTE_GEMINI_WEB_CORE` | `true` | Routes Gemini Web and Google AI Studio product entry points. | None. |
| `routing.vertex_ai_endpoints` | `ROUTE_VERTEX_AI_ENDPOINTS` | `true` | Routes four Vertex AI / Agent Platform rules: `aiplatform.googleapis.com`, `aiplatform.us.rep.googleapis.com`, `aiplatform.eu.rep.googleapis.com`, and the regional regex `^[a-z0-9-]+-aiplatform\.googleapis\.com$`. | Set to `false` when Antigravity enterprise inference and other Vertex AI traffic should stay on the airport upstream. |
| `routing.cursor_core` | `ROUTE_CURSOR_CORE` | `true` | Routes Cursor AI API, Tab, Agent, authorize/SSO portal, Cloud Agent VMs, and product-specific authentication. | Set to `false` when Cursor core traffic should stay on the airport upstream. `api2.cursor.sh` stays under this switch. |
| `routing.cursor_repository_indexing` | `ROUTE_CURSOR_REPOSITORY_INDEXING` | `false` | Routes Cursor repository-indexing hosts `repo[0-9]+.cursor.sh`. | Independent of `routing.cursor_core`. Default falls back to the original Profile/airport upstream. A missing field is completed as `false`. Set `true` to restore v5.8.1 residential routing for those hosts. Official docs and local 2026-08-17 logs jointly confirm `repo42.cursor.sh`. The numeric wildcard is this project's forward-compat policy, not an official Cursor wildcard contract. Privacy Mode does not stop indexing uploads. `disableHttp2` or a server-forced HTTP/1.1 fallback can put RepositoryService on shared `api2.cursor.sh`; domain rules cannot isolate that path while keeping most APIs, so default-off cannot claim all repo uploads are excluded. |
| `routing.grok_core` | `ROUTE_GROK_CORE` | `true` | Routes Grok Build (xAI grok CLI) inference API (`cli-chat-proxy.grok.com`), the Grok product domain, `auth.x.ai`, and `api.x.ai`. | Set to `false` when Grok should stay on the airport upstream. |
| `routing.grok_web_assets` | `ROUTE_GROK_WEB_ASSETS` | `true` | When `true`, injects `DOMAIN-SUFFIX,grok.com`. When `false`, replaces that suffix with exact hosts `grok.com`, `cli-chat-proxy.grok.com`, and `code.grok.com`. `DOMAIN-SUFFIX,api.x.ai` stays under `routing.grok_core`. | Requires `routing.grok_core = true`. `false` leaves `assets.grok.com` on the airport upstream. |
| `routing.cursor_process_fallback` | `ROUTE_CURSOR_PROCESS_FALLBACK` | `false` | Adds Cursor process-level fallback rules. | Requires `routing.ai_process_fallback = true` and can capture non-AI requests. |
| `routing.claude_code_auxiliary` | `ROUTE_CLAUDE_CODE_AUXILIARY` | `false` | Routes Claude Code install, update, docs, and package endpoints. | Auxiliary traffic, not inference. |
| `routing.ai_process_fallback` | `ENABLE_AI_PROCESS_FALLBACK` | `false` | Adds process-level fallbacks for known AI applications. | Captures non-AI requests from those processes. Process lookup is separate: the script writes top-level `find-process-mode: always`. A value nested under `profile:` is ignored by the kernel. |
| `routing.anthropic_ip_fallback` | `ENABLE_ANTHROPIC_IP_FALLBACK` | `true` | Uses Anthropic official inbound ranges for IP-only connections. | None. |
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

## Generate the local script

With `just`, the first run creates a missing local TOML. After you fill it in, and after every later TOML edit, run:

```bash
just render-local
```

Without `just`, after editing the local TOML:

```bash
node scripts/sync-local-config.js
```

`render-local` is one-way render, not two-way sync: it reads public `clash-verge-ai-residential.js` plus the local TOML and writes `clash-verge-ai-residential.local.js`. It does not modify the public template. The only write-back is missing switch auto-completion: missing switch keys are written into the local TOML from example defaults so you can see every available switch. User-written keys, comments, and line endings are not rewritten. Do not hand-edit the generated `.local.js`; change the TOML and generate again.

In Clash Verge Rev open **Profiles -> Global Extend Script**, double-click the script card, paste the **generated local script** in full, save, then refresh the current Profile:

![Clash Verge Rev Profiles page and Global Extend Script entry](../../assets/clash-verge-rev-global-extend-script.png)

`just sync` remains as a compatibility alias. New docs and generated files use `just render-local`.

### Copy from Windows to Ubuntu

You can copy the Windows `clash-verge-ai-residential.local.js` produced by `just render-local` into Ubuntu Clash Verge Rev Global Extend Script. The script contains no Windows paths, shell commands, or OS branches. The generated `.local.js` embeds the residential proxy address and credentials from the TOML, so it is sensitive: transfer over a trusted channel, restrict read permission, and do not commit it, upload it to a public drive, or print it in logs. After copying the rendered `.local.js` you should not also copy the local TOML, but that does not lower the protection required for `.local.js` itself.

The Ubuntu Profile must still uniquely resolve the `dialer-proxy` name in the script and provide a reachable airport node, UDP capability, and a compatible Clash Verge/Mihomo script host. Repository Windows/Ubuntu Node tests only cover syntax, rendering, and the rule contract. Whether Ubuntu host execution works after the copy, and whether login and model requests share one exit, must be confirmed with sanitized Connections. That check is **UNVERIFIED**.

Sync refuses the following before writing: unknown or duplicate tables/keys, non-boolean switches, missing proxy fields, invalid TOML strings, a type other than SOCKS5, a port out of range, an empty upstream name, or a `name` that does not match the template reserved name. Each switch must match exactly one boolean constant declaration in the public template, or the write is refused. Errors name the field or line number and leave no partial output. Fix the TOML and rerun.

## Mode that does not store credentials

You may leave `server`, `username`, and `password` as `"xxx"` and predefine a same-name `家宽-SOCKS5` node in each Clash Profile. At runtime the script reuses that node's endpoint and credentials. For no-auth SOCKS5, set both `username` and `password` to `""` in the TOML.

In either mode, do not commit the local TOML, the generated `.local.js`, generated Profiles, or unsanitized connection logs.

## Verification

Run the public-template checks and regression tests:

```bash
just ci
```

`just ci` equals `npm run ci` plus the monitor gate. It does not read or upload the local TOML. Generated local scripts are excluded from the template safety scan so credentials do not trip the public-repo check. Afterward, still confirm in Clash Verge Rev that `家宽-SOCKS5.dialer-proxy` resolves to a real airport group, and that AI requests hit `AI-家宽` in Connections.
