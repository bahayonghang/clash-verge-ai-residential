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

1. 任务的 `research/` 目录下存在 5 份调研文档（Anthropic、OpenAI、xAI、Cursor、Google），
   每条结论附来源 URL 或「无官方出处」标注。
2. 产出一份逐条判定表，覆盖当前全部 49 条注入规则，分类为「保留」「收窄」「移除」「补充」「存疑」。
3. 对每条「收窄」或「移除」项，给出具体的替换规则文本与受影响主机示例。
4. 对每条「补充」项，给出官方来源 URL 与该主机缺失时的功能影响。
5. 若产生代码改动，`just ci`（`node --check` + 回归测试 + 密钥扫描）通过。
6. 若产生代码改动，`tests/regression.test.js` 中新增负向断言，覆盖本次判定为「应移除」的主机；
   新增正向断言，覆盖判定为「必需」的主机。
7. `docs/routing-scope.md` 与 `CHANGELOG.md` 同步更新判定依据。

## 交付物

- `research/anthropic-domains.md`、`research/openai-domains.md`、`research/xai-grok-domains.md`、
  `research/cursor-domains.md`、`research/google-gemini-domains.md`
- `design.md`：逐条判定表与收窄方案
- `implement.md`：改动顺序、验证命令、回滚点
