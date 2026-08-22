# 技术设计：AI 家宽路由域名清单收窄

版本：v2（2026-08-19，经外部审阅后重建）。

v1 的三处实质错误已更正，见第 7 节「更正记录」。本文件的逐条表现在是**互斥账本**：
当前 49 条注入规则，每条恰好出现一次。

## 1. 判定口径

用户约束是「只有网页 Chat、本机 CLI、Cursor 等客户端连接官方的流量走家宽」。
按载体拆成两类，因为收窄代价不同。

### 载体 A：浏览器内的网页 Chat

`claude.ai`、`claude.com`、`chatgpt.com`、`grok.com`、`gemini.google.com`、`aistudio.google.com`。
同一浏览器会话并发访问同一产品的多个子域并携带同一套 Cookie。把其中一部分子域改走机场，
会让服务端在同一个已登录会话内观察到两个出口 IP。
`docs/routing-scope.md` 的 Authentication exit split 一节已记录这种分裂对风控的影响。

对载体 A，产品顶级域默认保持后缀匹配。收窄方案以默认关闭的开关提供。

### 载体 B：本机 CLI 与桌面/IDE 客户端

Claude Code、Codex、Grok CLI、Cursor、Antigravity。
非推理请求（文档站、更新下载、扩展市场、静态资源）不携带推理会话凭据，
与推理连接是独立 TCP 会话，改走机场不产生会话内 IP 分裂。

对载体 B，按官方文档逐条收窄。

### 证据门槛

用户已确认采用**证据优先**策略：刷新官方资料后仍无官方来源、也无脱敏 Connections 证据的规则，
退出激活清单，保留在 `allPossible*` 中仅用于迁移清理；取得证据后逐条恢复。

## 2. 互斥账本：当前 49 条规则逐条判定

判定取值：`保留`（规则文本不变）、`收窄`（同一目标、更窄的规则文本）、
`改类型`、`退出激活`（不再注入，保留在 `allPossible*`）。

### 2.1 DOMAIN-SUFFIX（19 条）

| # | 当前规则值 | 官方出处 | 判定 | 目标规则 |
|---|---|---|---|---|
| 1 | `claude.ai` | code.claude.com network-config | 保留 | 不变 |
| 2 | `claude.com` | 同上 | 保留 | 不变（收窄方案见 3.3，待证据） |
| 3 | `clau.de` | 无 | 退出激活 | — |
| 4 | `claudemcpclient.com` | 无 | 退出激活 | — |
| 5 | `claudemcpcontent.com` | Desktop `*.claudemcpcontent.com` | 保留 | 不变。官方 MCP Apps widget 域 |
| 6 | `claudeusercontent.com` | network-config 表 + 说明段 | 保留 | 不变 |
| 7 | `antigravity.google` | 无防火墙清单 | 收窄 | `DOMAIN,antigravity.google` |
| 8 | `chatgpt.com` | help.openai.com 9247338 `*.chatgpt.com` | 保留 | 不变 |
| 9 | `oaiusercontent.com` | 同上 `*.oaiusercontent.com` | 保留 | 不变 |
| 10 | `api.openai.com` | 同上 `*.openai.com` | 保留 | 不变 |
| 11 | `gemini.google.com` | 产品入口 | 保留 | 不变 |
| 12 | `aistudio.google.com` | 产品入口 | 保留 | 不变 |
| 13 | `api2.cursor.sh` | cursor.com 精确主机清单 | 收窄 | `DOMAIN,api2.cursor.sh` |
| 14 | `api5.cursor.sh` | 官方列出六个 `agent*` 子域 | 保留 | 不变 |
| 15 | `gcpp.cursor.sh` | 官方三个区域前缀 | 保留 | 不变 |
| 16 | `authenticate.cursor.sh` | 官方精确主机 | 收窄 | `DOMAIN,authenticate.cursor.sh` |
| 17 | `authentication.cursor.sh` | 官方另列 `prod.` 前缀 | 保留 | 不变 |
| 18 | `cursorvm.com` | 官方 `*.cursorvm.com` / `*.*.cursorvm.com` | 保留 | 不变 |
| 19 | `grok.com` | docs.x.ai enterprise | 保留 | 不变（收窄见 3.1 开关） |

### 2.2 DOMAIN（26 条）

