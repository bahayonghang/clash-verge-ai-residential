# 执行计划：生产组合根接真实端口

前置：`08-22-storage-reboot-after-recovery` 在本任务之后做。本任务删除 `AppFacade` 的 `dialog` 字段并改 `notify` 字段类型，都改 `boot()` 与 `recovery_only()`；反序会重复修改同一区域。

每步后跑 `cargo check --manifest-path residential-monitor/src-tauri/Cargo.toml`，避免把编译错误累积到最后一步。

## 1. 加依赖与插件注册

- [ ] 1.1 `src-tauri/Cargo.toml` 的 `[dependencies]` 加 `tauri-plugin-dialog` 与 `tauri-plugin-notification`（版本用 `2`）。
- [ ] 1.2 `lib.rs:1221` 的 builder 链首加 `.plugin(tauri_plugin_dialog::init())` 与 `.plugin(tauri_plugin_notification::init())`。
- [ ] 1.3 `cargo check`。确认 `capabilities/default.json` 未被构建要求改动；若被要求，加权限项并同步改写 description（保持不含文件系统、opener、SQL、凭据权限）。

验证：`cargo check` 通过；`git diff --stat src-tauri/capabilities/` 为空或仅含通知权限项。

## 2. 文件对话框适配器

- [ ] 2.1 `c2/shell.rs`：`FileDialogPort` 加 `Send + Sync` 约束（`pub trait FileDialogPort: Send + Sync`）。
- [ ] 2.2 `c2/shell.rs`：加 `DialogSpec` 结构体与 `dialog_spec(purpose: FilePurpose) -> DialogSpec` 纯函数，覆盖 `ReportExport`、`BackupCreate`、`BackupRestore`、`DiagnosticsExport` 四个 variant。标题与文件名走 `i18n::t`，不硬编码中文。
- [ ] 2.3 新建 `c2/dialog.rs`（或在 `c2/shell.rs` 内），实现 `TauriFileDialog { app: tauri::AppHandle }` 的 `FileDialogPort`，按 `FileMode` 分派 `blocking_pick_file` / `blocking_save_file`，用 `FilePath::into_path()` 取路径。
- [ ] 2.4 `c2/mod.rs` 导出新模块。

验证：`cargo check` 通过。

## 3. 摘掉 facade 的 dialog 字段

- [ ] 3.1 `facade.rs:196` 删 `pub dialog: FakeFileDialog`。
- [ ] 3.2 `facade.rs:334` 与 `:391` 删 `dialog: FakeFileDialog::default()`。
- [ ] 3.3 `facade.rs:1073-1076` 删 `pub fn pick_file`。
- [ ] 3.4 `facade.rs:18` 的 import 删 `FakeFileDialog`（保留 `FileMode`、`FilePurpose`，`lib.rs` 命令签名仍要）。

验证：`cargo check` 报错只剩 `lib.rs` 的 `pick_file` 命令，下一步修。

## 4. 改写 pick_file 命令与装配

- [ ] 4.1 `lib.rs:639-650` 改为 `async fn pick_file(dialog: State<'_, Arc<dyn FileDialogPort + Send + Sync>>, purpose, mode)`，不取 facade 锁。
- [ ] 4.2 `lib.rs:1223` 的 `setup()` 内 `app.manage(Arc::new(TauriFileDialog { app: app.handle().clone() }) as Arc<dyn FileDialogPort + Send + Sync>)`。放在 `attach_window_close(app)` 之前。
- [ ] 4.3 加 `use std::sync::Arc;` 与端口 import。

验证：`cargo check` 通过；`grep -n "state.lock" src-tauri/src/lib.rs | grep -n pick_file` 无结果（AC2）。

## 5. dialog_spec 单测

- [ ] 5.1 在 `c2/shell.rs` 的 `shell_seam_tests` 加 `dialog_spec_covers_all_purposes`：四个 variant 的标题与默认文件名非空。
- [ ] 5.2 加 `export_purposes_filter_matches_file_name`：`ReportExport` 与 `DiagnosticsExport` 的默认文件名扩展名在其 `extensions` 内。
- [ ] 5.3 `file_dialog_returns_only_injected_path`（`c2/shell.rs:231`）保留不改。

