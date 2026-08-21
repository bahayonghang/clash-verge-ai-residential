# Changelog

All notable changes are recorded here. The project follows Semantic Versioning for repository releases.

## [Unreleased]

### Added

- Dedicated residential page with live monitor, category aggregation, share of attributed observation, and report export. New command `residential_share`. Coverage with `covered_sec == 0` returns four `None` fields, not 0%.
- Ported the reports, alerts, settings/data, recovery, and unavailable pages onto the React shell. Share charts use Recharts. Export, retention, backup, alert-rule, and about behavior stay the same.
- C3 report queries accept `minute1` / `minute2` / `minute5` / `minute10` granularity on the raw tier. Existing `hour` / `day` / `month` values stay unchanged.
- C3 materializes `process`, `rule_group`, `chain`, and `network` dimension rows in addition to `host`. Category ranking on the dimension tier groups `category_id` and keeps `dimension_kind = host` so traffic is not counted five times.
- Ranking identity `__unknown__` marks a missing dimension value. The row stays in the ranking so rank sums can match totals.

### Changed

- Replaced the vanilla TypeScript + Catppuccin shell with a React + Tailwind desktop UI. Navigation is ten routes. Overview, live connections, and host / rule / chain / process pages ship in the new shell. `src/main.ts` and `src/styles.css` are removed.
- Residential classification lives in one module with two named functions. Accounting uses exact target match. Live “residential only” still matches a configured target or a node name that contains 家宽.
- `ReportFilters` now apply to raw totals, series, and rankings, including category. `filters.chain` matches the last chain hop. `filters.rule` matches the SQL rule key.
- Dimension-layer `exact_top_n` is false when the grouping has no five-dimension materialization. Queries before the `hourly_dim_v2` watermark return `capability_unsupported`.

### Planned

- Add sanitized real-profile integration fixtures.
- Add automated domain-source freshness checks where upstream providers publish machine-readable inventories.

## [5.10.1] - 2026-08-20

### Fixed

- Restored `daily-cloudcode-pa.googleapis.com` to the active residential catalog. Antigravity `language_server` sets `--cloud_code_endpoint` to this host. v5.10.0 retired the host as undocumented. Local logs show TLS handshake failures, and Clash Connections send the host to the original Profile upstream.

### Changed

- Default injected `AI-家宽` rule count is 45.

## [5.10.0] - 2026-08-19

### Added

- New `routing.grok_web_assets` switch (default `true`). When the switch is `true`, the script injects `DOMAIN-SUFFIX,grok.com`. When the switch is `false`, the script injects exact hosts `grok.com`, `cli-chat-proxy.grok.com`, and `code.grok.com`.
- New `routing.vertex_ai_endpoints` switch (default `true`). The switch controls four Vertex AI / Agent Platform rules: `aiplatform.googleapis.com`, `aiplatform.us.rep.googleapis.com`, `aiplatform.eu.rep.googleapis.com`, and the regional regex `^[a-z0-9-]+-aiplatform\.googleapis\.com$`.

### Changed

- Retired five hosts from the active residential catalog. The hosts stay in `allPossible*` so upgrades can clean old rules: `clau.de`, `claudemcpclient.com`, `a-api.anthropic.com`, `daily-cloudcode-pa.googleapis.com`, and `geminicloudassist.googleapis.com`.
- Narrowed four rules: `api2.cursor.sh` and `authenticate.cursor.sh` from suffix to exact; the `adminportal` regex to `DOMAIN,adminportal42.cursor.sh`; `antigravity.google` from suffix to exact.
- Changed `api.x.ai` from exact to suffix so regional hosts and `mtls.api.x.ai` match.
- Default injected `AI-家宽` rule count is 44.

### Notes

- `chatgpt.com` stays a suffix. Subdomains such as `help.` and `status.` stay on the residential link.
- Three `alkali*` AI Studio hosts stay in `gemini_web_core` and remain UNVERIFIED.
- `claudemcpcontent.com` stays a suffix for Claude Desktop MCP App widgets.

