# ResiWatch Windows 登录自启动设计

## 1. Boundary and data flow

本功能只把既有 C2 自启动契约接到生产组合根和设置 UI，不改变 collector、托盘或 single-instance 所有权。

```text
设置页 StartupSection
  -> useSettings 独立 autostart 请求状态
  -> get_autostart_state / set_autostart_enabled
  -> Rust TauriAutostartPort
  -> tauri-plugin-autostart ManagerExt
  -> Windows 当前用户登录启动项
  -> is_enabled 回读
  -> AutostartStateDto { enabled }
```

OS 是唯一状态源。SQLite、BootstrapDto 和 UI preferences 不增加 autostart 字段，避免“数据库说开、系统实际关”的双写漂移。

## 2. Production composition

### 2.1 Plugin registration

- 在 `Cargo.toml` 增加官方 `tauri-plugin-autostart` Rust 依赖并更新 `Cargo.lock`。
- 在 Tauri builder/setup 初始化插件，参数列表只取稳定常量 `identity::AUTOSTART_ARGUMENT`，当前值为 `--background`。
- 不安装前端 guest binding。插件权限只保护前端直接调用；本设计由自有 Rust command 调用 `ManagerExt`，因此 `capabilities/default.json` 不新增 `autostart:*` 权限。

### 2.2 Port ownership and side-effect-free tests

- 保留 `AutostartPort` 与 `apply_autostart` 作为可测试业务边界。
- 新增持有/借用 `AppHandle` 的 `TauriAutostartPort`，其 `set_enabled` 分派到 `enable`/`disable`，`is_enabled` 读取插件状态。真实 adapter 只由运行中的 Tauri commands 构造。
- command core 接收 `&dyn AutostartPort`（或等价可注入 manager seam）；自动测试只注入 `FakeAutostart`/mock manager，测试 enable/disable/readback/error 分派时不得实例化真实 plugin manager，也不得写 HKCU 启动项。
- 真实 adapter 的自动验证限于编译、builder 注册与参数常量检查；任何调用真实 enable/disable 的验证都归入经授权的安装态 gate。
- `FakeAutostart` 置于 `#[cfg(test)]` 或测试模块内，不能出现在非测试编译产物。
- `AppFacade` 删除 `autostart: FakeAutostart`。插件 handle 在 Tauri runtime 建立后才存在，不能在 `AppFacade::boot` 前伪造生产端口。
- Tauri command 在不持有 `Mutex<AppFacade>` 的情况下执行插件调用；若需要 locale，只短暂读取 locale 后立即释放 facade lock。

### 2.3 Commands and error contract

- `get_autostart_state(app, state) -> Result<AutostartStateDto, AppErrorDto>`：读取 OS 状态。
- `set_autostart_enabled(app, state, enabled) -> Result<AutostartStateDto, AppErrorDto>`：调用 `apply_autostart`，返回写后回读值。
- `AutostartStateDto` 只含严格布尔字段 `enabled`；前端边界必须拒绝缺失或非布尔值。
- 插件错误映射到稳定 code `autostart_unavailable`，`retryable=true`，action 为重试；用户文案按当前 locale 生成，日志只记录错误类，不记录 executable path、注册表内容或原始错误文本。

## 3. UI and request state

### 3.1 Placement

新增 `components/features/settings/startup-section.tsx`，由“连接与监控”分区渲染。卡片与控制器卡片并列，但不把自启动状态混入 `ControllerSettings`。

### 3.2 State machine

```text
idle -> loading -> ready(off/on)
ready(off) -- toggle --> confirming -- cancel --> ready(off)
confirming -- confirm --> saving -> readback -> ready(actual)
ready(on) -- toggle --> saving -> readback -> ready(actual)
loading/saving -- failure --> ready(lastConfirmed) + error
```

- `useSettings` 增加 `autostart` 子状态、独立 request sequence 与 `saving` guard；它不能复用当前 secret/collector/about 的共享序号。
- 每次进入“连接与监控”触发读取，但 `saving=true` 时该进入刷新必须直接跳过，不能发出可能覆盖最终 readback 的 `get`。set command 返回的写后回读是本次保存的最终提交；保存完成后显式 retry/re-enter 才可再次读取。
- 开启使用无新依赖的可访问内联确认区（`role="alertdialog"`、标题/说明关联、确认/取消按钮、Escape 取消、关闭后焦点返回 Switch）；取消不调用 command。关闭不增加阻碍。
- UI 不做 optimistic commit。后端响应经严格 decoder 后才更新 `enabled`。
- 错误显示在卡片附近并保留全页错误兼容；重试重新读取 OS 状态。

## 4. Lifecycle and compatibility

- 插件登记的 command line 必须是已安装 executable 加 `--background`。既有 `DesktopRuntime::start` 解析该参数，Tauri setup 隐藏主窗口并继续创建托盘与唯一 collector。
- single-instance 仍在 Tauri runtime 前抢占 owner。登录启动遇到已运行 owner 时只激活既有实例，不创建第二 writer/collector。
- fresh install 没有启动项即为关闭；初始化插件不得自动 enable。
- 不迁移数据库。若用户从 Windows 设置或安全工具改变启动项，下一次进入设置分区按 `is_enabled` 刷新。
- 产品名 `ResiWatch`、binary `residential-monitor`、安装目录和 `--background` 已稳定；本任务不新增旧产品名启动项兼容层，因为现有生产路径从未写过真实自启动项。

## 5. Security and permissions

- 不直接读写 Windows 注册表，不使用管理员权限，不创建 service/task scheduler。
- 不向 WebView 暴露插件 guest commands；只暴露两个应用 commands，参数仅一个布尔值。
- capability 继续不授予文件系统、opener、SQL、凭据或 autostart 插件权限。
- 原始平台错误和完整 executable path 不进入 IPC/log。

## 6. Validation and rollback

### Automated

- Rust：注入 fake 的写后回读、enable/disable 分派、错误脱敏映射、后台参数、production facade 无 fake、既有 desktop lifecycle；自动测试不得调用真实 plugin enable/disable。
- Frontend：decoder、加载、确认取消、成功 readback、失败保留、保存中重进分区、重复点击、stale response、keyboard/ARIA、中英文键。
- Workspace：`just ci`。

### Installed Windows evidence

`just tinstall`、启动项写入和真实登录会修改/中断用户环境，必须在实施阶段单独征得授权。安装态检查包括路径和参数、登录后隐藏窗口/托盘/唯一 collector、关闭后不再登录启动。证据未取得时 AC9 保持 `UNVERIFIED`，任务保持 `in_progress` 且不得完成/归档，除非用户明确改变验收范围。

### Rollback

若插件或安装态行为不可接受，移除设置卡、两个 commands、真实 adapter 与插件依赖即可；既有 `--background`、托盘、single-instance、数据库和 collector 不回滚。

## 7. Sources

- 当前 Tauri 2 官方 Autostart 文档：<https://v2.tauri.app/plugin/autostart/>（Windows 支持、Rust `ManagerExt`、init 参数、guest permissions；核对日期 2026-08-29）。
- 项目既有研究：`.trellis/tasks/archive/2026-08/08-18-residential-monitor-mvp/research/desktop-monitor-architecture.md:208-218`。
- 项目既有 C2 设计：`.trellis/tasks/archive/2026-08/08-18-monitor-desktop-realtime/design.md:68-70`。
