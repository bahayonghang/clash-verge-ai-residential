# 执行计划：AI 家宽路由域名清单收窄

版本：v2（2026-08-19，经外部审阅后重建）。
前置条件：`task.py start` 之后才能执行。当前状态 planning。

改动范围：`clash-verge-ai-residential.js`、`scripts/sync-local-config.js`、
`clash-verge-ai-residential.local.toml.example`、`tests/regression.test.js`、
`docs/routing-scope.md`、`docs/configuration.md`、`CHANGELOG.md`、`package.json`。

目标终态：激活规则 49 条 → 44 条。逐条依据见 `design.md` 第 2 节互斥账本。

## 步骤 1：5 条规则退出激活清单

用户已确认采用证据优先策略。**只从激活清单移除，必须保留在 `allPossible*` 中。**
`claudemcpcontent.com` 已有官方 Desktop 通配，用户确认保留，不在本步。

从 `CORE_SUFFIX_DOMAINS` 移除：

```
clau.de
claudemcpclient.com
```

从 `CORE_EXACT_DOMAINS` 移除（已用 `grep -n` 核实归属）：

```
a-api.anthropic.com                 （clash-verge-ai-residential.js:212）
daily-cloudcode-pa.googleapis.com   （:221）
geminicloudassist.googleapis.com    （:223）
```

三条 `alkali*` 用户确认保留在 `GEMINI_WEB_EXACT_DOMAINS`，标 UNVERIFIED，本步不移除。

同时在 `allPossibleSuffixDomains()` 与 `allPossibleExactDomains()` 中显式补入这 5 个值
（新增独立的 `RETIRED_*` 常量数组，并在两个 allPossible 函数中展开），
使旧规则与旧 DNS 键可被幂等清理。

## 步骤 2：`api.x.ai` 改类型（exact → suffix）

- 从 `GROK_EXACT_DOMAINS` 移除 `api.x.ai`，加入 `GROK_SUFFIX_DOMAINS`。
- `api.x.ai` 保留在 `allPossibleExactDomains()` 中，用于清理 v5.9 生成的
  `DOMAIN,api.x.ai,AI-家宽` 与 DNS 键 `api.x.ai`。
- 依据：docs.x.ai/developers/regions 的 `<region>.api.x.ai`、
  docs.x.ai/developers/advanced-api-usage/mtls 的 `mtls.api.x.ai`。

## 步骤 3：Cursor 两条 suffix → exact

- `CURSOR_SUFFIX_DOMAINS` 移除 `api2.cursor.sh`、`authenticate.cursor.sh`。
- `CURSOR_EXACT_DOMAINS` 加入这两个主机。
- 两个值保留在 `allPossibleSuffixDomains()` 中。
- 不改动 `api5.cursor.sh`、`gcpp.cursor.sh`、`authentication.cursor.sh`、`cursorvm.com`：
  官方文档确认这四项需要后缀匹配（`agent*` 子域、三个区域前缀、`prod.` 前缀、官方通配）。

## 步骤 4：`adminportal` 正则 → 精确主机

- `CURSOR_CORE_DOMAIN_REGEXES` 移除 `^adminportal[0-9]+\.cursor\.sh$`。
- `CURSOR_EXACT_DOMAINS` 加入 `adminportal42.cursor.sh`。
- 正则保留在 `allPossibleDomainRegexes()` 中，`adminportal42.cursor.sh` 进入
  `allPossibleExactDomains()`。
- 依据：cursor.com/docs/enterprise/network-configuration 只列出 `adminportal42.cursor.sh`。
- 附带收益：该主机改为 exact 后会获得 `nameserver-policy` 键，消除该主机的 DNS 不对称。

## 步骤 5：Antigravity suffix → exact

- `CORE_SUFFIX_DOMAINS` 移除 `antigravity.google`，`CORE_EXACT_DOMAINS` 加入。
- `antigravity.google` 保留在 `allPossibleSuffixDomains()` 中。

## 步骤 6：新增 `routing.grok_web_assets`（默认 `true`）

- 新增 `const ROUTE_GROK_WEB_ASSETS = true;`。
- `activeSuffixDomains()`：`true` 时用 `GROK_SUFFIX_DOMAINS = ["grok.com"]`。
- `activeExactDomains()`：`false` 时改用
  `GROK_STRICT_EXACT_DOMAINS = ["grok.com", "cli-chat-proxy.grok.com", "code.grok.com"]`。
- `allPossibleSuffixDomains()` 与 `allPossibleExactDomains()` 同时包含两组，保证双向切换都能清理。
- `scripts/sync-local-config.js` 映射表新增
  `{ table: "routing", key: "grok_web_assets", constant: "ROUTE_GROK_WEB_ASSETS", type: "boolean" }`。
