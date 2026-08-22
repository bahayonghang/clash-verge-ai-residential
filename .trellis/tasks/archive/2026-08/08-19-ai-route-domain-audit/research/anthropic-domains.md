# Anthropic / Claude 官方域名调研

调研日期：2026-08-19。检索工具：Exa（`web_search_exa` / `web_fetch_exa`）。

## 来源

- https://code.claude.com/docs/en/network-config （Enterprise network configuration，含 Network access requirements 表）
- https://code.claude.com/docs/en/corporate-proxy （同一页面的别名路径）
- https://code.claude.com/docs/en/self-hosted-environments-deploy （Network requirements）
- https://code.claude.com/docs/en/cloud-environments （Default allowed domains）
- https://platform.claude.com/docs/en/api/ip-addresses （IP addresses）

## 官方 Network access requirements 表（原文摘录）

> Claude Code requires access to the following URLs. Allowlist these in your proxy configuration
> and firewall rules, especially in containerized or restricted network environments.

| URL                   | Required for（官方原文）                                                                                                                                                               |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `api.anthropic.com`   | Claude API requests, including the WebFetch domain safety check, feature flag fetches, and telemetry event logging                                                                     |
| `claude.ai`           | claude.ai account authentication                                                                                                                                                       |
| `claude.com`          | claude.ai account sign-in opens a `claude.com` page in the browser, which redirects to `claude.ai`; pre-approved WebFetch documentation lookups also reach this host from the CLI      |
| `platform.claude.com` | Anthropic Console account authentication. OAuth token exchange, refresh, and revocation also go to this host for claude.ai accounts, so both Console and claude.ai sign-ins require it |

同表另有三行，2026-08-19 二次检索补齐（首次抓取时表格被截断，导致本文件早期版本
误判 `assets-proxy.anthropic.com` 与 `claudeusercontent.com` 无官方出处）：

| URL | Required for（官方原文） |
|---|---|
| （MCP connector 代理行） | claude.ai MCP connector 流量经该代理转发，可用 `ENABLE_CLAUDEAI_MCP_SERVERS=false` 或 `disableClaudeAiConnectors` 关闭 |
| `bridge.claudeusercontent.com` | [Claude in Chrome](/docs/en/chrome) extension WebSocket bridge |
| `*.frame.claudeusercontent.com` | [Artifact](/docs/en/artifacts) content reads. The CLI fetches an artifact's files from this host when Claude opens one... To disable the tool and drop this requirement, set `CLAUDE_CODE_DISABLE_ARTIFACT=1` or the `disableArtifact` setting |

同页表格下方的说明段落（关键，直接为 `assets-proxy.anthropic.com` 与
`claudeusercontent.com` 后缀提供依据）：

> The preceding table covers the standalone CLI. The Claude Desktop app and claude.ai in a
> browser load their application code and user content from additional Anthropic CDN hosts,
> including `assets-proxy.anthropic.com` and the other `*.claudeusercontent.com` origins that
> serve artifacts in those apps. Allowing `claude.ai` while blocking those hosts produces a
> blank page rather than an error.

来源：https://code.claude.com/docs/en/network-config

`*.frame.claudeusercontent.com` 在 cloud-environments 页被再次确认：

> If sessions in the environment work with artifacts, include `*.frame.claudeusercontent.com`
> in your list. Claude Code fetches artifact content from that host.

来源：https://code.claude.com/docs/en/cloud-environments

`bridge.claudeusercontent.com` 的运行时形态为 `wss://bridge.claudeusercontent.com/chrome/<session-id>`
（anthropics/claude-code issue #31206、#51844 的日志片段）。

## 官方对非推理主机的表述

self-hosted-environments-deploy 页原文：

> The runner doesn't reach `statsig.anthropic.com`, `*.sentry.io`, `claude.ai`, or
> `platform.claude.com`. These hosts appear in some older enterprise network checklists,
> but you don't need to allowlist them for runner or session traffic: feature-flag fetches
> go to `api.anthropic.com`.

同页把 `code.claude.com` 与 `claude.com` 的用途标注为：

> documentation lookups by the built-in claude-code-guide agent and pre-approved WebFetch
> requests during sessions.

来源：https://code.claude.com/docs/en/self-hosted-environments-deploy

结论：`statsig.anthropic.com`、`*.sentry.io` 属于特性开关与错误上报，官方明确说明不是必需。
脚本当前已把 `statsigapi.net`、`sentry.io` 归入 `CLAUDE_SHARED_SUFFIX_DOMAINS`，默认关闭，与官方一致。

## IP 段（关键结论）

https://platform.claude.com/docs/en/api/ip-addresses 原文：

> ## Inbound IP addresses
>
> These are the IP addresses where Anthropic services receive incoming connections.
>
> ### IPv4
>
> `160.79.104.0/23`
>
> ### IPv6
>
> `2607:6bc0::/48`
>
> ## Outbound IP addresses
>
> These are the stable IP addresses that Anthropic uses for outbound requests
> (for example, when making MCP tool calls to external servers).
>
> ### IPv4
>
> `160.79.104.0/21`

判定：脚本 `ANTHROPIC_INBOUND_IP_RULE_TEMPLATES` 使用 `160.79.104.0/23` 与 `2607:6bc0::/48`，
即 inbound 段，是客户端发起连接的目的地址，用作纯 IP 兜底方向正确。脚本注释中
「旧版 /21 是 Anthropic 发起外连时使用的 outbound 范围」与官方表述一致。**此项无需改动。**

官方另列出已停用段 `34.162.46.92/32`、`34.162.102.82/32`、`34.162.136.91/32`、
`34.162.142.92/32`、`34.162.183.95/32`，脚本未包含，无需清理。

## 逐条判定（2026-08-19 二次检索后更正）

