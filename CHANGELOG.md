# 更新日志

所有值得关注的变更都记录在此。本项目的仓库发行版遵循语义化版本规范。

## [未发布]

### 新增

- 新增三个默认启用的核心路由开关：`routing.anthropic_core`、`routing.gemini_api_core` 和 `routing.antigravity_core`。保留现有的默认域名、DNS 策略和出口目标。家宽审计输入现在可映射 25 个路由开关中的 13 个受支持域名开关；其余 12 个仍不受支持。
- 新增默认启用的 `routing.extra` 类别，用于独立维护的小型 AI 网站；初始配置将 `anyrouter.top` 后缀路由到家宽链路。
- 在 `docs/` 下新增本地双语 VitePress 文档站点（`just docs-dev` / `just docs-build`，Node.js 22+）。中文是默认语言，沿用现有的 `docs/*.md` 路径；英文页面位于 `docs/en/`。`docs/adr/` 保持不变，不属于该站点。扩展脚本的 `just ci` 门禁仍不会安装 VitePress。
- 桌面壳现已提供真实的系统文件对话框和 Windows Toast 通知。备份、恢复、备份校验、报告导出和诊断信息导出都会打开原生保存/打开对话框（打开对话框时，`pick_file` 命令不再持有 facade 锁）。告警和测试通知按钮会通过 `tauri-plugin-notification` 发出真实的 Windows Toast；将 `RESIDENTIAL_MONITOR_ALLOW_TOAST=0`（或 `false`）可关闭通知。参见 `residential-monitor/docs/notifications.md`。测试替身（`FakeFileDialog` / `FakeNotificationSink`）现在仅用于测试，不再出现在生产组合根或用户可见文案中。
- 新增家宽专用页面，提供实时监控、类别聚合、在已归因观测量中的占比以及报告导出功能。新增命令 `residential_share`。当 `covered_sec == 0` 时，覆盖率会返回四个 `None` 字段，而不是 0%。
- 将报告、告警、设置/数据、恢复和不可用页面迁移到 React 壳。占比图表使用 Recharts。导出、保留期、备份、告警规则和关于页面的行为保持不变。
- C3 报告查询在原始层支持 `minute1` / `minute2` / `minute5` / `minute10` 粒度。现有的 `hour` / `day` / `month` 值保持不变。
- 除 `host` 外，C3 还会实体化 `process`、`rule_group`、`chain` 和 `network` 维度行。维度层上的类别排名按 `category_id` 分组，并保留 `dimension_kind = host`，避免流量被重复计算五次。
- 排名标识 `__unknown__` 表示维度值缺失。该行会保留在排名中，以便排名之和能够与总量一致。
- 主机标识依次使用 `metadata.host`、`sniffHost` 和目标 IP。主机页面可以按规则、链路和进程检查剩余的 `__unknown__` 行。
- 未知进程行可以下钻到主机和链路。进程页面可以按家宽统计口径筛选。当进程归因不可用时，排名条会替换为字段缺失说明和当前帧的进程覆盖率。

### 变更

