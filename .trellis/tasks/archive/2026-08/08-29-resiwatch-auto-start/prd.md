# 为 ResiWatch 添加开机自动启动设置

## Goal

让用户在 ResiWatch 的设置页查看并控制 Windows 登录自启动。用户明确启用后，ResiWatch 在该用户登录 Windows 时以 `--background` 启动，主窗口保持隐藏，既有托盘与唯一 collector 继续负责后台监控；未启用时不写入任何自启动项。

本任务中的“开机自动启动”统一指 **Windows 用户登录后自启动**，不指系统登录前运行的 Windows Service。

## Background

- `src-tauri/src/c2/desktop.rs:207-250` 已有 `AutostartPort`、测试替身和 `--background` 命令行约定；`:289-302` 已覆盖后台启动时窗口隐藏。
- `src-tauri/src/c2/settings.rs:270-273` 已有“写入后读取 OS 实际状态”的纯业务流程。
- 生产 `AppFacade` 仍在 `src-tauri/src/c2/facade.rs:196,371,423` 持有并构造 `FakeAutostart`；前端没有调用点。
- `src-tauri/src/lib.rs:1219-1245` 已识别后台启动并隐藏主窗口，但 `Cargo.toml`、builder 和 command handler 尚未接入 `tauri-plugin-autostart`。
- 设置页当前固定为五个分区；`src/components/features/settings/index.tsx:102-125` 的“连接与监控”只加载控制器、采集器和凭据相关状态，没有启动行为卡片。
- 既有产品契约要求自启动默认关闭、仅在用户明确确认后写入、保存后读取 OS 实际状态，并使用 `--background`。详见 `docs/install.md:3,27,38`、`docs/first-run.md:7` 和归档 C2 设计。

## Requirements

### R1 真实系统能力

- 使用官方 Rust `tauri-plugin-autostart` 作为 Windows 登录自启动适配器，注册参数固定为 `--background`。
- Rust 是系统能力的唯一所有者。前端只调用应用自有 Tauri commands；不新增 `@tauri-apps/plugin-autostart`，不直接操作注册表，也不向 WebView capability 授予 autostart 插件命令权限。
- `FakeAutostart` 只保留在单元测试路径；生产 `AppFacade` 不得再持有或构造它。

### R2 OS 状态为权威

- 提供读取自启动实际状态和设置自启动状态的 Rust commands。
- 设置命令必须在 enable/disable 后再次调用 `is_enabled`，响应返回回读结果；不得把请求值或插件调用成功直接当作最终状态。
- 不向 SQLite 或 UI preference 写入第二份“期望状态”。用户或系统在应用外修改启动项后，重新进入设置页必须能看到新的 OS 状态。
- 读取或写入失败必须返回脱敏、可重试的应用错误，不泄露完整可执行文件路径或原始平台错误。

### R3 设置页交互

- 在既有“连接与监控”分区增加独立“启动与后台运行”卡片，不新增顶级设置分区。
- 卡片包含“登录 Windows 后自动启动 ResiWatch”开关、后台进入托盘的说明、加载/保存状态和失败反馈。
- 首次加载完成前开关不可操作。启用从关闭切换为开启时必须显示二次确认；取消确认不得写系统状态。关闭自启动可直接执行。
- 写入期间禁用重复操作；成功后显示后端回读的实际状态。失败时保留最近一次已确认状态并显示重试提示，不能先乐观翻转后把失败伪装成成功。
- 自启动状态使用独立请求序号或等价的过期响应保护，不能被 secret、collector、about 等并行请求互相作废。
- 中英文文案、键盘操作、焦点和 `aria` 状态必须完整。

### R4 生命周期兼容

- 自启动 owner 必须复用既有 `--background`、single-instance、托盘和唯一 collector 路径；不得新建第二个 writer/collector。
- 后台启动不显示或聚焦主窗口；用户从托盘或启动第二实例时才沿用既有窗口恢复流程。
- 首次安装或不存在启动项时保持关闭，不得由安装包、升级或应用启动偷偷启用。

### R5 文档与验证

- 更新安装、首次配置、升级/卸载和已知限制文档，使“已实现能力”“需要安装态验证”和“仍未验证证据”保持一致。
- 自动化覆盖 adapter 业务语义、command/DTO 错误映射、前端加载/确认/成功/失败/过期响应和无障碍状态。
- 最终通过 `just ci`。安装、写入登录自启动项以及真实登录/重启验证会修改本机或中断会话，执行前必须另行获得明确授权。

