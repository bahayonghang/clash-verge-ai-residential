# 修复 AnyRouter DNS 出口，并把 extra 收成可分站点的分类

## Goal

让 AnyRouter 在用户显式打开 extra 分类后仍走 `AI-家宽`，但不再使用住宅 DoH；公开默认关闭 extra。extra 是小站分类：分类总开关与站点开关独立，可以只开其中一部分站点。

## Background

- 官方入口是根域：控制台 `https://anyrouter.top/console`，Claude Code `ANTHROPIC_BASE_URL=https://anyrouter.top`。不使用 `www`。
- `DOMAIN-SUFFIX,anyrouter.top` 已能匹配根域与子域。当前 `ROUTE_EXTRA` 默认 `true`，并把同一后缀写入 `+.anyrouter.top` 住宅 DNS。
- 钉住 `47.246.23.200` 时，根域 TLS 握手失败。本机 `hosts` 曾写入该地址；Mihomo 会读系统 hosts。2026-09-16 已删除该行。去掉 hosts 后，fake-ip + 域名规则即可打开 `/console` 与登录页。
- 机场/本机 HTTPS DoH 给出 `43.109.133.202/203/204/205`，钉住根域 SNI 后 `/console` 与 `/` 完成 TLS 1.3、`CN=anyrouter.top`、HTTP 200。国内 DIRECT DoH 给出 `104.17.x`，握手失败。
- SOCKS5 通常转发主机名（`docs/dns-and-leak-model.md`）。住宅 DoH 仍是 Mihomo 自解析（hosts、filter、L3）时的 GeoDNS 风险，因此启用站点时也不写住宅 DNS。
- `www.anyrouter.top` 在健康 ESA 上仍返回 530，属第三方源站限制。
- 现有路由开关是扁平布尔键（`SWITCH_CONFIG_FIELDS`），没有嵌套表。Grok 用 `grok_core` + `grok_web_assets` 表达分类与子开关。

## Requirements

- R1：保留 AnyRouter 的 `DOMAIN-SUFFIX` 业务路由能力；根域与 `www` 在站点开启时仍匹配 `AI-家宽`。
- R2：AnyRouter 即使开启也不写入 `+.anyrouter.top` 住宅 DNS；查询回落到现有非 AI 默认 `nameserver`（机场上游 DoH）。不得硬编码 IP，不得改走 DIRECT 国内 DoH。
- R3：除 extra 站点外，现有 AI 域名的住宅 DNS 策略保持不变。
- R4：`routing.extra` 是 extra 分类总开关，公开默认 `false`。为假时，所有 extra 站点都不注入活动规则或 DNS 策略。
- R5：每个 extra 站点有独立扁平开关。当前唯一站点是 `routing.extra_anyrouter`，默认 `true`，但只在 `routing.extra = true` 时生效。可以 `extra=true` 且某个站点为 `false`。
- R6：关闭分类或关闭站点后，必须清掉该站点曾注入的规则与脚本管理的 DNS 键；连续两次 `main()` 输出一致。`anyrouter.top` 始终留在托管清理全集。
- R7：回归测试证明「业务路由」和「DNS 查询出口」可按站点解耦；并用补丁站点证明分类下可只开一部分。不得用单元测试声称第三方站点可用。
- R8：更新 Trellis 规范：新增路由域名要验证真实 TLS，以及 CDN/GeoDNS 与住宅 DNS 的耦合风险；extra 用站点登记表，不用再把规则和住宅 DNS 绑死。
- R9：本地 TOML、示例、渲染器、中英文开关表、路由范围、CHANGELOG、调优生成器与开关计数与上述合同一致。不编辑 `*.local.toml` / `*.local.js`。
- R10：不新增第二个生产 extra 站点。

## Acceptance Criteria

- [x] AC1：公开默认 `ROUTE_EXTRA = false`。默认输出不再包含 `DOMAIN-SUFFIX,anyrouter.top` 或 `+.anyrouter.top`，并与 v5.11 基准投影一致（不再附加 extra）。
- [x] AC2：`ROUTE_EXTRA = true` 且 `ROUTE_EXTRA_ANYROUTER = true` 时，输出含 `DOMAIN-SUFFIX,anyrouter.top,AI-家宽`；测试证明根域和 `www` 命中该规则；`nameserver-policy` 不含 `+.anyrouter.top`；Claude/OpenAI 等代表性 AI 域名仍映射 `RESIDENTIAL_DOH`。
- [x] AC3：仅关分类、仅关 `extra_anyrouter`、或两者都关时，不残留 AnyRouter 活动规则或 DNS policy；连续两次 `main()` 输出一致。旧输入里的 `DOMAIN-SUFFIX,anyrouter.top` 与 `+.anyrouter.top` 会被清掉。
- [x] AC4：补丁一个第二 extra 站点后，可出现「分类开 + A 开 + B 关」：A 有规则、B 无规则；两者都没有住宅 DNS（除非登记表显式要求）。
- [x] AC5：`SWITCH_CONFIG_FIELDS` 含 `extra` 与 `extra_anyrouter`；示例 TOML 为 `extra = false`、`extra_anyrouter = true`；缺失 `extra` 补 `false`，缺失 `extra_anyrouter` 补 `true`；已有本地 `extra = true` 不被改写。
- [x] AC6：调优生成器把 `anyrouter.top` 归到 `extra_anyrouter`；分类键 `extra` 无独立 host，进入 unsupported。开关完整性检查仍成立。
- [x] AC7：文档只推荐根域 `https://anyrouter.top` 与控制台 `https://anyrouter.top/console`；`www` 的 ESA 530 不作为修复成功条件。
- [ ] AC8：脱敏真实 Profile 在 `extra=true` 且无 hosts 钉 IP 时，访问 `https://anyrouter.top/console` 不再 TLS 握手失败；Connections 确认业务仍经 `AI-家宽`。
- [x] AC9：`just ci`、`just docs-build` 与 `git diff --check` 通过。

## Out of Scope

- 修复 AnyRouter ESA 源站、证书或 `www` 配置。
- 固定 ESA IP、关闭 TLS 校验、加入 `anyrouter.top` 以外的生产小站。
- 嵌套 `[routing.extra]` 表，或新增名为 `extra` 的 Mihomo `proxy-group`。
- 修改家宽 SOCKS5 凭据、控制器、ResiWatch，或手改 `*.local.toml` / `*.local.js`。

## Affected Files

- `clash-verge-ai-residential.js`
- `clash-verge-ai-residential.local.toml.example`
- `scripts/sync-local-config.js`
- `tests/regression.test.js`
- `tests/sync-local-config.test.js`
- `tests/install-agent-skills.test.js`
- `skills/residential-rule-tuning/scripts/build-inputs.js`
- `skills/residential-rule-tuning/SKILL.md`
- `skills/residential-rule-tuning/reference.md`
- `docs/configuration.md`、`docs/local-configuration.md`、`docs/routing-scope.md` 及 `docs/en/` 对应页
- `README.md`、`CHANGELOG.md`
- `.trellis/spec/frontend/quality-guidelines.md`
- `.trellis/spec/guides/index.md`
