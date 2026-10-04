# 物化事务、规则同事务、语句复用

## Goal

daily/core 物化与告警规则写入与 facts/outbox 一样在同一 Immediate 事务中失败即全回滚。每 tick persist 复用 prepared statement，避免对每行 `prepare`。

## Background

证据：`retention.rs:614-669` daily/core 无外层事务；`facade.rs:1404-1435` `upsert_rule` 先写规则再 `commit_alert_bundle`；`storage.rs:590-658` 每行 `execute` 新语句。规格：逐行 bind → step → reset；facts/coverage/alert/outbox 同一 writer 事务。

依赖：须在 `09-03-sqlite-writer-txn-fail-closed` 之后改 `storage.rs`。

## Requirements

- R1 daily + core + coverage 物化包在一个 `BEGIN IMMEDIATE`，失败 rollback。
- R2 告警规则行的持久化进入 `commit_alert_bundle` / `persist_slice`，不得在 bundle 外先 commit。
- R3 `persist_slice` / intern 使用 `StorageCoordinator` 上缓存的 statement，bind → step → reset。

## Acceptance Criteria

- [ ] AC1 core 第二句 insert 失败：daily/core/coverage 均无该窗口新行。
- [ ] AC2 `upsert_alert_rule` 在 bundle commit 失败后：`alert_rule` 无新版本行，内存引擎与库一致（或引擎回滚）。
- [ ] AC3 提交路径不再对每条 live row `prepare`（可用测试计数或代码结构断言 statement 字段存在）。
- [ ] AC4 既有 commit kill 点与 hourly 物化测试仍绿。

## Out of scope

- 自动 DELETE / VACUUM。
- 改 schema version。
- 子任务 1 已覆盖的 `commit_inner`/`save_targets`。