## [5.9.0] - 2026-08-18

### Added

- New `routing.cursor_repository_indexing` switch (default `false`) for Cursor repository-indexing hosts `repo[0-9]+.cursor.sh`. A missing local TOML field is completed as `false`. Set the field to `true` to restore v5.8.1 residential routing for those hosts without deleting the key.

### Changed

- Repository-indexing regexes are no longer part of `routing.cursor_core`. By default, `repo42.cursor.sh` and other `repo<N>.cursor.sh` hosts fall back to the original Profile/airport upstream. Cursor Chat, Tab, Agent, auth, and Cloud Agent stay on `routing.cursor_core` (still default `true`). `api2.cursor.sh` stays on cursor_core.

### Notes

- Official docs and local 2026-08-17 logs jointly confirm `repo42.cursor.sh` as the indexing host.
- `repo[0-9]+.cursor.sh` is this project's forward-compat policy, not an official Cursor wildcard contract.
- Privacy Mode does not stop indexing uploads.
- `disableHttp2` or a server-forced HTTP/1.1 fallback can put RepositoryService on shared `api2.cursor.sh`. Clash domain rules cannot isolate that path. This release does not claim that all repository uploads leave the residential link.

## [5.8.1] - 2026-08-17

### Changed

- Build one outbound name index during `main` so large airport profiles do not scan every proxy for each reachable leaf.
- Collapse reachable `udp: false` leaf warnings into one summary (at most 8 samples).

## [5.8.0] - 2026-08-17

### Added

- Five official ChatGPT exact hosts from OpenAI help article 9247338: `chat.openai.com`, `android.chat.openai.com`, `desktop.chat.openai.com`, `ios.chat.openai.com`, and `tcr9i.chat.openai.com`. The purpose of `tcr9i.chat.openai.com` is undocumented. Native ChatGPT desktop/iOS Connections results remain UNVERIFIED.

### Changed

- Restored `OPENAI_CORE_EXACT_DOMAINS` under `routing.openai_core`. Generated output uses exact `DOMAIN` rules and bare DNS keys only. A cleanup-only `chat.openai.com` suffix entry removes a mistaken `DOMAIN-SUFFIX,chat.openai.com` rule and `+.chat.openai.com` policy key; that suffix is never re-injected.

## [5.7.0] - 2026-08-16

### Added

- Claude catalog additions from the official Claude Code network configuration document: the `mcp-proxy.anthropic.com` MCP connector proxy and the `assets-proxy.anthropic.com` desktop/web asset proxy (the official document warns that blocking it breaks the app UI).
- Grok catalog additions from the official xAI enterprise deployment document: the `auth.x.ai` OAuth2/OIDC host (must-allow) and the `api.x.ai` direct API inference endpoint. The `x.ai` install host stays on the original Profile.
- A `warn` log when references to `AI-家宽` / `家宽-SOCKS5` are removed from a reachable upstream group. The recursion-prevention cleanup is no longer silent; the log names the group, the removed entries, and how to route AI traffic instead.
- An `info` log documenting that current Clash Verge Rev restores `tun` / `ipv6` authoritative fields after the global script runs; TUN DNS hijack and the IPv6 toggle must be configured in the app settings page. Docs now describe this host behavior and the fake-ip DNS resolution timing.

### Changed

- `api.openai.com` moved from an exact rule to a suffix rule so the official Codex data-residency prefixes `us.api.openai.com` / `eu.api.openai.com` also match. Rules generated by v5.6 in exact form are still cleaned up idempotently.

## [5.6.0] - 2026-08-16

### Added