- 更新日志统一改为简体中文，并精简扩展脚本文件头中与本文件重复的历史版本记录。
- Claude、OpenAI、Antigravity 和 Cursor 的专用进程回退现在要求启用各自对应的核心开关。Anthropic IP 回退也要求启用 `routing.anthropic_core`。认证、辅助、资产和全局捕获开关仍保持独立；核心流量被禁用后会回到原始 Profile 规则，而不是被强制送往机场出口。
- 现有的 `AI-家宽` 组只接受规范的单一家宽成员，以及 `name`、`type`、`proxies`、`disable-udp`、`icon` 和 `hidden` 字段。额外的提供方来源、过滤器和替代选择字段会被拒绝，且不会修改输入；输出仅保留受支持的展示元数据。配置被拒绝并不保证运行时流量一定会被阻断，因为脚本出错后，宿主可能保留其原始配置。
- 记录了现有 DNS 策略针对区域 Vertex 和可选 Cursor 索引正则路由的例外情况。未引入宽泛的域名后缀或新的解析器提供方；实际 DNS/UDP 路径和流量节省仍需运行时证据验证。
- ResiWatch 工具链：`typescript-eslint` 8.69.0、`@types/react-dom` 19.2.5，以及兼容范围内的 `cargo update`（`hyper` 1.11.1、`tauri-plugin-dialog` 2.7.3、`tauri-plugin-notification` 2.4.0）。GitHub Actions 的 `checkout` 和 `setup-node` 升级至 v7。破坏性升级包括：`lucide-react` 1.39.0、`sha2` 0.11、`rand` 0.10、`tokio-tungstenite` 0.30、`eslint-plugin-react-hooks` 7.1.1（扁平配置 `recommended`；`set-state-in-effect` 和 `refs` 仍关闭）、TypeScript 6.0.3、Vitest 4.1.11、Vite 8.2.2，以及 `@vitejs/plugin-react` 5.x 和作为可选压缩对等依赖的 `esbuild` 0.28。根目录的 `package.json` 仍保持零第三方依赖，且 `engines.node >=18`。本轮仍不引入 ESLint 10、TypeScript 7 和 plugin-react 6。
- 即使 `routing.ai_process_fallback` 为 `false`，扩展脚本也会写入顶层 `find-process-mode: always`。除非该开关已启用，否则仍不会注入 `PROCESS-NAME` / `PROCESS-PATH` 规则。Clash Verge 中嵌套在 `profile:` 下的值无法传递到内核。
- 使用 React + Tailwind 桌面 UI 替换了原生 TypeScript + Catppuccin 壳。导航包含十个路由。概览、实时连接以及主机/规则/链路/进程页面已随新壳提供。已移除 `src/main.ts` 和 `src/styles.css`。
- 家宽分类集中在一个模块中，并提供两个具名函数。统计使用目标的精确匹配。实时“仅家宽”仍会匹配已配置的目标，或名称中包含“家宽”的节点。
- `ReportFilters` 现在会应用于原始总量、序列和排名，包括类别。`filters.chain` 匹配链路的最后一跳。`filters.rule` 匹配 SQL 规则键。
- 当分组没有五维实体化数据时，维度层的 `exact_top_n` 为 `false`。查询 `hourly_dim_v2` 水位线之前的数据会返回 `capability_unsupported`。
- Windows 产品名称、开始菜单快捷方式、窗口标题和当前用户安装目录均为 `ResiWatch`。标识符和 exe 仍为 `residential-monitor`。侧边栏标语采用简短的“有边界、非账单”表述。
- 桌面 UI 路由页面使用 `React.lazy` 加载。React、Recharts 和 Radix 会生成独立的 Rollup 分块。Vite 的 500 kB 分块警告再次作为回归防线；不要通过提高 `chunkSizeWarningLimit` 来掩盖合并后的入口分块。

### 修复

