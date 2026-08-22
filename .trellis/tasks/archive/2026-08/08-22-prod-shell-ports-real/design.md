# 设计：生产组合根接真实端口

## 既有惯例

本仓已有「`boot()` 放默认实现，组合根装真实适配器」的做法：`lib.rs:310-325`。

```rust
let mut facade = AppFacade::boot(data_dir, &args, claim);
#[cfg(windows)]
attach_windows_credentials(&mut facade);
```

`attach_windows_credentials`（`lib.rs:316-325`）用 `crate::credential::windows_cm::WindowsCredentialManager` 换掉 `boot()` 里的 `FakeCredentialStore`（`facade.rs:331`）。因此 `FakeCredentialStore` 在 Windows 生产构建里不生效，与 `FakeFileDialog` / `FakeNotificationSink` 不同类，本任务不动它。

本任务沿用该命名与分工，新增两个 attach 步骤。差别在装配时机：`WindowsCredentialManager` 是单元结构体，`boot_facade()` 内即可装；对话框与通知插件都需要 `AppHandle`，只能在 `setup()` 内装。

## 约束

`AppFacade::boot` 在 Tauri 应用建立之前运行（`lib.rs:1200` 的 `boot_facade()` 先于 `:1221` 的 `tauri::Builder::default()`），因此 `boot()` 拿不到 `AppHandle`。两个插件的 Rust API 都需要一个实现 `Manager<R>` 的值。

`blocking_pick_file` / `blocking_save_file` 的文档写明：「This is a blocking operation, and should _NOT_ be used when running on the main thread.」非 `async` 的 Tauri 命令在主线程执行，会与事件循环死锁。

### 对话框为何不做成 facade 字段

沿用 attach 惯例会把对话框放成 `AppFacade` 字段。但 R2 要求对话框打开期间不持有 `Mutex<AppFacade>` 锁：`lib.rs:645` 现在在锁内调用，真实对话框会把锁持有到用户关闭对话框为止，阻塞 `lib.rs:1251` 的采集 tick。字段方案下 `pick_file` 必须取锁才能拿到端口，因此对话框改为独立 managed state，`AppFacade` 不再持有它。通知 sink 被摄入路径使用（`facade.rs:842`、`:1492`），仍留在 facade 内。

## 依赖

`src-tauri/Cargo.toml` 新增两项，均不加 npm 包（前端只调既有 `pick_file` / `test_notification` 命令，不用 JS API）：

- `tauri-plugin-dialog`
- `tauri-plugin-notification`

`lib.rs` 的 builder 注册：

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_notification::init())
    .manage(Mutex::new(facade))
```

capabilities 的权限项只约束前端 IPC 调用。Rust 侧调用不需要权限项，`capabilities/default.json` 预期不变。实现时确认：若插件注册导致构建要求权限项，则加入并同步改写 description。

## 文件对话框

### 端口与适配器

`c2::shell::FileDialogPort` 保持不变。新增适配器：

```rust
pub struct TauriFileDialog {
    app: tauri::AppHandle,
}

impl FileDialogPort for TauriFileDialog {
    fn pick(&self, purpose: FilePurpose, mode: FileMode) -> Option<PathBuf> {
        let spec = dialog_spec(purpose);           // 纯函数，可单测
        let builder = self.app.dialog().file()
            .set_title(spec.title)
            .set_file_name(spec.file_name)
            .add_filter(spec.filter_name, spec.extensions);
        let picked = match mode {
            FileMode::Open => builder.blocking_pick_file(),
            FileMode::Save => builder.blocking_save_file(),
        };
        picked.and_then(|path| path.into_path().ok())
    }
}
```

`dialog_spec(purpose) -> DialogSpec` 是纯函数：`FilePurpose` → 标题、默认文件名、过滤器。这是本节唯一可单测的部分，AC3 针对它。

`FilePath::into_path()` 处理平台差异；桌面返回文件系统路径。

### 装配位置

`AppFacade` 删除 `dialog` 字段（`facade.rs:196`）与 `:334`、`:391` 的构造，删除 `facade.rs:1073` 的 `pub fn pick_file`。`c2/shell.rs` 的 `FakeFileDialog` 保留，`#[cfg(test)]` 之外不再被引用；`facade.rs:18` 的 import 相应删除。

端口注册在 `setup()`（`lib.rs:1223`），此处 `app.handle()` 可用：

```rust
app.manage(Arc::new(TauriFileDialog { app: app.handle().clone() })
    as Arc<dyn FileDialogPort + Send + Sync>);
```

`FileDialogPort` 需加 `Send + Sync` 约束才能进 managed state。`FakeFileDialog` 的 `next` 是 `Mutex<Option<PathBuf>>`，已满足。

### 命令改写

`lib.rs:640-650` 的 `pick_file` 改为 `async`，不再取 facade 锁：

