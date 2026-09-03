# Routing scope

The residential link is reserved for core AI product traffic. A domain is not included merely because an AI web page loads it.

## Carrier caliber

Residential routing covers three kinds of traffic: in-browser Chat product sessions, local CLI calls to official endpoints, and desktop/IDE clients calling official AI endpoints. Everything else stays on the original Profile airport exit.

- **Carrier A (in-browser Chat)**: `claude.ai`, `claude.com`, `chatgpt.com`, `grok.com`, `gemini.google.com`, `aistudio.google.com`. One browser session concurrently hits several subdomains with the same cookies. Sending only some of those subdomains through the airport lets the server observe two exit IPs inside one signed-in session. Product apex domains stay suffix-matched by default.
- **Carrier B (local CLI and desktop/IDE clients)**: Claude Code, Codex, Grok CLI, Cursor, Antigravity. Non-inference requests (docs sites, update downloads, extension marketplaces, static assets) do not carry inference-session credentials, so they are narrowed to exact hosts from official docs.

## Included categories

| Product | Included traffic |
|---|---|
| Claude / Anthropic | Claude product domains, Messages API, `mcp-proxy.anthropic.com` MCP connector proxy, `assets-proxy.anthropic.com` asset proxy, `claudemcpcontent.com` MCP Apps widget isolation domain, `claudeusercontent.com` session content, and official inbound IP fallback |
| ChatGPT / OpenAI | ChatGPT product domain (full suffix, including `ws.chatgpt.com`), five official exact hosts (`chat.openai.com`, `android.chat.openai.com`, `desktop.chat.openai.com`, `ios.chat.openai.com`, `tcr9i.chat.openai.com`), OpenAI model API suffix `api.openai.com` (covers Codex official `us.` / `eu.` data-residency prefixes), and uploaded or generated user content. Optional `routing.openai_auth` only adds the bounded `auth.openai.com` suffix and exact `auth0.openai.com`. `routing.openai_web_assets` independently adds the `oaistatic.com` suffix. Both default to off. |
| Gemini | Gemini Web, Google AI Studio product RPC/streaming hosts, Gemini Developer API |
| Vertex AI / Agent Platform | `routing.vertex_ai_endpoints` defaults to `true` and controls `aiplatform.googleapis.com`, `aiplatform.us.rep.googleapis.com`, `aiplatform.eu.rep.googleapis.com`, and the regional regex `^[a-z0-9-]+-aiplatform\.googleapis\.com$` |
| Google Antigravity / Gemini Code Assist | Exact host `antigravity.google`, production Code Assist host `cloudcode-pa.googleapis.com`, and the Antigravity `language_server` `--cloud_code_endpoint` host `daily-cloudcode-pa.googleapis.com` |
| Cursor | Chat/API, Tab, Agent, Cloud Agent/Bugbot API, authorize endpoint, SSO admin portal `adminportal42.cursor.sh`, Cloud Agent VM hosts, and product-specific authentication; `routing.cursor_core` defaults to `true`. Repository indexing hosts `repo[0-9]+.cursor.sh` use the independent `routing.cursor_repository_indexing` switch, which defaults to `false` and falls back to the original Profile/airport upstream |
| Grok Build | `routing.grok_core` defaults to `true`. Default injects `DOMAIN-SUFFIX,grok.com` (covers `cli-chat-proxy.grok.com` inference API and `code.grok.com` session sync), `auth.x.ai` OAuth host, and `DOMAIN-SUFFIX,api.x.ai` (covers regional endpoints and `mtls.api.x.ai`). When `routing.grok_web_assets = false`, the `grok.com` suffix is replaced by three exact hosts: `grok.com`, `cli-chat-proxy.grok.com`, `code.grok.com`; the `api.x.ai` suffix is still injected |

Official sources:

