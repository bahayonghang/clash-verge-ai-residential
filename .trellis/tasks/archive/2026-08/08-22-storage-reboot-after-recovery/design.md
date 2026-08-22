# 设计：还原后重跑存储侧启动

## 结论先行

不把 `boot()` 改写成 `recovery_only()` + `reboot_storage()`。只把存储派生值的读取提取成 `StorageState::open`，两处复用。理由见下面的两个陷阱。

## 陷阱一：`workflow` 不能重建

`SettingsWorkflow::new(store, persistent_available)`（`c2/settings.rs:123`）内部新建 `ProcessLocalStore`（`:126`）作为会话内 secret 存储。

`lib.rs:311-325` 的 `attach_windows_credentials` 在 `boot()` 之后把 `facade.workflow` 整体换成 `SettingsWorkflow::new(WindowsCredentialManager, true)`。

因此 `reboot_storage()` 若重建 `workflow`：

1. 把真实的 `WindowsCredentialManager` 换回 `FakeCredentialStore`，Windows 凭据存储失效。
2. 新建 `ProcessLocalStore` 会清掉会话内 secret，用户在还原 / vacuum 后与控制器断开。

`reboot_storage()` 不触碰 `workflow`。`persistent_available` 也不随 `branch` 变化更新——现有三个方法同样不更新，本任务不改这一点。

## 陷阱二：`session_status` 不能置 `Connecting`

`boot()` 成功分支置 `session_status: Connecting`（`facade.rs:338`）。还原后采集会话仍连着还原前的地址，R7 要求不重连。此时置 `Connecting` 与实际不符。

`reboot_storage()` 不触碰 `session_status`。采集 tick 下一轮经 `apply_lifecycle` 自然更新。

`hub` 的观测阶段例外：与 `boot()`（`facade.rs:318-320`）一致，还原后的 `settings.address` 为空时置 `ObservationPhase::Unconfigured`。这是读新库设置得出的结论，不是重置 hub。

## StorageState

新增私有结构体，只装存储派生值：

```rust
struct StorageState {
    storage: StorageCoordinator,
    writer_epoch: u64,
    settings: ControllerSettings,
    wizard_complete: bool,
    ui_locale: UiLocale,
    ui_theme: UiTheme,
    ui_font: UiFont,
    ui_font_size: UiFontSize,
    ui_density: UiDensity,
    ui_sidebar_width: i32,
    live_table_layout: LiveTableLayout,
    engine: AccountingEngine,
    alerts: AlertEngine,
}

impl StorageState {
    fn open(db_path: &Path) -> Result<Self, StorageError> {
        let mut storage = StorageCoordinator::open(db_path)?;
        let writer_epoch = storage.reserve_writer_epoch()?;
        // …facade.rs:246-316 的读取原样搬入
    }
}
```

`facade.rs:227-316` 的读取逻辑原样搬入，不改语义。`reserve_writer_epoch` 失败在 `boot()` 里走 `drop(storage)` 后 `recovery_only`（`:242-243`）；改为 `?` 返回 `Err`，`storage` 随 `StorageState` 一起 drop，效果相同。

不引入 `UiChrome` 值对象。7 个 UI 字段在 `StorageState` 内保持独立字段，`AppFacade` 的字段与 7 个 `save_ui_*` 方法不动（父任务 Out of scope）。

## boot()

```rust
match StorageState::open(&db_path) {
    Ok(state) => {
        app_log::emit(Level::Info, "storage_open", json!({ "class": "ok" }));
        let hub = MonitorHub::new();
        if state.settings.address.trim().is_empty() {
            hub.set_observation_phase(ObservationPhase::Unconfigured);
        }
        Self { branch: NormalReady, storage: Some(state.storage), writer_epoch: state.writer_epoch, … }
    }
    Err(error) => {
        app_log::emit(Level::Error, "storage_open", json!({ "class": storage_error_class(&error) }));
        Self::recovery_only(desktop, data_dir, db_path)
    }
}
```

