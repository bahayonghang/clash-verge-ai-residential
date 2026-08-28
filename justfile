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
ci: monitor-check
    npm run ci

# 以 residential-monitor/package.json 为准，写入 Tauri 安装包、Cargo 与 lockfile 版本。
monitor-sync-version:
    node scripts/sync-monitor-version.js

# 子项目快速质量门：版本对齐、安装锁文件、前端检查、Rust 检查。不含 30 天库与 30 分钟峰值。
monitor-check:
    node scripts/sync-monitor-version.js --check
    npm --prefix residential-monitor ci
    npm --prefix residential-monitor run check
    cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check
    cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
    cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace
    npm run check:secrets

# 从 icon-source.png 生成 Windows ICO 与 PNG。不覆盖 tray-*.png。
monitor-icons:
    npm --prefix residential-monitor run icons

# 开发态桌面壳。
monitor-dev: monitor-sync-version
    npm --prefix residential-monitor run tauri:dev

# 调试家宽监控 Tauri 应用：Vite 热更新 + Rust 开发态。
tdev: monitor-dev

# 生成 NSIS 安装包。不在本机执行安装。构建前按 package.json 同步版本。
monitor-build: monitor-sync-version
    npm --prefix residential-monitor run tauri:build

# C5 自动硬化门：故障矩阵、并发 fixture、soak smoke、供应链。不含 30 天库、24 小时 soak、本机安装。
monitor-c5-auto:
    cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c5::
    cargo run --quiet --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- c5-fault
    cargo run --quiet --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- c5-supply

# 构建 NSIS 并静默安装到 %LOCALAPPDATA%\ResiWatch。不启动应用。会改本机安装态。
# 构建前按 residential-monitor/package.json 同步 Tauri/Cargo 版本。
# 安装前结束正在运行的 residential-monitor，避免 NSIS 覆盖被占用的文件。
# 以 /D= 指定持久目录，忽略注册表里指向 Temp 的上次安装位置。
[windows]
tinstall: monitor-build
    @node scripts/nsis-silent-install.js

# 家宽监控 v1 只提供 Windows 11 NSIS current-user 安装。
[unix]
tinstall:
    @echo "家宽监控 v1 只提供 Windows 11 NSIS current-user 安装。"
    @exit 1

# 单向渲染：本地 TOML + 公开模板 -> 被忽略的本地 Clash Verge 脚本。
# 首次执行会创建本地配置，避免带着示例占位值生成脚本。
# 本地 TOML 缺失的开关键会按示例默认值自动补全（含缺失的整个配置表）。
render-local:
    @node -e "const fs = require('node:fs'); const configPath = 'clash-verge-ai-residential.local.toml'; if (!fs.existsSync(configPath)) { fs.copyFileSync('clash-verge-ai-residential.local.toml.example', configPath); console.error('✗ 已生成 clash-verge-ai-residential.local.toml。请填写住宅 SOCKS5 信息后重新执行 just render-local。'); process.exit(1); }"
    @node scripts/sync-local-config.js

# 向后兼容旧命令；请使用 render-local，它更准确地表示单向生成行为。
sync: render-local
