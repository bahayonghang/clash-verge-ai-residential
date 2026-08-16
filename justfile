# Windows 不依赖 Git Bash；Unix 平台保留 POSIX shell。
[windows]
set shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

[unix]
set shell := ["sh", "-cu"]

[private]
default: help

# 列出所有可用的命令及说明
help:
    @just --list


# 公开模板、回归测试与模板安全检查。
ci:
    npm run ci

# 单向渲染：本地 TOML + 公开模板 -> 被忽略的本地 Clash Verge 脚本。
# 首次执行会创建本地配置，避免带着示例占位值生成脚本。
# 本地 TOML 缺失的开关键会按示例默认值自动补全（含缺失的整个配置表）。
render-local:
    @node -e "const fs = require('node:fs'); const configPath = 'clash-verge-ai-residential.local.toml'; if (!fs.existsSync(configPath)) { fs.copyFileSync('clash-verge-ai-residential.local.toml.example', configPath); console.error('已生成 clash-verge-ai-residential.local.toml。请填写住宅 SOCKS5 信息后重新执行 just render-local。'); process.exit(1); }"
    @node scripts/sync-local-config.js

# 向后兼容旧命令；请使用 render-local，它更准确地表示单向生成行为。
sync: render-local
