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

同页另有一行被检索片段截断，内容涉及 claude.ai MCP connector 代理（对应
`mcp-proxy.anthropic.com`），并说明连接器流量经该代理转发，可用
`ENABLE_CLAUDEAI_MCP_SERVERS=false` 或 `disableClaudeAiConnectors` 关闭。

来源：https://code.claude.com/docs/en/network-config

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

## 逐条判定

| 脚本条目                             | 规则类型           | 官方出处                   | 判定                                                                                                                                           |
| ------------------------------------ | ------------------ | -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `claude.ai`                          | DOMAIN-SUFFIX      | network-config 表          | 保留。官方列为账户认证必需。后缀额外覆盖 `downloads.claude.ai`（安装与自动更新），已记录为已知取舍                                             |
| `claude.com`                         | DOMAIN-SUFFIX      | network-config 表          | 保留后缀。官方仅列 `claude.com`，但 `platform.claude.com` 是 OAuth token 交换/刷新/吊销的必需主机，只有后缀匹配才覆盖。收窄为 exact 会破坏登录 |
| `api.anthropic.com`                  | DOMAIN             | network-config 表          | 保留。模型推理主通道                                                                                                                           |
| `mcp-proxy.anthropic.com`            | DOMAIN             | network-config 表          | 保留。claude.ai MCP connector 代理                                                                                                             |
| `assets-proxy.anthropic.com`         | DOMAIN             | 未在本次检索到的页面中出现 | 存疑。CHANGELOG v5.7 记录来源为官方网络配置文档，本次检索未复现该行；建议保留并标注待复核                                                      |
| `a-api.anthropic.com`                | DOMAIN             | 无官方出处                 | 存疑。本次检索未找到该主机的官方说明                                                                                                           |
| `clau.de`                            | DOMAIN-SUFFIX      | 无官方出处                 | 存疑。仓库初始提交引入，无依据记录。疑为短链域，非推理必需                                                                                     |
| `claudemcpclient.com`                | DOMAIN-SUFFIX      | 无官方出处                 | 存疑。仓库初始提交引入，无依据记录                                                                                                             |
| `claudemcpcontent.com`               | DOMAIN-SUFFIX      | 无官方出处                 | 存疑。仓库初始提交引入，无依据记录                                                                                                             |
| `claudeusercontent.com`              | DOMAIN-SUFFIX      | 无官方出处                 | 存疑。仓库初始提交引入，无依据记录                                                                                                             |
| `160.79.104.0/23` + `2607:6bc0::/48` | IP-CIDR / IP-CIDR6 | ip-addresses 页            | 保留。官方 inbound 段，方向正确                                                                                                                |

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
