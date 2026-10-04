<!-- TRELLIS:START -->
# Trellis Instructions

These instructions are for AI assistants working in this project.

This project is managed by Trellis. The working knowledge you need lives under `.trellis/`:

- `.trellis/workflow.md` — development phases, when to create tasks, skill routing
- `.trellis/spec/` — package- and layer-scoped coding guidelines (read before writing code in a given layer)
- `.trellis/workspace/` — per-developer journals and session traces
- `.trellis/tasks/` — active and archived tasks (PRDs, research, jsonl context)

If a Trellis command is available on your platform (e.g. `/trellis:finish-work`, `/trellis:continue`), prefer it over manual steps. Not every platform exposes every command.

If you're using Codex or another agent-capable tool, additional project-scoped helpers may live in:
- `.agents/skills/` — reusable Trellis skills
- `.codex/agents/` — optional custom subagents

Managed by Trellis. Edits outside this block are preserved; edits inside may be overwritten by a future `trellis update`.

<!-- TRELLIS:END -->

# Project contract

This `AGENTS.md` is the shared project contract for Claude Code, Codex, Grok
Build, Kimi Code, and OMP. Shared facts live in this file. Do not depend on
`@` imports to expand them. Claude-specific loader notes belong in `CLAUDE.md`
only. `AGENTS.md` must not import `CLAUDE.md`.

## Authorization and review

These rules apply to Claude Code, Codex, Grok Build, Kimi Code, and OMP.
Read-only review may run checks and write explicitly authorized research
artifacts. It must not repair product code or configuration. Self-fix in any
`trellis-check` role or skill applies only after implementation is approved,
within the approved task and file scope. A task directory, an `in_progress`
status, or an available write tool does not grant authorization. If a finding
needs additional files, behavior, permissions, or a lower validation gate,
return the finding to the planning/review model before expanding the work.

Harness capabilities and role prompts do not expand authorization. Do not
auto-trust projects or hooks, bypass permissions, or treat prompt recursion
guards as a hard sandbox. Dispatch only through the applicable workflow;
already dispatched implement/check agents must not spawn those roles again.

## What this repository contains

This repository is three surfaces, not a single no-dependency script:

- Clash Verge Rev **global extension script** `clash-verge-ai-residential.js`
  (repo root, vanilla CommonJS, Node.js ≥18, zero third-party dependencies).
  Users paste that file into Clash Verge to route core AI traffic through a
  residential SOCKS5 chain. Other traffic stays on the airport proxy.
- **ResiWatch** in `residential-monitor/` (React frontend, Tauri/Rust backend,
  SQLite ledger). Windows 11 NSIS current-user desktop app. It does not read
  `*.local.toml` and does not change the pasteable extension.
- **VitePress docs** in `docs/` (Node.js 22+). Default locale is Chinese at
  `docs/*.md`. English sources are `docs/en/`.

Domain terms: `CONTEXT.md`.

## Toolchains by path

Root / extension / `scripts/` / `tests/` — Node.js 18+, zero third-party
dependencies, `node:test` (no third-party test framework):

- `npm run check`
- `npm test`
- `npm run ci`
- `npm run check:secrets`

`residential-monitor/` — React frontend plus Tauri/Rust:

- `just monitor-check`
- `just ci` (`monitor-check`, then root `npm run ci`)

`docs/` — Node.js 22+, dependencies in `docs/package.json`:

- `just docs-build`
- `npm --prefix docs run build`

Docs is not part of `just ci`.

## Verification

Local full product gate: `just ci` (monitor + root). `just ci` is not equal to
`npm run ci`. `just ci` does not build docs.

Local docs: `just docs-build` (independent of `just ci`).

