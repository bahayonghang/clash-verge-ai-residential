# 写入事务失败必须回滚

## Goal

writer 连接在 `BEGIN IMMEDIATE` 之后任何错误都 rollback，避免后续 `begin` 失败导致采集停写。`save_targets` 的 bump / delete / insert 必须同一事务。

## Background

证据：`storage.rs:355-427` 在 `persist_slice` 之外的 `execute` 失败不 rollback；`storage.rs:533-549` 无 `BEGIN`。规格要求单 writer、失败 fail-closed。

## Requirements

- R1 `commit_inner` 在 `begin immediate` 之后，所有 `Err` 路径执行 `rollback`（含 `committed_bundle` insert、watermark、`commit` 本身失败）。
- R2 `save_targets` 三步包在 `BEGIN IMMEDIATE`；任一步失败 rollback，`policy_version` 与 `target_item` 保持进入前状态。
- R3 用测试夹具在 insert 失败或 kill 点后断言连接可再次 `begin immediate`，且行数回滚。

## Acceptance Criteria

- [ ] AC1 在 `begin` 之后、`persist_slice` 之前注入失败：返回错误，随后 `begin immediate` 成功，`committed_bundle` 无新行。
- [ ] AC2 `save_targets` 在 delete 之后 insert 失败：`target_item` 仍是旧集合，`policy_version` 未增加。
- [ ] AC3 既有 AfterFacts / AfterAlerts / AfterOutbox kill 测试仍通过。
- [ ] AC4 `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib storage::` 退出 0。

## Out of scope

- `persist_slice` N+1 / prepared statement（子任务 10）。
- daily/core 物化事务（子任务 10）。
- 改 C1/C3/C4 已发布 migration 文本。