- 不含 `host:port` 的控制器地址会返回 `invalid_address`。免打扰时段会阻止 `Activated`，而不只是抑制 Toast。恢复壳中的 `create_backup` 会返回 `recovery_only`，且不会复制实时数据库。
- `just tinstall` 使用 NSIS `/D=` 安装到 `%LOCALAPPDATA%\ResiWatch`。不会复用之前位于 `%TEMP%` 下或旧中文产品文件夹中的安装路径；数据会迁移到新的 `data\` 目录。
- Windows MSVC 不再为 `residential_monitor_lib` 输出 `linker_messages`。crate 类型仅保留 `rlib`；`cdylib` / `staticlib` 是移动端遗留项。
- 主机排名条的 Y 轴左侧不再裁剪较长的 FQDN。

### 计划

- 添加经过脱敏的真实 Profile 集成测试夹具。
- 在上游提供方发布机器可读清单时，添加自动化域名来源时效性检查。

## [5.11.0] - 2026-08-21

### 新增

- 新增独立的 `routing.openai_auth` 开关（默认 `false`），用于范围受限的 `auth.openai.com` 后缀和精确主机 `auth0.openai.com`。
- 新增独立的 `routing.openai_web_assets` 开关（默认 `false`），用于 `oaistatic.com` 后缀。
- 本地 TOML 渲染、缺失键补全、托管清理、DNS 策略、测试和开关文档现在均覆盖这两个控制项。

### 说明

- 这两个开关都不会启用 OpenAI 共用的 WorkOS、Intercom、Stripe、Cloudflare Challenge、Sentry 或 Datadog 依赖，也不会添加宽泛的 `openai.com` 后缀。
- 当目标 Profile 提供相同且可解析的上游名称和能力时，可以将渲染后的 `.local.js` 从 Windows 复制到 Ubuntu。渲染后的文件内嵌家宽端点和凭据，因此必须通过可信渠道传输，并作为机密加以保护。在没有经过脱敏的 Connections 证据时，Ubuntu 上 Clash 宿主的实际执行情况和端到端 ChatGPT 登录行为仍未经验证。

## [5.10.1] - 2026-08-20

### 修复

- 将 `daily-cloudcode-pa.googleapis.com` 恢复到活跃家宽目录。Antigravity 的 `language_server` 会将 `--cloud_code_endpoint` 设置为该主机。v5.10.0 曾因缺少文档而停用该主机。本地日志显示 TLS 握手失败，且 Clash Connections 会将该主机发送到原始 Profile 上游。

### 变更

- 默认注入的 `AI-家宽` 规则数量为 45。

## [5.10.0] - 2026-08-19

### 新增

- 新增 `routing.grok_web_assets` 开关（默认 `true`）。当该开关为 `true` 时，脚本注入 `DOMAIN-SUFFIX,grok.com`。当该开关为 `false` 时，脚本注入精确主机 `grok.com`、`cli-chat-proxy.grok.com` 和 `code.grok.com`。
- 新增 `routing.vertex_ai_endpoints` 开关（默认 `true`）。该开关控制四条 Vertex AI / Agent Platform 规则：`aiplatform.googleapis.com`、`aiplatform.us.rep.googleapis.com`、`aiplatform.eu.rep.googleapis.com`，以及区域正则 `^[a-z0-9-]+-aiplatform\.googleapis\.com$`。

### 变更

- 从活跃家宽目录中停用五个主机。这些主机仍保留在 `allPossible*` 中，以便升级时清理旧规则：`clau.de`、`claudemcpclient.com`、`a-api.anthropic.com`、`daily-cloudcode-pa.googleapis.com` 和 `geminicloudassist.googleapis.com`。
- 收窄四条规则：`api2.cursor.sh` 和 `authenticate.cursor.sh` 从后缀匹配改为精确匹配；将 `adminportal` 正则改为 `DOMAIN,adminportal42.cursor.sh`；`antigravity.google` 从后缀匹配改为精确匹配。
- 将 `api.x.ai` 从精确匹配改为后缀匹配，以匹配区域主机和 `mtls.api.x.ai`。
- 默认注入的 `AI-家宽` 规则数量为 44。

### 说明

- `chatgpt.com` 仍使用后缀匹配。`help.` 和 `status.` 等子域名仍通过家宽链路。
- 三个 `alkali*` AI Studio 主机仍保留在 `gemini_web_core` 中，且仍未经验证。
- `claudemcpcontent.com` 仍使用后缀匹配，用于 Claude Desktop MCP App 小组件。

## [5.9.0] - 2026-08-18

### 新增

- 新增 `routing.cursor_repository_indexing` 开关（默认 `false`），用于 Cursor 仓库索引主机 `repo[0-9]+.cursor.sh`。本地 TOML 字段缺失时会补全为 `false`。将该字段设置为 `true`，即可恢复 v5.8.1 对这些主机的家宽路由，无需删除该键。

### 变更

- 仓库索引正则不再属于 `routing.cursor_core`。默认情况下，`repo42.cursor.sh` 和其他 `repo<N>.cursor.sh` 主机会回退到原始 Profile/机场上游。Cursor Chat、Tab、Agent、认证和 Cloud Agent 仍使用 `routing.cursor_core`（默认仍为 `true`）。`api2.cursor.sh` 仍归属于 cursor_core。

### 说明

- 官方文档和本地 2026-08-17 日志共同确认 `repo42.cursor.sh` 是索引主机。
- `repo[0-9]+.cursor.sh` 是本项目的前向兼容策略，并非 Cursor 官方的通配符契约。
- Privacy Mode（隐私模式）不会阻止索引上传。
- `disableHttp2` 或服务器强制的 HTTP/1.1 回退可能会使 RepositoryService 使用共用的 `api2.cursor.sh`。Clash 域名规则无法隔离这条路径。本版本并未宣称所有仓库上传流量都不会经过家宽链路。

## [5.8.1] - 2026-08-17

### 变更

- 在 `main` 执行期间构建一次出站名称索引，使大型机场 Profile 无需为每个可达叶节点扫描所有代理。
- 将可达的 `udp: false` 叶节点警告合并为一条摘要（最多 8 个样例）。

## [5.8.0] - 2026-08-17

### 新增

- 根据 OpenAI 帮助文章 9247338，新增五个官方 ChatGPT 精确主机：`chat.openai.com`、`android.chat.openai.com`、`desktop.chat.openai.com`、`ios.chat.openai.com` 和 `tcr9i.chat.openai.com`。`tcr9i.chat.openai.com` 的用途没有文档说明。原生 ChatGPT 桌面端/iOS 的 Connections 结果仍未经验证。

### 变更

- 在 `routing.openai_core` 下恢复 `OPENAI_CORE_EXACT_DOMAINS`。生成的输出仅使用精确 `DOMAIN` 规则和不带前缀的 DNS 键。仅用于清理的 `chat.openai.com` 后缀条目会移除错误的 `DOMAIN-SUFFIX,chat.openai.com` 规则和 `+.chat.openai.com` 策略键；该后缀绝不会再次注入。

## [5.7.0] - 2026-08-16

### 新增

- 根据 Claude Code 官方网络配置文档，在 Claude 目录中新增：`mcp-proxy.anthropic.com` MCP 连接器代理，以及 `assets-proxy.anthropic.com` 桌面端/Web 资产代理（官方文档警告，阻止该代理会破坏应用 UI）。
- 根据 xAI 官方企业部署文档，在 Grok 目录中新增：`auth.x.ai` OAuth2/OIDC 主机（必须允许），以及 `api.x.ai` 直接 API 推理端点。`x.ai` 安装主机仍使用原始 Profile。
- 当从可达上游组中移除对 `AI-家宽` / `家宽-SOCKS5` 的引用时，新增一条 `warn` 日志。递归防护清理不再静默执行；日志会列出组名、被移除的条目，以及应如何改为路由 AI 流量。
- 新增一条 `info` 日志，说明当前 Clash Verge Rev 会在全局脚本运行后重新写入权威的 `tun` / `ipv6` 字段值；TUN DNS 劫持和 IPv6 开关必须在应用设置页面配置。文档现在也说明了这种宿主行为，以及 fake-ip DNS 的解析时机。

### 变更

- `api.openai.com` 从精确规则改为后缀规则，以便同时匹配官方 Codex 数据驻留前缀 `us.api.openai.com` / `eu.api.openai.com`。v5.6 以精确形式生成的规则仍会以幂等方式清理。

## [5.6.0] - 2026-08-16

### 新增

- 新增 `routing.openai_core` 开关（默认 `true`），用于控制 ChatGPT 产品、OpenAI 模型 API 以及用户上传/生成内容的路由。在本地 TOML 中将其设置为 `false`，可使 GPT 流量继续使用机场上游，而不是家宽链路。
- 新增 `routing.grok_core` 开关（默认 `true`），通过家宽链路路由 Grok Build（xAI grok CLI）推理 API `cli-chat-proxy.grok.com`（`/v1/responses` 推理和 `/v1/storage` 代码库/会话上传）以及 Grok 产品域名。Grok 第三方分析服务（`api.mixpanel.com`）、`x.ai` 安装主机和共用的 `storage.googleapis.com` 仍使用原始 Profile。
- 根据 Cursor 官方企业网络配置文档，在 Cursor 目录中新增：`authenticate.cursor.sh` 授权端点、`adminportal<N>.cursor.sh` SSO 门户（有界正则），以及 `*.cursorvm.com` Cloud Agent VM 主机。Marketplace、CDN、下载和更新主机仍被排除。
- 在 `just render-local` / `node scripts/sync-local-config.js` 期间自动补全本地 TOML：缺失的开关键（包括缺失的 `[routing]` / `[runtime]` 表）会使用示例默认值追加到本地 TOML。现有值、注释、行尾符和末尾换行会逐字保留；补全过程是幂等的，而缺失 `[home_proxy]` 凭据键时仍会采用失败关闭策略。

### 变更

- `routing.cursor_core` 现在默认为 `true`：无需显式启用即可注入 Cursor 规则和 DNS 策略。在本地 TOML 中将其设置为 `false`，可使 Cursor 继续使用机场上游。

### 修复

- 将观测到的 Anthropic 核心 API 主机 `a-api.anthropic.com` 路由到家宽连接和 DNS 路径，同时不将默认范围扩大到所有 `anthropic.com` 流量。

## [5.5.0] - 2026-07-23

### 新增

- 新增可选的 `[routing]` 和 `[runtime]` 本地 TOML 表，覆盖所有标量用户开关，同时保持与仅含 home proxy 的配置文件兼容。
- 新增“恰好一个布尔锚点”验证，以及针对部分开关覆盖的原子化本地脚本渲染。
- 新增验证：在构造 Mihomo DoH URL 前，拒绝包含 `#` 或 `&` 的上游名称。
- 完善 `just render-local` 和直接使用 Node 的设置流程，并提供 Clash Verge Rev Global Extend Script 截图。

