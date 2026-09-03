# 设计：按风险补测试

## 边界

- 测试落在既有模块的 `#[cfg(test)]` 旁，不新建并行测试 crate（`kill_gate.rs` 集成测除外，本任务不改它）。
- 生产函数签名保持。需要从测试调用的校验函数已经是 `pub`。
- Channel / Tauri command 薄封装不单测；断言 `AppFacade` 与纯函数，与现有 facade 测试一致。
- C2 测试仍不得 `use rusqlite` 写业务 SQL。storage / c4 store 测试可以。

## 模块与落点

| 模块 | 文件 | 夹具 |
|---|---|---|
| M1 地址与目标 | `c2/settings.rs` `settings_workflow_tests` | 无需 SQLite |
| M2 报告查询 | `c3/query.rs` `query_contract_tests` | `ReportQuery::default()` 改字段 |
| M3 家宽份额区间 | `c3/share.rs` `residential_share_tests` | 现有 `setup()` 种子库；坏区间在打开库前就会因校验失败，可不依赖种子内容 |
| M4 告警规则与静默 | `c4/types.rs` 新增 `validate_rule_tests`；`c4/engine.rs` `alert_engine_tests` | `rate_rule()` 变体；静默用固定 UTC |
| M5 Recovery 写禁 | `c2/facade.rs` 现有 Recovery boot 模式 | `user_version=99` 触发 `RecoveryOnly`（已有先例 `ui_sidebar_width_without_storage_stays_in_memory`） |
| M6 事务 kill | `storage.rs` 紧挨 `kill_after_alerts_rolls_back_facts_and_outbox` | 复用该测试的 `slice()` / `CommitBundle` |
| M7 CLI purge | `dbcli/mod.rs` `tests` | `tempfile` + `MaintFlags`；不真正删用户数据目录 |

## 契约细节

### 地址解析

`validate_address` 用 `rsplit_once(':')` 再 `trim_matches('[',']')`，然后 `reject_non_loopback` 只放行 `127.0.0.1` / `::1` / `localhost`。测试必须包含：

- `127.0.0.1:9097`、`localhost:9097`、`[::1]:9097` 成功
- `127.0.0.2:9097` 失败（字符串白名单，不是 `IpAddr::is_loopback()` 的整个 127/8）
- `0.0.0.0:9097`、`8.8.8.8:9097`、`example.com:9097` → `NonLoopback`
- `not-an-addr`、`127.0.0.1`（无端口）→ `InvalidAddress`
- 长度 `SETTING_VALUE_MAX+1` → `FieldTooLong`

### 报告查询

`ReportError::code()` 对所有 `InvalidQuery(_)` 都是 `"invalid_query"`。断言 `code()` 与失败，不依赖英文细节字符串作为唯一门。`MAX_RANGE_SECS = 400 * 86400`。cursor 已有 `select` 样本；补 `;` 单独样本。

份额路径走 `query_residential_share` 公开入口，保证 IPC/facade 同一函数被挡住。

### Recovery

构造：打开路径、`pragma user_version = 99`、再 `AppFacade::boot`。断言 `branch == RecoveryOnly` 且 `storage.is_none()`。调用四个写入口，断言 `code == "recovery_only"`。对 `save_targets` / `upsert_alert_rule` 另用 `rusqlite` 只读打开同一文件，确认 `target_item` / `alert_rule` 无新行（C2 测试文件已有 rusqlite 先例：`ui_sidebar_width_without_storage`）。

`upsert_alert_rule` 当前先改内存引擎再取 storage。Recovery 进程不会随后采集；restore 会重新 boot。本任务以「无 SQLite 行 + 错误码」为验收。若发现 durable 写入，才改生产顺序。

### 事务

`test_kill` 已是 `#[cfg(test)]` 字段。AfterFacts 在 `persist_slice` 成功之后注入；AfterOutbox 在 `persist_intents` 之后注入。重开 `StorageCoordinator` 计数。不改 kill 点语义。

### CLI purge

`run_purge` 先 `require_offline`。无离线时不应检查短语。有离线、`confirm=true`、无 phrase → `InvalidArgs`。错误短语路径可直接测 `confirm_delete`（已有 `wrong_phrase_does_not_delete`）；本模块补 `run_purge` 的离线门，避免只测底层漏掉 CLI 包装。

## 兼容

- 不新增 npm/Cargo 依赖。
- Windows 本机 `to_socket_addrs` 对 `localhost` 可能解析 IPv6；`reject_non_loopback` 在解析前已放行 host 名，测试不应依赖解析到哪条地址。
- `just ci` 含 `cargo test --workspace` 与前端 vitest；本任务若只改 `.rs`，vitest 仍应保持绿。

## 回滚

每个模块的测试是独立 `#[test]`。失败时删除或修复该模块新增测试，不影响其它模块。