```rust
#[tauri::command]
async fn pick_file(
    dialog: State<'_, Arc<dyn FileDialogPort + Send + Sync>>,
    purpose: FilePurpose,
    mode: FileMode,
) -> Result<Option<String>, AppErrorDto> {
    Ok(dialog.pick(purpose, mode).map(|p| p.to_string_lossy().into_owned()))
}
```

`async` 让命令在 `async_runtime::spawn` 的工作线程执行，满足 `blocking_*` 的线程要求。不取 `Mutex<AppFacade>` 锁，因此对话框打开期间采集循环（`lib.rs:1245` 的 tick）不被阻塞。

前端 `invoke("pick_file", { purpose, mode })` 的签名与返回不变，5 个调用点无需改动。

## 通知

### 字段类型

`facade.rs:205` 由 `pub notify: FakeNotificationSink` 改为：

```rust
pub notify: Box<dyn NotificationSink + Send>,
```

`NotificationSink`（`c4/notify.rs:53-56`）的两个方法 `capability(&self)` 与 `send(&mut self, ...)` 都是 object safe。

`c4/outbox::scan_once`（`c4/outbox.rs:209-215`）的签名由 `<S: NotificationSink>(… sink: &mut S …)` 改为 `sink: &mut dyn NotificationSink`。调用点 `facade.rs:842` 与 `facade.rs:1492` 传 `self.notify.as_mut()`。既有 outbox 测试传 `&mut FakeNotificationSink` 处改为 `&mut fake`（自动 unsize），无需改测试逻辑。

`c5/fault.rs:142` 与 `c4/outbox.rs:350` 自行构造 `FakeNotificationSink`，属测试代码，不受影响。全仓无测试断言 `facade.notify.sent`。

### AppHandle 的延迟装配

`boot()` 时无 `AppHandle`，因此：

```rust
pub struct WindowsNotificationSink {
    app: Option<tauri::AppHandle>,
    disabled: bool,          // RESIDENTIAL_MONITOR_ALLOW_TOAST=0|false
}
```

`boot()` 与 `recovery_only()` 构造 `Box::new(WindowsNotificationSink::new())`，`app` 为 `None`。`setup()` 内调用新增的 `AppFacade::attach_notification_handle(handle)` 补上。

`capability()` 与 `send()` 的判定表：

| 条件            | `capability().available`                         | `send()`                          |
| --------------- | ------------------------------------------------ | --------------------------------- |
| 非 Windows      | `false`，理由「v1 只在 Windows 11 提供系统通知」 | `Disabled("platform")`            |
| `disabled`      | `false`，理由说明已由环境变量关闭                | `Disabled("turned off")`          |
| `app` 为 `None` | `false`，理由说明桌面运行时尚未就绪              | `Disabled("runtime not ready")`   |
| 其余            | `true`                                           | 调用插件，`Ok(())` 或 `Temporary` |

这满足 R8：`available == true` 时 `send()` 不再恒定失败。删除 `allow_real` 与 `from_env`（`c4/notify.rs:90-101`），删除 `:135` 的 `Disabled("real toast requires reconfirmation")`。

### 发送实现

```rust
fn send(&mut self, payload: &NotifyPayload) -> Result<(), NotifyError> {
    let Some(app) = self.app.as_ref() else {
        return Err(NotifyError::Disabled("runtime not ready"));
    };
    app.notification()
        .builder()
        .title(&payload.title_zh)
        .body(&payload.body_zh)
        .show()
        .map_err(|_| NotifyError::Temporary("show failed"))
}
```

映射为 `Temporary` 让 outbox 重试（`NotifyError::permanent()` 在 `c4/notify.rs:48-50` 只对 `Permanent` 与 `Disabled` 返回 true）。

### 文案

`FakeNotificationSink` 的 `reason_zh`（`c4/notify.rs:72`）含类型名，但该 sink 不再进生产路径，无需改动。生产路径的理由字符串取自 `WindowsNotificationSink::capability()`，不含内部类型名，满足 R11。

## 环境变量

`RESIDENTIAL_MONITOR_ALLOW_TOAST` 语义反转：默认发送，值为 `0` 或 `false` 时不发送。依据：Windows 上真实 toast 是本任务的交付目标，而告警静默在后端无实现（全仓无 `silence` / `mute` / `suppress`），若默认关闭则该交付不生效；反转后保留一个关闭入口。`docs/` 写明该变量与「当前无应用内静默入口」。

## 手工验证的限制

Tauri 通知插件文档写明 Windows 上「Only works for installed apps. Shows powershell name & icon in development.」因此 AC9 的通知验证必须在安装版构建上做，`tauri dev` 下 toast 归属 PowerShell 名称与图标，不作为验收环境。文件对话框无此限制。

## 不做

- 对话框 `set_parent`。需要 `HasWindowHandle`，v1 不设父窗口，对话框不随主窗口置顶。
- `set_directory`。默认目录交给系统。
- `tauri-plugin-dialog` 的 message dialog 能力。
- `FakeAutostart`、`start_fixture`。
