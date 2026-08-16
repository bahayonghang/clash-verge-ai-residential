# Implement：v5.7.0 执行清单

前置：`npm run ci` 基线全绿（已验证，48 tests）。

## 代码

- [x] 1. `clash-verge-ai-residential.js`
  - [x] 1.1 CORE_EXACT_DOMAINS += `mcp-proxy.anthropic.com`、`assets-proxy.anthropic.com`（带注释与出处）
  - [x] 1.2 删除 OPENAI_CORE_EXACT_DOMAINS；`api.openai.com` 并入 OPENAI_CORE_SUFFIX_DOMAINS（注释说明 us./eu. 前缀依据）
  - [x] 1.3 新增 GROK_EXACT_DOMAINS = ["auth.x.ai", "api.x.ai"]；接入 activeExactDomains（ROUTE_GROK_CORE 门控）与 allPossibleExactDomains
  - [x] 1.4 allPossibleExactDomains 显式保留 `api.openai.com`（v5.6 遗留清理）
  - [x] 1.5 constants 导出同步（删 OPENAI_CORE_EXACT_DOMAINS、增 GROK_EXACT_DOMAINS）
  - [x] 1.6 removeInjectedReferencesFromGroup 删除前 warn（组名/被删引用/原因/指引）
  - [x] 1.7 hardenTun 与 config.ipv6 处加宿主覆盖注释；main() info 追加 Verge 权威字段提示行
  - [x] 1.8 SCRIPT_VERSION = "5.7.0"；文件头 v5.7 摘要
- [x] 2. `tests/regression.test.js`
  - [x] 2.1 版本断言 5.7.0；constants 解构同步
  - [x] 2.2 Claude/OpenAI/Grok 正向与负向断言更新（含 us./eu. 前缀、auth/api.x.ai、x.ai 负向）
  - [x] 2.3 托管清理测试：旧 exact api.openai.com 规则仍被清理；幂等测试 suffix 只注入一次
  - [x] 2.4 新增递归清理 warn 测试（console.warn spy）
- [x] 3. `package.json` version 5.7.0

## 文档

- [x] 4. docs/routing-scope.md：域名行 + 证据句 + downloads.claude.ai 取舍
- [x] 5. docs/dns-and-leak-model.md：Host-enforced fields 小节、fake-ip 生效时机精确化、geosite 依赖
- [x] 6. docs/troubleshooting.md：geosite 失败、私网 DIRECT 取舍、组引用清除 warn 三条
- [x] 7. docs/configuration.md：Verge 设置页为准的说明（若有相关段落）
- [x] 8. README.md：版本号与域名摘要行
- [x] 9. CHANGELOG.md：5.7.0 条目

## 验证

- [x] 10. `npm run ci` 全绿
- [x] 11. `node -e "require('./clash-verge-ai-residential.js')"` 冒烟（模块加载）
- [x] 12. 重新渲染本地脚本验证渲染管线：`node scripts/sync-local-config.js`
      （注意：不提交生成的 local.js / local.toml）

## 回滚点

所有改动一个提交；revert 即回滚。

## 审查门

- 域名变更均有 research/ai-endpoint-domains.md 中的官方 URL 背书
- 公开模板占位符检查仍通过（check:secrets）
- 本地凭据文件零改动、零提交
