# Changelog

All notable changes are recorded here. The project follows Semantic Versioning for repository releases.

## [Unreleased]

### Planned

- Add sanitized real-profile integration fixtures.
- Add automated domain-source freshness checks where upstream providers publish machine-readable inventories.

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
