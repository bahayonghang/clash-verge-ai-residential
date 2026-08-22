# 新增 Cursor 与 Grok Build 家宽路由及 TOML 键值自动补全

## Goal

1. 依据官方文档与 wire-level 分析，把 Cursor 与 Grok Build（xAI grok CLI）
   使用时连接到的域名纳入 `clash-verge-ai-residential.js` 的家宽路由清单，
   并在 `clash-verge-ai-residential.local.toml.example` 中提供两个默认打开
   的开关：`routing.cursor_core`（默认值从 `false` 改为 `true`）与新增的
   `routing.grok_core = true`。
2. 优化本地配置同步（`just render-local` / `scripts/sync-local-config.js`）：
   当本地 TOML 缺少 example 中已定义的键值对时自动补充，缺失的整个表也要
   补齐，同时完整保留用户已有键值、注释与行尾风格。

## Background

### Cursor 域名证据

- 官方企业网络配置文档
  （cursor.com/docs/enterprise/network-configuration）列出：
  - `api2.cursor.sh`：多数 API 请求（现有 suffix 已覆盖）
  - `api3.cursor.sh`：Cursor Tab（现有 exact 已覆盖）
  - `api4.cursor.sh`：Tab 区域端点（现有 exact 已覆盖）
  - `api5.cursor.sh`：Agent 请求与网络访问层 NAL（现有 suffix 已覆盖，
    含 `agent.api5.cursor.sh` 等 NAL 子域）
  - `us-asia/us-eu/us-only.gcpp.cursor.sh`：Tab 区域端点（现有
    `gcpp.cursor.sh` suffix 已覆盖）
  - `repo42.cursor.sh`：代码库索引（现有 regex `^repo[0-9]+\.cursor\.sh$` 已覆盖）
  - `authenticate.cursor.sh`：授权端点 —— **当前清单缺失**
  - `adminportal42.cursor.sh`：SSO 配置与域验证 —— **当前清单缺失**
  - `marketplace.cursorapi.com`、`cursor-cdn.com`、`downloads.cursor.com`、
    `anysphere-binaries.s3.us-east-1.amazonaws.com`、`*.cursorvm.com`：
    市场/CDN/更新/二进制/VM 服务
- 现有清单 `CURSOR_SUFFIX_DOMAINS` / `CURSOR_EXACT_DOMAINS` /
  `CURSOR_DOMAIN_REGEXES`（clash-verge-ai-residential.js:223-242）缺少
  `authenticate.cursor.sh`、`adminportal<N>.cursor.sh` 与 Cloud Agent VM
  域 `cursorvm.com`。

### Grok Build 域名证据

- wire-level 分析（grok 0.2.93，gist.github.com/cereblab/dc9a40bc26120f4540e4e09b75ffb547，
  经 Hacker News item 48877371 交叉确认）观察到：
  - `cli-chat-proxy.grok.com`：主推理 API（`POST /v1/responses`）、
    代码库/会话轨迹上传（`/v1/storage*`）、设置拉取
  - `grok.com`：事件遥测（`/_data/v1/events`）与隐私设置页
  - `storage.googleapis.com`：代码库上传的实际 GCS 后端
    （与 Claude Code 辅助清单中已有的主机相同）
  - `api.mixpanel.com`：第三方分析遥测
  - `x.ai`：安装脚本与隐私端点
- 当前脚本完全没有 Grok 相关清单与开关。

### 同步功能现状

- `justfile` 的 `render-local` 调用 `scripts/sync-local-config.js`：
  解析本地 TOML → 校验 → 渲染出 `clash-verge-ai-residential.local.js`。
- 本地 TOML 缺少开关键时（parseLocalToml 允许缺键），当前行为是
  `injectBooleanConstants` 静默跳过（scripts/sync-local-config.js:352-372），
  模板常量保持 JS 内置默认值：用户在本地 TOML 中看不到新键的存在，
  也不知道有新开关可用。
- `[home_proxy]` 缺键时 `validateHomeProxyConfig` 直接报错。
- 本任务为 example 新增 `grok_core` 并修改 `cursor_core` 默认值后，
  所有已存在的本地 TOML 都将缺键，正好放大该问题。

## Requirements

- R1. `CURSOR_*` 清单补充 `authenticate.cursor.sh`（suffix）、
  `adminportal[0-9]+.cursor.sh`（bounded regex）与 Cloud Agent VM 域
  `cursorvm.com`（suffix），并保持 docs/routing-scope.md 的验收规则：
  市场/CDN/更新/下载/分析类域名不进家宽。
- R2. 新增 Grok 清单与 `ROUTE_GROK_CORE` 常量：核心域为
  `grok.com`（suffix，覆盖 `cli-chat-proxy.grok.com` 推理 API 与 Grok
  产品域，和 chatgpt.com/claude.ai 的产品域处理一致）；Grok 使用的
  共享第三方（mixpanel、storage.googleapis.com、x.ai）默认不进家宽，
  以注释与文档记录。
