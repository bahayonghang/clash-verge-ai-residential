# 任务审阅记录（2026-08-19 第三次检索）

对照对象：`.trellis/tasks/08-19-ai-route-domain-audit` 的 `prd.md` / `design.md` / `implement.md` 与五份调研。
检索工具：Exa；Codex 源码取自 `github.com/openai/codex` 的 `main` 分支 raw 文件。
任务状态仍为 `planning`，本文件只记录审阅，不授权 `task.py start`。

## 用户目标（本轮）

把 Codex、Claude 等**官方后台**（推理 / 会话 / 必要认证）送到家宽，
并且不要把文档、状态、营销、下载、遥测等无关流量送到家宽。

用户给出的 Codex 默认后端已独立核验，见 `openai-codex-endpoints.md`。

## 成立的规划

| 项 | 说明 |
|---|---|
| 49 条互斥账本 | `design.md` 第 2 节一条规则一行，分项之和 49 |
| 证据优先 | 无官方来源且无 Connections 的规则退出激活、留在 `allPossible*` |
| Anthropic inbound IP | `160.79.104.0/23` 与 `2607:6bc0::/48` 方向正确，无需改 |
| Cursor 可收窄项 | `api2.cursor.sh`、`authenticate.cursor.sh` 改为 exact；`adminportal` 收到 `adminportal42` |
| `api5` / `gcpp` / `authentication` / `cursorvm.com` 保留后缀 | 与 2026-08-19 再取的 cursor.com 企业网络页一致 |
| `api.x.ai` 改 suffix | 覆盖官方 `<region>.api.x.ai` 与 `mtls.api.x.ai` |
| `vertex_ai_endpoints` 一次控制四条 | 修正 v1 开关口径分裂 |
| `grok_web_assets` | 官方把 `assets.grok.com` 标为无功能影响 |
| 不注入共享第三方 | OpenAI Intercom/Sentry、Claude statsig、Cursor 市场/CDN，与 `docs/routing-scope.md` 一致 |

Cursor 与 xAI 官方页 2026-08-19 再取，与现有 `cursor-domains.md`、`xai-grok-domains.md` 无实质冲突。

## 规划缺陷

### 1. Codex 默认后台未写入 OpenAI 调研（高）

`research/openai-domains.md` 只写「Codex 会话通道走 chatgpt.com 的 WebSocket」，
没有记录源码常量 `CHATGPT_CODEX_BASE_URL`，也没有记录 ChatGPT 登录 vs API Key 的主机分裂。

现行规则**已经覆盖**用户点名的三条 URL（见下表）。缺陷是依据不完整，
后续若按「官方 `*.chatgpt.com` 通配所以保留后缀」来辩护，会把防火墙放行清单
误当成家宽注入合同。同页的 `*.oaistatic.com` 已被仓库排除，说明该合同不成立。

| 用户给出的 URL | 主机 | 当前规则 |
|---|---|---|
| `https://chatgpt.com/backend-api/codex/responses` | `chatgpt.com` | `DOMAIN-SUFFIX,chatgpt.com` |
| `https://chatgpt.com/backend-api/codex/responses/compact` | `chatgpt.com` | 同上 |
| `https://api.openai.com/v1/responses` | `api.openai.com` | `DOMAIN-SUFFIX,api.openai.com` |

Clash 不能按路径匹配。无法只代理 `/backend-api/codex/*` 而放过同主机其它路径。

### 2. 漏记官方 ChatGPT WebSocket 主机 `ws.chatgpt.com`（高）

help.openai.com/9247338（2026-08-19 再取）原文表：

- ChatGPT → `wss://ws.chatgpt.com`（Conversation updates and notifications）
- Codex → `wss://chatgpt.com/`（Codex model sampling and streaming）

08-17 的摘录文件已经写出这两行，本任务的 `openai-domains.md` 没有把 `ws.chatgpt.com` 写成独立主机。

若把 `chatgpt.com` 从 suffix 收到 exact，必须同时注入 `DOMAIN,ws.chatgpt.com`，
否则 ChatGPT 网页对话的官方 WebSocket 会离开家宽。Codex 本身不受影响。

### 3. `implement.md`「不做的事」与用户本轮「不要无关流量」冲突（中）

现行计划明确不收窄 `chatgpt.com` / `claude.ai` / `claude.com`。
理由是载体 A 的会话 Cookie 与出口 IP 分裂。

该理由对**裸域内的路径**成立（无法拆），对**子域**不自动成立。
官方已给出 `ws.chatgpt.com` 这一条可枚举的会话子域，因此
`chatgpt.com` 存在比「整域后缀」更窄、仍覆盖 Codex + ChatGPT 网页通道的方案：

```
DOMAIN,chatgpt.com
DOMAIN,ws.chatgpt.com
DOMAIN-SUFFIX,api.openai.com
DOMAIN-SUFFIX,oaiusercontent.com
```

该方案仍会把裸域 `chatgpt.com` 上的帮助页带走，只排除子域上的 help/status/events。

### 4. 三条 `alkali*` 退出激活，与 PRD 的 AI Studio 网页会话冲突（中）

`alkalimakersuite-pa.clients6.google.com` 没有官方防火墙清单，
但 Google 开发者论坛与社区抓包把它标为 AI Studio 网页 `GenerateContent` RPC。
三条主机目前挂在 `GEMINI_WEB_EXACT_DOMAINS`，由 `gemini_web_core` 控制。

按证据优先退出后：`aistudio.google.com` 页面走家宽，实际推理 RPC 走机场。
这与载体 A「同一网页会话不要两个出口 IP」的口径相反。
9 条退出里，这一组的功能风险高于 `clau.de` 这类无产品入口的域名。

官方出处仍未找到。可选：保留并标 UNVERIFIED，或退出前先补 AI Studio 的脱敏 Connections。

### 5. `claudemcpcontent.com`「无官方出处」已过时（低）

https://claude.com/docs/claude-science/network-requirements 把
`*.claudemcpcontent.com` 列为 Claude Science 隔离 iframe 域。
Claude Code 的 network-config 表仍未列出该域。

若产品范围不含 Claude Science / MCP App iframe，退出激活仍可成立，
但依据应改为「官方只在 Science 文档出现，不在 Claude Code 推理通道」，
而不是「无官方出处」。

### 6. `a-api.anthropic.com` 仍无官方文档，但有第三方 DNS 指向 inbound IP（低）

netify 记录该主机解析到 `160.79.104.10` / `2607:6bc0::10`（Anthropic inbound 段）。
官方 network-config 与 API overview 只写 `api.anthropic.com`。
保持退出激活仍然符合证据门槛；若开启 `ENABLE_ANTHROPIC_IP_FALLBACK`，
即使域名规则退出，该 IP 仍可能被 CIDR 兜进家宽。需在 design 中写明这一重叠。

## 不在本轮改代码的范围

- 上游解析、DNS 结构、嗅探、residential-monitor
- 把 `auth.openai.com` / `accounts.google.com` 加进家宽
- 按 URL 路径拆 `chatgpt.com`（Clash 做不到）

## 本轮已定（写入 PRD）

- 保持 `DOMAIN-SUFFIX,chatgpt.com`。
- 保留 `DOMAIN-SUFFIX,claudemcpcontent.com`。
- 三条 `alkali*` 保留并标 UNVERIFIED。
- `a-api.anthropic.com` 按官方遥测用途退出激活。
