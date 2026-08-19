# OpenAI / ChatGPT / Codex 官方域名调研

调研日期：2026-08-19。检索工具：Exa（`web_search_exa`）。

## 来源

- https://help.openai.com/en/articles/9247338-network-recommendations-for-chatgpt-errors-on-web-and-apps
- https://help.openai.com/en/articles/12111596-ip-allowlisting-for-chatgpt
- https://help.openai.com/en/articles/10489721-login-and-authentication-faq-s-and-troubleshooting-sso-scim-and-domain-verification

## 官方放行清单（help ���章 9247338 原文）

> OpenAI uses the following domains. Please make sure that they are not blocked on your
> company network, and that any web/URL filtering is not responding with unexpected content:

```
*.auth.openai.com
*.chatgpt.com
*.ct.sendgrid.net
*.intercom.io
*.intercomcdn.com
*.oaistatic.com
*.oaiusercontent.com
*.openai.com
*.oaistatsig.com
android.chat.openai.com
auth0.openai.com
cdn.openaimerge.com
cdn.workos.com
challenges.cloudflare.com
chat.openai.com
desktop.chat.openai.com
forwarder.workos.com
humb.apple.com
images.workoscdn.com
ios.chat.openai.com
js.intercomcdn.com
js.stripe.com
o207216.ingest.sentry.io
o33249.ingest.sentry.io
rum.browser-intake-datadoghq.com
setup.auth.openai.com
setup.workos.com
tcr9i.chat.openai.com
workos.imgix.net
```

分类标注：

| 类别                    | 主机                                                                                                                                                                                                                                                                                      |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| OpenAI 第一方产品与推理 | `*.chatgpt.com`、`*.openai.com`、`chat.openai.com` 家族五个主机、`*.oaiusercontent.com`                                                                                                                                                                                                   |
| 第一方静态 CDN          | `*.oaistatic.com`、`cdn.openaimerge.com`                                                                                                                                                                                                                                                  |
| 第一方特性开关          | `*.oaistatsig.com`                                                                                                                                                                                                                                                                        |
| 认证                    | `*.auth.openai.com`、`setup.auth.openai.com`、`auth0.openai.com`                                                                                                                                                                                                                          |
| 共享第三方              | `*.ct.sendgrid.net`（邮件跟踪）、`*.intercom.io` / `*.intercomcdn.com` / `js.intercomcdn.com`（客服）、WorkOS 五项（SSO）、`challenges.cloudflare.com`（风控）、`js.stripe.com`（支付）、`humb.apple.com`、两个 Sentry ingest（错误上报）、`rum.browser-intake-datadoghq.com`（RUM 监控） |

`tcr9i.chat.openai.com` 在官方表中无用途说明。该主机名与 Arkose Labs 风控（tcr9i）
的命名一致，但官方文档未确认，标注为**无官方用途说明**。

## WebSocket 与 Codex（官方原文）

> Some ChatGPT and Codex features use secure WebSocket connections in addition to standard
> HTTPS requests. ... If your proxy or firewall cannot allowlist WebSocket traffic by URL
> path, allow WebSocket upgrades to chatgpt.com over TCP port 443 for Codex traffic.

结论：Codex 的会话通道走 `chatgpt.com` 的 WebSocket。脚本的 `DOMAIN-SUFFIX,chatgpt.com`
覆盖该路径。Clash 的域名规则不区分 HTTP 与 WebSocket，无需额外条目。

## ChatGPT Voice（官方原文）

> ChatGPT Voice connects to OpenAI servers over UDP port 3478. The current server IP address
> ranges are listed in chatgpt-voice.json. ... If UDP access is not allowed, TCP port 443 can
> be used instead, although UDP is preferred.

结论：脚本刻意不注入 UDP 3478 与 Voice IP 段。语音会退回 TCP 443，功能不中断，
但语音媒体流不经家宽。这是当前行为，与 `ROUTE_GLOBAL_REALTIME_PORTS = false` 一致。

## 数据驻留前缀

`us.api.openai.com` / `eu.api.openai.com` 属于 `*.openai.com` 官方通配范围。脚本用
`DOMAIN-SUFFIX,api.openai.com` 覆盖，探针确认 `us.api.openai.com`、`eu.api.openai.com` 命中。

## 逐条判定

| 脚本条目                                                 | 规则类型             | 官方出处                    | 判定                                                                                                                    |
| -------------------------------------------------------- | -------------------- | --------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `chatgpt.com`                                            | DOMAIN-SUFFIX        | 官方 `*.chatgpt.com`        | 保留。与官方通配一致。ChatGPT Web、桌面端与 Codex WebSocket 均在此域下                                                  |
| `api.openai.com`                                         | DOMAIN-SUFFIX        | 官方 `*.openai.com`         | 保留。覆盖 `us.` / `eu.` 数据驻留前缀                                                                                   |
| `oaiusercontent.com`                                     | DOMAIN-SUFFIX        | 官方 `*.oaiusercontent.com` | 保留。用户上传与生成内容                                                                                                |
| `chat.openai.com` 等五个 exact                           | DOMAIN               | 官方逐条列出                | 保留                                                                                                                    |
| `tcr9i.chat.openai.com`                                  | DOMAIN               | 官方列出，无用途说明        | 保留。官方清单成员，但用途未公开                                                                                        |
| `oaistatic.com`                                          | 未注入               | 官方列为静态 CDN            | 正确排除                                                                                                                |
| `oaistatsig.com`                                         | 未注入               | 官方列为特性开关            | 正确排除                                                                                                                |
| `auth.openai.com`                                        | 未注入               | 官方列为必需                | 有意排除。`docs/routing-scope.md` 的 Authentication exit split 已记录：登录 IP 与模型流量 IP 不同，风控可能要求额外验证 |
| WorkOS / Intercom / Sentry / Stripe / SendGrid / Datadog | 未注入（开关默认关） | 官方列为需放行              | 正确排除。均为共享第三方，非推理                                                                                        |

## 缺失项

| 主机                                                             | 官方用途                 | 缺失影响                                                                      |
| ---------------------------------------------------------------- | ------------------------ | ----------------------------------------------------------------------------- |
| `auth0.openai.com`、`setup.auth.openai.com`、`*.auth.openai.com` | 认证                     | 登录流量走机场出口。与已记录的 Authentication exit split 取舍一致，不是新缺陷 |
| `cdn.openaimerge.com`                                            | 官方清单成员，用途未说明 | 未知。当前未注入，也未在负向测试中断言                                        |

## 过度覆盖实测

`DOMAIN-SUFFIX,chatgpt.com` 把 `chatgpt.com` 下全部子域纳入家宽，包含
`help.`、`status.`、`ab.`、`events.`、`browser-intake.` 等前缀形式的主机。
OpenAI 官方本身以 `*.chatgpt.com` 通配放行该域，因此脚本与官方口径一致；
代价是该域下的遥测与状态页也占用家宽链路。
