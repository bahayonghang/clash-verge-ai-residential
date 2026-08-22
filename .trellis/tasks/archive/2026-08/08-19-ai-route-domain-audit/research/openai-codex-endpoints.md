# OpenAI Codex 官方后台端点（主机 vs 路径）

调研日期：2026-08-19。对照用户给出的默认后端地址做独立核验。

Clash / Mihomo 规则只匹配主机（SNI / Host），不匹配 URL 路径。
因此 `chatgpt.com/backend-api/codex/responses` 与同主机上的网页对话、帮助页无法用域名规则拆开。

## 来源

- https://github.com/openai/codex/blob/main/codex-rs/model-provider-info/src/lib.rs
- https://developers.openai.com/codex/config-sample
- https://developers.openai.com/codex/config-advanced
- https://developers.openai.com/codex/auth
- https://help.openai.com/en/articles/9247338-network-recommendations-for-chatgpt-errors-on-web-and-apps
- https://developers.openai.com/api/reference/resources/responses/methods/compact/

## 用户陈述核验

用户给出的默认后端：

| 登录方式 | 用户给出的地址 | 核验 |
|---|---|---|
| ChatGPT 账号 | `https://chatgpt.com/backend-api/codex/responses` | 成立 |
| ChatGPT 账号，上下文压缩 | `https://chatgpt.com/backend-api/codex/responses/compact` | 成立 |
| API Key | `https://api.openai.com/v1/responses` | 成立 |

官方源码常量（`codex-rs/model-provider-info/src/lib.rs`）：

```
pub const CHATGPT_CODEX_BASE_URL: &str = "https://chatgpt.com/backend-api/codex";
```

`WireApi` 默认且唯一合法值为 `Responses`，注释原文：

> The Responses API exposed by OpenAI at `/v1/responses`.

`ModelProviderInfo::to_api_provider` 的默认 base URL 逻辑（原文结构）：

- `AuthMode::Chatgpt` / `ChatgptAuthTokens` / `Headers` / `AgentIdentity` / `PersonalAccessToken` → `CHATGPT_CODEX_BASE_URL`
- 其余（含 API Key）→ `"https://api.openai.com/v1"`
- 若配置了 `base_url`，覆盖上述默认值

客户端再拼接 `/responses`。ChatGPT 登录态的线上地址就是
`https://chatgpt.com/backend-api/codex/responses`。

官方示例配置（https://developers.openai.com/codex/config-sample）把两个基址分开写：

```
# Base URL for ChatGPT auth flow (not OpenAI API).
chatgpt_base_url = "https://chatgpt.com/backend-api/"

# Optional base URL override for the built-in OpenAI provider.
# openai_base_url = "https://us.api.openai.com/v1"
```

`chatgpt_base_url` 标注为 ChatGPT **登录流**基址，不是模型 API。
模型采样基址是 `CHATGPT_CODEX_BASE_URL` 或 `api.openai.com/v1`。

## 压缩与 WebSocket（同一主机，不同路径 / 协议）

ChatGPT 登录态压缩：同一 base 再拼 `/responses/compact`。
GitHub issue #15046 的失败 URL 原文：

`https://chatgpt.com/backend-api/codex/responses/compact`

API Key 模式的压缩路径由官方 Responses API 给出：`POST /responses/compact`，
即 `https://api.openai.com/v1/responses/compact`。
Codex 测试 `compact_remote.rs` 断言 `compact_request.path() == "/v1/responses/compact"`。

GitHub issue #13406 记录的 WebSocket 主机（运行时日志，非防火墙文档）：

| 登录方式 | WebSocket |
|---|---|
| ChatGPT | `wss://chatgpt.com/backend-api/codex/responses` |
| API Key | `wss://api.openai.com/v1/responses` |

官方 help 9247338 的 WebSocket 表（2026-08-19 再取，比 08-17 摘录多出用途列）：

| Product area | Destination | Purpose |
|---|---|---|
| ChatGPT | `wss://ws.chatgpt.com` | Conversation updates and notifications |
| Codex | `wss://chatgpt.com/` | Codex model sampling and streaming |

官方补充：若代理不能按 URL 路径放行 WebSocket，则对 `chatgpt.com` 的 TCP 443 放行 Upgrade。

结论：

