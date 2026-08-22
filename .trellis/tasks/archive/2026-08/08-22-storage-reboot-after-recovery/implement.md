# 执行计划：还原后重跑存储侧启动

前置：`08-22-prod-shell-ports-real` 先完成。该任务删除 `AppFacade` 的 `dialog` 字段并改 `notify` 字段类型，两者都改 `boot()` 与 `recovery_only()` 的构造块；本任务同样改 `boot()`。反序会重复修改同一区域。

每步后跑 `cargo check --manifest-path residential-monitor/src-tauri/Cargo.toml`。

## 1. 提取 StorageState

- [ ] 1.1 在 `c2/facade.rs` 加私有 `struct StorageState`，字段按 design.md 列出的 13 项。
- [ ] 1.2 加 `impl StorageState { fn open(db_path: &Path) -> Result<Self, StorageError> }`，把 `facade.rs:227-316` 的读取原样搬入：`StorageCoordinator::open` → `reserve_writer_epoch` → `settings` → `wizard_complete` → 7 项 UI → `engine` targets → `alerts` 规则与实例。
- [ ] 1.3 `reserve_writer_epoch` 失败时在 `StorageState::open` 内 emit `writer_epoch_reserve` 日志再返回 `Err`，保留原有两条日志。
- [ ] 1.4 `MonitorHub::new()` 与观测阶段判断留在 `boot()`，不进 `StorageState`。

验证：`cargo check`。此步只新增代码，`boot()` 未改，编译应通过且出现 dead_code 警告。

## 2. boot() 改用 StorageState

- [ ] 2.1 `facade.rs:227` 的 `match StorageCoordinator::open(&db_path)` 改为 `match StorageState::open(&db_path)`。
- [ ] 2.2 成功分支：从 `state` 取值填 `Self { … }`，`storage: Some(state.storage)`。其余字段与现状一致（`branch: NormalReady`、`session_status: Connecting`、`bundle_seq: 1`、`workflow: SettingsWorkflow::new(FakeCredentialStore::new(), true)` 等）。
- [ ] 2.3 失败分支保留 `storage_open` 错误日志与 `Self::recovery_only(...)`。
- [ ] 2.4 删掉 `:234-245` 原来的 `reserve_writer_epoch` 块与 `:246-316` 的读取块。

验证：`cargo test --workspace`。既有 `writer_epoch` 用例（`facade.rs:1757`、`:1769`）必须仍通过——它们是本步的回归保护。

## 3. reboot_storage

- [ ] 3.1 加 `fn reboot_storage(&mut self) -> Result<(), StorageError>`，按 design.md 的赋值清单实现。
- [ ] 3.2 确认不写 `workflow`、`session_status`，不重建 `hub`。
- [ ] 3.3 `bundle_seq = 1`。

验证：`cargo check`。

## 4. 三个调用点改写

- [ ] 4.1 `restore_backup`（`:1530-1542`）：`match StorageCoordinator::open(&live)` 改为 `match self.reboot_storage()`。`Err` 分支保持返回 `restore_reopen` 错误，并置 `branch = RecoveryOnly`、`storage = None`。
- [ ] 4.2 `confirm_delete_local_data`（`:1575-1585`）：`all_declared_ok` 时调 `reboot_storage()`；`Err` 或未全部成功时 `branch = RecoveryOnly`。
- [ ] 4.3 `run_user_vacuum`（`:1612-1620`）：改为 `reboot_storage()`，`Err` 时 `branch = RecoveryOnly`。
- [ ] 4.4 三处入口的 `self.storage = None`（`:1515`、`:1559`、`:1601`）保留。

验证：`cargo test --workspace`。既有备份 / 还原 / 删除 / vacuum 用例必须仍通过。

## 5. 新增测试

按 AC 顺序加到 `c2_facade_contract_tests`（`:1711`）：

- [ ] 5.1 AC1：`boot` → 记 `writer_epoch` → 提交若干 bundle → `create_backup` → 继续提交 → `restore_backup` → 断言新 `writer_epoch` 大于还原前值，且大于还原库内 `max(writer_epoch)`。
- [ ] 5.2 AC2：还原后立即提交一帧，`CommitOutcome` 不是 `PayloadMismatch`、不是 `RetryWindowExpired`。
- [ ] 5.3 AC3：库内建规则 R1 → 备份 → 改为 R2 → 还原 → 读回为 R1。
- [ ] 5.4 AC4：`run_user_vacuum()` 后 `writer_epoch` 严格递增。
- [ ] 5.5 AC5：`confirm_delete_local_data` 成功后 `alerts` 无规则、`settings == ControllerSettings::default()`、`writer_epoch` 为新库首个 epoch。
- [ ] 5.6 AC6：重开失败时 `branch == RecoveryOnly` 且 `storage` 为 `None`。构造方式：还原一个无效候选文件，或在重开前让 db 路径不可用。
- [ ] 5.7 AC7：`restore_backup` 前后 `data_dir` 与 `desktop.launch_mode` 不变。
- [ ] 5.8 AC8：`restore_backup` 前后 `workflow.persistent_available()` 不变；还原前放入 session 的 secret 还原后仍可读。

验证：`cargo test --workspace`。

## 6. 全量门禁

- [ ] 6.1 `cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check`
- [ ] 6.2 `cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
- [ ] 6.3 `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`
- [ ] 6.4 `npm --prefix residential-monitor run check`
- [ ] 6.5 `just ci`
- [ ] 6.6 `git status` 确认未改路由脚本、`scripts/sync-local-config.js`、`tests/regression.test.js`（AC10）

## 回滚点

- 第 2 步后：`StorageState` 提取完成，行为不变，既有测试通过。此处可独立提交。
- 第 4 步后：三个调用点改完，新测试未加。
- 若 AC1 / AC2 的场景无法在测试内构造（例如 `create_backup` 需要真实文件路径而测试环境受限），先交付第 1 至 4 步与 AC3 至 AC8，把 AC1 / AC2 降级为手工验证并在 prd.md 记录原因，不静默跳过。
