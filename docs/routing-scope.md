# 路由范围

住宅链路只给核心 AI 产品流量。不能因为某个 AI 网页加载了某个域名，就把该域名纳入家宽。

## 载体口径

家宽只覆盖三类流量：网页 Chat 产品会话、本机 CLI 连接官方端点、桌面/IDE 客户端连接官方 AI 端点。其余流量留在原 Profile 的机场出口。

- **载体 A（浏览器内网页 Chat）**：`claude.ai`、`claude.com`、`chatgpt.com`、`grok.com`、`gemini.google.com`、`aistudio.google.com`。同一浏览器会话会并发访问多个子域并携带同一套 Cookie。把其中一部分子域改走机场，会让服务端在同一个已登录会话内观察到两个出口 IP。产品顶级域默认保持后缀匹配。
- **载体 B（本机 CLI 与桌面/IDE 客户端）**：Claude Code、Codex、Grok CLI、Cursor、Antigravity。非推理请求（文档站、更新下载、扩展市场、静态资源）不携带推理会话凭据，按官方文档收窄为精确主机。

## 纳入类别

| 产品 | 纳入的流量 |
|---|---|
| Claude / Anthropic | Claude 产品域名、Messages API、`mcp-proxy.anthropic.com` MCP 连接器代理、`assets-proxy.anthropic.com` 资源代理、`claudemcpcontent.com` MCP Apps widget 隔离域、`claudeusercontent.com` 会话内容，以及官方入站 IP 回退 |
| ChatGPT / OpenAI | ChatGPT 产品域名（整域后缀，含 `ws.chatgpt.com`）、五个官方 exact 主机（`chat.openai.com`、`android.chat.openai.com`、`desktop.chat.openai.com`、`ios.chat.openai.com`、`tcr9i.chat.openai.com`）、OpenAI 模型 API 后缀 `api.openai.com`（覆盖 Codex 官方的 `us.` / `eu.` 数据驻留前缀），以及上传或生成的用户内容。可选的 `routing.openai_auth` 只增加 `auth.openai.com` 有界后缀与 `auth0.openai.com` 精确主机；`routing.openai_web_assets` 独立增加 `oaistatic.com` 后缀；二者默认关闭 |
| Gemini | Gemini Web、Google AI Studio 产品 RPC/streaming 主机、Gemini Developer API |
| Vertex AI / Agent Platform | `routing.vertex_ai_endpoints` 默认 `true`，一次控制 `aiplatform.googleapis.com`、`aiplatform.us.rep.googleapis.com`、`aiplatform.eu.rep.googleapis.com` 与区域正则 `^[a-z0-9-]+-aiplatform\.googleapis\.com$` |
| Google Antigravity / Gemini Code Assist | 精确主机 `antigravity.google`、生产 Code Assist 主机 `cloudcode-pa.googleapis.com`，以及 Antigravity `language_server` 的 `--cloud_code_endpoint` 主机 `daily-cloudcode-pa.googleapis.com` |
| Cursor | Chat/API、Tab、Agent、Cloud Agent/Bugbot API、authorize 端点、SSO 管理门户 `adminportal42.cursor.sh`、Cloud Agent VM 主机和产品专属认证；`routing.cursor_core` 默认 `true`。仓库索引主机 `repo[0-9]+.cursor.sh` 由独立开关 `routing.cursor_repository_indexing` 控制，默认 `false`，回落原 Profile / 机场上游 |
| Grok Build | `routing.grok_core` 默认 `true`。默认注入 `DOMAIN-SUFFIX,grok.com`（覆盖 `cli-chat-proxy.grok.com` 推理 API 与 `code.grok.com` 会话同步）、`auth.x.ai` OAuth 主机、`DOMAIN-SUFFIX,api.x.ai`（覆盖区域端点与 `mtls.api.x.ai`）。`routing.grok_web_assets = false` 时只把 `grok.com` 后缀换成三条精确主机：`grok.com`、`cli-chat-proxy.grok.com`、`code.grok.com`；`api.x.ai` 后缀仍注入 |

官方来源：

