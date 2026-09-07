# 结构与变更边界

审查基线：`5578576787dba73ba1b96985a2fc408fc246bfed`，2026-09-07。

| 区域 | 关键入口/权威源 | 关键契约 | 当前检查 |
|---|---|---|---|
| 可粘贴 Clash 扩展 | `clash-verge-ai-residential.js:1767` 的 main；`:1517` 的 buildInjectedRules | 接收并克隆配置，校验后返回；唯一家宽成员；core/process/IP 开关与托管清理；不能引入 Node-only 宿主 API | 根 125 tests，公开模板 scanner |
| 本地渲染 | `scripts/sync-local-config.js:508` 的 syncLocalConfig；`:25` 起开关映射 | 公开模板+忽略的 TOML→忽略的 local.js；凭据不公开；失败不落半成品 | sync-local-config suite |
| ResiWatch 桌面壳 | `residential-monitor/src-tauri/src/lib.rs:1352` run / `:1424` invoke_handler；`c2/facade.rs:218` AppFacade / `:387` boot | Rust 拥有数据、凭据与生命周期；boot/recovery-only；Tauri IPC | fmt/clippy/workspace tests；真机另验 |
| React 视图 | `residential-monitor/src/App.tsx:64` App；`src/hooks/use-report.ts:150` useReport；`src/ipc/decoder.ts:189` decodeMonitorMessage | hooks 进入 IPC，边界解码；过期响应丢弃；未知不写零 | 69 files / 284 tests + typecheck/lint/build |
| 报告与核算 | `src-tauri/src/c3/query.rs`；`c3/share.rs:40` query_residential_share_on；`storage.rs`；`dbcli/` | SQLite 权威账本；核算口径与筛选口径区分；rank 不能当全量 audit | Rust workspace；30 天容量/现场仍 UNVERIFIED |
| 文档 | `CONTEXT.md`、`docs/adr/`、`docs/*.md`、`docs/en/`、`residential-monitor/docs/` | 词汇与既有决策；中英文配置与能力声明一致 | VitePress build（独立于 ci） |
| 项目协作 | `AGENTS.md`、`CLAUDE.md`、`.trellis/workflow.md`、`.trellis/spec/` | 创建任务≠批准实施；强模型审查、显式任务边界 | task validate + 人工验收追溯 |
| 业务 skill | `skills/residential-rule-tuning/`、`scripts/install-agent-skills.js` | 源库→项目本地副本；默认拒绝覆盖差异，force 先备份 | source tests 通过；实际 installed --check 失败 |

全仓审查采用关键入口和测试/工作流追踪，并非声称逐行验证所有 Rust/React 实现。`09-03-engineering-review-remediation` 及其子任务和 `09-07-js-routing-egress-optimization` 已归档；没有新证据时不重开其已修复问题。

## 本轮改造原则

保持单文件宿主结构、SQLite 所有权与既有 IPC 分层。当前基线没有复现产品单元测试失败；优先修正质量门和规则交付的实际断点，不为“常青”新增平台化运行时、包体系或兼容抽象。
