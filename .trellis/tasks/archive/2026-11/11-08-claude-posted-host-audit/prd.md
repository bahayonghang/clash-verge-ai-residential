# 核对帖中 Claude 主机的家宽归属

## Goal

判断 2026-11-05 / 2026-11-06 帖子里点名的 Claude Code 主机，是否已经由公开模板送入 `AI-家宽`。已经覆盖的不重复加规则。按仓库准入不能进入默认家宽的，保持排除。本任务不提供拦截遥测、规避平台封号的规则。

## Background

来源是两张截图，不是裸 IP：

- 买号管家 @MaihaoGuanjia，2026-11-05：列出 `api.anthropic.com`、`http-intake.logs.us5.datadoghq.com`、`bridge.claudeusercontent.com`、`downloads.claude.ai`、`platform.claude.com`，并称「封号机制」「有内鬼」。
- 拓跋鼠 @qwerty__cs，2026-11-06：补充 `clau.de`。
- 被引用的 @MinLiBuilds 称连续 24 小时进程流量里 Claude Code 只连 4 个地址，其中一个是 Datadog 日志接收端，载荷涉及设备/会话标识、账户及组织 UUID、模型版本和系统环境。该观察没有附带可复核的脱敏 Connections。

公开模板的准入见 `docs/routing-scope.md`「新域名准入」：只收推理、会话、agent/工具或补全流量；证据必须是官方文档或可复现的脱敏连接；匹配必须尽量窄；负向测试必须挡住市场、更新、下载、统计和无关共享流量。遥测、宽泛供应商后缀、marketplace/CDN 默认不新增。

## Confirmed Facts

| 主机 | 当前归属 | 结论 |
|---|---|---|
| `api.anthropic.com` | `ANTHROPIC_CORE_EXACT_DOMAINS`，`routing.anthropic_core` 默认 true。`clash-verge-ai-residential.js:221-223` | 已走家宽。官方网络文档同时把它标成 API、功能开关和遥测事件日志，不能为了躲开遥测把它拆出核心。 |
| `platform.claude.com` | `DOMAIN-SUFFIX,claude.com`。`tests/regression.test.js:2551` 断言走家宽 | 已覆盖。2026-11-06 官方网络文档把它列为 Console 登录，以及 claude.ai 账户的 OAuth 交换、刷新和撤销。 |
| `bridge.claudeusercontent.com` | `DOMAIN-SUFFIX,claudeusercontent.com`。`tests/regression.test.js:2552` 断言走家宽 | 已覆盖。官方用途是 Claude in Chrome 的 WebSocket bridge；组织 IP 允许列表场景要求它与 `api.anthropic.com`、`claude.ai` 同出口。当前同属 `AI-家宽`。 |
| `downloads.claude.ai` | `DOMAIN-SUFFIX,claude.ai`。`docs/routing-scope.md:50` | 已覆盖，且被记为已知取舍。官方用途是插件可执行文件、本机安装包、自动更新和版本检查，不是推理。拆出需要动态上游名规则，当前精确字符串清理模型无法安全删除。 |
| `http-intake.logs.us5.datadoghq.com` | `CLAUDE_SHARED_EXACT_DOMAINS`，仅当 `ROUTE_CLAUDE_SHARED_DEPENDENCIES` 为 true 才注入。公开默认和本机 `clash-verge-ai-residential.local.toml:30` 都是 false。`clash-verge-ai-residential.js:117-118,422-424` | 不进入默认家宽。这是 Datadog 日志入口。官方必放行表未列出它。打开共享依赖只会把遥测改走家宽并占用带宽，不会停止上报。 |
| `clau.de` | `RETIRED_CORE_SUFFIX_DOMAINS`。`clash-verge-ai-residential.js:204-207`。`tests/regression.test.js:2588-2589,2714-2718` 断言不走家宽、也不写住宅 DNS | 保持退役。v5.10 因无官方出处退出。帖子回复不是官方文档，也不是脱敏 Connections。 |

同页官方必放行、截图未点名、但已被后缀盖住的主机：`sso.claude.com`（`claude.com`）、`*.frame.claudeusercontent.com`（`claudeusercontent.com`，`tests/regression.test.js:2553`）。

官方入站网段当前脚本只注入 `160.79.104.0/23` 与 `2607:6bc0::/48`（`clash-verge-ai-residential.js:514-517`）。出站 `160.79.104.0/21` 与更宽的 `2607:6bc0::/32` 不在托管模板里；若 Profile 里已有，幂等测试要求保留（`tests/regression.test.js:2070-2071,2136-2139`）。

