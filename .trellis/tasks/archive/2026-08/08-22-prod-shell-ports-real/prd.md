# 生产组合根接真实文件对话框与通知 sink

父任务：`08-22-dev-main-merge-unblock`。

## Goal

`boot()` 交付真实的文件对话框与通知实现。备份、还原、备份校验、报告导出、诊断导出能真正弹出系统选路径对话框；告警产生时发出 Windows 通知。测试替身只在测试代码构造。

## Background

### 文件对话框

`c2/facade.rs:334` 与 `:391` 在生产与 recovery 两条启动分支都写 `dialog: FakeFileDialog::default()`。`c2/shell.rs:78-82` 的 `pick` 返回 `self.next` 的克隆，生产从不给 `next` 赋值，因此 `pick_file` 命令恒返回 `None`。

前端 5 个调用点把 `None` 当用户取消：

| 调用点 | purpose | 现象 |
|---|---|---|
| `src/hooks/use-settings.ts:562` | `backup-create` | `if (!picked) return "cancelled"`，备份不执行 |
| `src/hooks/use-settings.ts:576` | `backup-restore` | 同上，还原不执行 |
| `src/hooks/use-settings.ts:596` | `backup-restore` | 返回 `null`，校验不执行 |
| `src/hooks/use-report-archive.ts:457` | 报告导出 | 导出不执行 |
| `src/hooks/use-alerts.ts:256` | `diagnostics-export` | 状态显示 `report.export_cancel`，导出不执行 |

`.trellis/tasks/archive/` 内无「保留假对话框」的决策记录。`src-tauri/Cargo.toml` 与 `residential-monitor/package.json` 均无对话框依赖，真实适配器从未加入。

### 通知

`c2/facade.rs:205` 的字段类型是具体的 `FakeNotificationSink`，`:343` 与 `:400` 构造它。`WindowsNotificationSink` 只在 `c4/mod.rs:15` 导出，`boot()` 不构造。

`use-alerts.ts:211` 调 `test_notification` → `facade.rs:1423` 的 `self.notify.capability()`。`FakeNotificationSink` 的 `available` 默认 `false`，`c4/notify.rs:72` 返回 `"当前为进程内 FakeNotificationSink，未发送系统通知。"`，`use-alerts.ts:218` 把该字符串写进 `statusZh` 显示给用户。内部类型名进入产品文案。

`WindowsNotificationSink` 自身也不自洽：`capability()` 在 `allow_real` 且 Windows 下返回 `available: true`（`c4/notify.rs:122-127`），而 `send()` 在同样条件下恒返回 `Err(NotifyError::Disabled("real toast requires reconfirmation"))`（`:135`）。从未实现真实 toast。

已定（本任务）：接 `tauri-plugin-notification` 发真实 toast。

## Requirements

### 文件对话框

- R1. 新增 Rust 侧文件对话框实现，满足 `c2::shell::FileDialogPort`。`FilePurpose` 决定对话框标题与默认文件名，`FileMode::Open` 用打开对话框，`FileMode::Save` 用保存对话框。
- R2. 对话框不得在持有 `Mutex<AppFacade>` 锁时调用。当前 `lib.rs:645-648` 在锁内调用，真实对话框会把锁持有到用户关闭对话框为止，阻塞采集循环。
- R3. `AppFacade` 移除 `dialog` 字段。对话框端口改为独立的 Tauri managed state，`pick_file` 命令不再取 facade 锁。
- R4. 不引入文件系统读写权限。`capabilities/default.json` 的 description 声明不授予文件系统、opener、SQL、凭据权限，该声明在本任务后仍须成立。
- R5. `FakeFileDialog` 保留在 `c2/shell.rs`，仅供测试使用，不出现在任何生产构造路径。

### 通知

- R6. 新增 `tauri-plugin-notification` Rust 依赖并在 `lib.rs` 的 builder 注册。不引入 `@tauri-apps/plugin-notification` npm 包：前端只调用既有 `test_notification` 命令，不直接用 JS API。
- R7. `WindowsNotificationSink::send` 通过插件的 Rust API 提交通知，用 `NotifyPayload` 的 `title_zh` / `body_zh`。提交失败映射为 `NotifyError::Temporary`，由 outbox 重试；平台不支持或被关闭映射为 `NotifyError::Disabled`。
- R8. `capability()` 与 `send()` 必须一致：`capability().available` 为 `true` 时 `send()` 不得恒定失败。
- R9. `AppFacade::notify` 字段类型改为 trait 对象，`boot()` 与 `recovery_only()` 构造真实 sink。`c4/outbox::scan_once` 与 `facade.rs:842` 的调用相应改为动态派发。
- R10. 默认在 Windows 启用真实通知。`RESIDENTIAL_MONITOR_ALLOW_TOAST` 的语义反转为关闭开关：值为 `0` 或 `false` 时不发送。文档写明该变量与「无应用内静默入口」。
- R11. 用户可见文案不得出现 `FakeNotificationSink` 或任何内部类型名。
- R12. `FakeNotificationSink` 仅供测试使用，不出现在任何生产构造路径。

## Out of scope

- `FakeAutostart`（前端无调用点，`src/i18n/en.ts:357` 说明该功能待二次确认后才写入）。
- `start_operation` / `start_fixture` 命名与真实进度。
- 告警静默功能。见父任务 Out of scope。
- `alerts.notify_on` 等文案里的 `seam` 一词。
- `NotifyCapability` 各 `*_zh` 字段在英文 locale 下返回中文。
- `AppFacade` 其余字段与行数、`lib.rs` 拆分。
- 对话框父窗口归属（modal parenting）。v1 不设父窗口。
- 报告导出、备份、还原自身的逻辑与格式。

## Acceptance Criteria

- [x] AC1：`grep -n "Fake" src-tauri/src/c2/facade.rs` 在 `AppFacade` 结构体定义与 `boot()` / `recovery_only()` 两个构造块内不再匹配 `FakeFileDialog` 与 `FakeNotificationSink`。
- [x] AC2：`AppFacade` 无 `dialog` 字段；`lib.rs` 的 `pick_file` 命令不调用 `state.lock()`。
- [x] AC3：新增单测覆盖 `dialog_spec(purpose)` 纯函数：4 个 `FilePurpose` 各返回非空标题与默认文件名，`report-export` 与 `diagnostics-export` 的默认文件名扩展名与 `add_filter` 的扩展名一致。`FakeFileDialog` 仍有测试覆盖其注入语义。
- [x] AC4：新增单测：`WindowsNotificationSink` 在 `capability().available == true` 时，`send()` 的返回不是 `Err(NotifyError::Disabled(_))`。
- [x] AC5：新增单测：`AppFacade::boot` 后 `test_notification()` 返回的 `reason_zh` 不含 `Fake`。
- [x] AC6：`c4/outbox::scan_once` 接受 `&mut dyn NotificationSink`，既有 outbox 测试仍通过。
- [x] AC7：`capabilities/default.json` 不含文件系统、opener、SQL、凭据权限。若因通知插件新增权限项，description 同步更新。
- [x] AC8：`cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace` 通过；`npm --prefix residential-monitor run check` 通过；`npm run check:secrets` 通过。
- [ ] AC9：手工验证（安装版构建，非 `tauri dev`）：设置页点击备份弹出保存对话框，选路径后备份文件生成；告警页点击测试通知在 Windows 11 通知中心可见。`tauri dev` 下 toast 归属 PowerShell 名称与图标，不作为验收环境。