## Acceptance Criteria

- [x] **AC1 默认关闭且无隐式写入（R1、R4）**：无 OS 启动项的新安装/测试环境打开设置页时显示关闭；未点击并确认启用前，插件 `enable`/`disable` 均未被调用。
- [x] **AC2 实际状态回显（R2、R3）**：进入“连接与监控”后读取 `is_enabled`；应用外改变启动项后重新进入该分区，UI 跟随 OS 实际状态，而不是数据库或上次请求值。
- [x] **AC3 启用确认与回读（R2、R3）**：从关闭切换到开启先出现说明 `--background`/托盘行为的无新依赖内联确认区；取消无写入，确认后调用 enable + readback，只有回读 `true` 时开关显示开启。
- [x] **AC4 关闭、竞态与失败语义（R2、R3）**：关闭成功后 readback 为 `false`；保存期间重新进入连接分区不得发起覆盖性读取，最终写后回读拥有状态提交权；enable、disable 或 readback 任一步失败时显示本地化可重试错误，保留最近一次确认状态，且无未处理 Promise 或重复并发写入。
- [x] **AC5 生产组合根真实（R1）**：`AppFacade` 的字段和生产构造路径不含 `FakeAutostart`；`FakeAutostart` 受 `#[cfg(test)]` 或等价测试模块隔离；Tauri builder 注册官方插件并传入唯一参数 `--background`；前端无 autostart JS 插件依赖，capability 不新增插件权限。
- [x] **AC6 后台生命周期（R4）**：自动化继续证明 `--background` owner 启动隐藏窗口、第二实例不创建 collector；现有托盘打开、明确退出和窗口恢复契约不回归。
- [x] **AC7 前端可访问性与竞态（R3）**：组件/Hook 测试覆盖加载禁用、确认取消、成功、失败、保存中重进分区、快速重复操作和过期响应；中英文键完整，开关与内联确认区具备可读名称、焦点返回及正确 checked/disabled 状态。
- [x] **AC8 自动质量门且无本机副作用（R1、R5）**：`just ci` 通过，包括前端 typecheck/lint/test/build、Rust fmt/clippy/test、版本与 secret 检查；所有自动测试只使用 fake/注入 manager，不得调用真实 plugin enable/disable 或写 HKCU 启动项。
- [x] **AC9 Windows 安装态证据与关闭门（R4、R5）**：经用户另行授权后，在 NSIS current-user 安装版验证启动项指向 `%LOCALAPPDATA%\ResiWatch\residential-monitor.exe --background`，启用后一次真实 Windows 登录可隐藏窗口进入托盘且仅有一个 collector，关闭后下一次登录不再启动；取得该证据前任务保持 `in_progress` 且不得完成/归档。只有用户明确改变验收范围，才能把该门移出本任务。安装结果与当前 Run key 由命令采集，两个登录周期由用户于 2026-08-29 人工确认，详见 `evidence/installed-validation.md`。
- [x] **AC10 文档一致（R5）**：安装、首次配置、升级/卸载、release checklist/known limits 与最终验证状态一致；未取得的真机证据继续写为 `UNVERIFIED`。
- [x] **AC11 错误脱敏（R2）**：Rust 测试注入包含 executable path、注册表位置和原始平台文本的失败，断言 IPC `AppErrorDto` 与新增日志事件只含稳定 code/错误类，不含这些原文。

## Out of Scope

- 默认开启自启动、安装器静默启用、登录前运行、Windows Service 或计划任务。
- macOS/Linux 自启动支持和跨平台设置 UI 承诺；ResiWatch v1 仍以 Windows 11 NSIS current-user 为目标。
- 重做首次引导流程、增加新的顶级设置分区或改变当前五分区信息架构。
- 修改 collector、核算、数据库 schema、保留策略、通知、托盘菜单或 single-instance 语义。
- 自动执行 `just tinstall`、真实登录/重启、写注册表或发布安装包；这些动作需要新的明确授权。

## Risks and Deferred Evidence

- 插件 API 自动化只能证明注册、调用与错误语义，不能证明 Windows 在真实登录时执行该项；AC9 是独立安装态硬关闭门，取得证据前任务不得完成或归档。
- 自启动项可能被用户、系统设置或安全软件在应用外修改，因此 UI 只承诺进入分区时刷新，不承诺持续监听系统启动项变化。
- 当前仓库没有可用于升级测试的已发布 autostart 基线；本任务验证当前稳定产品名/二进制/参数，不伪造跨历史版本升级证据。
