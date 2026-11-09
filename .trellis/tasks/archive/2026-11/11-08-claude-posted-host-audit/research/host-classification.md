# Claude Code 帖中主机分类

日期：2026-11-08。只读核对，未改产品代码。

## 帖子在要求什么

截图里的「处理这几个 IP」来自买号账号对「封号 / 内鬼」的说法。名单全是主机名，没有 CIDR。

本仓库的 Claude 配置做的是另一件事：把核心产品、API、MCP 和会话内容送进 `AI-家宽`。它不生成用来藏遥测的 `REJECT` 规则。`docs/routing-scope.md` 把客服、遥测、功能开关、风控和其他共享第三方基础设施留在原 Profile。

## 逐条

### `api.anthropic.com`

- 清单：`clash-verge-ai-residential.js` `ANTHROPIC_CORE_EXACT_DOMAINS`。
- 开关：`ROUTE_ANTHROPIC_CORE`，公开默认 true。
- 官方网络配置（code.claude.com/docs/en/network-config，检索日期 2026-11-08，页面时间 2026-11-06）：Claude API、WebFetch 域名安全检查、功能开关，以及遥测事件日志都在这台主机。
- 动作：保持精确规则。不能为了「去掉内鬼」把它移出核心。

### `platform.claude.com`

- 命中 `ANTHROPIC_CORE_SUFFIX_DOMAINS` 的 `claude.com`。
- 回归：`tests/regression.test.js` 的 v5.10 正向主机表包含它。
- 官方用途：Console 账户登录；claude.ai 的 OAuth token 交换、刷新、撤销也走这里。首次运行连不上 `api.anthropic.com` 或 `platform.claude.com` 时，连接检查会指向网络配置页。
- 动作：不另加精确规则。后缀已经覆盖，重复规则只会增加清理面。

### `bridge.claudeusercontent.com`

- 命中 `claudeusercontent.com` 后缀。回归正向表包含它。
- 官方用途：Claude in Chrome 扩展的 WebSocket bridge。日文网络配置写明：组织开了 IP 允许列表时，这台主机必须和 `claude.ai`、`api.anthropic.com` 走同一代理出口。
- 动作：保持后缀。当前三条已经同属 `AI-家宽`。

### `downloads.claude.ai`

- 命中 `claude.ai` 后缀。
- 文档取舍：`docs/routing-scope.md` 写明它是安装包和自动更新主机，因顶级产品域后缀而走住宅链路。拆出需要带动态上游名的规则，精确字符串清理模型删不干净。
- 官方用途：插件可执行文件、本机安装器、自动更新、版本检查。另有 `storage.googleapis.com` 与 `registry.npmjs.org` 承担旧安装包和 npm，那两条在 `CLAUDE_CODE_AUXILIARY_EXACT_DOMAINS`，默认 `routing.claude_code_auxiliary = false`。
- 动作：不新增精确规则，也不在本任务里拆后缀。

### `http-intake.logs.us5.datadoghq.com`

- 位于 `CLAUDE_SHARED_EXACT_DOMAINS`，旁边还有 `browser-intake-us5-datadoghq.com`。
- 注入条件：`ROUTE_CLAUDE_SHARED_DEPENDENCIES`。公开脚本默认 false。本机 `clash-verge-ai-residential.local.toml` 的 `claude_shared_dependencies` 也是 false。
- 官方必放行表的检索片段没有这台主机。帖子称它接收设备/会话/账户/组织标识；这与「遥测默认不进家宽」一致。
- 打开该开关的效果是遥测改走家宽，不是停止上报。官方另把遥测事件日志放在 `api.anthropic.com`。
- 动作：默认清单不动。不新增 REJECT。不代改本地 TOML。

### `clau.de`

- `RETIRED_CORE_SUFFIX_DOMAINS`，注释写明 v5.10 因无官方出处退出，只留给 `allPossible*` 清理旧规则。
- 回归：默认规则不匹配 `clau.de`；nameserver-policy 不含 `clau.de` 与 `+.clau.de`。
- 帖子只说「不止这几个还有」。没有官方页面，没有脱敏连接证明它承载会话或推理。
- 动作：继续退役。

## 截图没写、但官方同页有的主机

| 主机 | 归属 |
|---|---|
| `sso.claude.com` | `claude.com` 后缀。SSO 登录页。 |
| `*.frame.claudeusercontent.com` | `claudeusercontent.com` 后缀。Artifact 读取。回归用 `x.frame.claudeusercontent.com` 做正向样本。 |
| `mcp-proxy.anthropic.com`、`assets-proxy.anthropic.com` | 已是核心精确主机。 |
| `storage.googleapis.com`、`registry.npmjs.org` | 辅助清单，默认关闭。 |

## 入站 IP，不是这张截图的内容

脚本模板：

- `IP-CIDR,160.79.104.0/23,{GROUP},no-resolve`
- `IP-CIDR6,2607:6bc0::/48,{GROUP},no-resolve`

只在 `anthropic_core` 与 `anthropic_ip_fallback` 同时开启时注入。本机两个开关都是 true。

检索冲突，2026-11-08：

- 英文、俄文 platform.claude.com IP 页片段：入站另有 `153.61.224.0/22`；出站为 `160.79.104.0/21` 与 `153.61.208.0/20`。
- 中文、法文同路径，以及 docs.claude.com：入站仍只有 `160.79.104.0/23` 与 `2607:6bc0::/48`。中文出站仍是 `160.79.104.0/21`，并把旧的五条 `/32` 标为停用。
- `code.claude.com` 在本环境解析到 Fake-IP `198.18.4.196`，页面正文没能直接抓到。

`153.61.224.0/22` 是 `153.61.224.0–153.61.227.255`。出站 `153.61.208.0/20` 是 `153.61.208.0–153.61.223.255`。两者相邻、不重叠。出站段是 Anthropic 访问外部服务器时的源地址，不能写成客户端目的 CIDR。

## 2026-11-08 X 帖复核

原帖 [2118337998437888476](https://x.com/MinLiBuilds/status/2118337998437888476) 的图是四台主机：`api.anthropic.com`、`http-intake.logs.us5.datadoghq.com`、`bridge.claudeusercontent.com`、`downloads.claude.ai`。同线程回复 [2118480432912371719](https://x.com/MinLiBuilds/status/2118480432912371719) 和 48 小时帖 [2119041020918902856](https://x.com/MinLiBuilds/status/2119041020918902856) 的图补上第五台 `platform.claude.com`。流量最大的是 `api.anthropic.com`，Datadog 只有几 MiB。

@qwerty__cs 提到的 `clau.de` 没有出现在这两次进程监测里。

@IzumiHakumei 2026-11-08 把三段写在一起：`160.79.104.0/21`、`153.61.208.0/20`、`153.61.224.0/22`。前两段是官方出站源地址，后一段是英文 IP 页的入站目的地址。规则只补入站 `/22`。

@CedarWithCode 建议用 Clash `REJECT` 丢掉 Datadog。那是遥测拦截，不进入本仓库。@Darkfibr3 对 2.1.317 的拆包列出 GrowthBook、Datadog、Sentry、OTLP、BigQuery、Perfetto、Intercom；这些仍归共享依赖或明确排除，默认不走家宽。官方网络文档写明遥测事件日志也在 `api.anthropic.com`。

## 采纳的规则变更

`ANTHROPIC_INBOUND_IP_RULE_TEMPLATES` 增加 `IP-CIDR,153.61.224.0/22,{GROUP},no-resolve`。负向测试确认默认输出不含 `160.79.104.0/21` 与 `153.61.208.0/20`。域名清单不改。
