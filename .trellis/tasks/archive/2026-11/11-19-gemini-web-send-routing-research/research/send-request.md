# Gemini 网页版发送纯文本时的请求

- 核查日期为 2026-11-19。范围是消费者网页版、公开的一手观察、Chrome 网络诊断，以及仓库中对应的域名规则。

## 能定位到哪一个请求

带 Domain 列的故障截图已确认 `processSession` 的主机是 `geminiweb-pa.clients6.google.com`。会话主机与网页入口在本机配置中的路由，以及尚需验证的实际策略链，见[本机调研](routing-findings.md)。

Google 帮助将 `gemini.google.com` 列为网页版入口。其公开的防火墙说明列出应用使用的主机，未说明每次点击发送时使用的路径或依赖项。[Gemini 使用说明](https://support.google.com/gemini/answer/13275745?co=GENIE.Platform%3DDesktop&hl=en) · [Google Workspace 防火墙说明](https://knowledge.workspace.google.com/admin/generative-ai/gemini-app/gemini-app-firewall-settings)

本次读取的公开前端脚本同时含有 `processSession` 和 `StreamGenerate`。因而只用后一个名称过滤 Network 会漏掉截图中已经出现的请求。[Gemini 网页](https://gemini.google.com/app)

仓库在 `clash-verge-ai-residential.js:280` 收录 `gemini.google.com`。截图中的会话主机位于 `clients6.google.com` 下，因此入口后缀规则无法覆盖。规则能否命中还取决于浏览器的连接进入对应核心、采用规则模式和实际规则顺序。

## 一次发送怎样判读

在点击发送前打开开发者工具的 Network，勾选 Domain 列，清空已有行，发送一条纯文本。按点击时间找 `processSession` 或其他 Fetch/XHR。如果看不到，清除筛选，回到 All 检查同一时刻的其他请求或前置失败。记下域名、请求简称、状态或错误名、完成情况和同一时刻的 FlClash 策略链。[Chrome Network 面板说明](https://developer.chrome.com/docs/devtools/network/reference/)

| 观察 | 可以判断的范围 |
| --- | --- |
| 主请求报 `ERR_BLOCKED_BY_CLIENT` | Chrome 报告客户端阻止了这条请求。可在使用相同出口的干净配置中复测，区分扩展或浏览器设置。 |
| 主请求超时、连接重置或 DNS 失败 | 结合 FlClash 同时段、相同域名的连接判断在哪条链路上失败。单个网络错误无法区分走错出口和当前出口故障。 |
| 主请求返回 `403`、`429` 等 | 已有 HTTP 响应。具体原因还要看服务返回的错误类别，状态码本身无法识别出口。 |
| 主请求 `200`，网页仍报错 | 检查请求是否完整结束、响应是否明确返回业务错误，以及另一条同时发生的请求是否失败。流式请求也可能在收到响应头后中断。 |
| 只看到字体、头像或统计失败 | 先在同一次操作中找到发消息请求，再判读附带请求与对话的关系。 |

只分享以上字段即可。查询串、请求正文、响应原文和 HAR 都可能包含对话或会话数据。本次未获得两个浏览器的成对发送记录，无法把具体域名或某个浏览器设置定为故障原因。
