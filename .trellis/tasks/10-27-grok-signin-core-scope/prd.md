# 修复 Grok 登录核心覆盖与路由依据

## Goal

修复用户在本次审查后明确要求修复的两个问题：关闭 Grok 网页资源路由时仍保留官方必需登录主机 auth.grok.com 的家宽路由和住宅 DNS；更正把 TLS 通配证书当作路由范围依据的说明。

## Authorization

用户请求“请创建trellis任务修复这两个问题”，明确授权创建任务与实施这两项修复。先持久化并审阅本计划，再 start 和派发实施。用户后续请求“请提交改动并归档任务”，明确授权提交本次改动、归档任务及记录会话。未授权推送、安装、发布、修改本地 TOML、生成本地 JS 或修改运行中代理配置。

## Requirements

- grok_core=true 时，无论 grok_web_assets 值如何，auth.grok.com 均有精确 DOMAIN 规则和裸主机名住宅 DNS 键。
- grok_core=false 时，清理该主机已生成的精确规则和 DNS；无 grok.com 家宽后缀残留。保留非托管用户规则。
- grok_web_assets=false 时不新增 grok.com 后缀，不带入 assets.grok.com、connectors-gateway.grok.com 或 auth.grok.com 的子域。
- 依据为官方最小登录业务名单与最小流量原则；TLS 握手只说明被测试路径的证书验证结果，不能决定路由范围或证明未测路径可用。
- 中文/英文配置文档、范围说明、README、CHANGELOG 与注释保持一致；不改已归档的历史证据。

## Acceptance

- [x] 修复前新增的行为测试失败；修复后通过。
- [x] 覆盖 grok_core 与 grok_web_assets 四种组合的规则/DNS、正负主机。
- [x] 从开启时的输出关闭开关后旧规则/DNS被清掉，重复执行幂等，保留用户自定义规则。
- [x] 默认明确增加一条 DOMAIN,auth.grok.com,AI-家宽，默认规则数量为 60；固定历史投影不被改写。
- [x] 下游 build-inputs 将 auth.grok.com 归属 grok_core，而非 grok_web_assets 独有流量。
- [x] npm run ci、文档构建通过；canonical just ci 通过，使用已安装默认工具链与临时 target 绕开本机包装器/链接故障，未降低检查门。
- [x] 不改任何 *.local.toml / *.local.js / 运行配置，不推送；提交和归档遵循用户后续明确授权。

## Implementation boundary

轻量修复，沿用 GROK_EXACT_DOMAINS 和现有生成/清理函数，不新增抽象、依赖或开关。允许修改 clash-verge-ai-residential.js、tests/regression.test.js、tests/install-agent-skills.test.js、README.md、CHANGELOG.md、docs/{configuration,local-configuration,routing-scope}.md、对应 docs/en/ 文档。仅当生成脚本测试需要时可修改 tests/sync-local-config.test.js。主会话负责当前任务文件和 .trellis/spec/frontend/quality-guidelines.md 的窄范围约定补充。

## Evidence

- 当前会话只读审查已通过 VM 执行父提交和当前提交，确认 strict 模式 auth.grok.com 缺少路由和 DNS。
- https://docs.x.ai/x-subscription/sign-in-domains 当前会话已读取，最小名单包括 auth.grok.com；https://docs.x.ai/x-subscription/grok-bot-sign-in 说明 Grok Bot 使用共享 xAI 登录。
- 原始来源：.trellis/tasks/archive/2026-10/10-26-grok-bot-residential-hosts/research/grok-bot-live-hosts.md。该历史证据不重写。
- 本会话此前 npm run ci 260 测试与文档构建通过；just ci 的 Rust 步骤受本机 cargo wrapper / rustup 链接启动异常阻塞。