| 脚本条目 | 规则类型 | 官方出处 | 判定 |
|---|---|---|---|
| `claude.ai` | DOMAIN-SUFFIX | network-config 表 | 保留。官方列为账户认证必需。后缀额外覆盖 `downloads.claude.ai`（安装与自动更新），已记录为已知取舍 |
| `claude.com` | DOMAIN-SUFFIX | network-config 表 | 保留后缀，但**理由已更正**。此前写「只有后缀能覆盖 `platform.claude.com`」不成立：可以并列 `DOMAIN,claude.com` + `DOMAIN,platform.claude.com` + `DOMAIN,code.claude.com` + `DOMAIN,docs.claude.com`。保留后缀的实际理由是浏览器登录重定向链（官方原文：sign-in opens a `claude.com` page，redirects to `claude.ai`）跳数未知，精确枚举可能漏掉中间跳。收窄需脱敏 Connections 证据 |
| `api.anthropic.com` | DOMAIN | network-config 表 | 保留。模型推理主通道 |
| `mcp-proxy.anthropic.com` | DOMAIN | network-config 表 | 保留。claude.ai MCP connector 代理 |
| `assets-proxy.anthropic.com` | DOMAIN | network-config 表格下方说明段 | **保留（更正）**。官方原文：Desktop 与 claude.ai 从该 CDN 主机加载应用代码与用户内容，屏蔽后出现白屏而非报错 |
| `claudeusercontent.com` | DOMAIN-SUFFIX | network-config 表 + 说明段、cloud-environments | **保留（更正）**。官方列出 `bridge.claudeusercontent.com`（Claude in Chrome WebSocket 桥）与 `*.frame.claudeusercontent.com`（Artifact 内容读取），并明确提到「the other `*.claudeusercontent.com` origins」。后缀匹配有官方依据 |
| `a-api.anthropic.com` | DOMAIN | Desktop 精简放行清单 + 3P telemetry 表 | 退出激活。官方列出，用途是 3P Desktop「Analytics events」，不是 Messages API。标准 Desktop 精简清单也有该主机，未写用途 |
| `clau.de` | DOMAIN-SUFFIX | 无官方出处 | 退出激活清单。仓库初始提交引入，无依据记录 |
| `claudemcpclient.com` | DOMAIN-SUFFIX | 无官方出处 | 退出激活清单。仓库初始提交引入，无依据记录 |
| `claudemcpcontent.com` | DOMAIN-SUFFIX | Desktop 通配 `*.claudemcpcontent.com` | 保留。官方 MCP Apps widget 隔离域；用户已确认 |
| `160.79.104.0/23` + `2607:6bc0::/48` | IP-CIDR / IP-CIDR6 | ip-addresses 页 | 保留。官方 inbound 段，方向正确 |

## 更正记录

本文件第一版把 `assets-proxy.anthropic.com` 与 `claudeusercontent.com` 判为「无官方出处」。
原因是首次用 `web_fetch_exa` 抓取 network-config 页时，返回的 markdown 未包含
Network access requirements 表格与其下方说明段。二次检索用 `web_search_exa` 的
highlights 取回了该段原文。两条判定已更正为「保留」。

教训：该页的表格在 markdown 转换中会被丢弃，只能通过搜索 highlights 或直接查看
渲染页面确认。后续复核该页时不要只依赖单次 fetch 结果。


## 过度覆盖实测

以 `ruleMatchesHost` 同语义探针测试，当前规则把下列主机纳入 `AI-家宽`：

`www.claude.com`、`docs.claude.com`、`code.claude.com`、`status.claude.com`、
`support.claude.com`、`console.claude.com`、`blog.claude.com`、`downloads.claude.ai`、
`www.claude.ai`、`status.claude.ai`、`cdn.claude.ai`。

其中 `code.claude.com` 与 `docs.claude.com` 是 Claude Code CLI 的文档抓取目标
（官方标注为 pre-approved WebFetch / claude-code-guide 文档查询），属于本机 CLI 连接官方端点，
在用户设定的范围内。`status.claude.com`、`blog.claude.com`、`www.claude.com`
是状态页与营销页，不属于推理、会话或认证。

`statsig.anthropic.com` 未被当前规则匹配，与官方「非必需」表述一致。

## 更正记录（2026-08-19 第三次检索）

完整主机表见 `claude-backend-endpoints.md`。

1. **`claudemcpcontent.com` 已有官方 Desktop 通配。**
   https://code.claude.com/docs/en/desktop#network-access-requirements 列出
   `*.claudemcpcontent.com`。3P telemetry 页用途为 MCP Apps widget。
   Claude Science 文档再次列出该通配。Claude Code CLI 表未列。
   用户已确认保留后缀。

2. **`a-api.anthropic.com` 已有官方列出，用途是分析事件。**
   Desktop 精简放行清单包含该主机。
   https://claude.com/docs/third-party/claude-desktop/telemetry 标为「Analytics events」。
   不是 Messages API。退出激活的依据改为「官方遥测主机」。
   若 `ENABLE_ANTHROPIC_IP_FALLBACK` 保持开启，该主机仍可能被 inbound CIDR 兜进家宽。

3. **Claude Code 推理主机没有变。**
   模型请求走 `api.anthropic.com`（同主机也承载 feature flag 与 telemetry）。
   登录走 `claude.ai` / `claude.com` / `platform.claude.com`。
   `code.claude.com` 官方标注为文档查找，「Blocking this host only affects documentation lookups」。

4. **Desktop 精简清单还列出 `a.claude.ai`、`assets.claude.ai`、`downloads.claude.ai`、
   `*.livepreview.claude.ai`。** 现行 `DOMAIN-SUFFIX,claude.ai` 已覆盖这些子域。
