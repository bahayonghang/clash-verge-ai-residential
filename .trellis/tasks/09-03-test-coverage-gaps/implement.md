# Implement：按模块补测试

## 前置

- 用户批准本规划摘要之后才能 `task.py start`。
- 不改 `*.local.toml` / `*.local.js` / 公开模板凭据。
- 不改 `ref/neko-master/`。
- 每模块先窄测，再进入下一模块。最后 `just ci`。

## 顺序

### M1 — 地址与目标校验

文件：`residential-monitor/src-tauri/src/c2/settings.rs`

- 扩展 `settings_workflow_tests`。
- 窄测：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c2::settings::`

### M2 — 报告 `validate_query`

文件：`residential-monitor/src-tauri/src/c3/query.rs`

- 在 `query_contract_tests` 增加分断言（区间、limit、top_n、时区、cursor `;`）。保留现有 cursor/`select` 测试。
- 窄测：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c3::query::query_contract_tests`

### M3 — 家宽份额区间

文件：`residential-monitor/src-tauri/src/c3/share.rs`

- 倒置与 `MAX_RANGE_SECS+1`。断言 `error.code() == "invalid_query"`。
- 窄测：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c3::share::`

### M4 — 告警规则与静默

文件：`c4/types.rs`、`c4/engine.rs`

- `validate_rule` 表驱动错误分支。
- `load_rules` 257 条 enabled rate 规则 → `InvalidRule("too many rules")`。
- `in_quiet`：`quiet_start_min=22*60`、`quiet_end_min=6*60`，窗口内三连击无 `Activated`；窗口外仍三连击激活。时区 `UTC`，避免本机 DST。
- 窄测：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c4::`

### M5 — Recovery 写禁

文件：`c2/facade.rs`

- 辅助函数构造 `RecoveryOnly` facade（复制现有 `user_version=99` 写法）。
- 四个入口 + SQLite 无新行。
- 窄测：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c2::facade:: -- recovery`

### M6 — commit kill 点

文件：`storage.rs`

- `AfterFacts`、`AfterOutbox` 各一测，断言 receipt/minute/event/outbox 为 0。
- 窄测：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib storage:: -- kill_`

### M7 — CLI purge 门闩

文件：`dbcli/mod.rs`

- `run_purge` 无 `offline_confirmed`；有离线+confirm、无 phrase。
- 窄测：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib dbcli::`

### 收尾

- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`
- `just ci`
- 若只改测试且 `just ci` 绿，CHANGELOG 可写 Unreleased 一条英文 test 记录；无产品行为变化则可不写。

## 风险文件

| 文件 | 风险 | 回退 |
|---|---|---|
| `c2/settings.rs` | 误改 `validate_address` 生产逻辑 | 只留测试模块 diff |
| `c2/facade.rs` | Recovery 测试偶发依赖本机时间 | 用固定 `user_version` 夹具 |
| `storage.rs` | kill 点语义被改 | 只加测试，不改 `CommitKillPoint` 分支 |
| `c4/engine.rs` | 静默测试被 DST 打偏 | 强制 `timezone: "UTC"` |
| `dbcli/mod.rs` | purge 测到真实数据目录 | 只用 `tempfile` 种子库 |

## 产品修复门槛

仅当新测试失败且失败原因是规格已写明的 fail-closed 被违反。允许的修复示例：Recovery 入口先检查 `storage` 再改内存。禁止顺手重构校验函数。
