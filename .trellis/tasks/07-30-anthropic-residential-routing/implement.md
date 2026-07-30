# Implementation Plan

## Ordered Checklist

- [x] 在 `clash-verge-ai-residential.js` 的 `CORE_EXACT_DOMAINS` 中加入
  `a-api.anthropic.com`，保持 `api.anthropic.com` 与
  `api.openai.com` 的现有精确匹配。
- [x] 扩展 `tests/regression.test.js` 的核心域名测试，证明
  `a-api.anthropic.com` 命中 `AI-家宽`，而 Anthropic 的文档、状态及
  网站相邻主机不命中。
- [x] 扩展配置级规则顺序断言：当 Profile 已有
  `DOMAIN-SUFFIX,anthropic.com,GPT` 时，新精确规则存在、位于其前面，
  且原规则仍被保留。
- [x] 扩展 DNS policy 断言，证明 `api.anthropic.com` 和
  `a-api.anthropic.com` 使用 `RESIDENTIAL_DOH`，未批准的相邻主机没有
  专用住宅 policy。
- [x] 将新规则纳入现有托管清理/两次执行的断言，证明无重复和无漂移。
- [x] 在 `CHANGELOG.md` 的 `Unreleased` 下记录修复。
- [x] 检查最终 diff，仅包含任务、脚本、回归测试和 changelog 的必要
  变更，不包含本地 TOML、生成脚本、真实代理信息或截图。

## Validation

按从窄到宽执行：

```powershell
node --test tests/regression.test.js
npm run check
npm test
npm run check:secrets
npm run ci
just ci
git diff --check
```

随后使用脱敏真实 Profile 在 Clash Verge Rev 中刷新扩展配置，确认：

- `a-api.anthropic.com` 的 Chains 为 `AI-家宽 / 家宽-SOCKS5`；
- Rule 显示新注入的精确域名规则，而不是原 Profile 的
  `DomainSuffix(anthropic.com)`；
- 相邻的非核心 Anthropic 主机仍按原 Profile 路由。

若无法完成宿主验证，将其标记为 `UNVERIFIED` 并单独报告。

## Check Result

- `node --test tests/regression.test.js`: 29/29 passed.
- `npm test`: 40/40 passed.
- `npm run check`, `npm run check:secrets`, `npm run ci`, `just ci`, and
  `git diff --check`: passed.
- `.trellis/spec` sync: no update needed; existing domain, DNS, managed-rule,
  negative-test, and idempotence contracts already cover this fix.
- Sanitized real Profile / Clash Verge Rev Connections verification:
  `UNVERIFIED`.

## Rollback Points

- 源码与测试修改均为单一精确域名增量，可整体反向应用。
- 不运行本地 renderer，不修改 `*.local.toml` 或 `*.local.js`。
- 不修改用户订阅、Merge 配置或远程仓库状态。