- Claude Code / Desktop 网络配置：https://code.claude.com/docs/en/network-config.md
- Claude Desktop MCP Apps widget 通配 `*.claudemcpcontent.com`
- OpenAI 企业防火墙清单：https://help.openai.com/en/articles/9247338-network-recommendations-for-chatgpt-errors-on-web-and-apps
- Codex 数据驻留前缀：`us.api.openai.com` / `eu.api.openai.com`
- Cursor 企业网络配置：https://cursor.com/docs/enterprise/network-configuration
- xAI 企业部署：https://docs.x.ai/build/enterprise ；区域端点：https://docs.x.ai/developers/regions ；mTLS：https://docs.x.ai/developers/advanced-api-usage/mtls
- Vertex AI 端点与 Antigravity Enterprise：https://antigravity.google/docs/enterprise
- Anthropic 入站网段：https://platform.claude.com/docs/en/api/ip-addresses.md

Cursor 依据：官方企业网络配置文档列出精确主机 `authenticate.cursor.sh`、`adminportal42.cursor.sh` 和 `*.cursorvm.com` 虚拟机主机，以及此前已覆盖的 API、Tab 和 Agent 端点。`api2.cursor.sh` 与 `authenticate.cursor.sh` 从 v5.10 起改为 `DOMAIN` 精确匹配。官方网络文档与本机 2026-08-17 Cursor 索引日志共同确认 `repo42.cursor.sh` 为仓库索引主机；`repo[0-9]+.cursor.sh` 是本项目的前向兼容策略，不是 Cursor 官方通配合同。默认 `routing.cursor_repository_indexing = false` 只让这些索引专属主机回落原 Profile，不阻止 Chat/Agent 发送代码上下文，也不能在 `disableHttp2` 或服务端强制 HTTP/1.1 把 RepositoryService 放到共享 `api2.cursor.sh` 时继续隔离索引。`api2.cursor.sh` 仍由 `routing.cursor_core` 控制。Privacy Mode 不会停止索引上传。Grok 依据：docs.x.ai/build/enterprise 把 `cli-chat-proxy.grok.com` 与 `auth.x.ai` 列为必需，把 `code.grok.com` 列为可选会话通道，把 `assets.grok.com` 标注为无功能影响。v5.7 依据：Claude Code 官方网络配置文档列出了 `mcp-proxy.anthropic.com` 和 `assets-proxy.anthropic.com`。

已知取舍：`downloads.claude.ai`（安装程序和自动更新主机）位于 `claude.ai` 后缀下，因此也会经过住宅链路。若将其拆出，需要注入包含动态解析上游名称的规则，而当前基于精确字符串的托管规则清理模型无法安全清理此类规则。更新下载频率较低，剩余影响仅为占用住宅链路带宽。`chatgpt.com` 后缀同样覆盖 `help.`、`status.`、`ab.`、`events.` 等子域。

## 明确排除

下列类别留在原 Profile 路由：

- Cursor Marketplace、扩展安装、应用下载、CDN、更新、Remote-SSH/WSL 服务端资产、网站、文档和论坛。
- Grok Build 第三方分析（`api.mixpanel.com`）、`x.ai` 安装脚本/隐私端点，以及代码库上传用的共享 `storage.googleapis.com`。
- YouTube、Maps、Google Search、Google Fonts、Gstatic、广告、统计和其他通用 Google 服务。
- OpenAI/Claude 的客服、遥测、功能开关、风控、支付和其他共享第三方基础设施。
- OpenAI 第一方登录主机与 `oaistatic.com` 网页资源默认也留在原 Profile；分别显式开启 `routing.openai_auth`、`routing.openai_web_assets` 后才进入家宽，且不会因此开启共享第三方依赖。
- 公共 DoH/DoT、通用 STUN/TURN，以及宽泛 UDP 端口捕获。
- Cursor、Grok、Claude、ChatGPT、Antigravity 的进程级全量路由。

v5.10 起不再注入、但仍保留在 `allPossible*` 中供升级清理的主机：

- `clau.de`、`claudemcpclient.com`（无官方出处）
- `a-api.anthropic.com`（官方 Desktop 清单用途为 Analytics events / 遥测。开启 `routing.anthropic_ip_fallback` 时，若该主机解析到 inbound CIDR，仍可能被 IP 规则命中）
- `geminicloudassist.googleapis.com`（Cloud Assist MCP，不是 Antigravity Agent 网关）

收窄后不再匹配的主机：

- `assets.grok.com`（仅在 `routing.grok_web_assets = false` 时排除；官方标注无功能影响）
- `docs.antigravity.google`、`download.antigravity.google`、`www.antigravity.google`
- `adminportal<N≠42>.cursor.sh`（例如 `adminportal0.cursor.sh`、`adminportal999.cursor.sh`）
- `www.api2.cursor.sh`、`feature.api2.cursor.sh`

