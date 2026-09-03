# ResiWatch 自启动现状与方案证据

核对日期：2026-08-29。

## Confirmed current state

| Layer | Evidence | Finding |
|---|---|---|
| Product contract | `docs/install.md:3,27,38`; `docs/first-run.md:7`; archived C2 design `:68-70` | 登录自启动默认关闭、用户明确确认、参数为 `--background`，OS 实际状态为权威；安装态证据不能由 dev/单测替代。 |
| Desktop lifecycle | `src-tauri/src/c2/desktop.rs:103-116,289-302`; `src-tauri/src/lib.rs:1219-1245` | `--background` 已进入 launch mode，并在 setup 隐藏主窗口。 |
| Business seam | `src-tauri/src/c2/desktop.rs:207-250`; `src-tauri/src/c2/settings.rs:270-273` | 已有 `AutostartPort`、fake 和 write-then-readback 逻辑。 |
| Production composition | `src-tauri/src/c2/facade.rs:196,371,423` | 生产 `AppFacade` 仍使用 `FakeAutostart`。 |
| Tauri builder | `src-tauri/src/lib.rs:1220-1223,1265-1295`; `src-tauri/Cargo.toml:39-41` | 仅注册 dialog/notification，无 autostart plugin 或 commands。 |
| Frontend | `src/components/features/settings/index.tsx:14-16,102-125`; `connection-section.tsx:75-204`; `hooks/use-settings.ts:58-99` | 设置保持五分区；连接分区和 hook 均无自启动状态或操作。 |
| Capability | `src-tauri/capabilities/default.json:1-12` | WebView 只有 core window 权限，没有 autostart guest permission。 |

结论：当前不是缺少概念设计，而是“业务骨架和后台参数已存在，生产 OS adapter、IPC、设置 UI 与安装态证据未接通”。

## Official Tauri 2 contract

来源：<https://v2.tauri.app/plugin/autostart/>，核对日期 2026-08-29。

- 官方插件支持 Windows，Rust API 通过 `ManagerExt` 获取 autolaunch manager。
- 初始化可传任意应用参数，适合直接注册既有 `--background`。
- enable、disable、is_enabled 均可从 Rust 调用。
- 文档列出的 `autostart:allow-*` permissions 面向 guest/plugin commands。本项目可以沿用“系统能力由 Rust 拥有”的既有架构，只暴露自有应用 commands，不安装 JS guest binding。

## Option analysis

| Option | Result |
|---|---|
| 官方 Rust plugin + 自有 commands | 采用。匹配既有架构、稳定参数与 OS readback，避免前端直接获得系统权限。 |
| 前端安装 `@tauri-apps/plugin-autostart` 并授予 permissions | 拒绝。扩大 WebView capability，绕开项目规定的 Rust-owned system seams。 |
| 手写 Windows Registry | 拒绝。重复平台逻辑、升级/卸载与路径转义风险更高，也偏离既有官方插件决策。 |
| Windows Service/Task Scheduler | 拒绝。需要更高权限和完全不同生命周期，超出“用户登录后启动桌面托盘应用”的需求。 |

## Planning consequences

1. 任务是单个紧耦合跨层功能，不拆父/子任务：plugin adapter、commands 和 UI 必须一起交付才有用户价值。
2. 不新增数据库 migration；实际系统状态每次进入设置分区读取。
3. 开启需要用户确认，关闭直接执行；任何写入后都 readback。
4. 自动化门与安装态门分开。`just tinstall`、真实启动项和登录/重启属于需要额外授权的本机动作。
5. 现有 `--background`、single-instance、tray 和 collector 测试必须作为回归门，而不是重新实现生命周期。
6. 自动测试不得实例化真实 plugin manager 或调用 enable/disable；真实 adapter 通过可注入 command core 与编译覆盖，HKCU 写入只属于授权后的安装态 gate。
7. 开启确认固定为无新依赖的可访问内联确认区；保存期间进入分区的读取必须跳过，避免旧 OS 状态覆盖最终写后回读。
8. AC9 是任务归档前硬关闭门，不是可无限延期后仍能声明完成的附注。
