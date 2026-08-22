# AI 家宽路由域名清单审计

## 背景

`clash-verge-ai-residential.js` v5.9.0 当前向 `AI-家宽` 注入 49 条规则（19 条 `DOMAIN-SUFFIX`、
26 条 `DOMAIN`、2 条 `DOMAIN-REGEX`、2 条 IP 段）。本地部署开启的路由开关为
`openai_core`、`gemini_web_core`、`cursor_core`、`grok_core`、`anthropic_ip_fallback`。

用户提出的约束：只有以下三类流量应当走家宽链路。

1. 网页 Chat 产品会话（Claude、ChatGPT、Grok、Gemini、AI Studio）。
2. 本机 CLI 连接官方端点（Claude Code、Codex、Grok CLI、Gemini CLI）。
3. 桌面 / IDE 客户端连接官方 AI 端点（Cursor、Claude Desktop、ChatGPT Desktop、Antigravity）。

其余流量一律留在原 Profile 的机场出口。

## 问题

对当前规则集做主机探针（`ruleMatchesHost` 同语义）后确认，多个产品顶级域使用 `DOMAIN-SUFFIX`
匹配，会把该域下的全部子域纳入家宽，其中包含文档站、状态页、营销页、下载与遥测子域。
探针命中的过度代理主机示例：

- `DOMAIN-SUFFIX,claude.com` 命中 `docs.claude.com`、`code.claude.com`、`status.claude.com`、
  `support.claude.com`、`console.claude.com`、`blog.claude.com`、`www.claude.com`。
- `DOMAIN-SUFFIX,claude.ai` 命中 `downloads.claude.ai`、`status.claude.ai`。
- `DOMAIN-SUFFIX,chatgpt.com` 命中该域下任意子域，包含 `help.`、`status.`、`ab.`、`events.` 等前缀。
- `DOMAIN-SUFFIX,grok.com` 命中该域下任意子域，包含 `assets.`、`static.`、`status.` 等前缀。
- `DOMAIN-SUFFIX,cursorvm.com` 命中该域下任意子域，不限于 Cloud Agent 虚拟机主机。
- `DOMAIN-SUFFIX,antigravity.google` 命中 `docs.` 与 `download.` 子域。
- `DOMAIN-REGEX,^[a-z0-9-]+-aiplatform\.googleapis\.com$` 的 `[a-z0-9-]+` 段不限定为 Vertex AI 区域名。
- `api2.cursor.sh`、`api5.cursor.sh`、`gcpp.cursor.sh`、`authenticate.cursor.sh`、
  `authentication.cursor.sh` 是单个主机，却使用 `DOMAIN-SUFFIX` 匹配，额外覆盖 `*.<host>`。

`docs/routing-scope.md` 已记录 `downloads.claude.ai` 属于已知取舍。其余命中项此前没有记录。

## 证据策略（已定，2026-08-19）

外部审阅指出一处目标冲突：本 PRD 要求「其余流量一律留在机场」且「新增或保留的域名必须有
官方来源或经脱敏 Connections 证据」，而 v1 执行计划同时决定不删除任何存疑规则。两者不能并存。

用户已选定 **证据优先**：刷新官方资料后仍无官方来源、也无脱敏 Connections 证据的规则，
退出激活清单，保留在 `allPossible*` 中仅用于迁移清理；取得证据后逐条恢复。

刷新官方资料后，第三次检索再改写两条：

- `a-api.anthropic.com` 已有官方列出，用途是 3P Desktop 分析事件，按遥测排除，退出激活。
- `claudemcpcontent.com` 已有官方 Desktop 通配（MCP Apps widget），用户确认保留后缀。

仍无官方出处、按证据优先退出的集合：`clau.de`、`claudemcpclient.com`、
`daily-cloudcode-pa.googleapis.com`、`geminicloudassist.googleapis.com`。

三条 `alkali*` 无官方防火墙清单，用户确认保留并标 UNVERIFIED，避免 AI Studio 网页会话出口分裂。

接受的风险：若其中某条实际承载会话流量，退出激活后会出现出口 IP 分裂。
该风险由 `implement.md` 步骤 11 的部署后取证覆盖，出现问题时回到 Plan 阶段按证据恢复。

## 目标

1. 用官方文档核对每一条已注入域名，判定其属于「推理 / 会话 / 认证必需」还是「文档、状态、营销、
   CDN、下载、更新、遥测」。
2. 给出应移除、应收窄、应补充的具体清单，每条附官方来源 URL 或明确标注「无官方出处」。
3. 用官方文档核对 `160.79.104.0/23` 与 `2607:6bc0::/48` 的性质（inbound 还是 outbound），
   判定 `ENABLE_ANTHROPIC_IP_FALLBACK` 是否达成其声称的兜底作用。
