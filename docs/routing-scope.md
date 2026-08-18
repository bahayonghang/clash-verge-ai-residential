# Routing Scope

The residential link is reserved for core AI product traffic. A domain is not included merely because an AI web page loads it.

## Included categories

| Product | Included traffic |
|---|---|
| Claude / Anthropic | Claude 产品域名、Messages API、`mcp-proxy.anthropic.com` MCP 连接器代理、`assets-proxy.anthropic.com` 资源代理、MCP/会话内容，以及官方入站 IP 回退 |
| ChatGPT / OpenAI | ChatGPT 产品域名、五个官方 exact 主机（`chat.openai.com`、`android.chat.openai.com`、`desktop.chat.openai.com`、`ios.chat.openai.com`、`tcr9i.chat.openai.com`）、OpenAI 模型 API 后缀 `api.openai.com`（覆盖 Codex 官方的 `us.` / `eu.` 数据驻留前缀），以及上传或生成的用户内容 |
| Gemini | Gemini Web, Google AI Studio product RPC/streaming hosts, Gemini Developer API, and Vertex AI regional/global model endpoints |
| Google Antigravity / Gemini Code Assist | Product domain and product-specific Code Assist/agent APIs |
| Cursor | Chat/API, Tab, Agent, Cloud Agent/Bugbot API, authorize endpoint, SSO admin portal, Cloud Agent VM hosts, and product-specific authentication; `routing.cursor_core` is `true` by default. Repository indexing hosts `repo[0-9]+.cursor.sh` use the independent `routing.cursor_repository_indexing` switch, which defaults to `false` and falls back to the original Profile/airport upstream |
| Grok Build | Grok 产品域名，覆盖 `cli-chat-proxy.grok.com` 推理 API 和 `/v1/storage` 代码库/会话上传；还包括 `auth.x.ai` OAuth 主机和 `api.x.ai` 直连 API 端点；`routing.grok_core` 默认为 `true` |

Cursor 依据：官方企业网络配置文档列出了 `authenticate.cursor.sh`、`adminportal<N>.cursor.sh` 和 `*.cursorvm.com` 虚拟机主机，以及此前已覆盖的 API、Tab 和 Agent 端点。官方网络文档与本机 2026-08-17 Cursor 索引日志共同确认 `repo42.cursor.sh` 为仓库索引主机；`repo[0-9]+.cursor.sh` 是本项目的前向兼容策略，不是 Cursor 官方通配合同。默认 `routing.cursor_repository_indexing = false` 只让这些索引专属主机回落原 Profile，不阻止 Chat/Agent 发送代码上下文，也不能在 `disableHttp2` 或服务端强制 HTTP/1.1 把 RepositoryService 放到共享 `api2.cursor.sh` 时继续隔离索引。`api2.cursor.sh` 仍由 `routing.cursor_core` 控制。Privacy Mode 不会停止索引上传。Grok 依据：对 grok 0.2.93 的线级抓包表明，`cli-chat-proxy.grok.com` 承载 `/v1/responses` 推理和 `/v1/storage` 上传；同一主机也承载 Grok Web 和事件遥测，无法在域名层拆分。v5.7 依据：Claude Code 官方网络配置文档列出了 `mcp-proxy.anthropic.com` 和 `assets-proxy.anthropic.com`；xAI 官方企业部署文档将 `auth.x.ai` 列为必须放行的主机，并将 `api.x.ai` 列为直连 API 主机；Codex 官方配置参考记录了 `us.api.openai.com` 和 `eu.api.openai.com` 数据驻留前缀，因此 `api.openai.com` 的规则由精确匹配改为后缀匹配。v5.8 依据：OpenAI 官方 help 文章 9247338 明文列出上述五个 `chat.openai.com` 家族主机；`tcr9i.chat.openai.com` 在该表中无用途说明。不注入 `DOMAIN-SUFFIX,chat.openai.com`。真实 ChatGPT 桌面/iOS Connections 结果为 UNVERIFIED。

已知取舍：`downloads.claude.ai`（安装程序和自动更新主机）位于 `claude.ai` 后缀下，因此也会经过住宅链路。若将其拆出，需要注入包含动态解析上游名称的规则，而当前基于精确字符串的托管规则清理模型无法安全清理此类规则。更新下载频率较低，剩余影响仅为占用住宅链路带宽。

## Explicit exclusions

The following classes stay on the original Profile route:

- Cursor Marketplace, extension installation, application downloads, CDN, updates, Remote-SSH/WSL server assets, website, documentation, and forum.
- Grok Build third-party analytics (`api.mixpanel.com`), the `x.ai` install script/privacy endpoints, and the shared `storage.googleapis.com` backend for codebase uploads.
- YouTube, Maps, Google Search, Google Fonts, Gstatic, advertising, analytics, and other generic Google services.
- OpenAI/Claude customer support, telemetry, feature flags, fraud prevention, payment, and other shared third-party infrastructure.
- Public DoH/DoT, generic STUN/TURN, and broad UDP port captures.
- Process-wide routing for Cursor, Grok, Claude, ChatGPT, and Antigravity.

## Acceptance rule for new domains

A new domain should be added only when all conditions hold:

1. It is used by model inference, response streaming, AI chat/session control, agent/tool execution, code completion, or repository indexing.
2. The evidence is an official document or a sanitized connection record tied to a reproducible feature.
3. The narrowest practical match can be expressed with `DOMAIN`, a constrained `DOMAIN-SUFFIX`, or a bounded `DOMAIN-REGEX`.
4. Negative tests prove that marketplace, update, download, media, advertising, analytics, and unrelated shared traffic remain outside `AI-家宽`.

## Authentication exit split

Shared login hosts remain on the original Profile by default. In particular, `auth.openai.com` and `accounts.google.com` are not added to the residential route, while core chat/model traffic uses the residential exit. Strict risk-control systems can therefore observe different login and model-traffic IPs and may request additional verification. This is an intentional narrow-scope trade-off, not a reason to add either shared authentication domain without evidence.

## Managed-rule ownership after v5.5

The script replaces rules that the current version can generate, including output from a switch that was later disabled. It no longer contains automatic migration lists for pre-v5.4 broad rules or retired v5.4 Cursor entries. Unknown rules targeting `AI-家宽` are treated as user-owned and preserved. If generated output was manually persisted in a subscription or Global Extend Config (Merge), remove stale entries there using the exact search list in [Troubleshooting](troubleshooting.md), then refresh the Profile.