| # | 当前规则值 | 官方出处 | 判定 | 目标规则 |
|---|---|---|---|---|
| 20 | `api.anthropic.com` | network-config 表 | 保留 | 不变 |
| 21 | `a-api.anthropic.com` | Desktop 精简清单 + 3P「Analytics events」 | 退出激活 | 官方遥测主机，不是 Messages API |
| 22 | `mcp-proxy.anthropic.com` | network-config 表 | 保留 | 不变 |
| 23 | `assets-proxy.anthropic.com` | network-config 说明段 | 保留 | 不变 |
| 24 | `cloudcode-pa.googleapis.com` | Code Assist 防火墙清单 | 保留 | 不变 |
| 25 | `daily-cloudcode-pa.googleapis.com` | 无 | 退出激活 | — |
| 26 | `cloudaicompanion.googleapis.com` | Code Assist 防火墙清单（primary endpoint） | 保留 | 不变 |
| 27 | `geminicloudassist.googleapis.com` | 无 | 退出激活 | — |
| 28 | `generativelanguage.googleapis.com` | Gemini Developer API | 保留 | 不变 |
| 29 | `aiplatform.googleapis.com` | Antigravity Enterprise + Vertex AI 全局端点 | 保留 | 不变 |
| 30 | `chat.openai.com` | help 9247338 | 保留 | 不变 |
| 31 | `android.chat.openai.com` | 同上 | 保留 | 不变 |
| 32 | `desktop.chat.openai.com` | 同上 | 保留 | 不变 |
| 33 | `ios.chat.openai.com` | 同上 | 保留 | 不变 |
| 34 | `tcr9i.chat.openai.com` | 同上（官方无用途说明） | 保留 | 不变 |
| 35 | `alkalicore-pa.clients6.google.com` | 无官方防火墙清单 | 保留 | 不变。AI Studio 网页 RPC，UNVERIFIED |
| 36 | `alkalimakersuite-pa.clients6.google.com` | 无官方防火墙清单 | 保留 | 不变。社区标为 GenerateContent 主机，UNVERIFIED |
| 37 | `webchannel-alkalimakersuite-pa.clients6.google.com` | 无官方防火墙清单 | 保留 | 不变。AI Studio 流式通道，UNVERIFIED |
| 38 | `aiplatform.us.rep.googleapis.com` | Deployments and endpoints 主机名表 | 保留 | 不变 |
| 39 | `aiplatform.eu.rep.googleapis.com` | 同上 | 保留 | 不变 |
| 40 | `api3.cursor.sh` | cursor.com 精确主机清单 | 保留 | 不变 |
| 41 | `api4.cursor.sh` | 同上 | 保留 | 不变 |
| 42 | `authenticator.cursor.sh` | 同上 | 保留 | 不变 |
| 43 | `api.cursor.com` | Cloud Agents API / Bugbot API 文档 | 保留 | 不变 |
| 44 | `auth.x.ai` | docs.x.ai enterprise（必需） | 保留 | 不变 |
| 45 | `api.x.ai` | docs.x.ai regions + mtls | 改类型 | `DOMAIN-SUFFIX,api.x.ai` |

### 2.3 DOMAIN-REGEX（2 条）

| # | 当前规则值 | 官方出处 | 判定 | 目标规则 |
|---|---|---|---|---|
| 46 | `^[a-z0-9-]+-aiplatform\.googleapis\.com$` | Vertex AI 端点全表 | 保留 | 不变 |
| 47 | `^adminportal[0-9]+\.cursor\.sh$` | 官方仅 `adminportal42.cursor.sh` | 收窄 | `DOMAIN,adminportal42.cursor.sh` |

### 2.4 IP（2 条）

| # | 当前规则值 | 官方出处 | 判定 | 目标规则 |
|---|---|---|---|---|
| 48 | `IP-CIDR,160.79.104.0/23` | platform.claude.com ip-addresses（inbound） | 保留 | 不变 |
| 49 | `IP-CIDR6,2607:6bc0::/48` | 同上 | 保留 | 不变 |

### 2.5 对账

| 判定 | 条数 |
|---|---|
| 保留 | 39 |
| 退出激活 | 5 |
| 收窄 | 4 |
| 改类型 | 1 |
| **合计** | **49** |

改动后激活规则数：**44 条**（49 − 5）。

## 3. 开关

### 3.1 `routing.grok_web_assets`（默认 `true`）