4. 把结论写入任务工件，并给出可执行的修改方案与回归测试方案。

## 非目标

- 不改动上游解析（`resolveUpstreamName`）、递归链防护、DNS 结构或嗅探逻辑。
- 不新增路由开关以外的功能。
- 不改动 `residential-monitor/` 子项目。

## 约束

- 公开模板中的 `server` / `username` / `password` 必须保持 `"xxx"` / `""`（`npm run check:secrets` 强制）。
- 不手工编辑 `*.local.js`；如需更新用 `just render-local` 重新生成。
- 新增或保留的域名必须有官方来源或经脱敏的 Connections 证据，并配套负向测试
  （`docs/routing-scope.md` 的 Acceptance rule for new domains）。
- 代码注释、错误信息、`docs/` 用中文；`CHANGELOG.md`、`package.json`、CI 用英文。

## 验收标准

1. 任务的 `research/` 目录下存在调研文档（Anthropic、OpenAI、OpenAI Codex 端点、xAI、Cursor、
   Google、审阅记录），每条结论附来源 URL 或「无官方出处」标注；被推翻的早期判定必须留有更正记录。
2. `design.md` 的逐条表是**互斥账本**：当前 49 条注入规则，每条恰好出现一次，
   判定取值限于 `保留` / `收窄` / `改类型` / `退出激活`，且分项之和等于 49。
3. 对每条「收窄」或「改类型」项，给出具体的替换规则文本与受影响主机示例。
4. 对每条「退出激活」项，给出其必须保留在哪个 `allPossible*` 清单中。
5. 无运行时证据的判断必须在 `design.md` 中标记 UNVERIFIED，并在 `implement.md` 中
   有对应的部署后取证步骤。单元级 `ruleMatchesHost` 断言不得作为产品可用性的验收依据。
6. 若产生代码改动，`just ci`（`node --check` + 回归测试 + 密钥扫描）通过。
7. 若产生代码改动，`tests/regression.test.js` 中新增负向断言覆盖全部「退出激活」与「收窄」
   排除掉的主机；新增正向断言覆盖判定为必需的主机；新增规则总数断言固化对账结果。
8. `docs/routing-scope.md` 与 `CHANGELOG.md` 同步更新判定依据与 UNVERIFIED 清单。

## 已核实的 Codex 后台（2026-08-19，见 `research/openai-codex-endpoints.md`）

用户给出的默认后端已对照官方源码与文档核实，全部成立。

| 登录方式 | 后台 | 主机 | 当前规则是否覆盖 |
|---|---|---|---|
| ChatGPT 账号 | `https://chatgpt.com/backend-api/codex/responses` | `chatgpt.com` | 覆盖（`DOMAIN-SUFFIX,chatgpt.com`） |
| ChatGPT 账号 | `https://chatgpt.com/backend-api/codex/responses/compact` | `chatgpt.com` | 覆盖 |
| ChatGPT 账号 | `wss://chatgpt.com/backend-api/codex/responses` | `chatgpt.com` | 覆盖 |
| API Key | `https://api.openai.com/v1/responses` | `api.openai.com` | 覆盖（`DOMAIN-SUFFIX,api.openai.com`） |

Clash 只匹配主机，不匹配路径。不能只代理 `/backend-api/codex/*` 而放过同主机其它路径。
ChatGPT 网页官方 WebSocket 主机是 `ws.chatgpt.com`（help 9247338），
被现行后缀覆盖。

## 已定：三条 `alkali*` 保留并标 UNVERIFIED（2026-08-19）

用户确认三条 `*.clients6.google.com` 留在 `gemini_web_core` 激活清单。
无官方防火墙出处。部署后用 AI Studio 脱敏 Connections 补证。

## 已定：`claudemcpcontent.com` 保留后缀（2026-08-19）

用户确认保留 `DOMAIN-SUFFIX,claudemcpcontent.com`。
依据：Claude Desktop 官方通配，MCP Apps widget 隔离域，与 `claudeusercontent.com` 同类。

## 已定：`chatgpt.com` 保持整域后缀（2026-08-19）

用户确认保持 `DOMAIN-SUFFIX,chatgpt.com`。
`help.` / `status.` / `ab.` / `events.` 等子域继续走家宽，记为已知取舍。
可枚举方案（`DOMAIN,chatgpt.com` + `DOMAIN,ws.chatgpt.com`）不实施。

## 交付物

- `research/anthropic-domains.md`、`research/openai-domains.md`、`research/xai-grok-domains.md`、
  `research/cursor-domains.md`、`research/google-gemini-domains.md`、
  `research/openai-codex-endpoints.md`、`research/claude-backend-endpoints.md`、
  `research/review-findings.md`
- `design.md`：逐条判定表与收窄方案
- `implement.md`：改动顺序、验证命令、回滚点
