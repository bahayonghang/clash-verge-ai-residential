# Windows 不依赖 Git Bash；Unix 平台保留 POSIX shell。
[windows]
set shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

[unix]
set shell := ["sh", "-cu"]

# 公开模板、回归测试与模板安全检查。
ci:
    npm run ci

# 单向渲染：本地 TOML + 公开模板 -> 被忽略的本地 Clash Verge 脚本。
render-local:
    node scripts/sync-local-config.js

# 向后兼容旧命令；请使用 render-local，它更准确地表示单向生成行为。
sync: render-local