- Claude Code / Desktop network config: https://code.claude.com/docs/en/network-config.md
- Claude Desktop MCP Apps widget wildcard `*.claudemcpcontent.com`
- OpenAI enterprise firewall list: https://help.openai.com/en/articles/9247338-network-recommendations-for-chatgpt-errors-on-web-and-apps
- Codex data-residency prefixes: `us.api.openai.com` / `eu.api.openai.com`
- Cursor enterprise network config: https://cursor.com/docs/enterprise/network-configuration
- xAI enterprise: https://docs.x.ai/build/enterprise ; regions: https://docs.x.ai/developers/regions ; mTLS: https://docs.x.ai/developers/advanced-api-usage/mtls
- Vertex AI endpoints and Antigravity Enterprise: https://antigravity.google/docs/enterprise
- Anthropic inbound ranges: https://platform.claude.com/docs/en/api/ip-addresses.md

Cursor evidence: the official enterprise network document lists exact hosts `authenticate.cursor.sh`, `adminportal42.cursor.sh`, and `*.cursorvm.com` VM hosts, plus the previously covered API, Tab, and Agent endpoints. `api2.cursor.sh` and `authenticate.cursor.sh` became `DOMAIN` exact matches from v5.10. Official network docs and local 2026-08-17 Cursor indexing logs jointly confirm `repo42.cursor.sh` as a repository-indexing host; `repo[0-9]+.cursor.sh` is this project's forward-compat policy, not an official Cursor wildcard contract. Default `routing.cursor_repository_indexing = false` only sends those indexing-only hosts back to the original Profile. It does not stop Chat/Agent from sending code context, and it cannot keep isolating indexing when `disableHttp2` or a server-forced HTTP/1.1 path puts RepositoryService on shared `api2.cursor.sh`. `api2.cursor.sh` stays under `routing.cursor_core`. Privacy Mode does not stop indexing uploads. Grok evidence: docs.x.ai/build/enterprise lists `cli-chat-proxy.grok.com` and `auth.x.ai` as required, `code.grok.com` as an optional session channel, and `assets.grok.com` as having no functional impact. v5.7 evidence: the Claude Code official network document lists `mcp-proxy.anthropic.com` and `assets-proxy.anthropic.com`.

Known trade-off: `downloads.claude.ai` (installer and auto-update host) sits under the `claude.ai` suffix, so it also uses the residential link. Splitting it out would require a rule that contains a dynamically resolved upstream name, which the current exact-string managed-rule cleanup model cannot safely remove. Update downloads are infrequent; the leftover cost is residential bandwidth. The `chatgpt.com` suffix also covers `help.`, `status.`, `ab.`, `events.`, and similar subdomains.

## Explicit exclusions

The following classes stay on the original Profile route:

- Cursor Marketplace, extension installation, application downloads, CDN, updates, Remote-SSH/WSL server assets, website, documentation, and forum.
- Grok Build third-party analytics (`api.mixpanel.com`), the `x.ai` install script/privacy endpoints, and the shared `storage.googleapis.com` backend for codebase uploads.
- YouTube, Maps, Google Search, Google Fonts, Gstatic, advertising, analytics, and other generic Google services.
- OpenAI/Claude customer support, telemetry, feature flags, fraud prevention, payment, and other shared third-party infrastructure.
- OpenAI first-party login hosts and `oaistatic.com` web assets also stay on the original Profile by default; they enter residential only after `routing.openai_auth` or `routing.openai_web_assets` is explicitly enabled, and enabling either does not enable shared third-party dependencies.
- Public DoH/DoT, generic STUN/TURN, and broad UDP port captures.
- Process-wide routing for Cursor, Grok, Claude, ChatGPT, and Antigravity.

Hosts no longer injected from v5.10, but still kept in `allPossible*` for upgrade cleanup:

- `clau.de`, `claudemcpclient.com` (no official source)
- `a-api.anthropic.com` (official Desktop list purpose is Analytics events / telemetry. With `routing.anthropic_ip_fallback` on, the host can still hit IP rules if it resolves to an inbound CIDR)
- `geminicloudassist.googleapis.com` (Cloud Assist MCP, not the Antigravity Agent gateway)

Hosts that no longer match after narrowing:

- `assets.grok.com` (excluded only when `routing.grok_web_assets = false`; official docs mark it as no functional impact)
- `docs.antigravity.google`, `download.antigravity.google`, `www.antigravity.google`
- `adminportal<N≠42>.cursor.sh` (for example `adminportal0.cursor.sh`, `adminportal999.cursor.sh`)
- `www.api2.cursor.sh`, `feature.api2.cursor.sh`

