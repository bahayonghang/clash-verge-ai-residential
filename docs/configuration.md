# 配置

本地凭据和开关写在被忽略的 `clash-verge-ai-residential.local.toml`，再用 `just render-local` 或 `node scripts/sync-local-config.js` 生成 `clash-verge-ai-residential.local.js`。不要改公开模板，也不要手改生成脚本；改 TOML 后重新渲染。

```toml
[home_proxy]
name = "家宽-SOCKS5"
type = "socks5"
server = "xxx"
port = 443
username = "xxx"
password = "xxx"
udp = true
dialer-proxy = "🚀节点选择"
```

生成的本地脚本和本地 TOML 都被 `.gitignore` 排除。受跟踪的起点是 `clash-verge-ai-residential.local.toml.example`。完整步骤、校验规则和开关表见 [本地配置](local-configuration.md)。

支持两种模式：

- 在忽略的本地 TOML 里填写真实 endpoint 和凭据，再把生成的本地脚本粘进 Clash Verge Rev。
- 在每个 Profile 里预置同名 `家宽-SOCKS5` 节点，把 `server`、`username`、`password` 留成 `xxx`；脚本复用该节点的 endpoint 和凭据。

无认证 SOCKS5 必须把 `username` 和 `password` 都设为 `""`。任一字段仍是 `xxx`、且 Profile 里没有同名节点补值时，脚本会 fail closed。

## Profile 上游候选

`dialer-proxy` 只接受一个代理或组名。候选数组是解析顺序，不是 Mihomo 配置值：

```javascript
const PROFILE_UPSTREAM_OVERRIDES = {
  "Profile A": ["🚀节点选择", "Proxy", "自动选择"],
  "Profile B": ["Proxy", "🚀节点选择", "自动选择"]
};
```

最终写入 `HOME_PROXY_TEMPLATE["dialer-proxy"]` 的值仍是跨 Profile 的首选默认。解析顺序见 [多 Profile](multi-profile.md)。

## 开关

`[routing]` 与 `[runtime]` 都是可选表，允许只覆盖需要改的键。同步时，本地 TOML 缺失的开关键（包括整个缺失的表）会按示例默认值补全并写回；已有键值、注释和行尾风格保持不变。默认值刻意压低家宽流量；打开共享依赖或进程级兜底会改变隐私、成本和范围。不要根据 `ROUTE_*` / `ENABLE_*` 前缀猜测键名。没有脱敏 Connections 证据时，保持共享依赖和进程级兜底关闭。步骤与校验见 [本地配置](local-configuration.md)。

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
| `routing.ai_process_fallback` | `ENABLE_AI_PROCESS_FALLBACK` | `false` | 为已知 AI 应用增加进程级兜底。 | 会把进程中的非 AI 请求一并路由。查找进程由脚本写到 Mihomo 顶层 `find-process-mode: always`，与本开关无关。写在 `profile:` 下的值内核不用。 |
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

## Clash Verge Rev 设置

建议的运行时设置：

- 规则模式。
- 若 Clash Verge Merge 仍把 `find-process-mode` 写在 `profile:` 下，请在 Mihomo YAML 顶层放 `find-process-mode: always`。内核不读 `profile.find-process-mode`。
- 需要系统级拦截或进程规则时启用 TUN。
- 在 Clash Verge Rev 的 TUN 设置中启用 DNS 劫持。TUN 已启用时，脚本还会补充 `any:53` 和 `tcp://any:53`。当前版本会在全局脚本运行后从设置页恢复 `tun` 和 `ipv6`；这两个字段以设置页为准，并在该页关闭 IPv6。
- 浏览器的私有/安全 DNS 若绕过系统解析器，应关掉。
- 选中的上游组不得解析到 `DIRECT`、`REJECT` 或家宽节点自身。
- 目标功能需要 UDP 时，选中的机场线路和住宅 SOCKS5 都必须支持 UDP。机场订阅节点若省略 `udp` 字段，Mihomo 默认禁用 UDP。
