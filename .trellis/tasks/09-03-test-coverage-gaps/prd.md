# 按业务风险补齐核心测试覆盖

## Goal

补齐 ResiWatch 里会改变核算、存储、告警或本机数据的未测分支。优先参数校验、Recovery 写禁、SQLite 事务回滚、loopback 与 CLI 删除确认。不测无业务后果的代码。

## Background

2026-09-03 对照规格与现有 `#[test]`。根扩展脚本 fail-closed 与域名正/负样本已密。ResiWatch 主路径（核算、报告 Top N、档案 LRU、告警触发、崩溃 kill_gate）已有回归。缺口集中在校验失败、Recovery 入口、未使用的 commit kill 点、CLI purge 门闩。证据：`research/coverage-audit.md`。

## Requirements

- R1 只加自动测试（及为让测试编译所需的 `pub(crate)` / 测试夹具）。不改路由域名、公开模板凭据、schema、IPC 字段名。
- R2 按风险从高到低补模块；每完成一个模块跑该模块对应测试，最后跑完整套件。
- R3 每个新测试必须断言可观察契约：错误码、未写入的行数、回滚后库状态。禁止只读常量或只匹配源码字符串。
- R4 若新测试证明规格已写明的 fail-closed 被打破，在本任务内做最小产品修复，并在任务 notes 记下文件与原因。
- R5 测试继续用现有框架：Rust `#[cfg(test)]` + `tempfile`；前端若触及则 vitest；根脚本若触及则 `node:test`。不引入 coverage 工具或第三方测试库。
- R6 不跑本机 Credential Manager、NSIS、30 天库、真实自启动。

## Acceptance Criteria

- [ ] AC1 `validate_address` / `validate_targets` 覆盖：合法 `127.0.0.1` / `localhost` / `[::1]`；拒绝 `8.8.8.8`、`0.0.0.0`、无端口、超长地址、空目标、超长目标名、超过 `TARGET_COUNT_MAX`。错误为现有 `SettingsError` 变体，不是泛存储失败。
- [ ] AC2 `validate_query` 覆盖：区间倒置、超过 `MAX_RANGE_SECS`、`page.limit` 为 0 或大于 `PAGE_MAX`、`top_n` 为 0 或大于 `TOP_N_MAX`、未知时区、cursor 含 `;` 或 `select`。`code()` 为 `invalid_query`。
- [ ] AC3 `query_residential_share` / `validate_share_range` 对倒置区间与超大区间返回 `invalid_query`，不返回把缺口写成 0 的份额。
- [ ] AC4 `validate_rule` 覆盖空/过长 rule id、负 cooldown、非正阈值、缺 direction/recovery、滞回不小于阈值、Health 选择器错误、PeriodUsage 缺 period/timezone。`AlertEngine::load_rules` 在启用规则超过 `MAX_ENABLED_RULES` 时 `invalid_rule`。
- [ ] AC5 静默窗口：`start > end` 的跨日区间在窗口内不激活；窗口外可按既有三连击激活。
- [ ] AC6 `BootBranch::RecoveryOnly` 下 `run_report`、`save_targets`、`upsert_alert_rule`、`create_backup` 返回 `code == "recovery_only"`，SQLite 无对应新行。
- [ ] AC7 `CommitKillPoint::AfterFacts` 与 `AfterOutbox` 回滚后 `committed_bundle`、`connection_minute`、`alert_event`、`notification_outbox` 计数均为 0。`AfterAlerts` 现有测试保持。
- [ ] AC8 `maint::run_purge`：无 `--offline-confirmed` fail closed；有离线无 `--phrase` 且 `--confirm` 时 `InvalidArgs`；错误短语不删文件（可复用 `c5::confirm_delete` 行为）。
- [ ] AC9 每个模块窄测通过后，`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace` 与 `just ci` 退出码 0。
- [ ] AC10 不提交 `*.local.toml` / `*.local.js`；公开模板凭据仍为占位。

## Out of scope

- 为提高行覆盖率测试 UI 组件、i18n 词典、图标、Radix 封装。
- 根脚本新增域名正向表（已有正/负样本）。
- C5 30 天库、24 小时 soak、本机 Credential Manager、NSIS 安装态、真实 HKCU 自启动。
- 引入 llvm-cov / istanbul / 新测试框架。
- 改 C1/C3/C4 已发布 migration 文本。
- 前端把 `use-alerts` 的 `as AlertRule[]` 升级成严格解码（Rust 仍是权威校验者）。

## Key decisions

- 单任务，不拆父子。交付是一组按模块落地的回归，不是多个独立产品面。
- 根扩展与前端 DTO 本轮不扩，除非实施中发现与上表同级的 fail-closed 空洞。
- 产品代码默认不动。只有新测试打到规格已写明、现码违反的契约时才改生产实现。
