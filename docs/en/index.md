# Clash Verge AI Residential

A Clash Verge Rev global extension script. It sends only core AI traffic from Claude, ChatGPT, Gemini, Antigravity, Cursor, and Grok Build through a residential SOCKS5 chain. Marketplace, downloads, YouTube, and other non-AI traffic stay on the original Profile.

```text
this machine -> current Profile airport group/node -> residential SOCKS5 -> AI service
```

The full boundary is in [Routing scope](routing-scope.md). Do not look for switch tables or domain lists on this page.

## Open the docs site locally

This site is local preview only. It is not published to GitHub Pages. Node.js 22+ is required. From the repository root:

```bash
npm --prefix docs install
just docs-dev
```

Without `just`, use `npm run docs:dev`. Build with `just docs-build`. The extension gate `just ci` still runs on Node 18+ and does not install the docs dependencies.

The Chinese site is at [/](/). Chinese sources live at `docs/*.md`.

## Usage

- [Local configuration](local-configuration.md): local TOML, `just render-local`, credential handling, and the full switch table
- [Configuration](configuration.md): two usage modes, upstream candidates, Clash Verge settings
- [Routing scope](routing-scope.md): includes, exclusions, UNVERIFIED items, new-domain admission
- [Multi-profile](multi-profile.md): `dialer-proxy` resolution order and recursion protection
- [DNS and leak model](dns-and-leak-model.md): DNS paths and leaks the script cannot cover alone
- [Troubleshooting](troubleshooting.md): common failures and leftover rules

The root [README.md](https://github.com/bahayonghang/clash-verge-ai-residential/blob/dev/README.md) remains the GitHub overview. GitHub also opens the Chinese Markdown files under `docs/`.

## Agent

These pages are for in-repo agent skills, not end-user manuals.

- [Domain docs](agents/domain.md)
- [Issue tracker](agents/issue-tracker.md)
- [Triage labels](agents/triage-labels.md)
- [Residential rule tuning](agents/residential-rule-tuning.md)

Recorded decisions stay in repository files under `docs/adr/`. They are not part of this site.