- `clash-verge-ai-residential.local.toml.example` 新增 `grok_web_assets = true`。

## 步骤 7：新增 `routing.vertex_ai_endpoints`（默认 `true`）

该开关必须一次性控制**四条**规则，否则设计与实现口径不一致（v1 的错误）：

| 规则 | 所在常量 |
|---|---|
| `DOMAIN,aiplatform.googleapis.com` | `CORE_EXACT_DOMAINS` |
| `DOMAIN,aiplatform.us.rep.googleapis.com` | `GEMINI_WEB_EXACT_DOMAINS` |
| `DOMAIN,aiplatform.eu.rep.googleapis.com` | `GEMINI_WEB_EXACT_DOMAINS` |
| `DOMAIN-REGEX,^[a-z0-9-]+-aiplatform\.googleapis\.com$` | `GEMINI_DOMAIN_REGEXES` |

- 把这四项抽到独立常量 `VERTEX_AI_EXACT_DOMAINS` 与 `VERTEX_AI_DOMAIN_REGEXES`。
- `activeExactDomains()` / `activeDomainRegexes()` 依 `ROUTE_VERTEX_AI_ENDPOINTS` 决定是否展开。
- 三个 allPossible 函数无条件包含这四项。
- 默认 `true`。依据：antigravity.google/docs/enterprise 确认 Antigravity CLI 与
  Antigravity 2.0 经 Agent Platform API 与 global/us/eu 端点做推理。
- 同步 `scripts/sync-local-config.js` 与 example TOML。

## 步骤 8：测试

`tests/regression.test.js` 新增断言。

### 正向（必须走家宽）

```
platform.claude.com
bridge.claudeusercontent.com
x.frame.claudeusercontent.com
assets-proxy.anthropic.com
eu-west-1.api.x.ai
mtls.api.x.ai
api.x.ai
api2.cursor.sh
authenticate.cursor.sh
prod.authentication.cursor.sh
agent.api5.cursor.sh
us-eu.gcpp.cursor.sh
adminportal42.cursor.sh
api.cursor.com
antigravity.google
aiplatform.us.rep.googleapis.com
aiplatform.eu.rep.googleapis.com
cloudaicompanion.googleapis.com
us-central1-aiplatform.googleapis.com
widget.claudemcpcontent.com
alkalicore-pa.clients6.google.com
alkalimakersuite-pa.clients6.google.com
webchannel-alkalimakersuite-pa.clients6.google.com
```

### 负向（不得走家宽）

```
clau.de
claudemcpclient.com
a-api.anthropic.com
daily-cloudcode-pa.googleapis.com
geminicloudassist.googleapis.com
adminportal0.cursor.sh
adminportal999.cursor.sh
www.api2.cursor.sh
docs.antigravity.google
download.antigravity.google
www.antigravity.google
```

### 开关组合（参考现有 `withPatchedCursorSwitches` 模式）

- `grok_web_assets = false`：`grok.com`、`cli-chat-proxy.grok.com`、`code.grok.com` 走家宽；
  `assets.grok.com` 不走家宽。
- `grok_web_assets = true`（默认）：`assets.grok.com` 走家宽。
- `vertex_ai_endpoints = false`：`aiplatform.googleapis.com`、两个 `.rep.` 主机、
  `us-central1-aiplatform.googleapis.com` 四者**全部**不走家宽。
  该断言是步骤 7 的正确性守卫，防止只关正则的实现回归。
- `vertex_ai_endpoints = true`（默认）：四者全部走家宽。

### 幂等清理

构造含以下旧规则的输入，断言 `cleanExistingManagedRules` 全部清除：

```
DOMAIN-SUFFIX,clau.de,AI-家宽
DOMAIN-SUFFIX,claudemcpclient.com,AI-家宽
DOMAIN,a-api.anthropic.com,AI-家宽
DOMAIN,daily-cloudcode-pa.googleapis.com,AI-家宽
DOMAIN,geminicloudassist.googleapis.com,AI-家宽
DOMAIN,api.x.ai,AI-家宽
DOMAIN-SUFFIX,api2.cursor.sh,AI-家宽
DOMAIN-SUFFIX,authenticate.cursor.sh,AI-家宽
DOMAIN-SUFFIX,antigravity.google,AI-家宽
DOMAIN-REGEX,^adminportal[0-9]+\.cursor\.sh$,AI-家宽
```

同时断言未知规则 `DOMAIN-SUFFIX,example-user-rule.com,AI-家宽` 保留。