| 值 | 注入 |
|---|---|
| `true` | `DOMAIN-SUFFIX,grok.com` |
| `false` | `DOMAIN,grok.com` + `DOMAIN,cli-chat-proxy.grok.com` + `DOMAIN,code.grok.com` |

依据：docs.x.ai/build/enterprise 把 `cli-chat-proxy.grok.com`（推理代理）与 `auth.x.ai` 列为必需，
把 `code.grok.com`（会话同步、分享、WebSocket 中继）与 `assets.grok.com`（头像与 UI 资源，
标注 `no functional impact`）列为可选。`false` 分支必须显式包含 `code.grok.com`，
否则远程会话同步与分享链接会退到机场出口。

### 3.2 `routing.vertex_ai_endpoints`（默认 `true`）

该开关一次性控制**全部四条** Vertex AI / Agent Platform 规则，避免设计与实现口径不一致：

- `DOMAIN,aiplatform.googleapis.com`
- `DOMAIN,aiplatform.us.rep.googleapis.com`
- `DOMAIN,aiplatform.eu.rep.googleapis.com`
- `DOMAIN-REGEX,^[a-z0-9-]+-aiplatform\.googleapis\.com$`

默认 `true`。antigravity.google/docs/enterprise 确认 Antigravity CLI 与 Antigravity 2.0
通过 Agent Platform API（`aiplatform.googleapis.com`）和 global/us/eu 端点做推理，
默认关闭会切断 Antigravity 企业部署的推理链路。
`false` 供不使用 Antigravity 企业版、也不希望通用 GCP 机器学习流量占用家宽的用户选择。

### 3.3 `claude.com` 的收窄方案（本次不实施）

官方可枚举的 `claude.com` 主机为 `claude.com`、`platform.claude.com`、`code.claude.com`、
`docs.claude.com`（前两者见 network-config 表，后两者见 cloud-environments 默认允许域）。
因此收窄为四条 `DOMAIN` 规则在技术上可行。

本次不实施，理由是官方原文把登录描述为
「sign-in opens a `claude.com` page in the browser, which redirects to `claude.ai`」，
未说明重定向链经过哪些主机，精确枚举可能漏掉中间跳。
实施前需要脱敏 Connections 证据记录一次完整登录过程中命中的 `*.claude.com` 主机。

## 4. 兼容性：退出激活后的旧规则清理

`buildManagedRuleSet()` 依据 `allPossibleSuffixDomains()` / `allPossibleExactDomains()` /
`allPossibleDomainRegexes()` 构造清理集合，与激活开关无关。
`buildManagedDnsPolicyKeySet()` 用同一组清单构造 DNS 键。

已实测验证：条目留在 allPossible 清单但移出激活清单后，旧规则被幂等清除，未知用户规则保留。

```
输入：DOMAIN-SUFFIX,chat.openai.com,AI-家宽 / DOMAIN,api.openai.com,AI-家宽
      DOMAIN-SUFFIX,claude.com,AI-家宽 / DOMAIN-SUFFIX,example-user-rule.com,AI-家宽
输出：DOMAIN-SUFFIX,example-user-rule.com,AI-家宽
```

**硬约束**：本次全部 5 条「退出激活」、4 条「收窄」、1 条「改类型」所涉及的旧规则值，
必须保留在对应的 `allPossible*` 清单中，否则升级用户配置里的旧规则不会被清理。

需要保留在 `allPossible*` 中的迁移清理项：

| 清单 | 需保留的值 |
|---|---|
| `allPossibleSuffixDomains()` | `clau.de`、`claudemcpclient.com`、`antigravity.google`、`api2.cursor.sh`、`authenticate.cursor.sh`、`grok.com` |
| `allPossibleExactDomains()` | `a-api.anthropic.com`、`daily-cloudcode-pa.googleapis.com`、`geminicloudassist.googleapis.com`、`api.x.ai`、`adminportal42.cursor.sh`、`grok.com`、`cli-chat-proxy.grok.com`、`code.grok.com`、`antigravity.google` |
| `allPossibleDomainRegexes()` | `^adminportal[0-9]+\.cursor\.sh$`、`^[a-z0-9-]+-aiplatform\.googleapis\.com$` |

## 5. DNS 与规则的不对称

`buildNameserverPolicy()` 只遍历 `activeSuffixDomains()` 与 `activeExactDomains()`，
不遍历 `activeDomainRegexes()`。收窄后仅剩一条正则（Vertex AI 区域端点）受影响：
路由走家宽，DNS 解析走机场 DoH。

