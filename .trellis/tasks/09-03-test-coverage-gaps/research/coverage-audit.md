# 测试覆盖审计（2026-09-03）

范围：根扩展脚本、`tests/`、ResiWatch 前端 vitest、`src-tauri` Rust 单测与 `tests/kill_gate.rs`。
方法：对照规格契约与 `#[test]` / vitest 名称，不引入 coverage 工具，不把 getter/常量当缺口。

## 已有密度（不做覆盖率冲刺）

- 根 `npm test`：路由 fail-closed、DNS、幂等、占位凭据、本地渲染、模板扫描。`tests/regression.test.js` 已覆盖上游循环、UDP 拒绝、`xxx` 凭据、进程路由默认关。
- ResiWatch Rust：C1 核算、C2 facade/hub/collector、C3 报告/档案/retention/backup、C4 引擎触发/滞回、C5 purge/vacuum、storage `kill_after_alerts` 与 `kill_gate` 进程崩溃，均有回归。
- 前端：DTO/Channel 解码、报告请求序号、IPC 边界（`components/**` 不 `invoke`）、删除短语、自启动 fake。

## 高风险缺口（本任务范围内）

| ID | 契约 | 证据 | 现有测试 | 风险 |
|---|---|---|---|---|
| G1 | 控制器地址只接受 loopback；目标名/数量有上限 | `c2/settings.rs:84-114` `validate_address` / `validate_targets` | 仅 `rejects_non_loopback`（`8.8.8.8:9090`） | 非 loopback 或超长目标进入采集 |
| G2 | 报告查询拒绝坏区间、分页、Top N、时区、SQL 形 cursor | `c3/query.rs:747-766` `validate_query` | 仅 cursor 含 `select` 与 `range_end == start`（`query.rs:1030-1037`） | 超大窗口 / `limit=0` / 未知时区打进 SQL |
| G3 | 家宽份额同样拒绝坏区间 | `c3/share.rs:49-61` `validate_share_range` | 份额守恒与缺口 None，无 InvalidQuery | 家宽页扫 400 天以上 |
| G4 | 告警规则参数与启用上限 | `c4/types.rs:234-274` `validate_rule`；`c4/engine.rs:147-155` `MAX_ENABLED_RULES=256` | 引擎测触发/滞回，不测 InvalidRule 各分支 | 无效规则经 IPC 写入 |
| G5 | 跨日静默窗口 | `c4/engine.rs:773-785` `in_quiet` | 无 | 夜间规则误触发或该静不静 |
| G6 | Recovery Shell 不得写报告/目标/告警/备份 | `c2/facade.rs` `ok_or_else(recovery_only)` | 只断言 boot 保持 `RecoveryOnly`，不调这些入口 | 损坏库上误写 |
| G7 | 同一 writer 事务：facts / alerts / outbox 中途失败整笔回滚 | `storage.rs:355-422` `CommitKillPoint::{AfterFacts,AfterAlerts,AfterOutbox,BeforeCommit}` | 仅 `kill_after_alerts_rolls_back_facts_and_outbox` | 部分提交导致告警与用量分叉 |
| G8 | `monitor-db maint purge` 需要 `--offline-confirmed` 与 `--phrase` | `dbcli/maint.rs:288-365` | `vacuum_requires_offline_confirmed`；purge 短语/离线未测 | CLI 无确认删库 |

## 明确不做

- 根脚本再堆正向域名表测试（已有正/负样本）。
- 前端 Radix/布局/色井、`use-alerts` 里 `as AlertRule[]` 弱解码（规格写明 Rust 是权威校验者）。
- C5 30 天库、24h soak、本机 Credential Manager、NSIS 安装态。
- `c2/dialog.rs` Tauri 对话框适配器、`c2/contract.rs` 常量模块。
- 为提高行覆盖而测 `unwrap_or_else(|_| "{}")` 一类序列化兜底。

## 规格锚点

- `.trellis/spec/residential-monitor/backend/modules-and-errors.md`：TCP 只接受 loopback；Recovery 失败保留当前库；错误码稳定。
- `.trellis/spec/residential-monitor/backend/secrets-and-cancellation.md`：C3 操作可取消；CLI purge 需 `--offline-confirmed`。
- `.trellis/spec/residential-monitor/storage/sqlite-contract.md`：facts/coverage/alert/outbox 同一 writer 事务；低空间与未来 schema fail closed。