对 `buildNameserverPolicy({})` 断言：上述 5 个退出激活的主机不再出现对应的
`+.<domain>` 或 `<domain>` 键。

### 规则总数

断言 `buildInjectedRules().filter(r => r.includes(AI_GROUP)).length === 44`。
该断言把 `design.md` 第 2.5 节的对账固化，任何漏改或多改都会失败。

## 步骤 9：文档与版本

- `docs/routing-scope.md`：
  - 更新 Included categories，补本次官方来源 URL。
  - Explicit exclusions 新增 `assets.grok.com`、`docs.antigravity.google`、
    `download.antigravity.google`、`adminportal<N≠42>.cursor.sh`，以及 5 条退出激活的域名。
  - 新增「载体 A / 载体 B」口径小节。
  - 新增 UNVERIFIED 小节，逐条抄录 `design.md` 第 6 节。
- `docs/configuration.md`：记录两个新开关。
- `CHANGELOG.md`：新增 5.10.0 条目（英文，ASD-STE100），说明退出激活的 5 条与收窄的 4 条。
- `SCRIPT_VERSION`、`package.json` 提升到 5.10.0，同步 `tests/regression.test.js` 版本断言。

## 步骤 10：本地渲染

```bash
just render-local
```

不手工编辑 `*.local.js`。

## 验证命令

```bash
just ci        # node --check + 回归测试 + 密钥扫描
```

## 步骤 11：部署后运行时取证（必做，不可用单元测试替代）

`design.md` 第 6 节列出的 UNVERIFIED 项无法由 `ruleMatchesHost` 单元测试证明。
渲染并加载新 Profile 后，逐项采集脱敏 Clash Connections 证据：

| 场景 | 操作 | 需记录 |
|---|---|---|
| Claude 网页登录 | 完整登录一次 | 命中的 `*.claude.com` / `*.claude.ai` 主机与各自出口 |
| Claude Desktop / Artifacts | 打开一个 Artifact | 是否命中 `*.frame.claudeusercontent.com`、`assets-proxy.anthropic.com` |
| Claude Desktop / MCP App | 打开一个 MCP App widget | 是否命中 `*.claudemcpcontent.com` |
| Grok 网页版 | 登录并发一轮对话 | `grok.com` 下命中的子域；确认无认证出口分裂 |
| Grok CLI | 一轮推理 + 一次会话分享 | `cli-chat-proxy.grok.com`、`code.grok.com` |
| Antigravity | 启动、登录、一轮推理 | 是否命中 `docs.` / `download.antigravity.google`；命中的 aiplatform 端点 |
| Cursor | 登录、Tab、Agent、Cloud Agent | `api.cursor.com` 是否出现；SSO 是否只用 adminportal42 |
| AI Studio 网页 | 登录并发一轮对话 | 是否命中三条 `alkali*`；未命中则回到 Plan 再退出 |
| 5 条退出激活域名 | 日常使用一周 | 是否有任一条出现在 Connections 中 |

采集结果写入任务的 `research/runtime-evidence.md`。
出现「退出激活的域名实际承载会话流量」时，回到 Plan 阶段按证据恢复该条，不在 Execute 内直接加回。

## 回滚点

- 步骤 1-5 是清单调整，回滚方式是还原 `clash-verge-ai-residential.js` 中对应数组。
- 步骤 6-7 引入新开关，回滚需同时还原脚本常量、`scripts/sync-local-config.js` 映射表
  与 example TOML，否则 `npm test` 的「生产映射覆盖全部用户布尔开关」断言会失败。
- 本地 TOML 已有键不会被覆盖；缺键按 example 默认值补全。回滚后需再次 `just render-local`。

## 不做的事

- 不收窄 `claude.ai`、`claude.com`、`gemini.google.com`、`aistudio.google.com`：
  收窄会在同一登录会话内产生两个出口 IP。`claude.com` 的精确枚举方案见 `design.md` 3.3，
  待 Connections 证据。
- 不收窄 `chatgpt.com`：用户已确认保持 `DOMAIN-SUFFIX,chatgpt.com`。
  `ws.chatgpt.com` 随后缀覆盖；`help.` / `status.` 等子域走家宽，记为已知取舍。
- 不改动 `ENABLE_ANTHROPIC_IP_FALLBACK`：官方确认 `160.79.104.0/23` 与 `2607:6bc0::/48`
  是 inbound 段，当前用法正确。
- 不新增 `auth.openai.com` / `accounts.google.com` / `oauth2.googleapis.com`：
  `docs/routing-scope.md` 已记录为有意取舍。
- 不改动上游解析、递归链防护、DNS 结构、嗅探逻辑。