- Codex 采样 / 压缩 / WebSocket 的主机是裸域 `chatgpt.com`，不是子域。
- ChatGPT **网页**对话的官方 WebSocket 主机是 `ws.chatgpt.com`。
- `DOMAIN,chatgpt.com` 已覆盖 Codex ChatGPT 登录态的 HTTPS 与 WebSocket。
- `DOMAIN,chatgpt.com` **不**覆盖 `ws.chatgpt.com`。
- `DOMAIN-SUFFIX,chatgpt.com` 同时覆盖两者，以及 `help.` / `status.` 等未列入会话通道的子域。

## 数据驻留

https://developers.openai.com/codex/config-advanced 原文示例：

```
openai_base_url = "https://us.api.openai.com/v1"
```

以及：

```
[model_providers.openaidr]
base_url = "https://us.api.openai.com/v1" # Replace 'us' with domain prefix
```

脚本当前 `DOMAIN-SUFFIX,api.openai.com` 覆盖 `us.api.openai.com` 与 `eu.api.openai.com`。
该项与 Codex API Key / 驻留前缀一致，不是缺口。

ChatGPT 工作区驻留：官方写「sign in with ChatGPT 时 Codex 遵守 workspace residency」，
不要求自定义 `us.api.openai.com` provider。

## 当前脚本覆盖

| 流量 | 主机 | 当前规则 | 是否覆盖 |
|---|---|---|---|
| Codex ChatGPT 登录，Responses | `chatgpt.com` | `DOMAIN-SUFFIX,chatgpt.com` | 覆盖 |
| Codex ChatGPT 登录，compact | `chatgpt.com` | 同上 | 覆盖 |
| Codex ChatGPT 登录，WebSocket | `chatgpt.com` | 同上 | 覆盖 |
| Codex API Key，Responses / compact / WS | `api.openai.com` | `DOMAIN-SUFFIX,api.openai.com` | 覆盖 |
| Codex 数据驻留 `us.` / `eu.` | `us.api.openai.com` 等 | 同上 | 覆盖 |
| ChatGPT 网页 WebSocket | `ws.chatgpt.com` | 被 chatgpt.com 后缀覆盖 | 覆盖 |
| Codex / ChatGPT 登录 token 交换 | `auth.openai.com` | 未注入 | 有意不覆盖 |

`chatgpt.com` 上无法按路径拆分的流量（同一主机）：

- `/backend-api/codex/responses`（Codex 推理，要走家宽）
- `/backend-api/codex/responses/compact`（Codex 压缩，要走家宽）
- `/backend-api/conversation` 与 `/backend-api/f/conversation`（ChatGPT 网页对话，PRD 要走家宽；社区观测，官方未按路径公布）
- 裸域上的营销页、帮助页（无法用域名规则排除）

后缀额外纳入、官方 9247338 未逐条标明用途的子域示例：`help.chatgpt.com`、`status.chatgpt.com`。
`ws.chatgpt.com` 是官方标明的 ChatGPT 会话 WebSocket，不属于「无关」。

## 对规划的含义

1. 用户点名的 Codex 默认后台，**已经**被当前 `chatgpt.com` 后缀与 `api.openai.com` 后缀覆盖。缺口不在「漏掉 Codex」，而在「后缀把无关子域一并带走」。
2. 官方 9247338 的 `*.chatgpt.com` 是企业防火墙 **不要拦截** 的清单，不是「全部走家宽」的合同。同页的 `*.oaistatic.com`、Intercom、Sentry 已被本仓库排除，说明该页不能直接当注入清单。
3. 若收窄 `chatgpt.com` 后缀，最低完整集合是：
   - `DOMAIN,chatgpt.com`（Codex ChatGPT 登录后台 + ChatGPT 网页 HTTPS）
   - `DOMAIN,ws.chatgpt.com`（官方 ChatGPT WebSocket）
   - 保留 `DOMAIN-SUFFIX,api.openai.com` 与 `oaiusercontent.com`
4. 收窄后仍无法把裸域 `chatgpt.com` 上的帮助页从家宽拆走。只能拆子域。
5. `auth.openai.com` 继续走机场：Codex `codex login` 的 token 交换与模型流量出口 IP 不同。`docs/routing-scope.md` 已记录该取舍。

## UNVERIFIED

| 项 | 内容 |
|---|---|
| ChatGPT 网页对话的 HTTPS 主机集合 | 社区观测为裸域 `chatgpt.com` 的 `/backend-api/...`；无官方逐条主机表。是否还有其它会话子域，需脱敏 Connections |
| `experimental_realtime_ws_base_url` | 官方配置键存在，默认主机未在本轮源码摘录中展开 |
| Codex 插件目录 `chatgpt.com/backend-api/plugins/featured` | GitHub issue 日志出现；与推理同主机，收窄裸域后仍覆盖 |
