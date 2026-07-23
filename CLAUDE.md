# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`clash-verge-ai-residential.js` (repo root, ~1.7k lines, vanilla CommonJS) is a Clash Verge Rev
**global extension script** — not a built app. Users paste it into Clash Verge to route only core AI
traffic (Claude, ChatGPT, Gemini, Cursor, …) through a residential SOCKS5 chain while all other traffic
stays on their airport proxy. Node ≥18, zero dependencies, no bundler/transpiler/test framework.

## Commands

- `just ci` (= `npm run ci`) — full gate: `node --check` syntax lint + regression tests + secret scan.
  Run before every commit.
- `npm test` — runs `tests/*.test.js` directly (plain Node scripts, no framework).
- `just render-local` — regenerate the local profile from `*.local.toml`. First run copies
  `*.local.toml.example` → `*.local.toml` and exits 1 asking you to fill it in.

## Must not break

- **Never put real credentials in the public template.** `server`/`username`/`password` in
  `HOME_PROXY_TEMPLATE` stay `"xxx"`/`""`; `npm run check:secrets` enforces this in CI.
- **Never commit or hand-edit generated/local files.** `*.local.toml` and `*.local.js` are gitignored;
  `*.local.js` is generated — regenerate with `just render-local`, don't edit it.
- **New routed domains need justification** — an official source or sanitized Connections evidence plus a
  negative test. Broad provider suffixes, marketplace/CDN, and telemetry are rejected by default
  (see README + PR template).

## Conventions

- **Language split:** code comments, error messages, and `docs/` are **Chinese**; `CHANGELOG.md`,
  `package.json`, CI, and PR/issue templates are **English**. Match the file you're in.
- **Commits:** `<type>: [AI] <gitmoji> <Chinese subject>` (e.g. `feat: [AI] ✨ 添加本地配置渲染`).
- 2-space indent, double quotes, `"use strict"`, CommonJS. No linter/formatter configured — keep
  syntax clean (`npm run check`).

@AGENTS.md
