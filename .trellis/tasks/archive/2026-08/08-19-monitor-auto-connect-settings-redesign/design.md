# 技术设计：启动即监控与设置页现代化

## 设计目标

1. 把“打开应用”定义为可观察的自动恢复触发：owner 冷启动、`--background` owner 启动、托盘重新打开，以及第二实例向既有 owner 发出的激活请求；只有已保存且合法的控制器地址才触发。
2. 复用现有 Rust collector、lifecycle、health、Channel 与设置补偿，不创建第二条后台循环，不把前端按钮或单帧 probe 当作监控事实。
3. 在既有五页桌面工作台内重组 `settings-data`，用 SkillPort 的二级导航 / 分组密度提升可扫性，同时保留本产品主题、中文语义、本地隐私和危险操作边界。

## 边界与数据流

```text
持久 controller 设置
  -> AppFacade::boot / owner activation
  -> open/reconnect policy（仅有地址且 NormalReady）
  -> 唯一 collector_loop_tick
  -> ControllerSession / plan_tick / apply_tick_result
  -> AppFacade health + LiveProjection + Monitor Channel
  -> main.ts bootstrap/reducer/overview + settings status

用户设置草稿
  -> settings view state（地址、targets、section；secret 仅 input.value / 进程内）
  -> save/test/reconnect/disconnect command
  -> Rust SettingsWorkflow / lifecycle
  -> 新 bootstrap / Channel 状态
```

Rust 负责地址、loopback、凭据和生命周期校验；前端只解码 DTO、保存会话级视图草稿并渲染状态。设置页不读 SQLite、不读文件、不持有 secret 的 HTML 字符串。

## 自动连接策略

### 触发矩阵

| 入口 | 已保存合法地址 | 无地址 / Recovery | 已手动断开但窗口保持打开 |
|---|---|---|---|
| owner 冷启动 | 复用启动 collector，首拍取帧 | 保持未配置，不试 9097 | 不适用 |
| `--background` owner | 复用隐藏窗口下 collector | 保持未配置 | 不适用 |
| 托盘 / 菜单打开窗口 | 若 paused / cancelled，调用一次既有 reconnect seam | 仅显示窗口 | 调用 reconnect，符合“重新打开”意图 |
| 第二实例激活 owner | owner 执行同一打开窗口 + reconnect seam | 仅显示窗口 | 调用 reconnect |
| 普通 UI 重绘 / Channel resync | 不触发 | 不触发 | 不触发 |

“仅已保存配置”以 `settings.address` 非空且 Rust `plan_tick` 可解析为 loopback 为准；前端输入框的回退值不是配置。失败后保持现有健康错误分类和现有采集节拍，不增加指数重试器或额外日志敏感字段。

### 单实例激活

保留现有 named mutex 判定和 `InstanceClaim`。如果当前实现没有把 `FocusExisting` 传给 owner，则在 Windows 现有 `windows-sys` Threading 能力上增加一个按稳定 identifier 命名的轻量激活事件：second instance 设置事件后退出，owner 在 Tauri setup 注册一次等待任务，收到事件后调用统一 `open_main_window`。非 Windows 的现有进程锁路径保持；不引入 Tauri plugin 或新运行时依赖。激活等待任务只发 UI / reconnect 请求，不拥有 collector。

如果并行托盘任务已经提供 owner 激活 seam，直接复用该 seam，不重复注册事件。

## 设置页组成

`settings-data` 保持顶级 route。`renderSettings` 拆成纯的分组模型和渲染函数，建议新增 `src/settings-view.ts`（或等价本地模块），不引入组件框架：

- `SettingsSection = appearance | connection | data | about | danger`。
- `SettingsDraft` 只保存地址、targets、section、主题 / locale 选择和必要的 pending 状态；secret 延续现有 `settingsSecret`，只由 `applySecretField` 写入 `input.value`。
- `connection` 默认激活，首屏展示权威 session / collector 状态、地址、secret、targets、保存、测试、重连、断开；“测试”文案明确是单帧探测。
- `appearance` 使用四个主题选项卡片和中英分段控件，仍调用 `save_ui_theme` / `save_ui_locale`。
- `data` 按日志、备份恢复、保留 / 汇总、VACUUM 划分；`about` 独立；`danger` 隔离删除预览 / 确认。
- 通过事件委托处理二级导航，`aria-current` 表示当前项。动态重绘前从 DOM 同步草稿，重绘后恢复 focus、secret 显隐和字段值。

## 视觉系统与可用性

这是既有 Catppuccin 深色 / 浅色主题内的局部结构重构，不建立新的 DESIGN 世界。沿用 `--sidebar`、`--main`、`--card`、`--accent`、`--border`、`--focus` 等变量；新 CSS 只添加 settings-scoped surface / row / nav token。

- 桌面：全局侧栏 + 约 13rem 设置二级栏 + 可滚动内容面；1200×800 中连接状态和核心字段同屏。
- 窄窗：二级栏变为可横向滚动的紧凑导航，内容单列，无页面水平溢出。
- 结构边界使用细 divider；层次主要靠 surface 色块和留白，不用泛化阴影。
- 所有动态数字用 `font-variant-numeric: tabular-nums`；按钮与导航最低 40 px；focus-visible 轮廓独立于 hover。
- 过渡只指定 `background-color, color, border-color, opacity, transform`；按压最多 `scale(0.96)`；`prefers-reduced-motion` 下禁用装饰性 transform。
- 本地图标继续使用同一 `currentColor` 线性风格；不新增远程资源。

## 兼容与回滚

- 不改变五个顶级 route、Bootstrap / Channel DTO、C3 / C4 / C5 command、数据库 schema 或 Credential Manager 语义。
- 自动连接可通过停用 open/reconnect policy 回到现有显式连接入口；设置页可通过恢复旧 `renderSettings` 与 scoped CSS 回滚，数据库与凭据不受影响。
- 发生并行托盘任务冲突时，以其已验证的 `lib.rs` 接线为基线，按函数语义手工合并，不使用整文件回退。

## 验证策略

- Rust：纯策略与生命周期测试、有效配置 / 无配置 / manual disconnect / tray reopen / second-instance signal fixture；fmt、clippy、workspace tests。
- TypeScript：设置分组状态、draft 保留、主题 / locale、secret 不进 HTML、按钮动作与现有 DTO / i18n tests；typecheck、lint、test、build。
- 视觉：Tauri WebView 1200×800 与窄窗口实拍，四主题至少抽查 Mocha / Latte 并走键盘、focus、error、loading、reduced-motion。
- 按 `impeccable` 要求只跑一次 detector；再由独立 finish reviewer 复核截图与方向契约。浏览器 fixture 不冒充桌面证据。
