# CLAUDE.md

Claude Code entry for this repository. Shared product map, toolchains,
verification gates, safety boundaries, language split, and spec navigation
live in `AGENTS.md`. Do not treat this file as the five-tool contract.

@AGENTS.md

## Claude Code loader

Claude Code reads this file each session and expands `@AGENTS.md`. `AGENTS.md`
must not import this file.

Commands, gates, and must-not-break rules: `AGENTS.md`. `just ci` is not
`npm run ci` and does not build docs.

Local extension render: `just render-local`. First run copies
`*.local.toml.example` → `*.local.toml` and exits 1 until that file is filled
in. Do not hand-edit `*.local.js`.

## Agent skills

### Issue tracker

Issues live in GitHub Issues for bahayonghang/clash-verge-ai-residential. See
`docs/agents/issue-tracker.md`.

### Triage labels

The five canonical roles use matching label strings: `needs-triage`,
`needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See
`docs/agents/triage-labels.md`.

### Domain docs

single-context. See `docs/agents/domain.md`.

### 家宽规则优化

源文件在 `skills/residential-rule-tuning/`。用 `just install-skills` 安装到本仓库已存在的平台 skill 目录。见 `docs/agents/residential-rule-tuning.md`。
