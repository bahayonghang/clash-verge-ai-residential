# Gemini 网页能打开、发送时报错

2026-11-19（美东），用户将更新后的脚本同步到本机 FlClash，在普通浏览器发出消息并收到 Gemini 的完整文字回复，网页对话验收通过。此前失败的四条 `processSession` 请求均访问 **`geminiweb-pa.clients6.google.com`**。更新前的配置把该主机交给订阅原有选择组，更新后它与网页入口都匹配 `AI-家宽`。

## 发送按钮发出的请求

1. 用户报告，同一 Google 账号在指纹浏览器可以对话，普通浏览器发送纯文本后提示 `Something went wrong (1)`。带 Domain 列的截图可见四条 `processSession` 都返回 HTTP 200，其中一条 XHR 显示 30.26 秒、0.1 kB，另有一条结束会话的 ping。HTTP 状态和耗时无法区分长轮询正常结束、服务返回业务错误或流中断。两条红色请求均为 `play.google.com` 的 `log`，分别为 ping 和 fetch，状态为 `(blocked:other)`。图中没有二者阻断对话的因果证据。
2. 本机匿名读取 Google 的 `/app`，再读取页面从 `gemini.gstatic.com` 预加载的应用脚本。在这一版页面中，启动设置 `GK6dn` 给出的地址是 `https://geminiweb-pa.clients6.google.com`。应用脚本读取这个值作为服务地址，在其中一条会话分支拼接 `/v1/processSession`。这与故障浏览器显示的主机及请求名一致。[Google Gemini 网页](https://gemini.google.com/app)
3. 截图一条成功的 `batchexecute` 的发起方文件名与本机已安装的 Voyager 2.3.3 扩展资源一致。扩展声明匹配 Gemini 页面，该脚本也识别 `/v1/processSession`。现有证据只能说明扩展参与了页面请求处理。若需要排查扩展，可单独停用它并刷新，保持其他条件相同，再启用复测。

## 更新前的本机路由与连通性

运行的是 FlClash 0.9.3，核心监听本机代理端口。FlClash 生成的配置为规则模式，TUN 与自动路由开启，Windows 存在 FlClash 的 IPv4 默认路由。系统显式代理开关关闭。以下为配置中从上到下匹配的域名规则，序号从 1 开始。现场连接仍须通过 FlClash 的「连接」确认。

| 主机 | 生成配置里的域名规则 | 证据 |
| --- | --- | --- |
| `gemini.google.com` | 第 25 条，`DOMAIN-SUFFIX,gemini.google.com` → `AI-家宽` | 网页入口。家宽组仅包含住宅 SOCKS5。 |
| `geminiweb-pa.clients6.google.com` | 第 2356 条，`DOMAIN-KEYWORD,google` → 订阅原有选择组 | 故障截图实见的会话服务地址。前置的三条进程规则均未指定常见普通浏览器，该组的静态成员树中没有项目的家宽节点。 |
| `play.google.com` | 第 2356 条 → 订阅原有选择组 | 故障截图仅见两条日志请求被阻止。 |
| `gemini.gstatic.com` | 第 2560 条，`DOMAIN-SUFFIX,gstatic.com` → 订阅原有选择组 | 首页引用的脚本主机。匿名实测脚本可加载。 |
| `www.gstatic.com`、`ssl.gstatic.com`、`accounts.google.com` | 精确主机 → `AI-家宽` | 网页模块和账号相关；本机相关开关已开。 |
| `waa-pa.clients6.google.com` | 第 2356 条 → 订阅原有选择组 | 官方防火墙清单列出；本次首页含 DNS 预取提示。没有证据显示其在发送瞬间失败。 |

更新前的模板覆盖网页入口及三个 AI Studio 主机，开启 `gemini_web_core` 时也未包含此次会话主机。

本机通过系统默认网络路径和显式 FlClash 本机代理分别匿名读取网页、`gemini.gstatic.com` 与 `www.gstatic.com` 上页面实际引用的 JavaScript，两种方式都收到 HTTP 200。该测试仅覆盖 GET。ResiWatch 最近 24 小时的只读 `share` 返回 `coverage.status = uncovered`，没有可回放的失败连接。两项 Gemini 定向回归测试通过，覆盖模板的现有规则契约。

收到带域名的截图后再次核对本机。Windows 系统解析器为网页入口和会话主机分别返回了 FlClash 所用地址范围中的 fake IP，Windows 对这两个地址的选路均经过 FlClash 虚拟网卡。这支持系统接管路径正常。复查时未观察到发往这两个地址的现存 TCP 443 连接，FlClash 生成配置中的外部控制接口也未启用。这些观测无法还原截图对应的连接、UDP 流或最终出口。

## 调研阶段的对照方案

1. 故障浏览器已确认会话主机。下一次发送时，可在 FlClash「连接」按这个精确域名查看策略链。记录命中规则和是否含 `AI-家宽` 即可。
2. 对照规则为 `DOMAIN,geminiweb-pa.clients6.google.com,AI-家宽`，临时放在通用 Google 规则之前。沿用同一浏览器、账号、模型、机场节点和扩展，刷新并确保新连接实际命中家宽，再发送同一条纯文本。若能收到回复，移除规则并再次建立新连接复测。只有这一个变化可重复地改变发送结果，才足以把出口差异归因到本次故障。本轮只做只读调研。
3. 如果会话主机已命中家宽，或切换这一条线路没有影响，保持网络条件固定，做 Voyager 开关对照。两种情形都无需先扩大全部 Google 域名的家宽范围。Google 说明防火墙需要允许相关主机访问，没有要求它们使用同一个出口。其官方错误表也没有将单独的 `(1)` 定义为路由或地区错误。[Google 的 Gemini 防火墙清单](https://knowledge.workspace.google.com/admin/generative-ai/gemini-app/gemini-app-firewall-settings) · [Google 的错误代码说明](https://support.google.com/gemini/answer/18647528?hl=zh-Hans&co=GENIE.Platform%3DDesktop)

记录仅保留公开主机、请求简称与状态。未保存原始截图、URL 查询串、响应正文、账号或代理节点信息。

## 原始模板的后续验证

用户选择修改仓库的公开模板，自行同步到 FlClash。模板现在为会话主机生成精确路由和住宅 DNS 策略。在临时目录使用示例住宅配置和本机订阅中脱敏后的通用 Google 规则运行 `just sync`，加载生成物后确认精确规则出现一次，优先于原 Google 规则。定向测试还验证了相邻主机、子域排除以及开关关闭再开启时的清理和恢复。

另外通过配置的住宅 SOCKS5 分别向 Cloudflare（1.1.1.1）和 Google（8.8.8.8）查询该主机的 IPv4 地址，再通过同一 SOCKS5 分别连接解析结果。两个结果均以会话主机名通过证书验证，协商 TLS 1.3。

完整 `just ci` 重跑通过，根目录 270 项测试全部通过。第一次全量运行的 CRLF 渲染测试在 Windows 临时文件重命名时遇到 `EPERM`，该项随后单独通过。

## 同步后的普通浏览器验收

- 用户在本机 FlClash 应用新脚本后，用普通浏览器发送消息。用户确认对话恢复，截图同时显示已发送的消息和 Gemini 生成的回复。
- 同步后只读核对 FlClash 生成配置：在规则模式中，精确规则 `DOMAIN,geminiweb-pa.clients6.google.com,AI-家宽` 仅出现一次，位于第 52 条。原有通用 Google 规则位于第 2357 条。会话主机的住宅 DNS 策略使用裸主机名。
- 本次验收覆盖这台机器上的网页文字对话。没有采集成功请求的实际连接链，也没有撤销规则后复测，故仅记录应用修改后恢复的结果。