失败分支的日志 class 现在能区分 `storage_open` 与 `writer_epoch_reserve` 两种来源。原代码为两条独立日志（`:229` 与 `:237`）。保留两条：`StorageState::open` 内部在 `reserve_writer_epoch` 失败时自行 emit `writer_epoch_reserve`，再返回 `Err`。

## reboot_storage

```rust
fn reboot_storage(&mut self) -> Result<(), StorageError> {
    let db_path = self.data_dir.join("monitor.sqlite3");
    let state = StorageState::open(&db_path)?;
    self.storage = Some(state.storage);
    self.writer_epoch = state.writer_epoch;
    self.bundle_seq = 1;
    self.settings = state.settings;
    self.wizard_complete = state.wizard_complete;
    self.ui_locale = state.ui_locale;
    // …其余 6 项 UI
    self.live_table_layout = state.live_table_layout;
    self.engine = state.engine;
    self.alerts = state.alerts;
    if self.settings.address.trim().is_empty() {
        self.hub.set_observation_phase(ObservationPhase::Unconfigured);
    }
    self.branch = BootBranch::NormalReady;
    Ok(())
}
```

不触碰：`workflow`、`session_status`、`desktop`、`closes`、`operations`、`session`、`data_dir`、`snapshots`、`space`、`recovery`、`raw_retain_days`、`notify`、`controller_epoch_ready`、`last_frame_utc`、`metadata_coverage`、`last_period_eval_utc`、`last_logged_session`。

`bundle_seq = 1` 与 `boot()`（`facade.rs:345`）一致。新 epoch 下 seq 从 1 起，不会与还原库的 `(epoch, seq)` 主键碰撞。

## 三个调用点

| 方法                                        | 现在                                                 | 改为                                                                                                          |
| ------------------------------------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `restore_backup`（`:1530-1542`）            | `StorageCoordinator::open` → 设 `storage` / `branch` | `self.reboot_storage()`；`Err` 时 `branch = RecoveryOnly`、`storage = None`，返回既有的 `restore_reopen` 错误 |
| `confirm_delete_local_data`（`:1575-1585`） | 同形                                                 | `report.all_declared_ok` 时 `reboot_storage()`；`Err` 或未全部成功时 `branch = RecoveryOnly`                  |
| `run_user_vacuum`（`:1612-1620`）           | 同形                                                 | `self.reboot_storage()`；`Err` 时 `branch = RecoveryOnly`                                                     |

三处入口都已先置 `self.storage = None`（`:1515`、`:1559`、`:1601`），保证旧连接在文件操作前关闭。这一行保留。

R5 的中间态要求：`reboot_storage` 只在 `StorageState::open` 全部成功后才写 `self`，`?` 提前返回时 `self.storage` 仍是入口处置的 `None`。

## 为什么 vacuum 也要重跑

vacuum 不改数据，但 `:1601` 关闭了连接，`:1612` 重开。`reserve_writer_epoch` 只在打开时调用一次并向 `bundle_epoch` 插行；不重跑则新连接沿用旧 epoch，与 `boot()` 后的不变式不符。规则、targets、设置在 vacuum 前后相同，重读是多余但无害的读操作，换取三个调用点走同一条路径。

## 测试

放在 `facade.rs` 的 `c2_facade_contract_tests`（`:1711`）。已有 `writer_epoch` 递增用例（`:1757`、`:1769`）可作参照。

AC1 / AC2 需要「备份内含 epoch N 的 bundle 历史，还原后 facade 仍持 epoch N」这一场景：先提交若干 bundle 建立历史，`create_backup`，再提交，然后 `restore_backup`。断言还原后的 `writer_epoch` 大于还原库内 `max(writer_epoch)`。

AC3 读取告警规则用 `facade.rs:1261` 的 `load_rules` 路径。
