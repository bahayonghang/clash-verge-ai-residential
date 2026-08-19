# Research: Claude / Claude Code / Claude Desktop / claude.ai official backend destinations

- **Query**: Deep-verify Claude / Claude Code / Claude Desktop / claude.ai official BACKEND request destinations (inference, session, auth token exchange), not docs/status/marketing. Host vs path: Clash cannot match `/v1/messages` separately from other `api.anthropic.com` paths.
- **Scope**: mixed (official Anthropic docs + current script lists; no sanitized Connections capture)
- **Date**: 2026-08-19

## Findings

### Files Found

| File Path | Description |
|---|---|
| `clash-verge-ai-residential.js:185-217` | Active Claude suffix/exact hosts: `claude.ai`, `claude.com`, `clau.de`, `claudemcpclient.com`, `claudemcpcontent.com`, `claudeusercontent.com`, `api.anthropic.com`, `a-api.anthropic.com`, `mcp-proxy.anthropic.com`, `assets-proxy.anthropic.com` |
| `clash-verge-ai-residential.js:335-346` | Shared deps (default off): Statsig, Intercom, Sentry, Datadog |
| `clash-verge-ai-residential.js:377-382` | Auxiliary (not default-on): `storage.googleapis.com`, `raw.githubusercontent.com`, `formulae.brew.sh`, `registry.npmjs.org` |
| `clash-verge-ai-residential.js:434-438` | Inbound IP fallback: `160.79.104.0/23`, `2607:6bc0::/48` |
| `docs/routing-scope.md:9,18` | Scope text; known over-proxy of `downloads.claude.ai` under `DOMAIN-SUFFIX,claude.ai` |
| `.trellis/tasks/08-19-ai-route-domain-audit/research/anthropic-domains.md` | Earlier pass. Several rows are superseded below (see Corrections). |

### Code Patterns

Active Claude matching is suffix on product apexes plus four `anthropic.com` exact hosts. Clash rules match hostname only. A `DOMAIN` or `DOMAIN-SUFFIX` rule cannot select `/v1/messages` and exclude feature-flag or telemetry paths on the same host.

### External References

Official pages fetched 2026-08-19 (markdown and HTML):

