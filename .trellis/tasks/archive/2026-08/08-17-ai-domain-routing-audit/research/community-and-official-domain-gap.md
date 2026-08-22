# 核心 AI 域名缺口对照（2026-08-17）

对照对象：`clash-verge-ai-residential.js` v5.7.0 默认开启清单。
准入规则：`docs/routing-scope.md`（官方文档或脱敏 Connections + 负向测试；拒绝宽泛 provider suffix、市场/CDN/遥测）。

## 对照来源

| 类型 | 来源 | 取回日期 |
|---|---|---|
| 官方 | https://code.claude.com/docs/en/network-config | 2026-08-17 |
| 官方 | https://help.openai.com/en/articles/9247338-network-recommendations-for-chatgpt-errors-on-web-and-apps | 2026-08-17；摘录见 `openai-9247338-allowlist-excerpt.md` |
| 官方 | https://cursor.com/docs/enterprise/network-configuration | 2026-08-17 |
| 官方 | https://docs.x.ai/build/enterprise | 2026-08-17 |
| 官方 | https://antigravity.google/docs/enterprise | 2026-08-17 |
| 社区聚合 | https://github.com/VPSDance/ai-proxy-rules | 2026-08-17 |
| 社区 | https://github.com/blackmatrix7/ios_rule_script（OpenAI / Claude / Gemini） | 2026-08-17 |
| 社区 | https://github.com/v2fly/domain-list-community `data/openai`、`data/anthropic` | 2026-08-17 |

社区规则集普遍使用 `openai.com` / `anthropic.com` / `cursor.sh` 宽后缀、进程名兜底、遥测与 CDN。它们适合「凡是 AI 相关都代理」，与本仓库家宽链路的窄范围准入不一致。下表只记录对本仓库默认清单有决策价值的条目。

## 官方清单 vs 脚本默认开启项

### Claude / Anthropic

| 主机 | 官方定位 | 脚本现状 | 结论 |
|---|---|---|---|
| `api.anthropic.com` | API / flag / 遥测 | exact | 已覆盖 |
| `claude.ai` / `claude.com` | 产品与登录 | suffix | 已覆盖；`platform.claude.com` 随 `claude.com` 覆盖 |
| `mcp-proxy.anthropic.com` | MCP connector 代理 | exact | 已覆盖（v5.7） |
| `assets-proxy.anthropic.com` | 桌面/网页资产代理 | exact | 已覆盖（v5.7） |
| `*.claudeusercontent.com` | 制品 | `claudeusercontent.com` suffix | 已覆盖 |
| `code.claude.com` | 文档查找 | 未纳入 | 保持排除（辅助文档，非推理） |
| Datadog / brew / npm / GCS | 遥测或安装 | 默认关 | 保持排除 |
| `160.79.104.0/23`、`2607:6bc0::/48` | 官方入站段 | IP 回退 | 已覆盖 |

社区额外出现、官方网络表未列：`claude.app`、`claude.new`、`claude.site`、`claudepages.dev`、`claudestudio.com`、`antspace.dev`、`modelcontextprotocol.*`。无官方或 Connections 证据，不加。

社区把 Anthropic 入站写成 `160.79.104.0/21` 与 `2607:6bc0::/32`。官方文档是 `/23` 与 `/48`。保持官方段。

### ChatGPT / OpenAI / Codex

| 主机 | 官方定位 | 脚本现状 | 结论 |
|---|---|---|---|
| `*.chatgpt.com`、`ws.chatgpt.com` | Web / Codex 流式 | `chatgpt.com` suffix | 已覆盖 |
| `api.openai.com`、`us.` / `eu.` 前缀 | 模型 API / Codex 驻留 | `api.openai.com` suffix | 已覆盖（v5.7） |
| `*.oaiusercontent.com` | 上传与生成内容 | suffix | 已覆盖 |
| `chat.openai.com` | 官方 allowlist 明文 | **未匹配** | `chatgpt.com` suffix 不覆盖该主机；改为 exact |
| `android.chat.openai.com` / `desktop.chat.openai.com` / `ios.chat.openai.com` / `tcr9i.chat.openai.com` | 官方 allowlist 明文；`tcr9i` 无用途说明 | **未匹配** | 四个独立 exact。不用 `chat.openai.com` suffix，避免未列出的子域 |
| `*.oaistatic.com` | 静态 CDN | 负向测试排除 | 保持排除 |
| `*.auth.openai.com`、`auth0.openai.com` | 认证 | 有意排除 | 保持排除（`docs/routing-scope.md` 登录/推理出口分离） |
| Intercom / WorkOS / Stripe / Sentry / Datadog / Statsig | 共享依赖 | 默认关 | 保持排除 |
| `sora.com` | 社区与 v2fly 列入；官方 9247338 未列 | 未纳入 | 属另一产品，默认不加 |
| `chat.com` / `ai.com` | 短链跳转 | 未纳入 | 仅首跳，非推理主机 |
| `chatgpt.livekit.cloud` | 社区 Voice；官方 Voice 写 UDP 3478 | 未纳入 | 不加。官方 Voice 未点名 `tcr9i` |