2026-11-08 检索到的英文与俄文 `platform.claude.com` IP 页还列出了入站 `153.61.224.0/22` 和出站 `153.61.208.0/20`。同日中文、法文页以及 `docs.claude.com` 仍只有旧的 `/23` 与 IPv6 `/48`。直接打开 `code.claude.com` 被 Fake-IP `198.18.4.196` 拦住，因此新网段仍是未核实的检索片段，不在本任务默认范围内。`153.61.224.0/22` 与出站 `153.61.208.0/20` 相邻但不重叠；即使以后核实入站段，也不得把出站段写进客户端目的规则。

## Requirements

| ID | 要求 |
|---|---|
| R1 | 六个点名主机不新增重复的 `DOMAIN` / `DOMAIN-SUFFIX` 规则。 |
| R2 | `clau.de` 继续只留在退役清理列表，不重新注入，不写住宅 DNS。 |
| R3 | Datadog 日志主机继续只挂在默认关闭的 `routing.claude_shared_dependencies` 上。 |
| R4 | 不新增 `REJECT` / `REJECT-DROP`，不用规则去藏设备、会话、账户或组织标识。 |
| R5 | 不改 `*.local.toml`、不手改 `*.local.js`、不改开关默认值。 |
| R6 | 分类、文件锚点和证据缺口记在本任务，不把社交帖写成官方出处。 |
| R7 | `anthropic_core` 与 `anthropic_ip_fallback` 同时开启时注入入站 `153.61.224.0/22`。默认输出不含出站 `160.79.104.0/21` 与 `153.61.208.0/20`。关闭任一开关时，该入站规则与既有入站规则一起清理。 |

## Acceptance Criteria

- [x] AC1 → R1–R3：`npm run ci` 里现有正向断言仍覆盖 `platform.claude.com`、`bridge.claudeusercontent.com`、`api.anthropic.com`；`clau.de` 与默认配置下的 Datadog 主机仍不进入 `AI-家宽`。
- [x] AC2 → R4：差异中没有新的 Claude 遥测拦截规则。
- [x] AC3 → R5：`clash-verge-ai-residential.js` 的活跃域名清单与六个点名主机相关的部分无新增；本地 TOML/JS 无本任务改动。
- [x] AC4 → R6：本 PRD 的分类表与 `research/host-classification.md` 一致，并写明英文 IP 页与中文/法文页面的网段差异。
- [x] AC5 → R7：默认规则含 `IP-CIDR,153.61.224.0/22,AI-家宽,no-resolve` 一次；不含两条出站网段。`ROUTE_ANTHROPIC_CORE` 或 `ENABLE_ANTHROPIC_IP_FALLBACK` 关闭时，三条入站规则都不留下。默认 `AI-家宽` 规则数为 61。`npm run ci` 268 项通过。未跑真实 Clash Profile。

若后续批准的范围仍是「不改公开模板」，AC1–AC4 由核对本身满足，不需要产品补丁。

## Out of Scope

- 为规避封号而丢弃、阻断或改写 Datadog、Statsig、Sentry 或其他遥测。官方文档写明遥测事件也走 `api.anthropic.com`，阻断第三方日志端点并不能去掉平台侧信号。
- 把 `routing.claude_shared_dependencies` 默认改成 true，或代用户改本地 TOML。
- 恢复 `clau.de`、`claudemcpclient.com`、`a-api.anthropic.com`。
- 加入出站网段 `160.79.104.0/21`、`153.61.208.0/20`，或把 `anthropic.com` 收成宽泛后缀。
- 在未直接核对官方 IP 页之前，把 `153.61.224.0/22` 写入脚本。
- ResiWatch、文档站工具链、凭据和控制器。

## Decision

2026-11-08 用户要求继续核对 X 帖并完善规则。六个点名主机仍不新增域名规则。采纳的规则变更只有一条：在 `anthropic_core` 与 `anthropic_ip_fallback` 同时开启时注入官方入站 `153.61.224.0/22`。出站 `160.79.104.0/21` 与 `153.61.208.0/20` 不注入。不新增 `REJECT`，不改本地 TOML。

X 证据见 `research/host-classification.md`。英文与俄文 IP 页检索片段列出新入站段；同日中文、法文页面仍是旧名单。直接抓取被 Fake-IP 拦住，因此文档写明这个口径差异，不把社交帖当成新域名出处。
