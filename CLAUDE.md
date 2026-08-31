# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`clash-verge-ai-residential.js` (repo root, ~1.7k lines, vanilla CommonJS) is a Clash Verge Rev
**global extension script** — not a built app. Users paste it into Clash Verge to route only core AI
traffic (Claude, ChatGPT, Gemini, Cursor, …) through a residential SOCKS5 chain while all other traffic
stays on their airport proxy. Node ≥18, zero dependencies, no bundler/transpiler or third-party test framework.

## Commands

- `just ci` (= `npm run ci`) — full gate: `node --check` syntax lint + regression tests + secret scan.
  Run before every commit.
- `npm test` — runs the explicitly listed suites with Node's built-in `node:test` runner; no third-party framework.
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

## Agent skills

### Issue tracker

Issues live in GitHub Issues for bahayonghang/clash-verge-ai-residential. See `docs/agents/issue-tracker.md`.

### Triage labels

The five canonical roles use matching label strings: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

single-context. See `docs/agents/domain.md`.

### 家宽规则优化

源文件在 `skills/residential-rule-tuning/`。用 `just install-skills` 安装到本仓库已存在的平台 skill 目录。见 `docs/agents/residential-rule-tuning.md`。