## UNVERIFIED

These judgments have official docs or unit-level rule-match evidence only, not post-deploy runtime evidence. After loading a new Profile, confirm with sanitized Clash Connections.

| Item | What is unverified |
|---|---|
| Four deactivated rules | Whether `clau.de`, `claudemcpclient.com`, `a-api.anthropic.com`, or `geminicloudassist.googleapis.com` actually carries session or inference traffic. `daily-cloudcode-pa.googleapis.com` was confirmed in v5.10.1 with local process arguments and Clash Connections, and was reactivated |
| Three `alkali*` hosts | Whether the AI Studio web app actually requests those `clients6.google.com` hosts |
| `antigravity.google` narrowing | Whether the Antigravity client hits `docs.` / `download.` subdomains, and whether disconnecting them affects startup or update prompts |
| `adminportal42.cursor.sh` narrowing | Whether enterprise SSO setup uses only host number 42 |
| `api.cursor.com` | Whether the Cursor desktop “Cloud” entry uses this host or `api2.cursor.sh`; official docs only prove API-key programmatic calls |
| `grok_web_assets = false` | Whether Grok web shows auth or asset exit-splitting in that mode |
| Exact `claude.com` enumeration | The full set of `*.claude.com` hosts on the login redirect chain |
| `tcr9i.chat.openai.com` | Official list member; purpose unpublished |
| OpenAI auth and web-asset switches | Node regressions only prove the rule and DNS switch contract for `auth.openai.com`, `auth0.openai.com`, and `oaistatic.com`. Real login redirects, Cloudflare/SSO/support dependencies, Ubuntu Clash host execution, and a single exit still need sanitized Connections |

## Acceptance rule for new domains

Add a domain only when all of the following hold:

1. It is used by model inference, response streaming, AI chat/session control, agent/tool execution, code completion, or repository indexing.
2. The evidence is an official document or a sanitized connection record tied to a reproducible feature.
3. The narrowest practical match can be expressed with `DOMAIN`, a constrained `DOMAIN-SUFFIX`, or a bounded `DOMAIN-REGEX`.
4. Negative tests prove that marketplace, update, download, media, advertising, analytics, and unrelated shared traffic remain outside `AI-家宽`.

## Authentication exit split

Authentication traffic stays on the original Profile by default. With OpenAI `routing.openai_auth = false`, `auth.openai.com`, its children such as `setup.auth.openai.com`, and exact `auth0.openai.com` do not enter residential, while ChatGPT core session and model traffic continue to use residential. With `routing.openai_web_assets = false`, `oaistatic.com` also stays on the original Profile. That default split is an intentional AI-only boundary.

To reduce the OpenAI first-party auth versus core-traffic exit split, set `routing.openai_auth = true`. That switch only adds `DOMAIN-SUFFIX,auth.openai.com` and `DOMAIN,auth0.openai.com`. It does not add `DOMAIN-SUFFIX,openai.com`, and it does not enable `routing.openai_web_assets` or `routing.openai_shared_dependencies`. WorkOS, Intercom, Stripe, Cloudflare Challenge, Sentry, Datadog, and other third-party redirects or dependencies may still use the airport exit. Turning the switch on does not prove the whole login chain shares one exit, and it does not guarantee fewer platform challenges. If `oaistatic.com` must share the exit, enable `routing.openai_web_assets` separately.

Google still uses the independent and broader `routing.antigravity_google_auth`. Off by default, `accounts.google.com` and other shared Google login entries stay on the airport; turning it on affects other Google products on the same account system. The OpenAI switches do not change that.

## Managed-rule ownership after v5.5

The script replaces rules that the current version can generate, including output from a switch that was later disabled. It no longer contains automatic migration lists for pre-v5.4 broad rules or retired v5.4 Cursor entries. Unknown rules targeting `AI-家宽` are treated as user-owned and preserved. If generated output was manually persisted in a subscription or Global Extend Config (Merge), remove stale entries there using the exact search list in [Troubleshooting](troubleshooting.md), then refresh the Profile.