### 变更

- Cursor 核心路由现在需要显式启用，且默认关闭；仍可通过 `routing.cursor_core = true` 使用收窄后的目录。
- 移除三条冗余的 Cursor 目录匹配项，这些匹配已被保留的后缀规则或有界仓库规则覆盖。
- 开关变化时，当前版本的托管规则仍会被替换；而目标为 `AI-家宽` 的未知规则仍归用户所有。
- 文档记录了继续采用严格 DNS 所带来的首次查询延迟权衡，以及登录流量使用原始 Profile、模型流量使用家宽出口的分流方式。

### 移除

- 移除尚未发布的 v5.4 之前旧版迁移目录、目标重定向、组引用迁移和旧组清理逻辑。
- 如果 v5.4 生成的输出曾被手动持久化到订阅或 Merge 层，请在刷新前从对应位置移除以下现已停用、归用户所有的规则：
  - `DOMAIN,repo42.cursor.sh,AI-家宽`
  - `DOMAIN-REGEX,^[a-z0-9-]+\.api5\.cursor\.sh$,AI-家宽`
  - `DOMAIN-REGEX,^(?:us-asia|us-eu|us-only)\.gcpp\.cursor\.sh$,AI-家宽`

## [5.4.0] - 2026-07-22

### 新增

- 稳定的公开入口文件：`clash-verge-ai-residential.js`。
- 为 Claude、ChatGPT、Gemini、Google Antigravity 和 Cursor 核心推理/Agent 流量提供仅限 AI 的路由。
- 支持多 Profile 的 `dialer-proxy` 解析，并将 `🚀节点选择` 作为首选默认项。
- 提供递归代理组和 `include-all` 防护。
- 提供 AI 专用 DNS 策略，并将非 AI 的境外 DNS 绑定到当前 Profile 上游。
- 新增 28 项配置级回归测试。
- 在 Node.js 18、20 和 22 上运行 CI。
- 新增模板安全检查，拒绝提交家宽 SOCKS5 凭据。

### 变更

- 明确将 Cursor Marketplace、下载、CDN、更新资产、YouTube、Maps、广告和共用遥测排除在家宽路由之外。
- 使用稳定的仓库路径取代带版本号的归档文件名；发行版本通过 Git 标签表示。

### 安全性

- 公开模板中的家宽端点和凭据仍保留为占位符。
- 当必需凭据或上游组无法安全解析时，运行时配置会采用失败关闭策略。