## UNVERIFIED

以下判断只有官方文档或单元级规则匹配证据，没有部署后运行时证据。加载新 Profile 后需用脱敏 Clash Connections 补证。

| 项 | 未验证的内容 |
|---|---|
| 4 条退出激活规则 | `clau.de`、`claudemcpclient.com`、`a-api.anthropic.com`、`geminicloudassist.googleapis.com` 是否有任一条实际承载会话或推理流量。`daily-cloudcode-pa.googleapis.com` 已在 v5.10.1 用本机进程参数与 Clash Connections 证实，并恢复激活 |
| 三条 `alkali*` | AI Studio 网页是否实际请求这些 `clients6.google.com` 主机 |
| `antigravity.google` 收窄 | Antigravity 客户端是否访问 `docs.` / `download.` 子域，以及断开后是否影响启动或更新提示 |
| `adminportal42.cursor.sh` 收窄 | 企业 SSO 配置流程是否只使用编号 42 的主机 |
| `api.cursor.com` | Cursor 桌面客户端「Cloud」入口走该主机还是 `api2.cursor.sh`；官方只证明 API key 编程调用 |
| `grok_web_assets = false` | Grok 网页版在该模式下是否出现认证或资源加载的出口分裂 |
| `claude.com` 精确枚举 | 完整登录重定向链命中的 `*.claude.com` 主机集合 |
| `tcr9i.chat.openai.com` | 官方清单成员，用途未公开 |
| OpenAI 认证与网页资源开关 | Node 回归只证明 `auth.openai.com`、`auth0.openai.com` 与 `oaistatic.com` 的规则和 DNS 开关合同；实际登录重定向、Cloudflare/SSO/支持依赖、Ubuntu Clash 宿主执行及是否保持单一出口仍需脱敏 Connections 补证 |

## 新域名准入

同时满足以下条件才新增域名：

1. 用于模型推理、响应流、AI 聊天/会话控制、agent/工具执行、代码补全或仓库索引。
2. 证据是官方文档，或可复现功能对应的脱敏连接记录。
3. 能用 `DOMAIN`、有约束的 `DOMAIN-SUFFIX` 或有界 `DOMAIN-REGEX` 表达最窄可行匹配。
4. 负向测试证明市场、更新、下载、媒体、广告、统计和无关共享流量仍不进入 `AI-家宽`。

## 认证出口分裂

认证流量默认仍留在原 Profile。OpenAI 在 `routing.openai_auth = false` 时，`auth.openai.com`、其子域（如 `setup.auth.openai.com`）以及精确主机 `auth0.openai.com` 不进家宽，ChatGPT 核心会话和模型流量继续走家宽；`routing.openai_web_assets = false` 时，`oaistatic.com` 也留在原 Profile。这种默认分裂是有意的 AI-only 边界。

需要降低 OpenAI 第一方认证与核心流量的出口分裂时，可显式设置 `routing.openai_auth = true`。该开关只增加 `DOMAIN-SUFFIX,auth.openai.com` 与 `DOMAIN,auth0.openai.com`，不添加 `DOMAIN-SUFFIX,openai.com`，也不会联动 `routing.openai_web_assets` 或 `routing.openai_shared_dependencies`。因此 WorkOS、Intercom、Stripe、Cloudflare Challenge、Sentry、Datadog 等第三方跳转或依赖仍可能使用机场出口；开启该开关不等于证明整条登录链同出口，也不能保证减少平台验证。`oaistatic.com` 如确需同出口，应另行开启 `routing.openai_web_assets`。

Google 仍采用独立且范围更广的 `routing.antigravity_google_auth`。默认关闭时 `accounts.google.com` 等共享 Google 登录入口留在机场；开启会影响使用同一账号体系的其他 Google 产品，不因 OpenAI 开关而改变。

## v5.5 之后的托管规则归属

脚本会替换当前版本能生成的规则，包括后来被关掉的开关曾经产出的规则。它不再包含 pre-v5.4 宽规则或已退役 v5.4 Cursor 条目的自动迁移列表。指向 `AI-家宽` 的未知规则视为用户所有并保留。若生成结果曾被手工写进订阅或 Global Extend Config (Merge)，用 [故障排查](troubleshooting.md) 里的精确搜索列表删掉残留，再刷新 Profile。
