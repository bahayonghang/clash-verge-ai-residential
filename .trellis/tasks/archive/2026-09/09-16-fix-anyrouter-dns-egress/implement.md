# Implement: extra 分类与 AnyRouter DNS 解耦

## Ordered checklist

1. 在 `clash-verge-ai-residential.js` 加入 `EXTRA_SITES` 与 `ROUTE_EXTRA_ANYROUTER = true`；把 `ROUTE_EXTRA` 改为 `false`。
2. 由登记表派生活动 extra 域名、清理全集，以及 `residentialDns === false` 的 DNS 豁免。`activeSuffixDomains()` 改用活动 extra 后缀；`buildNameserverPolicy` 跳过豁免后缀。
3. 导出登记表、新常量，以及测试需要的派生列表。
4. `scripts/sync-local-config.js` 增加 `routing.extra_anyrouter`。示例 TOML：`extra = false`、`extra_anyrouter = true`，并写清分类/站点关系。
5. 更新 `tests/regression.test.js`：
   - 默认投影与 v5.11 一致，不再插入 AnyRouter 规则或住宅 DNS。
   - 补丁 `ROUTE_EXTRA` + `ROUTE_EXTRA_ANYROUTER` 证明路由在、DNS 不在。
   - 分类关 / 站点关 / 两者关的清理与幂等。
   - 补丁第二 extra 站点证明只开一部分。
   - 代表性 AI 域名住宅 DNS 不变。
6. 更新 `tests/sync-local-config.test.js`：`extra` 从「缺失补 true」改为补 `false`；新增 `extra_anyrouter` 缺失补 `true`；开启 extra 时断言有规则、无 `+.anyrouter.top`。
7. 更新 `skills/residential-rule-tuning/scripts/build-inputs.js`、skill/reference 文案与开关计数；同步 `tests/install-agent-skills.test.js`。
8. 更新中英文配置表、路由范围、README、CHANGELOG。文档推荐 `https://anyrouter.top` 与 `/console`。
9. 更新 `.trellis/spec/frontend/quality-guidelines.md` 与 `.trellis/spec/guides/index.md`：路由与 DNS 可解耦；新增域名要看真实 TLS 与 GeoDNS。

## Validation

- `just ci`
- `just docs-build`
- `git diff --check`
- 本机不改 `*.local.toml`。若用户已有 `extra = true`，自行 `just render-local` 后加载脱敏 Profile，访问 `https://anyrouter.top/console`，用 Connections 看是否仍经 `AI-家宽`。验收前确认 hosts 无 `anyrouter.top`。

## Risky files / rollback

- `clash-verge-ai-residential.js` 的 DNS 与活动域名聚合：改错会让其它 AI 域名丢掉住宅 DNS。
- `tests/regression.test.js` 的 v5.11 投影测试：默认 extra 关闭后应回到基准，不要再 splice AnyRouter。
- `SWITCH_CONFIG_FIELDS` 与四份开关表、调优完整性检查必须一起改。
- 回退：还原公开模板常量与字段映射；不要手改用户本地 TOML。

## Before `task.py start`

- `prd.md` / `design.md` / `implement.md` 已与用户确认。
- `implement.jsonl` 与 `check.jsonl` 已有真实 spec/research 行。
- 不在本清单里加入第二个生产站点或 hosts 钉 IP。