### Cursor

官方细粒度列表与脚本一致：`api2/3/4/5.cursor.sh`、`gcpp.cursor.sh`、`repo<N>.cursor.sh`、`adminportal<N>.cursor.sh`、`authenticate` / `authenticator` / `authentication.cursor.sh`、`*.cursorvm.com`、`api.cursor.com`。

`prod.authentication.cursor.sh` 被现有 `authentication.cursor.sh` suffix 覆盖。

官方仍把 `marketplace.cursorapi.com`、`cursor-cdn.com`、`downloads.cursor.com`、S3 二进制列为更新/市场。脚本正确排除。

社区 `cursor.sh` / `cursor.com` / `cursorapi.com` 宽后缀与仓库负向测试冲突，不加。

### Grok Build

官方 required：`cli-chat-proxy.grok.com`、`auth.x.ai`。官方 additional：`api.x.ai`、`code.grok.com`、`assets.grok.com`。脚本已覆盖（`grok.com` suffix + 两个 exact）。

官方继续把 `x.ai` 标为安装脚本。脚本正确排除。

社区额外：`grok.x.com`、`grokipedia.com`、`x.ai` suffix。官方企业网络表未列前两者；后者过宽。不加。

### Gemini / Antigravity

| 主机 | 证据 | 脚本现状 | 结论 |
|---|---|---|---|
| `gemini.google.com`、`aistudio.google.com` | 产品入口 | suffix | 已覆盖 |
| `generativelanguage.googleapis.com`、`aiplatform.googleapis.com`、区域 regex | Developer API / Vertex | exact + regex | 已覆盖 |
| `cloudcode-pa.googleapis.com`、`daily-cloudcode-pa.googleapis.com` | Gemini CLI / Antigravity 网关 | exact | 已覆盖 |
| `cloudaicompanion` / `geminicloudassist` | Code Assist | exact | 已覆盖 |
| `alkali*-pa.clients6.google.com` | AI Studio RPC | exact | 已覆盖 |
| `antigravity.google` | 产品域 | suffix | 已覆盖 |
| `antigravity-pa.googleapis.com`、`antigravity.googleapis.com`、`antigravity-unleash.goog` | 仅社区规则 | 未纳入 | 无官方 allowlist，暂不加 |
| `daily-cloudcode-pa.sandbox.googleapis.com` | 第三方客户端源码 | 未纳入 | 非官方生产网关，暂不加 |
| `makersuite.google.com`、`bard.google.com`、`generativeai.google`、`ai.google.dev` | 旧入口或文档站 | 未纳入 | 非当前核心推理主机 |
| NotebookLM / Jules / Flow / Labs / Colab | 社区扩品 | 未纳入 | 超出脚本现有产品集 |

Antigravity 官方企业页只要求启用 `aiplatform.googleapis.com`，没有发布完整客户端 allowlist。第三方讨论仍把生产推理指到 `cloudcode-pa.googleapis.com`。

## 社区规则相对本仓库的系统差异

| 社区做法 | 本仓库对应策略 |
|---|---|
| `DOMAIN-SUFFIX,openai.com` / `anthropic.com` / `cursor.sh` | 拒绝；会吞掉官网、文档、状态页、市场 |
| `PROCESS-NAME` 全量代理 | `ENABLE_AI_PROCESS_FALLBACK` 默认关 |
| 遥测、支付、客服、CDN | 默认关或负向测试锁定 |
| Copilot / Windsurf / NotebookLM / Sora / Jules | 不在当前脚本产品集 |
| 更宽的 Anthropic CIDR | 使用官方 `/23` 与 `/48` |

## 当前可执行缺口

1. **官方证据已齐、默认清单未覆盖：** 五个 exact 主机 `chat.openai.com`、`android.chat.openai.com`、`desktop.chat.openai.com`、`ios.chat.openai.com`、`tcr9i.chat.openai.com`。
2. **官方产品功能、本轮不加：** ChatGPT Voice（`chatgpt.livekit.cloud` + UDP 3478）。
3. **仅社区出现、证据不足：** `antigravity-pa.googleapis.com` 等。

## 明确不加（本轮对照）

`oaistatic.com`、`oaistatsig.com`、`auth.openai.com`、`www.openai.com`、`www.anthropic.com`、`x.ai` 后缀、`cursor.sh` / `cursor.com` 宽后缀、`marketplace.cursorapi.com`、`cursor-cdn.com`、`downloads.cursor.com`、Intercom / Sentry / Datadog / Stripe、`apis.google.com` / `clients6.google.com` 宽后缀、YouTube / Maps / Gstatic。
