# 新增 extra 小站路由分组

## Goal

提供一个独立、可配置的 `extra` 小站路由类别，让用户可以把不属于现有大型 AI 产品分组的小型 AI 站点通过 `AI-家宽` 路由；首个站点为用户指定的 `https://anyrouter.top/`。

## Background

- 根扩展脚本通过布尔常量、域名清单、活动域名聚合和托管规则清理共同管理路由。
- `[routing]` 本地开关是跨文件契约：公开模板、TOML 渲染器、示例配置和中英文配置文档必须保持一致。
- 用户提供的产品入口是 `https://anyrouter.top/`；本任务不推断或加入其他站点。

## Requirements

1. 在 `clash-verge-ai-residential.js` 中新增独立的 `ROUTE_EXTRA` 开关，默认值为 `true`。
2. 新增集中管理的小站域名清单，当前仅包含 `anyrouter.top`。
3. `ROUTE_EXTRA = true` 时，以 `DOMAIN-SUFFIX` 覆盖 `anyrouter.top` 及其子域，并为该后缀生成住宅 DNS 策略；关闭时不注入对应规则或 DNS 策略。
4. `anyrouter.top` 必须始终留在托管清理全集中，使开关从开变关或重复运行时可以移除旧规则和 DNS 策略。
5. 在 `clash-verge-ai-residential.local.toml.example` 的 `[routing]` 中公开 `extra = true`，说明它用于独立控制小站流量。
6. 在 `scripts/sync-local-config.js` 注册 `routing.extra` 到 `ROUTE_EXTRA` 的布尔映射，使旧本地 TOML 能按示例默认值自动补全。
7. 同步更新 `docs/configuration.md`、`docs/local-configuration.md` 及对应 `docs/en/` 页面中的开关表。
8. 增加正向、负向、DNS 和本地渲染回归覆盖；负向域名至少证明相邻但无关的 `anyrouter.com` 不会命中。
9. 保持公开家宽模板凭据占位符不变，不编辑或生成任何 `*.local.toml` / `*.local.js`。
10. 将 `extra` 域名归属同步到 ResiWatch 调优输入生成器，并更新公开路由范围说明及相关开关计数，避免审计工具把可归属域名误标为 unsupported。
11. 清理 `clash-verge-ai-residential.js` 文件头中重复维护的历史版本变更记录，保留用途、数据路径、运行入口和公开模板安全提示。
12. 将 `CHANGELOG.md` 全文改为简体中文，保持版本、日期、技术标识、链接和事实不变；同步项目语言规范的权威来源。

## Acceptance Criteria

- [x] 默认公开脚本包含 `DOMAIN-SUFFIX,anyrouter.top,AI-家宽`，并且 `anyrouter.top` 与其子域会命中该规则。
- [x] 默认 DNS 策略包含 `+.anyrouter.top` 的住宅解析器映射。
- [x] `anyrouter.com` 不会被 `extra` 规则路由到 `AI-家宽`。
- [x] 渲染 `routing.extra = false` 后，生成脚本不包含 `anyrouter.top` 的活动规则或 DNS 策略；公开模板保持不变。
- [x] 示例 TOML、渲染器字段映射和中英文开关表均声明 `routing.extra` / `ROUTE_EXTRA` / `true`。
- [x] 旧版缺少该键的本地 TOML 会自动补全 `routing.extra = true`。
- [x] 关闭开关后再次运行能够清理先前注入的 `anyrouter.top` 托管规则，且脚本保持幂等。
- [x] `npm run ci` 通过；若环境允许，最终运行项目要求的 `just ci`。
- [x] 扩展脚本文件头不再包含 v5.6–v5.11 的重复变更日志，必要运行说明仍保留。
- [x] `CHANGELOG.md` 全文及章节标题均为简体中文，技术标识与版本历史未被删减。
- [x] `AGENTS.md` 与前端质量规范明确 `CHANGELOG.md` 使用中文。

## Out of Scope

- 不新增名为 `extra` 的 Mihomo `proxy-group`；所有流量仍进入现有单例 `AI-家宽` 组。
- 不加入 `anyrouter.top` 之外的其他小站、共享依赖、CDN、遥测或宽泛 provider 后缀。
- 不修改 ResiWatch、真实本地凭据或生成的本地脚本。

## Files in Scope

- `clash-verge-ai-residential.js`
- `clash-verge-ai-residential.local.toml.example`
- `scripts/sync-local-config.js`
- `tests/regression.test.js`
- `tests/sync-local-config.test.js`
- `docs/configuration.md`
- `docs/local-configuration.md`
- `docs/en/configuration.md`
- `docs/en/local-configuration.md`
- `docs/routing-scope.md`
- `docs/en/routing-scope.md`
- `README.md`
- `CHANGELOG.md`
- `docs/index.md`
- `docs/en/index.md`
- `skills/residential-rule-tuning/scripts/build-inputs.js`
- `skills/residential-rule-tuning/SKILL.md`
- `skills/residential-rule-tuning/reference.md`
- `tests/install-agent-skills.test.js`
- `AGENTS.md`
- `.trellis/spec/frontend/quality-guidelines.md`
