# Changelog

All notable changes are recorded here. The project follows Semantic Versioning for repository releases.

## [Unreleased]

### Planned

- Add sanitized real-profile integration fixtures.
- Add automated domain-source freshness checks where upstream providers publish machine-readable inventories.

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