- New `routing.openai_core` switch (default `true`) controlling ChatGPT product, OpenAI model API, and uploaded/generated user-content routing. Setting it to `false` in the local TOML keeps GPT traffic on the airport upstream instead of the residential link.
- New `routing.grok_core` switch (default `true`) routing the Grok Build (xAI grok CLI) inference API `cli-chat-proxy.grok.com` (`/v1/responses` inference and `/v1/storage` codebase/session uploads) plus the Grok product domain through the residential link. Grok third-party analytics (`api.mixpanel.com`), the `x.ai` install host, and shared `storage.googleapis.com` stay on the original Profile.
- Cursor catalog additions from the official enterprise network configuration document: the `authenticate.cursor.sh` authorize endpoint, the `adminportal<N>.cursor.sh` SSO portal (bounded regex), and the `*.cursorvm.com` Cloud Agent VM hosts. Marketplace, CDN, download, and update hosts remain excluded.
- Local TOML auto-completion during `just render-local` / `node scripts/sync-local-config.js`: missing switch keys (including a missing `[routing]` / `[runtime]` table) are appended to the local TOML using the example defaults. Existing values, comments, line endings, and the trailing newline are preserved verbatim; completion is idempotent, and missing `[home_proxy]` credential keys still fail closed.

### Changed

- `routing.cursor_core` now defaults to `true`: Cursor rules and DNS policy are injected without opt-in. Set it to `false` in the local TOML to keep Cursor on the airport upstream.

### Fixed

- Route the observed Anthropic core API host `a-api.anthropic.com` through the residential connection and DNS paths without broadening the default scope to all `anthropic.com` traffic.

## [5.5.0] - 2026-07-23

### Added

- Optional `[routing]` and `[runtime]` local TOML tables covering every scalar user switch while preserving home-proxy-only configuration files.
- Exact-one boolean-anchor validation and atomic local-script rendering for partial switch overrides.
- Validation that rejects upstream names containing `#` or `&` before constructing a Mihomo DoH URL.
- Complete `just render-local` and direct Node setup paths, with the Clash Verge Rev Global Extend Script screenshot.

### Changed

- Cursor core routing is now opt-in and disabled by default; the narrow catalog remains available through `routing.cursor_core = true`.
- Removed three redundant Cursor catalog matches covered by retained suffix or bounded repository rules.
- Current-version managed rules are still replaced when switches change, while unknown rules targeting `AI-家宽` remain user-owned.
- Documented the retained strict-DNS first-query latency trade-off and the original-Profile login versus residential model-traffic exit split.

### Removed

- Removed the unreleased pre-v5.4 legacy migration catalogs, retargeting, group-reference migration, and legacy-group cleanup.
- If v5.4 generated output was manually persisted in a subscription or Merge layer, remove these now-retired user-owned rules there before refreshing:
  - `DOMAIN,repo42.cursor.sh,AI-家宽`
  - `DOMAIN-REGEX,^[a-z0-9-]+\.api5\.cursor\.sh$,AI-家宽`
  - `DOMAIN-REGEX,^(?:us-asia|us-eu|us-only)\.gcpp\.cursor\.sh$,AI-家宽`

## [5.4.0] - 2026-07-22

### Added

- Stable public entry file: `clash-verge-ai-residential.js`.
- AI-only routing for Claude, ChatGPT, Gemini, Google Antigravity, and Cursor core inference/agent traffic.
- Multi-Profile `dialer-proxy` resolution with `🚀节点选择` as the preferred default.
- Recursive proxy-group and `include-all` protection.
- AI-specific DNS policy with non-AI overseas DNS bound to the current Profile upstream.
- 28 configuration-level regression tests.
- CI across Node.js 18, 20, and 22.
- Template safety check to reject committed residential SOCKS5 credentials.

### Changed

- Cursor Marketplace, downloads, CDN, update assets, YouTube, Maps, advertising, and shared telemetry are explicitly excluded from the residential route.
- Versioned archive filenames were replaced by stable repository paths; release versions are represented by Git tags.

### Security

- Residential endpoint and credentials remain placeholders in the public template.
- Runtime configuration fails closed when required credentials or upstream groups cannot be resolved safely.