- R3. `ROUTE_CURSOR_CORE` 默认值 `false` → `true`；example TOML
  `routing.cursor_core = true`；新增 `routing.grok_core = true`。两个
  开关均默认打开，且参与规则、DNS nameserver-policy、托管规则清理集合
  （active*/allPossible* 全套函数）。
- R4. `scripts/sync-local-config.js` 在同步时自动补全本地 TOML 缺失的
  开关键（含整个缺失的 `[routing]` / `[runtime]` 表）：默认值取自 example
  文件（单一事实来源），按 SWITCH_CONFIG_FIELDS 顺序插入，保留用户已有
  值、注释与行尾风格，幂等（无缺失时不改写文件），原子写入。
- R5. `[home_proxy]` 缺键不自动补全（含凭据，必须由用户手填），保持
  现有报错行为。
- R6. 回归测试覆盖：新 Cursor/Grok 域名命中规则与 DNS policy、默认
  开关状态、负向边界（marketplace/CDN/更新/mixpanel/x.ai 不进家宽）、
  幂等重建；sync 测试覆盖缺失键补全、缺表补全、完整文件不被改写、
  home_proxy 缺键仍报错、CRLF 保持。
- R7. 更新 README 与 docs/routing-scope.md、docs/local-configuration.md
  中与 Cursor/Grok 及同步行为相关的描述；版本号 5.5.0 → 5.6.0 并补
  CHANGELOG 条目。

## Acceptance Criteria

- [x] `ROUTE_CURSOR_CORE = true`、`ROUTE_GROK_CORE = true` 时：
  `authenticate.cursor.sh`、`adminportal42.cursor.sh`、`*.cursorvm.com`、
  `grok.com`、`cli-chat-proxy.grok.com` 的 DOMAIN/DOMAIN-SUFFIX/DOMAIN-REGEX
  规则指向 `AI-家宽`，且对应 nameserver-policy 指向 `RESIDENTIAL_DOH`。
- [x] 开关关闭时上述域名不产生规则与 DNS policy；重复执行 main() 不产生
  重复规则；曾开启后关闭的托管规则仍能被精确清理。
- [x] `marketplace.cursorapi.com`、`cursor-cdn.com`、`downloads.cursor.com`、
  `anysphere-binaries.s3.us-east-1.amazonaws.com`、`api.mixpanel.com`、
  `x.ai` 默认不进家宽（负向测试）。
- [x] example TOML 包含 `cursor_core = true` 与 `grok_core = true`，且
  与 JS 模板常量默认值一致。
- [x] 本地 TOML 缺少 `grok_core` 或整个 `[runtime]` 表时，运行同步后
  本地 TOML 被补全（值来自 example），用户已有键值与注释逐字保留，
  渲染产物正确；再次运行不再改写 TOML 文件。
  已用开发者真实本地 TOML 验证：仅追加缺失的 `grok_core = true` 一行，
  用户显式 `cursor_core = false` 被保留。
- [x] 本地 TOML 完整时同步不改动该文件（幂等）。
- [x] `[home_proxy]` 缺键仍报错并指向用户手填。
- [x] `npm run ci` 全绿（check + test + check:secrets），48 个测试通过。
- [ ] 在 Clash Verge Rev 中用真实 Profile 从 Connections 验证新增域名
  （authenticate.cursor.sh / adminportal42.cursor.sh / cursorvm.com /
  grok.com）实际命中 `AI-家宽`。当前状态：`UNVERIFIED`——需要宿主
  环境手工验证，Node 测试不能替代。

## Key Decisions

- 「2 个相关选项」= `routing.cursor_core`（已有键，默认值改为 `true`，
  因为用户要求 Cursor 相关路由默认打开，否则新增域名不会生效）与新增
  `routing.grok_core = true`。不新增第三个开关（如 grok 遥测），Grok 的
  共享第三方域名以注释 + 文档记录，与 v5.5 的 AI-only 哲学一致。
- Cursor 市场与更新类域名虽然出现在官方网络文档中，但按
  docs/routing-scope.md 验收规则属于排除类，不纳入 cursor_core。
- `grok.com` 采用 suffix 而非仅 exact `cli-chat-proxy.grok.com`：
  与 claude.ai / chatgpt.com / gemini.google.com 的产品域处理保持一致，
  同主机遥测与推理无法在域名层拆分。

## Out of Scope

- Cursor/Grok 进程级全量代理（PROCESS 规则开关维持现状默认关闭）。
- Grok 遥测/安装域名（api.mixpanel.com、x.ai）与共享存储
  （storage.googleapis.com）的家宽路由开关。
- 修改真实住宅代理地址、凭据或用户本地生成的文件。
- 重写 TOML 解析器或引入第三方 TOML 依赖（保持零依赖、文本级补全）。
