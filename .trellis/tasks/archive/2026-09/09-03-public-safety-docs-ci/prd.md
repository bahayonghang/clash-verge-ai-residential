# 公开面安全扫描与 CI

## Goal

公开仓库不能静默带上真实家宽凭据；安全报告范围包含 ResiWatch；CI 与 `just monitor-check` 一样检查版本对齐；skill 的 grok 开关与运行时注入一致。

## Background

证据：`check-template-safety.js` 只钉 `HOME_PROXY_TEMPLATE`；example TOML 目前是 `xxx`，但任意非 token 形态的真实密码不会失败。`SECURITY.md:20-22` 范围不含 `residential-monitor`。`.github/workflows/ci.yml` monitor job 未跑 `sync-monitor-version.js --check`。`skills/.../build-inputs.js:33-38` 把 `GROK_EXACT_DOMAINS`（`auth.x.ai`）算在 `grok_web_assets`；运行时 `grokActiveExactDomains` 在 `grok_core` 下始终包含 `auth.x.ai`，`grok_web_assets` 控制的是 `grok.com` 后缀 vs 精确主机。

## Requirements

- R1 `check-template-safety` 对 `clash-verge-ai-residential.local.toml.example` 的 `[home_proxy]` `server`/`username`/`password` 只允许 `""` 或 `"xxx"`。
- R2 `SECURITY.md` Scope 写明 ResiWatch / `residential-monitor`、安装脚本、本机数据目录。
- R3 CI monitor job 增加 `node scripts/sync-monitor-version.js --check`。
- R4 skill `SUPPORTED_SWITCH_BUILDERS` 与 `grokActiveSuffixDomains` / `grokActiveExactDomains` 一致；回归测试比较开关开/关时的注入主机。
- R5 扫描忽略 `ref/`，避免第三方夹具误报。

## Acceptance Criteria

- [ ] AC1 example TOML 把 `password` 改成非占位后 `npm run check:secrets` 退出非 0。
- [ ] AC2 当前仓库 `check:secrets` 仍退出 0。
- [ ] AC3 CI yaml 含版本对齐检查。
- [ ] AC4 skill JSON 在 `grok_web_assets=false` 时不把 `auth.x.ai` 算作该开关独有流量。
- [ ] AC5 `SECURITY.md` 出现 `ResiWatch` 或 `residential-monitor`。

## Out of scope

- 改运行时 grok 默认开关。
- 网络依赖审计。
