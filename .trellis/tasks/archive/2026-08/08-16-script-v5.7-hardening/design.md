# Design：v5.7.0 健壮性优化

## 边界

只动 `clash-verge-ai-residential.js` 的域名常量、warn 可观测性与版本号；
测试与文档同步。不改变 DNS 构建算法、规则注入顺序、上游解析逻辑、
sync-local-config 的开关映射（无新开关）。

## 变更 1：域名常量（R1）

```js
// CORE_EXACT_DOMAINS 追加（Claude，官方 network-config 文档）：
"mcp-proxy.anthropic.com",
"assets-proxy.anthropic.com"

// OPENAI_CORE_EXACT_DOMAINS 改为空（迁移历史）：
//   api.openai.com 移入 OPENAI_CORE_SUFFIX_DOMAINS（覆盖 us./eu. 数据驻留前缀）。
//   OPENAI_CORE_EXACT_DOMAINS 保留常量但为空数组？——不，直接删除常量会破坏
//   constants 导出与测试引用。决定：保留常量，值为 []，标注历史用途？
//   更简洁：删除常量并同步删除 constants 导出与测试引用（测试同仓修改，无兼容负担）。
//   采用：删除 OPENAI_CORE_EXACT_DOMAINS，新增无 exact 需求。

// GROK_SUFFIX_DOMAINS 保持 ["grok.com"]；新增：
const GROK_EXACT_DOMAINS = ["auth.x.ai", "api.x.ai"];
```

集合函数接线：
- `activeSuffixDomains()`：无结构变化（OPENAI_CORE_SUFFIX_DOMAINS 已并入）。
- `activeExactDomains()`：新增 `...(ROUTE_GROK_CORE ? GROK_EXACT_DOMAINS : [])`。
- `allPossibleSuffixDomains()`：`OPENAI_CORE_SUFFIX_DOMAINS` 已在列 —— 但需确认
  迁移后 `api.openai.com` 出现在 suffix 池（自动满足，因为它就在
  OPENAI_CORE_SUFFIX_DOMAINS 里）。旧的 exact 形态 `DOMAIN,api.openai.com,AI-家宽`
  的清理依赖 `allPossibleExactDomains()` 含 `api.openline...`——迁移后 exact 池
  不再含它，旧规则清不掉。**解决**：`allPossibleExactDomains()` 显式追加
  `"api.openai.com"`（注释说明为 v5.6 遗留清理保留），确保幂等迁移。
- `allPossibleExactDomains()` 同步追加 `GROK_EXACT_DOMAINS`。
- `buildManagedDnsPolicyKeySet()` 经 allPossible* 自动覆盖
  `+.api.openai.com`（新）与 `api.openai.com`（遗留）两个键。

DNS policy：`buildNameserverPolicy` 对 suffix 域写 `+.api.openai.com`；
mihomo `+.` 匹配裸域 + 子域（已核实），`api.openai.com` 与 `us.api.openai.com`
均命中，无需额外裸域键。

## 变更 2：递归清理 warn（R2）

`removeInjectedReferencesFromGroup(group)` 在删除前收集被删引用，
非空时 `warn`：组名、被删名称列表、原因（防止 dialer-proxy 链递归）、
指引（AI 流量请用规则指向 AI-家宽，不要把 AI-家宽 放进上游组）。
该函数被 `hardenReachableUpstreamGraph` 对每个可达组调用，warn 每组最多一次。

## 变更 3：宿主权威字段提示（R3）

`main()` 末尾 info 日志追加一行：Clash Verge Rev 新版会在脚本后还原
`tun`/`ipv6` 权威字段；TUN dns-hijack 与 IPv6 开关需在 Verge 设置页配置。
代码中 `hardenTun` / `config.ipv6 = false` 保留（旧版宿主仍生效），
并在 `hardenTun` 定义处加注释说明宿主覆盖行为。

## 变更 4：版本与元数据（R5）

- `SCRIPT_VERSION = "5.7.0"`；package.json `"version": "5.7.0"`。
- 文件头注释追加 v5.7 摘要（域名补齐、递归清理 warn、宿主字段提示）。

## 测试设计

regression.test.js：
- 版本断言更新 5.7.0。
- "Claude、ChatGPT..." 正向断言追加 `mcp-proxy.anthropic.com`、
  `assets-proxy.anthropic.com`、`us.api.openai.com`、`eu.api.openai.com`、
  `api.openai.com`（裸域仍命中 suffix）。
- Grok 测试：正向追加 `auth.x.ai`、`api.x.ai`；负向保持 `x.ai`、`api.mixpanel.com`。
- 托管清理测试：输入含旧 `DOMAIN,api.openai.com,AI-家宽`，断言被清理；
  幂等测试断言 `DOMAIN-SUFFIX,api.openai.com,AI-家宽` 只出现一次。
- 新增：递归清理 warn 测试 —— console.warn spy 断言含组名与被删引用。
- 负向断言 `x.ai` / `www.x.ai`? 注意 `api.x.ai` 是 exact，`www.x.ai` 不命中 ✓；
  `auth.x.ai` exact 不影响 `console.x.ai`。

## 文档变更

- routing-scope.md：Grok/Claude/OpenAI 行更新 + 证据句；"Explicit exclusions"
  保持；新增"downloads.claude.ai 随 claude.ai 后缀走家宽"取舍句。
- dns-and-leak-model.md：新增"Host-enforced fields"小节（tun/ipv6 被 Verge 还原）；
  精确化 fake-ip + socks5 域名直传下 nameserver-policy 的生效时机；
  geosite.dat 硬依赖一句。
- troubleshooting.md：新增 geosite 初始化失败条目、私网 DIRECT 前置取舍条目、
  用户组引用被清除条目。
- configuration.md：若有 ipv6/tun 相关段落，补"以 Verge 设置页为准"。
- README.md：版本号、Domain 变更示例更新（如列出新域）。

## 回滚

单 commit 内完成脚本+测试+文档；出问题 `git revert` 单提交即可，
无数据迁移、无状态。

## 风险

- `api.openai.com` suffix 化的唯一放大面：openai 未来的非 API 子域（如
  status.api.openai.com 不存在；官方子域即数据驻留端点）。可接受。
- warn 噪音：仅当用户组真的引用了保留名才会触发，正常配置零输出。
