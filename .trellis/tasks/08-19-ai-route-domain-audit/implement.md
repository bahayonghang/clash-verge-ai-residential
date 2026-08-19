# 执行计划：AI 家宽路由域名清单收窄

前置条件：`task.py start` 之后才能执行。当前任务状态为 planning，本文件是待审核的计划。

## 顺序

### 步骤 1：修复 `api.x.ai` 匹配缺口（无开关，直接改）

- `clash-verge-ai-residential.js`：把 `api.x.ai` 从 `GROK_EXACT_DOMAINS` 移到
  `GROK_SUFFIX_DOMAINS`。
- 保留 `api.x.ai` 在 `allPossibleExactDomains()` 中（用于清理 v5.9 生成的
  `DOMAIN,api.x.ai,AI-家宽` 与 DNS 键 `api.x.ai`）。
- 依据：docs.x.ai/developers/regions 的 `<region>.api.x.ai`、
  docs.x.ai/developers/advanced-api-usage/mtls 的 `mtls.api.x.ai`。

验证：

```bash
node -e 'const s=require("./clash-verge-ai-residential.js");
const r=s.buildInjectedRules();console.log(r.filter(x=>x.includes("x.ai")))'
```

预期出现 `DOMAIN-SUFFIX,api.x.ai,AI-家宽`，不再出现 `DOMAIN,api.x.ai,AI-家宽`。

### 步骤 2：Cursor 精确化（后缀改精确）

- `CURSOR_SUFFIX_DOMAINS` 移除 `api2.cursor.sh`、`authenticate.cursor.sh`。
- `CURSOR_EXACT_DOMAINS` 加入这两个主机。
- 两个主机同时保留在 `allPossibleSuffixDomains()` 中，用于清理旧的后缀规则。
- 不改动 `api5.cursor.sh`、`gcpp.cursor.sh`、`authentication.cursor.sh`、`cursorvm.com`：
  官方文档确认这四项需要后缀匹配。

### 步骤 3：Antigravity 精确化

- `CORE_SUFFIX_DOMAINS` 移除 `antigravity.google`。
- 新增到 `CORE_EXACT_DOMAINS`。
- `antigravity.google` 保留在 `allPossibleSuffixDomains()` 中。

### 步骤 4：新增 `routing.grok_web_assets` 开关（默认 `true`）

- `clash-verge-ai-residential.js`：新增 `const ROUTE_GROK_WEB_ASSETS = true;`。
- `GROK_SUFFIX_DOMAINS` 拆为两组：开关开启时用 `["grok.com"]`；
  关闭时改用 `GROK_CORE_EXACT_DOMAINS = ["grok.com", "cli-chat-proxy.grok.com", "code.grok.com"]`。
- `allPossibleSuffixDomains()` / `allPossibleExactDomains()` 同时包含两组，保证双向切换都能清理。
- `scripts/sync-local-config.js` 的映射表新增
  `{ table: "routing", key: "grok_web_assets", constant: "ROUTE_GROK_WEB_ASSETS", type: "boolean" }`。
- `clash-verge-ai-residential.local.toml.example` 新增 `grok_web_assets = true`。

### 步骤 5：新增 `routing.vertex_ai_regional` 开关（默认 `true`）

- 新增 `const ROUTE_VERTEX_AI_REGIONAL = true;`。
- `activeDomainRegexes()` 中 `GEMINI_DOMAIN_REGEXES` 改为受该开关控制。
- `allPossibleDomainRegexes()` 保持包含该正则。
- 同步 `scripts/sync-local-config.js` 与 example TOML。

### 步骤 6：测试

`tests/regression.test.js` 新增断言：

正向（必须走家宽）：

```
eu-west-1.api.x.ai
mtls.api.x.ai
api2.cursor.sh
authenticate.cursor.sh
prod.authentication.cursor.sh
agent.api5.cursor.sh
us-eu.gcpp.cursor.sh
antigravity.google
platform.claude.com
```

负向（不得走家宽）：

```
www.api2.cursor.sh
docs.antigravity.google
download.antigravity.google
www.antigravity.google
```

开关组合测试（参考现有 `withPatchedCursorSwitches` 模式）：

- `grok_web_assets = false` 时：`grok.com`、`cli-chat-proxy.grok.com`、`code.grok.com` 走家宽；
  `assets.grok.com` 不走家宽。
- `vertex_ai_regional = false` 时：`us-central1-aiplatform.googleapis.com` 不走家宽；
  `aiplatform.googleapis.com` 仍走家宽。

幂等清理测试：

- 输入含 `DOMAIN,api.x.ai,AI-家宽`、`DOMAIN-SUFFIX,api2.cursor.sh,AI-家宽`、
  `DOMAIN-SUFFIX,antigravity.google,AI-家宽` 的旧规则集合，
  断言 `cleanExistingManagedRules` 全部清除。
- 断言未知规则 `DOMAIN-SUFFIX,example-user-rule.com,AI-家宽` 保留。

### 步骤 7：文档与版本

- `docs/routing-scope.md`：更新 Included categories 表，补充本次的官方来源 URL；
  在 Explicit exclusions 中加入 `assets.grok.com`、`docs.antigravity.google`、
  `download.antigravity.google`；新增一节说明载体 A / 载体 B 的收窄口径差异。
- `docs/configuration.md`：记录两个新开关。
- `CHANGELOG.md`：新增版本条目（英文，ASD-STE100）。
- `package.json` 与 `SCRIPT_VERSION` 提升到 5.10.0。
- `tests/regression.test.js` 中的版本断言同步。

### 步骤 8：本地渲染

```bash
just render-local
```

不手工编辑 `*.local.js`。

## 验证命令

```bash
just ci        # node --check + 62+ 个回归测试 + 密钥扫描
```

全部通过后再提交。

## 回滚点

- 步骤 1-3 是纯清单调整，回滚方式是还原 `clash-verge-ai-residential.js` 中的三处数组。
- 步骤 4-5 引入新开关，回滚需同时还原脚本常量、`scripts/sync-local-config.js` 映射表
  与 example TOML，否则 `npm test` 中的「生产映射覆盖全部用户布尔开关」断言会失败。
- 本地 TOML 已有键不会被覆盖；缺键按 example 默认值补全。回滚后需再次 `just render-local`。

## 不做的事

- 不删除第 2.4 节列出的 17 条存疑规则。用途未查明前删除会造成不可预期的功能回退。
- 不收窄 `claude.ai`、`claude.com`、`chatgpt.com`、`gemini.google.com`、`aistudio.google.com`：
  收窄会在同一登录会话内产生两个出口 IP。
- 不改动 `ENABLE_ANTHROPIC_IP_FALLBACK`：官方文档确认 `160.79.104.0/23` 与 `2607:6bc0::/48`
  是 inbound 段，当前用法正确。
- 不新增 `auth.openai.com` / `accounts.google.com`：`docs/routing-scope.md` 已记录为有意取舍。
- 不改动上游解析、递归链防护、DNS 结构、嗅探逻辑。