验证：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`。AC3。

## 6. 通知 sink 重写

- [ ] 6.1 `c4/notify.rs`：`WindowsNotificationSink` 字段改为 `app: Option<tauri::AppHandle>` 与 `disabled: bool`。加 `new()` 读 `RESIDENTIAL_MONITOR_ALLOW_TOAST`，值为 `0` 或 `false` 时 `disabled = true`。删 `from_env` 与 `allow_real`。
- [ ] 6.2 加 `attach(&mut self, app: tauri::AppHandle)`。
- [ ] 6.3 按 design.md 的判定表重写 `capability()`。
- [ ] 6.4 重写 `send()`：走 `app.notification().builder().title().body().show()`，失败映射 `NotifyError::Temporary`。删掉 `:135` 的 `Disabled("real toast requires reconfirmation")`。
- [ ] 6.5 改 `notify_seam_tests`：`test_and_real_share_trait` 里 `WindowsNotificationSink { allow_real: false }` 改为 `WindowsNotificationSink::new()`（`app` 为 `None`），断言 `capability().available == false`。
- [ ] 6.6 加 `capability_and_send_agree`：`app` 为 `None` 时 `capability().available == false` 且 `send()` 返回 `Disabled`。AC4 的可测部分——`available == true` 需要真实 `AppHandle`，用断言「不存在使 `available == true` 且 `send` 恒 `Disabled` 的分支」的形式覆盖：即 `send` 内除 `app` 为 `None`、`disabled`、非 Windows 三种情况外不返回 `Disabled`。

验证：`cargo test --workspace`。AC4。

## 7. outbox 改动态派发

- [ ] 7.1 `c4/outbox.rs:209` 的 `scan_once<S: NotificationSink>(… sink: &mut S …)` 改为 `sink: &mut dyn NotificationSink`，删泛型参数。
- [ ] 7.2 `c4/outbox.rs:350` 附近的测试调用点确认仍编译（`&mut fake` 自动 unsize）。

验证：`cargo test --workspace`，outbox 既有测试通过。AC6。

## 8. facade 的 notify 字段

- [ ] 8.1 `facade.rs:205` 改为 `pub notify: Box<dyn NotificationSink + Send>`。
- [ ] 8.2 `facade.rs:343` 与 `:400` 改为 `notify: Box::new(WindowsNotificationSink::new())`。
- [ ] 8.3 `facade.rs:842` 与 `:1492` 的 `&mut self.notify` 改为 `self.notify.as_mut()`。
- [ ] 8.4 `facade.rs:30` 的 import 删 `FakeNotificationSink`，加 `WindowsNotificationSink`。
- [ ] 8.5 加 `pub fn attach_notification_handle(&mut self, app: tauri::AppHandle)`，转调 sink 的 `attach`。因字段是 trait 对象，`attach` 不在 `NotificationSink` trait 上；改为在 `AppFacade` 保存 handle 后重建 sink，或把 `attach` 加进 trait 并给 `FakeNotificationSink` 空实现。选后者，改动更小。
- [ ] 8.6 `lib.rs` 的 `setup()` 内取 facade 锁调用 `attach_notification_handle(app.handle().clone())`。放在 `:1228` 读 `ui_locale` 的同一个锁作用域内，避免多取一次锁。

验证：`cargo check`；`grep -n "Fake" src-tauri/src/c2/facade.rs` 在结构体定义与两个构造块内无匹配（AC1）。

## 9. test_notification 文案断言

- [ ] 9.1 在 `facade.rs` 的 `c2_facade_contract_tests` 加测试：`AppFacade::boot` 后 `test_notification()` 返回的 `reason_zh` 不含 `"Fake"`。

验证：`cargo test --workspace`。AC5。

## 10. 文档

- [ ] 10.1 `docs/` 内写明 `RESIDENTIAL_MONITOR_ALLOW_TOAST`（默认发送，`0` / `false` 关闭）与「当前无应用内静默入口」。文件用中文。
- [ ] 10.2 `CHANGELOG.md` 加一条英文记录：真实文件对话框与 Windows 通知接入。

## 11. 全量门禁

- [ ] 11.1 `cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check`
- [ ] 11.2 `cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
- [ ] 11.3 `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`
- [ ] 11.4 `npm --prefix residential-monitor run check`
- [ ] 11.5 `npm run check:secrets`
- [ ] 11.6 `just ci`

验证：AC8。

## 12. 手工验证

- [ ] 12.1 出安装版构建（非 `tauri dev`）。
- [ ] 12.2 设置页点备份：弹出保存对话框，选路径后备份文件生成。
- [ ] 12.3 设置页点还原、备份校验；告警页点诊断导出；报告页点导出。各自弹出对话框。
- [ ] 12.4 告警页点测试通知：Windows 11 通知中心可见，标题与正文为应用文案。
- [ ] 12.5 对话框打开期间实时页继续刷新（确认未持有 facade 锁）。

验证：AC9。

## 回滚点

- 第 5 步后：对话框链路完成且有测试，通知未动。此处可独立提交。
- 第 9 步后：通知链路完成。
- 若 `tauri-plugin-notification` 在 Windows 安装版之外无法验证，回滚范围限于第 1、6、7、8、9 步；第 2 至 5 步（对话框）独立成立，可单独合并。