Dependency security: `just dependency-audit` is independent of `just ci` and
`just docs-build`. Run it for dependency/lockfile and audit-gate changes. It
runs full monitor/docs npm audits, including dev dependencies, with
`--include=dev --audit-level=high`, then
`cargo audit --file residential-monitor/src-tauri/Cargo.lock`. The explicit
`--include=dev` keeps development dependencies in the audit when environment
or npm configuration sets `omit=dev`.
Prerequisites: npm, Cargo, an already installed `cargo-audit`, and network
access to the configured npm registry and RustSec advisory database. Do not
install tools globally without authorization. Registry/database/network or
missing-tool failures are blocked checks, not vulnerability findings or PASS.
Keep RustSec unmaintained/unsound warnings separate from vulnerabilities and
record the affected target. A successful `npm ci` does not satisfy the audit
gate. These rules apply to Claude Code, Codex, Grok Build, Kimi Code, and OMP.

Hosted GitHub CI (`.github/workflows/ci.yml`):

- Ubuntu Node.js 18/20/22 and Windows Node.js 22 run `npm run ci`.
- Windows `monitor` job: version alignment, then seven separate pwsh native
  steps (frontend install, full npm audit, frontend check, Rust fmt, clippy,
  workspace tests, secret scan).
- Ubuntu Node.js 22 `docs` job: `npm --prefix docs ci`, independent
  `npm --prefix docs audit --include=dev --audit-level=high`, then
  `npm --prefix docs run build`.
- Aggregate job named `Required checks` needs `[test, monitor, docs]`.
- RustSec audit remains an independent local check; hosted CI does not run it.

Root-only `npm run ci` is syntax check plus listed Node tests plus the secret
scan. Use `just ci` when monitor must pass as well.

For host integration, DNS, or routing changes, also test a sanitized real Clash
profile when practical. Node tests do not emulate the Clash JavaScript host or
Mihomo.

## Boundaries

- Never put real credentials in the public template. `HOME_PROXY_TEMPLATE`
  `server` / `username` / `password` stay `"xxx"` / `""`. `npm run check:secrets`
  enforces this.
- Never commit `*.local.toml` or `*.local.js`. By default, agents must not
  hand-edit either file. For personal tuning, produce recommendations for
  the user to apply to `*.local.toml`. An agent may edit that TOML only when
  the user explicitly authorizes the exception and names the change scope;
  a skill or task does not supply that exception. Never hand-edit generated
  `*.local.js`. Generate local output with `just render-local`. First run
  copies `*.local.toml.example` to `*.local.toml` and exits 1 until the user
  fills in the file.
- New routed domains need an official source or sanitized Connections evidence
  plus a negative test. Broad provider suffixes, marketplace/CDN, and telemetry
  are rejected by default (README and PR template).
- Production controller, credential, and routing-algorithm changes need an
  active task that names those files. Do not mix those edits into contract,
  docs-toolchain, or harness work.

## Spec navigation

- `.trellis/spec/frontend/` — pasteable extension and local renderer at repo
  root. That package is not the ResiWatch React app.
- `.trellis/spec/residential-monitor/` — package index. It points at
  `backend/`, `frontend/`, and `storage/` layer indexes.
- `python .trellis/scripts/get_context.py --mode packages` lists first-layer
  specs.

Read the layer that matches the files you change. Do not apply the root
frontend spec to `residential-monitor/src/`.

## Language split

- Code comments, error messages, `CHANGELOG.md`, and `docs/` (except `docs/en/`) are Chinese.
- `docs/en/` is the English docs-site tree.
- `package.json`, CI, and PR/issue templates are English.
- This Trellis spec tree is English.

Match the file you edit. Root extension style: 2-space indent, double quotes,
`"use strict"`, CommonJS. No root linter/formatter. Keep syntax clean with
`npm run check`. That style does not apply to `residential-monitor/` or `docs/`.

Commits: `<type>: [AI] <gitmoji> <Chinese subject>` (example:
`feat: [AI] ✨ 添加本地配置渲染`).

Business skill source: `skills/residential-rule-tuning/`. Install with
`just install-skills`. See `docs/agents/`.