在 `enhanced-mode: fake-ip` 下客户端拿到 fake IP，真实解析在出口侧完成，
该不对称不改变最终出口 IP。本次记录该现象，不作为必须修复项。

`adminportal` 收窄为 `DOMAIN` 后会获得对应的 DNS policy 键，该主机的不对称随之消除。

## 6. UNVERIFIED 清单

以下判断只有官方文档或单元级规则匹配证据，**没有部署后运行时证据**。
按 `docs/routing-scope.md` 既有约定标记为 UNVERIFIED，实施后需用脱敏 Clash Connections 补证。

| 项 | 未验证的内容 |
|---|---|
| 5 条退出激活规则 | 是否有任一条实际承载会话或推理流量；退出后是否出现出口 IP 分裂 |
| 三条 `alkali*` | AI Studio 网页是否实际请求这些 `clients6.google.com` 主机 |
| `antigravity.google` 收窄 | Antigravity 客户端是否访问 `docs.` / `download.` 子域，以及断开后是否影响启动或更新提示 |
| `adminportal42.cursor.sh` 收窄 | 企业 SSO 配置流程是否只使用编号 42 的主机 |
| `api.cursor.com` | Cursor 桌面客户端「Cloud」入口走该主机还是 `api2.cursor.sh`；官方只证明 API key 编程调用 |
| `grok_web_assets = false` | Grok 网页版在该模式下是否出现认证或资源加载的出口分裂 |
| `claude.com` 精确枚举 | 完整登录重定向链命中的 `*.claude.com` 主机集合 |
| `tcr9i.chat.openai.com` | 官方清单成员，用途未公开 |

## 7. 更正记录（v1 → v2）

v1 有三处实质错误，均由外部审阅指出并经二次检索确认：

1. **官方证据误判。** v1 把 `assets-proxy.anthropic.com`、`claudeusercontent.com`、
   `aiplatform.us.rep.googleapis.com`、`aiplatform.eu.rep.googleapis.com`、
   `cloudaicompanion.googleapis.com`、`api.cursor.com` 判为无官方出处。
   六项全部有官方出处。原因是首次抓取 code.claude.com/docs/en/network-config 时
   markdown 转换丢弃了表格，以及未检索 Deployments and endpoints、Cloud Agents API 等页面。
   「存疑」集合从 16 条降到 9 条。

2. **`claude.com` 保留理由不成立。** v1 称「只有后缀能覆盖 `platform.claude.com`」。
   实际可以并列多条 `DOMAIN` 规则。已改为按重定向链未知、待 Connections 证据处理（3.3 节）。

3. **`vertex_ai_regional` 开关定义与实现不一致。** v1 正文称关闭后「只保留全局端点」，
   实施计划却只关闭正则，两个 `.rep.` exact 主机仍会注入；同时正文建议「默认关」，
   开关表写 `true`。已重新定义为 `routing.vertex_ai_endpoints`，一次性控制四条规则，
   默认 `true`，依据是 Antigravity Enterprise 文档。

另有三处结构问题一并修复：

4. v1 的逐条表不是互斥账本：`DOMAIN,api.x.ai` 只出现在「补充」一节，未计入任何判定；
   「存疑」小计写 17，列表实为 16。本版改为一条当前规则一行，合计恰好 49。
5. v1 建议收窄 `adminportal` 正则，但未进入实施计划。本版列入 2.3 与实施步骤。
6. v1 未标注 UNVERIFIED 项，验收只到规则生成层。本版新增第 6 节。

## 8. 第三次审阅补记（2026-08-19）

完整条目见 `research/review-findings.md`。

1. **主机 vs 路径。** Clash 规则只匹配主机。Codex 的
   `/backend-api/codex/responses` 与同主机上的 ChatGPT 网页路径无法拆开。
2. **9247338 的定位。** 该页是企业防火墙不要拦截的清单，不能单独证明必须保留后缀。
   用户已确认保持 `DOMAIN-SUFFIX,chatgpt.com`（第 2.1 第 8 行维持「保留」）。
   `ws.chatgpt.com` 随后缀覆盖。`help.` / `status.` 子域走家宽，记为已知取舍。
3. **IP 兜底重叠。** `a-api.anthropic.com` 即使退出域名清单，
   在 `ENABLE_ANTHROPIC_IP_FALLBACK` 开启时仍可能命中 inbound CIDR。
