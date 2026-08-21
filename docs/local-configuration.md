# 本地 TOML 配置与同步

公开的 `clash-verge-ai-residential.js` 必须始终保留 `xxx` 占位符。本地凭据和脚本开关使用 `clash-verge-ai-residential.local.toml` 保存，再单向渲染为 `clash-verge-ai-residential.local.js`。这两个本地文件均被 `.gitignore` 排除，只有示例文件 `clash-verge-ai-residential.local.toml.example` 会进入版本控制。

## 前置条件

- Node.js 18 或更高版本。
- 可选的 [just](https://github.com/casey/just) 命令行工具；没有 `just` 时可直接运行 Node 渲染器。

在项目根目录执行以下命令。Justfile 会在 Windows 使用内置 Windows PowerShell，在 macOS/Linux 使用 `sh`，因此 Windows 不需要安装 Git Bash 或额外的 Unix shell。

## 首次配置

推荐首次执行 `just render-local`。如果不存在 `clash-verge-ai-residential.local.toml`，命令会自动从 `clash-verge-ai-residential.local.toml.example` 创建它，并提示你填写配置后重新执行。随后编辑本地 TOML：

```toml
[home_proxy]
name = "家宽-SOCKS5"
type = "socks5"
server = "residential.example.com"
port = 1080
username = "your-username"
password = "your-password"
udp = true
dialer-proxy = "🚀节点选择"

[routing]
cursor_core = true
cursor_repository_indexing = false
grok_core = true
```

字段含义：

| 字段 | 说明 |
| --- | --- |
| `name` | 必须与公开模板的 `HOME_PROXY_NAME` 一致，目前为 `家宽-SOCKS5`。 |
| `type` | 当前只允许 `socks5`。 |
| `server` / `port` | 家宽 SOCKS5 主机与 `1-65535` 的端口。 |
| `username` / `password` | 认证信息；无认证服务必须将两个值都设为 `""`。 |
| `udp` | SOCKS5 服务支持 UDP 时设为 `true`。 |
| `dialer-proxy` | 本地机场 Profile 中实际存在的上游代理组或节点名。 |

TOML 的字符串使用双引号；用户名或密码中有双引号、反斜杠时需使用 TOML 转义。`#` 在引号内是值的一部分，在引号外开始注释。

没有 `just` 时，先在文件不存在的前提下手动复制示例。Windows PowerShell：

```powershell
if (-not (Test-Path clash-verge-ai-residential.local.toml)) {
  Copy-Item clash-verge-ai-residential.local.toml.example clash-verge-ai-residential.local.toml
}
```

macOS/Linux：

```bash
test -e clash-verge-ai-residential.local.toml || \
  cp clash-verge-ai-residential.local.toml.example clash-verge-ai-residential.local.toml
```

然后编辑同一个本地 TOML。不要把真实值写入公开 JavaScript 模板。

## 开关配置

`[routing]` 和 `[runtime]` 都是可选表，并且允许只写需要覆盖的键。同步时，本地 TOML 缺失的开关键（包括整个缺失的表）会按示例文件的默认值自动补全并写回本地文件；已有键值、注释和行尾风格逐字保留，`[home_proxy]` 的凭据字段缺键仍会报错要求手填。因此，旧版只含 `[home_proxy]` 的本地 TOML 也可以直接渲染，渲染器会顺手补齐缺失开关。以下 TOML 键与 JavaScript 常量是一一映射，不要根据 `ROUTE_*` / `ENABLE_*` 前缀自行猜测键名。

### 路由范围

| TOML 键 | JavaScript 常量 | 默认值 | 作用 | 依赖或风险 |
| --- | --- | --- | --- | --- |
| `routing.openai_shared_dependencies` | `ROUTE_OPENAI_SHARED_DEPENDENCIES` | `false` | 路由 OpenAI 的 WorkOS、客服、遥测、支付等共享依赖。 | 会扩大到非模型流量。 |
| `routing.openai_core` | `ROUTE_OPENAI_CORE` | `true` | 路由 ChatGPT 产品、OpenAI 模型 API 和用户上传/生成内容。 | 关闭后 GPT 流量改走机场上游。 |
| `routing.openai_auth` | `ROUTE_OPENAI_AUTH` | `false` | 路由第一方登录主机 `auth.openai.com`（含其子域）和精确主机 `auth0.openai.com`。 | 与核心流量、网页资源和共享第三方依赖相互独立；不会匹配整个 `openai.com`。 |
| `routing.openai_web_assets` | `ROUTE_OPENAI_WEB_ASSETS` | `false` | 路由 `oaistatic.com` 网页静态资源后缀。 | 与第一方登录及共享第三方依赖独立；仅在页面资源确需同出口时开启。 |
| `routing.claude_shared_dependencies` | `ROUTE_CLAUDE_SHARED_DEPENDENCIES` | `false` | 路由 Claude 的统计、客服、风控等共享依赖。 | 会扩大到非模型流量。 |
| `routing.antigravity_google_auth` | `ROUTE_ANTIGRAVITY_GOOGLE_AUTH` | `false` | 路由 Antigravity 使用的共享 Google 登录入口。 | 影响其他 Google 产品的认证流量。 |
| `routing.antigravity_project_apis` | `ROUTE_ANTIGRAVITY_PROJECT_APIS` | `false` | 路由 Service Usage、Resource Manager、IAM、API Hub 等项目 API。 | 属于项目配置而非推理。 |
| `routing.antigravity_update_and_telemetry` | `ROUTE_ANTIGRAVITY_UPDATE_AND_TELEMETRY` | `false` | 路由 Antigravity 更新、扩展市场和遥测。 | 会扩大到更新和统计流量。 |
| `routing.gemini_web_core` | `ROUTE_GEMINI_WEB_CORE` | `true` | 路由 Gemini Web 和 Google AI Studio 产品入口。 | 无。 |
| `routing.vertex_ai_endpoints` | `ROUTE_VERTEX_AI_ENDPOINTS` | `true` | 路由四条 Vertex AI / Agent Platform 规则：`aiplatform.googleapis.com`、`aiplatform.us.rep.googleapis.com`、`aiplatform.eu.rep.googleapis.com`，以及区域正则 `^[a-z0-9-]+-aiplatform\.googleapis\.com$`。 | 不使用 Antigravity 企业推理或其他 Vertex AI 流量时可改为 `false`，这些主机改走机场上游。 |
| `routing.cursor_core` | `ROUTE_CURSOR_CORE` | `true` | 路由 Cursor AI API、Tab、Agent、授权/SSO 门户、Cloud Agent VM 和产品专属认证。 | 不需要 Cursor 核心流量走家宽时可显式改为 `false`。`api2.cursor.sh` 始终由本开关控制。 |
| `routing.cursor_repository_indexing` | `ROUTE_CURSOR_REPOSITORY_INDEXING` | `false` | 路由 Cursor 仓库索引主机 `repo[0-9]+.cursor.sh`。 | 与 `routing.cursor_core` 独立。默认回落原 Profile / 机场上游；缺字段按 `false` 补全；显式 `true` 恢复 v5.8.1 的 repo 家宽路由。官方与本机 2026-08-17 日志共同确认的精确主机是 `repo42.cursor.sh`；数字通配是项目前向兼容策略，不是 Cursor 官方通配合同。Privacy Mode 不会停止索引上传。`disableHttp2` 或服务端强制 HTTP/1.1 时，RepositoryService 可能改走共享的 `api2.cursor.sh`，域名规则无法在保留多数 API 的同时隔离该路径，因此不能宣称已排除全部仓库上传。 |
| `routing.grok_core` | `ROUTE_GROK_CORE` | `true` | 路由 Grok Build（xAI grok CLI）推理 API（`cli-chat-proxy.grok.com`）、Grok 产品域、`auth.x.ai` 与 `api.x.ai`。 | 不需要 Grok 走家宽时可显式改为 `false`。 |
| `routing.grok_web_assets` | `ROUTE_GROK_WEB_ASSETS` | `true` | 为 `true` 时注入 `DOMAIN-SUFFIX,grok.com`；为 `false` 时把该后缀换成精确主机 `grok.com`、`cli-chat-proxy.grok.com`、`code.grok.com`。`DOMAIN-SUFFIX,api.x.ai` 仍由 `routing.grok_core` 控制。 | 依赖 `routing.grok_core = true`。`false` 时 `assets.grok.com` 改走机场上游。 |
| `routing.cursor_process_fallback` | `ROUTE_CURSOR_PROCESS_FALLBACK` | `false` | 增加 Cursor 进程级兜底规则。 | 仅在 `routing.ai_process_fallback = true` 时生效，会捕获非 AI 请求。 |
| `routing.claude_code_auxiliary` | `ROUTE_CLAUDE_CODE_AUXILIARY` | `false` | 路由 Claude Code 安装、更新、文档和包管理端点。 | 属于辅助流量而非推理。 |
| `routing.ai_process_fallback` | `ENABLE_AI_PROCESS_FALLBACK` | `false` | 为已知 AI 应用增加进程级兜底并启用进程查找。 | 会把进程中的非 AI 请求一并路由。 |
| `routing.anthropic_ip_fallback` | `ENABLE_ANTHROPIC_IP_FALLBACK` | `true` | 使用 Anthropic 官方入站网段覆盖纯 IP 连接。 | 无。 |
| `routing.shared_realtime_infrastructure` | `ROUTE_SHARED_REALTIME_INFRASTRUCTURE` | `false` | 路由通用 STUN/TURN 实时通信基础设施。 | 可能捕获其他应用的实时流量。 |
| `routing.global_realtime_ports` | `ROUTE_GLOBAL_REALTIME_PORTS` | `false` | 按通用实时 UDP 端口增加规则。 | 仅在 `routing.shared_realtime_infrastructure = true` 时生效，范围很宽。 |
| `routing.public_encrypted_dns` | `ROUTE_PUBLIC_ENCRYPTED_DNS` | `false` | 路由公共 DoH/DoT 服务。 | 会影响共享 DNS 流量。 |

### 运行时行为

| TOML 键 | JavaScript 常量 | 默认值 | 作用 | 依赖或风险 |
| --- | --- | --- | --- | --- |
| `runtime.allow_final_rule_upstream_fallback` | `ALLOW_FINAL_RULE_UPSTREAM_FALLBACK` | `true` | 候选未命中时尝试当前 Profile 最后一个 `MATCH` / `FINAL` 目标。 | 目标仍需通过结构与递归校验。 |
| `runtime.allow_heuristic_upstream_fallback` | `ALLOW_HEURISTIC_UPSTREAM_FALLBACK` | `false` | 根据组名语义猜测上游。 | 仅在更早候选未命中时使用，可能选错出口。 |
| `runtime.preserve_unmanaged_nameserver_policy` | `PRESERVE_UNMANAGED_NAMESERVER_POLICY` | `false` | 保留订阅中脚本未托管的 `nameserver-policy`。 | 会放宽严格 DNS 重建边界。 |
| `runtime.enable_domain_sniffer` | `ENABLE_DOMAIN_SNIFFER` | `true` | 加固域名嗅探以补偿纯 IP 连接和 DNS 映射缺失。 | 不会全局改写目标地址。 |
| `runtime.harden_existing_tun_dns_hijack` | `HARDEN_EXISTING_TUN_DNS_HIJACK` | `true` | 为已经启用的 TUN 补齐 DNS 劫持项。 | 仅在 Profile 已启用 TUN 时生效。 |
| `runtime.enable_tun_strict_route` | `ENABLE_TUN_STRICT_ROUTE` | `false` | 为已有 TUN 开启 `strict-route`。 | 依赖 TUN 已启用且 `runtime.harden_existing_tun_dns_hijack = true`，可能影响虚拟机或特殊路由。 |
| `runtime.warn_on_reachable_udp_disabled` | `WARN_ON_REACHABLE_UDP_DISABLED` | `true` | 对可达叶子显式关闭 UDP 汇总为一条警告（最多 8 个样本）。 | 顶层上游禁用 UDP 仍会直接失败。 |

## 生成本地脚本

使用 `just` 时，首次运行会自动创建缺失的本地 TOML；填写后，以及每次修改 TOML 后，执行：

```bash
just render-local
```

没有 `just` 时，编辑本地 TOML 后执行等价的 Node 命令：

```bash
node scripts/sync-local-config.js
```

`render-local` 表示单向渲染，而非双向同步：它读取公开的 `clash-verge-ai-residential.js` 与本地 TOML，生成 `clash-verge-ai-residential.local.js`，不会修改公开模板。唯一的例外是缺失开关的自动补全：本地 TOML 缺少的开关键会按示例默认值写回本地文件，方便你看到并调整所有可用开关；用户已写的键值、注释与行尾风格不会被改写。不要手动编辑生成的 `.local.js`；修改 TOML 后重新生成即可。

在 Clash Verge Rev 中打开 **Profiles -> Global Extend Script**，双击脚本卡片，将**生成的本地脚本**全部粘贴并保存，然后刷新当前 Profile：

![Clash Verge Rev 的 Profiles 页面与 Global Extend Script 入口](../assets/clash-verge-rev-global-extend-script.png)

`just sync` 仍作为兼容别名保留，但新文档和自动生成文件均使用 `just render-local`。

### 从 Windows 复制到 Ubuntu

可以把 Windows 上由 `just render-local` 生成的 `clash-verge-ai-residential.local.js` 直接复制到 Ubuntu 的 Clash Verge Rev Global Extend Script；脚本本身不包含 Windows 路径、Shell 命令或操作系统分支。生成的 `.local.js` 已嵌入 TOML 中的住宅代理地址与认证凭据，本身也是敏感文件；应通过受信通道传输、限制读权限，且不得提交到仓库、上传到公开网盘或在日志中展示。复制已渲染的 `.local.js` 后无需也不应再复制本地 TOML，但这并不降低 `.local.js` 自身的凭据保护要求。

Ubuntu 的 Profile 仍必须能唯一解析脚本中的 `dialer-proxy` 名称，并提供可达的机场节点、UDP 能力和兼容的 Clash Verge/Mihomo 脚本环境。仓库的 Windows/Ubuntu Node 测试只验证语法、渲染与规则合同；复制后是否能在 Ubuntu 宿主执行、以及登录与模型请求是否命中同一出口，必须用脱敏 Connections 记录人工确认，目前为 **UNVERIFIED**。

同步会在写入前拒绝以下配置：未知或重复的表/键、非布尔开关、缺少代理字段、无效 TOML 字符串、非 SOCKS5 类型、端口超出范围、空上游名称，或 `name` 与模板保留名称不一致。每个开关必须在公开模板中恰好匹配一个布尔常量声明，否则也会在写入前失败。错误会直接显示字段或行号，且不会留下半成品；修正 TOML 后重新运行即可。

## 不保存凭据的模式

也可以保留 `server`、`username` 和 `password` 为 `"xxx"`，并在每个 Clash Profile 中预置同名的 `家宽-SOCKS5` 节点。运行时脚本会复用 Profile 中该节点的 endpoint 和凭据。无认证 SOCKS5 则在 TOML 中把 `username`、`password` 都改为 `""`。

无论使用哪种模式，都不要提交本地 TOML、生成的 `.local.js`、生成 Profile 或未脱敏的连接日志。

## 校验

运行全部公开模板检查与回归测试：

```bash
just ci
```

`just ci` 等价于 `npm run ci`。它不会读取或上传本地 TOML；本地生成脚本也会被模板安全扫描排除，以免凭据干扰公开仓库检查。完成后仍应在 Clash Verge Rev 中确认 `家宽-SOCKS5.dialer-proxy` 能解析到实际机场组，并从 Connections 验证 AI 请求命中 `AI-家宽`。
