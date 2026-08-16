# 修复 Anthropic 子域家宽分流

## Goal

确保已确认属于 Claude / Anthropic 核心 API 的请求及其 DNS 查询稳定命中
`AI-家宽`，不再被 Profile 中更宽泛的 `DOMAIN-SUFFIX,anthropic.com,...`
规则接管，同时保持项目既有的 AI-only 路由边界。

## Background

- 用户提供的 Clash Verge Rev Connections 记录显示：
  `api.anthropic.com` 命中 `AI-家宽`，而 `a-api.anthropic.com` 命中原
  Profile 的 `DomainSuffix(anthropic.com)` 并走普通 GPT 代理。
- `clash-verge-ai-residential.js:175-178` 当前只在
  `CORE_EXACT_DOMAINS` 中声明 `api.anthropic.com`；没有
  `a-api.anthropic.com`，也刻意没有使用宽泛的 `anthropic.com` 后缀。
- `clash-verge-ai-residential.js:1089-1094` 从同一活动域名集合生成连接规则，
  `clash-verge-ai-residential.js:1257-1263` 从该集合生成住宅 DNS policy。
  因而当前遗漏会同时影响流量出口和 DNS 出口。
- `tests/regression.test.js:458-466` 仅覆盖 `api.anthropic.com`，未覆盖
  `a-api.anthropic.com`，所以现有测试没有发现该缺口。
- `docs/routing-scope.md:27-34` 要求新增域名采用最窄的可行匹配，并用负向
  测试证明文档、支持、遥测等非核心流量仍不进入家宽链路。

## Requirements

- R1. 将用户已观测到的 Anthropic 核心 API 主机纳入默认
  `AI-家宽` 连接规则。
- R2. 同一主机的 `nameserver-policy` 必须使用 `RESIDENTIAL_DOH`，保持
  DNS 与应用流量出口一致。
- R3. 保持单一域名清单驱动规则、DNS policy 和当前版本托管清理集合，
  不增加平行配置源或手写重复规则。
- R4. 保持 AI-only 边界，只新增 `a-api.anthropic.com` 精确匹配，不将
  整个 `anthropic.com` 后缀默认送入家宽。
- R5. 增加连接规则、DNS policy、负向边界、配置重建及幂等回归覆盖。
- R6. 更新与 Anthropic 默认路由范围直接相关的用户文档；不改动本地
  SOCKS5 凭据、TOML 开关或无关产品规则。

## Acceptance Criteria

- [x] `api.anthropic.com` 与 `a-api.anthropic.com` 均在默认配置中命中
  `AI-家宽`，且生成规则位于原 Profile 的宽泛规则之前。
- [x] 两个核心 API 主机均由 `nameserver-policy` 指向
  `RESIDENTIAL_DOH`。
- [x] Anthropic 文档、支持、状态、遥测等未获准的相邻主机不被新增规则
  捕获。
- [x] 重复执行扩展脚本不会产生重复规则或漂移 DNS policy；升级时当前
  版本托管规则仍可被精确替换。
- [x] `npm run ci` 与 `just ci` 通过。
- [ ] 使用脱敏后的真实 Profile 在 Clash Verge Rev Connections 中验证
  `a-api.anthropic.com` 命中 `AI-家宽`；若本次无法执行，必须明确记录为
  未验证，不得用 Node 测试替代。当前状态：`UNVERIFIED`。

## Key Decision

- 用户选择最窄修复：保留 `api.anthropic.com` 并新增
  `a-api.anthropic.com` 精确规则。未来若观察到其他核心端点，应继续按
  `docs/routing-scope.md` 的证据和负向测试要求逐个评估，而不是提前扩大
  到 `DOMAIN-SUFFIX,anthropic.com`。

## Out of Scope

- 将所有 `anthropic.com` 网站、文档、状态页、客服、分析或共享第三方
  依赖默认送入家宽。
- 开启进程级 Claude/Claude Code 全量代理。
- 修改用户订阅、Merge 配置中由用户维护的 `anthropic.com` 规则。
- 修改真实住宅代理地址、凭据或本地生成文件。