- [Enterprise network configuration](https://code.claude.com/docs/en/network-config) — CLI allowlist table
- [Desktop application — Network access requirements](https://code.claude.com/docs/en/desktop#network-access-requirements) — Desktop/web host lists
- [IP addresses](https://platform.claude.com/docs/en/api/ip-addresses) — inbound vs outbound CIDR
- [Self-hosted environments deploy — Network requirements](https://code.claude.com/docs/en/self-hosted-environments-deploy#network-requirements)
- [Configure cloud environments — Default allowed domains](https://code.claude.com/docs/en/cloud-environments#default-allowed-domains) — VM egress, not the user client
- [Authentication](https://code.claude.com/docs/en/authentication)
- [MCP](https://code.claude.com/docs/en/mcp)
- [API overview](https://platform.claude.com/docs/en/api/overview)
- [Data usage — Telemetry services](https://code.claude.com/docs/en/data-usage)
- [Claude Desktop on 3P — Telemetry and egress](https://claude.com/docs/third-party/claude-desktop/telemetry)
- [Tenant Restrictions](https://support.claude.com/en/articles/13198485-enforce-network-level-access-control-with-tenant-restrictions)
- [Custom connectors / remote MCP](https://support.claude.com/en/articles/11175166-get-started-with-custom-connectors-using-remote-mcp)
- [Claude Science network requirements](https://claude.com/docs/claude-science/network-requirements)
- [status.claude.com](https://status.claude.com) / [status.claude.ai](https://status.claude.ai) — product split on the status page

### Related Specs

- `.trellis/tasks/08-19-ai-route-domain-audit/prd.md` — residential route is for chat session, official CLI endpoints, and official desktop/IDE AI endpoints
- `.trellis/tasks/08-19-ai-route-domain-audit/research/anthropic-domains.md` — first-pass domain table; see Corrections

---

## Clash host vs path (constraint)

Clash `DOMAIN` / `DOMAIN-SUFFIX` / `DOMAIN-REGEX` match the hostname (and SNI). They do not match URL path.

Official 3P Desktop egress text:

> All traffic is HTTPS on port 443. Allowlist by hostname (SNI); path-level rules aren't required.

Source: https://claude.com/docs/third-party/claude-desktop/telemetry

Consequence: a rule for `api.anthropic.com` carries every path on that host. Anthropic states that this host is not inference-only.

---

## 1. Claude Code CLI: inference vs login

### Official CLI allowlist (full table)

Source: https://code.claude.com/docs/en/network-config

> Claude Code requires access to the following URLs. Allowlist these in your proxy configuration and firewall rules, especially in containerized or restricted network environments. The first-run setup connectivity check points here when it can't reach `api.anthropic.com` or `platform.claude.com`

| URL | Official “Required for” (quoted) | Role for this audit |
|---|---|---|
| `api.anthropic.com` | “Claude API requests, including the WebFetch domain safety check, feature flag fetches, and telemetry event logging” | Inference channel. Same host also carries feature flags and telemetry. **Cannot split in Clash.** |
| `claude.ai` | “claude.ai account authentication” | Login / session for subscription accounts. Not described as the CLI model endpoint. |
| `claude.com` | “claude.ai account sign-in opens a `claude.com` page in the browser, which redirects to `claude.ai`; pre-approved WebFetch documentation lookups also reach this host from the CLI” | Login landing + docs fetch. Not the model endpoint. |
| `platform.claude.com` | “Anthropic Console account authentication. OAuth token exchange, refresh, and revocation also go to this host for claude.ai accounts, so both Console and claude.ai sign-ins require it” | **Auth token exchange** for Console **and** claude.ai. Required for `/login` even when inference is `api.anthropic.com`. |
| `mcp-proxy.anthropic.com` | “MCP connectors from claude.ai … Connector traffic routes through this proxy; connectors are enabled by default for claude.ai-authenticated users.” | Connector proxy, not Messages API. |
| `downloads.claude.ai` | “Plugin executable downloads; native installer, native auto-updater, and update version checks” | Installer / updater. Not inference. |
| `storage.googleapis.com` | Plugin metadata / Artifact upload (fallback to `api.anthropic.com` if blocked); pre-2.1.116 installer | Shared Google host. Not first-party AI. |
| `registry.npmjs.org` | Plugin / npx MCP / npm install of Claude Code | Package registry. |
| `bridge.claudeusercontent.com` | “Claude in Chrome extension WebSocket bridge” | Live browser-bridge channel. |
| `*.frame.claudeusercontent.com` | “Artifact content reads. The CLI fetches an artifact's files from this host when Claude opens one” | Artifact file fetch. |
| `raw.githubusercontent.com` | Changelog / release notes | Docs feed. |
| `http-intake.logs.us5.datadoghq.com` | “Operational telemetry events … Optional: disable with DISABLE_TELEMETRY or DO_NOT_TRACK” | Telemetry. |
| `browser-intake-us5-datadoghq.com` | “Operational error reports … Optional” | Error reporting. |
| `formulae.brew.sh` | Homebrew update checks | Package metadata. |
| `code.claude.com` | “Claude Code documentation lookups by the built-in claude-code-guide agent and pre-approved WebFetch requests. Blocking this host only affects documentation lookups” | Documentation site / docs fetch. **Not a backend.** |

Same page, third-party providers:

> When using Amazon Bedrock, Google Cloud's Agent Platform, Microsoft Foundry, or a signed-in Claude apps gateway session, model traffic and authentication go to your provider or gateway instead of `api.anthropic.com`, `claude.ai`, or `platform.claude.com`.

Default API base (platform docs):

> The Claude API is a RESTful API at `https://api.anthropic.com` … Messages API … (`POST /v1/messages`)

Source: https://platform.claude.com/docs/en/api/overview

Self-hosted runner (session process, not the developer laptop):

> `api.anthropic.com` … Runner control plane and session streaming, model inference, feature flags, product analytics …

Source: https://code.claude.com/docs/en/self-hosted-environments-deploy

### CLI host split (official)

| Host | Inference | Session / control plane | Auth (login + token exchange) | Docs / install / telemetry |
|---|---|---|---|---|
| `api.anthropic.com` | Yes (default `ANTHROPIC_BASE_URL`) | Yes for self-hosted runner streaming | No (OAuth browser/token host is `platform.claude.com`) | Yes: feature flags + “telemetry event logging” on this same host |
| `claude.ai` | No for standalone CLI | No | Yes: “account authentication” | 3P Desktop also sends analytics events here (see §3) |
| `platform.claude.com` | No | No | Yes: Console login **and** OAuth token exchange / refresh / revocation for claude.ai accounts | Console UI / docs live here too (`docs.claude.com` redirects to `platform.claude.com/docs`) |
| `code.claude.com` | No | No | No | Docs only. Official: blocking “only affects documentation lookups” |

First-run connectivity check names `api.anthropic.com` **or** `platform.claude.com`, which matches “API host vs auth host”.

---

## 2. claude.ai web chat: conversation / streaming hosts

Official sources do **not** publish a path-level web-chat API. They do split the **products**:

status.claude.com / status.claude.ai list distinct components:

- `claude.ai`
- Claude Console (`platform.claude.com`)
- Claude API (`api.anthropic.com`)
- Claude Code
- Claude Cowork

Tenant Restrictions proxy application list:

> Application: `claude.ai`, `api.anthropic.com`, `claude.com`, `anthropic.com`

Source: https://support.claude.com/en/articles/13198485-enforce-network-level-access-control-with-tenant-restrictions

Desktop/web extra hosts (same network-config page, section “Desktop and claude.ai”):

> The preceding table covers the standalone CLI. The Claude Desktop app and claude.ai in a browser load their application code and user content from additional Anthropic CDN hosts, including `assets-proxy.anthropic.com` and the other `*.claudeusercontent.com` origins that serve artifacts in those apps. Allowing `claude.ai` while blocking those hosts produces a blank page rather than an error.

Desktop reduced-wildcard list (official, not CLI table) names these `claude.ai` hosts explicitly:

```
claude.ai
a.claude.ai
a-cdn.claude.ai
assets.claude.ai
downloads.claude.ai
*.livepreview.claude.ai
```

Source: https://code.claude.com/docs/en/desktop#network-access-requirements

**Conclusion from official text:** web chat is not “claude.ai apex only”. Anthropic lists additional `claude.ai` subdomains for Desktop/web. Purpose of `a.claude.ai` is not stated. Whether the browser posts conversation SSE to apex `claude.ai`, to `a.claude.ai`, or also to `api.anthropic.com` is **UNVERIFIED** (no official path map).

Community captures (not official; do not use as allowlist evidence):

- `https://claude.ai/api/organizations/{org}/chat_conversations/.../completion` (cookie `sessionKey`, SSE)
- `wss://claude.ai/v1/sessions/ws/{id}/subscribe` (claude.ai/code web UI)
- CLI cloud sessions use `https://api.anthropic.com` (`/v1/sessions`, `/v1/session_ingress/...`)

Mark **UNVERIFIED**. Clash cannot match those paths even if they were confirmed.

HTTP fetch 2026-08-19 (Exa, not local DNS): `https://www.claude.ai` serves the product; `https://status.claude.ai` serves the status page.

---

## 3. Claude Desktop

### Standard Desktop (Anthropic account)

Wildcard allowlist:

```
anthropic.com
*.anthropic.com
claude.ai
*.claude.ai
claude.com
*.claude.com
claude.app
*.claude.app
*.claudeusercontent.com
*.claudemcpcontent.com
```

Reduced list (official comment: “Certain subdomains are dynamically generated and must remain wildcards”):

```
anthropic.com
api.anthropic.com
a-api.anthropic.com
a-cdn.anthropic.com
s-cdn.anthropic.com
assets-proxy.anthropic.com
claude.ai
a.claude.ai
a-cdn.claude.ai
assets.claude.ai
downloads.claude.ai
*.livepreview.claude.ai
claude.com
platform.claude.com
*.livepreview.claude.app
*.claudeusercontent.com
*.claudemcpcontent.com
```

Source: https://code.claude.com/docs/en/desktop#network-access-requirements

Auth for standard Desktop:

> Claude Desktop and cloud sessions do not call `apiKeyHelper` or read these environment variables: they use OAuth, except desktop sessions running a third-party inference configuration

Source: https://code.claude.com/docs/en/authentication

Local/SSH Code tab signed in through claude.ai uses the same CLI network variables as a terminal session. Cowork / 3P Code tab: the app owns the provider connection.

### Claude Desktop on 3P (inference off Anthropic)

Source: https://claude.com/docs/third-party/claude-desktop/telemetry

> When Claude Desktop on third-party (3P) is configured with Google Cloud's Agent Platform, Amazon Bedrock, or Microsoft Foundry, the app sends conversation content only to your configured inference endpoint.

> All traffic is HTTPS on port 443. Allowlist by hostname (SNI); path-level rules aren't required.

| Host | Official purpose in 3P egress tables |
|---|---|
| `downloads.claude.ai` | Always required: “VM workspace bundle and Claude CLI binary, fetched at session start. **Without this, Cowork sessions cannot start**” (unless offline installer) |
| `claude.ai` | Auto-update feed **and** “Analytics events” |
| `api.anthropic.com` | Auto-update feed; “Claude Code usage telemetry”; “MCP connector directory” |
| `a-api.anthropic.com` | “Analytics events” |
| `a-cdn.anthropic.com` | “Analytics SDK” |
| `www.claudeusercontent.com` | “Artifact preview iframe” |
| `*.claudemcpcontent.com` | MCP Apps widgets; “own generated subdomain” |
| `assets.claude.ai` | “Fonts loaded by MCP App widget iframes” |
| `releases.claude.com` | Optional update feed when `updateViaUpdatesHost` is true (instead of `claude.ai` + `api.anthropic.com` for the feed) |

This is the **only official purpose statement** for `a-api.anthropic.com`: analytics events, in 3P Desktop. Standard Desktop lists the same host in the firewall set and does not name a purpose. Treat standard-mode role of `a-api.anthropic.com` as **UNVERIFIED** beyond “allowlisted next to `api.anthropic.com`”. Do not treat it as the Messages API host.

`s-cdn.anthropic.com` appears only on the standard Desktop reduced list. Purpose **UNVERIFIED**.

---

## 4. Over-proxy hosts under `DOMAIN-SUFFIX,claude.ai` / `DOMAIN-SUFFIX,claude.com`

Current script uses both suffixes (`clash-verge-ai-residential.js:187-188`). Any hostname under those apexes matches `AI-家宽`.

Classification below uses official text where it exists. HTTP existence checked via Exa fetch 2026-08-19. Local `nslookup` is **not** evidence: this machine’s Clash fake-ip answers `198.18.0.0/16` for every name, including queries sent to `8.8.8.8`.

| Host | Under which suffix | Official role | Audit class |
|---|---|---|---|
| `claude.ai` | apex | CLI: account authentication. Status component. Tenant-restriction target. 3P: update feed + analytics events. Web product. | Auth + web session surface. Streaming path **UNVERIFIED**. |
| `www.claude.ai` | `claude.ai` | Serves product (HTTP fetch). Not named in reduced Desktop list. | Product alias. **UNVERIFIED** as a distinct backend. |
| `a.claude.ai` | `claude.ai` | Named in Desktop reduced list. No purpose text. | **UNVERIFIED** (could be API-like; no official path). |
| `a-cdn.claude.ai` | `claude.ai` | Named in Desktop reduced list. No purpose text. | CDN. Not inference. |
| `assets.claude.ai` | `claude.ai` | Desktop list. 3P: “Fonts loaded by MCP App widget iframes”. | Assets. |
| `downloads.claude.ai` | `claude.ai` | Installer, auto-updater, plugin binaries, Cowork VM bundle. | Install/update. Already recorded in `docs/routing-scope.md`. |
| `*.livepreview.claude.ai` | `claude.ai` | Desktop reduced list; “dynamically generated”. | Preview. Not Messages API. |
| `status.claude.ai` | `claude.ai` | HTTP fetch: same status product as `status.claude.com`. | Status page. Not backend. |
| `cdn.claude.ai` | `claude.ai` | Not in official reduced list. | **UNVERIFIED**. |
| `code.claude.ai` | `claude.ai` | HTTP fetch failed. Not in official reduced list. Product path is `claude.ai/code`. | **UNVERIFIED**. Do not assume it is a backend. |
| `claude.com` | apex | Login landing that redirects to `claude.ai`; CLI docs WebFetch. | Auth landing + docs. |
| `www.claude.com` | `claude.com` | Marketing / pricing / “Continue with Google” (HTTP fetch). | Marketing + possible login landing. |
| `platform.claude.com` | `claude.com` | Console auth. OAuth token exchange/refresh/revocation for claude.ai **and** Console. Status: “Claude Console”. | **Auth backend** (also serves Console UI and docs). |
| `code.claude.com` | `claude.com` | Docs site. CLI: “Blocking this host only affects documentation lookups”. | Documentation. **Not a backend.** |
| `docs.claude.com` | `claude.com` | HTTP fetch redirects to `platform.claude.com/docs`. Listed in cloud-environment default allowlist (VM egress). | Documentation alias. |
| `status.claude.com` | `claude.com` | Official status page. | Status. Not backend. |
| `support.claude.com` | `claude.com` | Anthropic Help Center. | Support/docs. Not backend. |
| `console.claude.com` | `claude.com` | HTTP fetch: “Claude Platform” Console shell. Not in Desktop reduced list (`platform.claude.com` is). | Console UI alias. **UNVERIFIED** whether it is required if `platform.claude.com` is already allowed. |
| `blog.claude.com` | `claude.com` | HTTP fetch failed. | **UNVERIFIED**. Marketing if it exists. |
| `releases.claude.com` | `claude.com` | 3P optional update feed. | Updates. Not inference. |
| `console.anthropic.com` | n/a (not under those suffixes) | Claude Science: “sign-in fallback page” with one-time code. HTTP fetch failed here. | Auth fallback. **UNVERIFIED** for Claude Code/Desktop. |

`claude.app` / `*.livepreview.claude.app` are official Desktop hosts and are **not** covered by `claude.ai` / `claude.com` suffixes.

---

## 5. `code.claude.com`: documentation site vs backend

Official (CLI table):

> `code.claude.com` — Claude Code documentation lookups by the built-in claude-code-guide agent and pre-approved WebFetch requests. Blocking this host only affects documentation lookups.

Official (self-hosted):

> `code.claude.com` and `claude.com` — Documentation lookups by the built-in claude-code-guide agent and pre-approved WebFetch requests during sessions. Blocking these hosts only affects documentation lookups.

The docs themselves are served from `https://code.claude.com/docs/...`. This host is a documentation site that the CLI may fetch. It is not listed as API, OAuth, or session streaming.

`claude.ai/code` is the Claude Code on the web **product** (different host).

---

## 6. MCP connector: `mcp-proxy.anthropic.com`

CLI table (quoted above): connector traffic for claude.ai MCP connectors, including org-admin connectors, routes through this proxy. Default on for claude.ai-authenticated users. Disable: `ENABLE_CLAUDEAI_MCP_SERVERS=false` or `disableClaudeAiConnectors`.

Help Center (custom remote MCP):

> When you add a custom connector, Claude connects to your remote MCP server from Anthropic's cloud infrastructure, rather than from your local device. This is true across every Claude client, including claude.ai, Claude Desktop, Cowork, and the mobile apps.

> Even though Cowork and Claude Desktop run on your computer, remote connectors are configured and brokered through your Claude account. The connection to your MCP server originates from Anthropic's servers, not from your machine's network interface.

Source: https://support.claude.com/en/articles/11175166-get-started-with-custom-connectors-using-remote-mcp

Those Anthropic-originated calls use **outbound** IPs (`160.79.104.0/21`), not the client’s residential path.

Self-hosted sessions do **not** use `mcp-proxy.anthropic.com`:

> `mcp-proxy.anthropic.com` isn't required either: self-hosted sessions don't use it, and delivery of your organization's claude.ai connectors to sessions, when enabled for your organization, routes through `api.anthropic.com`.

Allowlist paths named for delivered connectors on that host: `/v2/ccr-sessions/*`, `/v1/code/sessions/*`, `/v1/code/mcp/*`.

Local MCP in Desktop via `claude_desktop_config.json` uses the local network, not this proxy.

Claude Science (separate product) uses `*.mcp.claude.com` for hosted science connectors. That host is **not** in the Claude Code/Desktop tables.

---

## 7. `claudeusercontent.com` / `assets-proxy.anthropic.com` — session-bearing or assets?

### `assets-proxy.anthropic.com`

Official: Desktop and claude.ai “load their application code and user content” from this host (and other `*.claudeusercontent.com` origins). Blocking `claude.ai` while blocking this host “produces a blank page rather than an error”.

This is an app/CDN + user-content proxy required for UI. Official text does not call it the chat completion endpoint. Blocking it breaks the app. Session-cookie use on this host is **UNVERIFIED**.

### `claudeusercontent.com` (suffix)

Official named names:

| Host | Official purpose | Session-bearing? |
|---|---|---|
| `bridge.claudeusercontent.com` | Claude in Chrome **WebSocket bridge** | Yes: live WS control channel, not static assets |
| `*.frame.claudeusercontent.com` | Artifact file reads (CLI and cloud sessions) | User content / artifact files, not Messages API |
| `www.claudeusercontent.com` | 3P: “Artifact preview iframe” | Preview iframe |
| other `*.claudeusercontent.com` | “origins that serve artifacts” | Artifact origins; Desktop requires the wildcard because names are generated |

Not “just assets”. The Chrome bridge is a live channel. Artifact frames carry user-generated pages. They are not the CLI Messages API.

### `*.claudemcpcontent.com`

Official Desktop wildcard. 3P: MCP Apps widgets on generated subdomains. Interactive connector UI, not inference.

---

## 8. Official inbound IPs `160.79.104.0/23` and `2607:6bc0::/48`

Source: https://platform.claude.com/docs/en/api/ip-addresses

> Anthropic services use fixed IP addresses for both inbound and outbound connections. You can use these addresses to configure your firewall rules for secure access to the Claude API and Console. These addresses will not change without notice.

**Inbound** (where Anthropic **receives** client connections):

- IPv4 `160.79.104.0/23`
- IPv6 `2607:6bc0::/48`

**Outbound** (Anthropic **originates** MCP / web search / web fetch to customer servers):

- IPv4 `160.79.104.0/21`

Phased out (do not allowlist): `34.162.46.92/32`, `34.162.102.82/32`, `34.162.136.91/32`, `34.162.142.92/32`, `34.162.183.95/32`.

Script templates (`clash-verge-ai-residential.js:434-438`) use inbound `/23` and `/48`. Direction matches “client → Anthropic”. The page titles the ranges for “Claude API **and Console**”, so IP fallback cannot separate inference from Console.

`/23` inbound is a subset of `/21` outbound numerically; they are different roles.

Claude Platform on AWS inbound is AWS IP ranges, not these CIDRs.

---

## Auth token exchange (all surfaces)

Official chain for Claude Code `/login` with a claude.ai account:

1. Browser opens `claude.com`, which redirects to `claude.ai`.
2. OAuth token exchange, refresh, and revocation go to `platform.claude.com` (required for **both** Console and claude.ai).
3. After login, model requests go to `api.anthropic.com` (unless a third-party provider or `ANTHROPIC_BASE_URL` is set).

Claude Science repeats the same split: `claude.ai` = browser sign-in; `platform.claude.com` = “Completing sign-in (the OAuth token exchange)”; `api.anthropic.com` = “The Claude API for every request Claude makes”.

Source: https://claude.com/docs/claude-science/network-requirements

`claude setup-token` uses the same browser authorization flow as `/login`. The resulting token “can only make model requests” and cannot fetch claude.ai connectors.

---

## Telemetry on the inference host (official)

CLI table assigns **feature flag fetches and telemetry event logging** to `api.anthropic.com` in the same cell as “Claude API requests”.

Data-usage: metrics and error reports also go to Datadog hosts; those are optional. Feature-flag evaluation used by Remote Control is tied to `DISABLE_TELEMETRY` / `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`.

Self-hosted: “feature-flag fetches go to `api.anthropic.com`” and the runner does not need `statsig.anthropic.com`.

Cloud-environment **Trusted** VM allowlist still includes `statsig.anthropic.com` (VM egress, not the laptop client).

3P Desktop: `api.anthropic.com` is used for update feed, sandbox usage telemetry, and MCP directory even when inference is Bedrock/Vertex/Foundry.

**Clash cannot drop telemetry on `api.anthropic.com` while keeping `/v1/messages`.** Anthropic already documents mixed use of that host.

---

## Corrections to `research/anthropic-domains.md`

| Earlier row | This pass |
|---|---|
| `a-api.anthropic.com` — “无官方出处”, recommended 退出激活 | **Has official listing** on Desktop reduced allowlist. 3P telemetry table: “Analytics events”. Not the documented Messages API host. |
| `claudemcpcontent.com` — “无官方出处” | **Official** Desktop wildcard `*.claudemcpcontent.com` (MCP Apps widgets). |
| `assets-proxy` / `claudeusercontent.com` | Confirmed again: official Desktop/web; Chrome bridge is session-bearing WS. |
| `code.claude.com` as “本机 CLI 连接官方端点” in the over-proxy list | Official purpose is documentation lookup only. Blocking “only affects documentation lookups”. |

---

## Caveats / Not Found

- No official document lists claude.ai web-chat URL paths (`/api/organizations/.../chat_conversations/...`). Community paths are **UNVERIFIED**.
- No official purpose text for `a.claude.ai`, `a-cdn.claude.ai`, `s-cdn.anthropic.com` beyond “allow these hosts”.
- Standard-mode role of `a-api.anthropic.com` besides the 3P “Analytics events” row is **UNVERIFIED**.
- No sanitized Clash Connections capture in this pass. Host existence via local DNS is invalid under fake-ip.
- `blog.claude.com`, `code.claude.ai`, `cdn.claude.ai`, `console.anthropic.com`: no successful fetch and/or no Desktop reduced-list entry.
- `clau.de`, `claudemcpclient.com`: still no official network-config / Desktop / 3P egress listing.
- Cloud “Default allowed domains” is VM outbound (package registries, `docs.claude.com`, `statsig.anthropic.com`, …). It is not a laptop-client inference map.
- IP inbound ranges cover API **and Console**. They are not an inference-only signal.
