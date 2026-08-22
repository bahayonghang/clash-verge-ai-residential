# 还原、vacuum、删除后重跑存储侧启动

父任务：`08-22-dev-main-merge-unblock`。

## Goal

`restore_backup`、`confirm_delete_local_data`、`run_user_vacuum` 重开数据库后，`AppFacade` 的存储派生状态与新数据库一致。还原后的第一帧提交不再因陈旧 `writer_epoch` 落到 `PayloadMismatch`。

## Background

`boot()` 打开数据库后按序建立一批存储派生状态（`c2/facade.rs:227-320`）：

| 状态 | 来源 | 行 |
|---|---|---|
| `writer_epoch` | `storage.reserve_writer_epoch()` | `:234` |
| `settings` | `get_setting("controller")` | `:246` |
| `wizard_complete` | `get_setting("wizard_complete")` | `:252` |
| `ui_locale` / `ui_theme` / `ui_font` / `ui_font_size` / `ui_density` / `ui_sidebar_width` / `live_table_layout` | 各自 setting key | `:258-301` |
| `engine` targets | `storage.load_targets()` | `:303` |
| `alerts` 规则 | `c4::store::load_rules` | `:309` |
| `alerts` 实例 | `c4::store::load_instances` | `:312` |

三个恢复类方法都只改 `storage` 与 `branch`，不重跑上表任何一项：

- `restore_backup`（`:1514-1543`）：`storage = None` → 还原文件 → `StorageCoordinator::open` → `storage = Some(_)`、`branch = NormalReady`。
- `confirm_delete_local_data`（`:1554-1597`）：同形，`:1577-1579`。
- `run_user_vacuum`（`:1599-1622`）：同形，`:1612-1615`。

### `writer_epoch` 的失败路径

`reserve_writer_epoch`（`storage.rs:452-477`）从数据库取 `max(writer_epoch)`，加一，并向 `bundle_epoch` 插入一行。它是按数据库计算的 fencing token。

`committed_bundle` 的主键是 `(writer_epoch, bundle_seq)`（`storage.rs:131`）。`commit_inner`（`storage.rs:321-344`）先按 `(writer_epoch, bundle_seq)` 查既有行：命中且 `payload_hash` 不同则返回 `CommitOutcome::PayloadMismatch`（`:337`）。

还原一份来自同一安装早前会话的备份后，facade 保留旧 epoch（设为 7），而还原库的 `committed_bundle` 已含 epoch 7 的历史行。下一帧提交 `(7, seq)`：

1. 命中既有行且 hash 不同 → `PayloadMismatch`。`facade.rs:850-856` 记 `commit_bundle` class `payload_mismatch` 并返回错误。
2. 或 `bundle_seq + RETRY_WINDOW_RECEIPTS < min(bundle_seq)` → `RetryWindowExpired`（`storage.rs:352`）。
3. 或未命中而插入成功，把当前会话的数据混入还原 epoch 的 bundle 历史，并由 `:373` 的 `on conflict do update` 改写该 epoch 的 `highest_contiguous_seq` 与 `durable_watermark`。

第 3 条改写还原库的水位记账。前两条使实时摄入停止接受该 bundle。

### 其余陈旧状态

- `alerts`：内存中仍是还原前库的规则与实例。用户在还原后看到的告警规则与库内 `c4` 表不一致。
- `engine` targets、`settings`、7 项 UI 设置：同样陈旧。
- 前端不在这三个操作后重新 bootstrap（`use-settings.ts:583`、`:620`、`:427` 之后无 `get_bootstrap` 调用），因此 UI 也继续显示还原前的值。

## Requirements

- R1. 提取存储侧启动序列为一个可复用单元，`boot()` 与三个恢复类方法共用。
- R2. `restore_backup` 成功重开库后，重跑该序列：`writer_epoch` 重新 reserve，`alerts` 规则与实例重新加载，`engine` targets、`settings`、`wizard_complete`、7 项 UI 设置重新读取。
- R3. `confirm_delete_local_data` 在 `report.all_declared_ok` 且重开成功时，重跑该序列。删除后库为空，各项回到默认值。
- R4. `run_user_vacuum` 重开库后，重跑该序列。vacuum 不改数据，但连接已关闭重开，`writer_epoch` 必须重新 reserve。
- R5. 重开或重跑失败时 `branch` 落到 `RecoveryOnly`，与现有失败处理一致。不得留下 `storage = Some(_)` 而派生状态未建立的中间态。
- R6. `bundle_seq` 在重跑后重置为 `1`，与 `boot()` 一致（`facade.rs:345`、`:402`）。
- R7. 重跑不重置非存储派生状态：`desktop`、`closes`、`operations`、`session`、`data_dir`、`snapshots`、`space`、`recovery`。采集会话不因还原而重连，`session_status` 由采集 tick 的 `apply_lifecycle` 自然更新，重跑不直接写它。`hub` 对象不重建；仅当新库的 `settings.address` 为空时置 `ObservationPhase::Unconfigured`，与 `boot()`（`facade.rs:318-320`）一致。
- R8. 重跑不触碰 `workflow`。`lib.rs:311-325` 的 `attach_windows_credentials` 在 `boot()` 之后把 `workflow` 换成由 `WindowsCredentialManager` 支撑的实例；重建 `workflow` 会换回 `FakeCredentialStore`，并因新建 `ProcessLocalStore`（`c2/settings.rs:126`）清掉会话内 secret，使用户在还原后与控制器断开。

## Out of scope

- 前端在这三个操作后重新 bootstrap。属前端改动，另开任务；本任务只保证后端状态一致。
- `writer_epoch` 与备份历史重叠的更强防护（例如还原时清理 `bundle_epoch`）。本任务只让 epoch 重新 reserve。
- `AppFacade` 拆分、字段数量、行数。
- `recovery_only()` 分支的行为。
- 备份格式、还原算法、vacuum 实现、删除确认短语。

## Acceptance Criteria

- [x] AC1：新增单测：`boot()` → 记下 `writer_epoch` → 提交若干 bundle → `create_backup` → 继续提交 → `restore_backup` 该备份 → 断言 `writer_epoch` 严格大于还原前的值，且大于还原库内 `max(writer_epoch)`。
- [x] AC2：新增单测：还原后立即提交一帧，`CommitOutcome` 不是 `PayloadMismatch`、不是 `RetryWindowExpired`。
- [x] AC3：新增单测：在库 A 建告警规则 R1、备份；改为规则 R2；还原 A 后 `list_rules()`（或等价读取路径）返回 R1，不返回 R2。
- [x] AC4：新增单测：`run_user_vacuum()` 后 `writer_epoch` 严格大于调用前的值。
- [x] AC5：新增单测：`confirm_delete_local_data` 成功后 `alerts` 无用户规则（库内仅迁移种子的 `health-*` 规则，AC 原文「无规则」据此修正）、`settings` 为 `ControllerSettings::default()`、`writer_epoch` 为新库的首个 epoch。
- [x] AC6：新增单测：重开失败时 `branch == RecoveryOnly` 且 `storage` 为 `None`。
- [x] AC7：新增单测：`restore_backup` 前后 `data_dir` 与 `desktop.launch_mode` 不变（R7 的可观测部分）。
- [x] AC8：新增单测：`restore_backup` 前后 `workflow.persistent_available()` 不变，且还原前放入 session 的 secret 在还原后仍可读（R8）。
- [x] AC9：`cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace` 通过；`just ci` 通过。
- [x] AC10：`git diff` 不含 `clash-verge-ai-residential.js`、`scripts/sync-local-config.js`、`tests/regression.test.js`。
