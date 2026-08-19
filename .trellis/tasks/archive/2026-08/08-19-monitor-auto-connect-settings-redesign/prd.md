# 启动即监控与设置页现代化

## Goal

让已经完成控制器配置的用户打开 residential-monitor 后无需再点“测试连接”即可进入持续监控，并把当前堆叠式设置 / 数据管理页重构为参考 SkillPort 信息架构的现代桌面设置工作台。

用户价值：启动后的连接状态与监控行为可预期，常用设置更容易定位，危险操作与普通设置不再混在同一条长页面中。

## Background and Confirmed Facts

- Rust `collector_loop_tick` 已拥有约 1 Hz 的唯一持续采集循环；`test_controller` 只做单帧探测，不能作为持续监控实现。
- 正常分支启动时 `AppFacade.session_status` 为 `Connecting`，已有合法持久地址时采集循环会自行取帧；首次安装的 `ControllerSettings.address` 为空，当前前端只把 `127.0.0.1:9097` 显示成输入框回退值，并未保存。
- WebView 仅通过 Tauri Channel 订阅权威状态；隐藏、重建或不存在不得停止 Rust collector、writer、coverage 与 health。
- 当前 `settings-data` route 由四个纵向 `.panel` 组成，外观、控制器、数据、关于和删除入口处于同一长页面；固定五个顶级 route 不变。
- 参考截图只作为 SkillPort 的设置导航、分组、控件密度和状态层级参考；截图中的技能管理功能与文案不属于本产品需求。
- 前端为 Vanilla TypeScript + Vite，不引入 UI 框架、远程 URL、CDN 或新运行时依赖。

## Requirements

### R1 Task boundaries and ordering

- 本父任务管理两个可独立验收的子任务：`08-19-monitor-auto-connect` 与 `08-19-monitor-settings-redesign`。
- 先冻结并实现启动连接 / 采集状态契约，再让设置页消费该契约和状态文案；设置页不得另起采集循环或把单帧测试标成“正在监控”。

### R2 Automatic connection and monitoring

- 普通前台启动与 `--background` 启动都遵循同一 Rust 生命周期；只有单实例 owner 可以启动采集。
- 已配置路径在应用启动后自动进入连接 / 采集流程，成功、连接中、未配置、鉴权失败、端点缺失、已断开、采集暂停和存储故障保持独立可诊断状态。
- 自动行为必须复用现有 `collector_loop_tick`、`reconnect_now` / `resume_collector` 与 Channel bootstrap，不创建第二个定时器或 WebView 所有的后台循环。
- 手动“断开连接”在窗口保持打开时仍是明确用户意图；只有冷启动、从托盘 / 第二实例重新打开主窗口或显式重连才恢复自动采集，不允许普通界面重绘悄悄覆盖断开动作。
- secret 继续只存在于 Credential Manager 或当前进程内存，自动连接不得把 secret 放入 SQLite、日志、Channel、URL、错误或诊断。

### R3 Settings information architecture

- 保留顶级 `settings-data` route，在其内部采用 SkillPort 式二级设置导航与单一内容面，至少包含：外观与语言、连接与监控、数据与备份、关于、危险区域。
- 默认打开“连接与监控”，突出启动行为、当前连接状态、控制器地址、TCP secret、重点目标以及测试 / 重连 / 断开动作。
- 外观与语言使用比原生下拉框更可扫的主题选项与语言分段控件；只呈现现有四个主题与中 / 英，不新增 Accent Color、字体系统或 SkillPort 专属能力。
- 数据与备份保留日志目录、备份、恢复、保留预览、物化汇总和用户主动 VACUUM；关于与危险删除分别独立，删除全部本地数据保持二次确认短语和分项结果。
- 设置引导文案改为就地帮助与状态说明，不再用长编号列表占据首屏。

### R4 Visual and interaction quality

- 设计模式为桌面 Operate：高频操作可扫、表单标签明确、状态靠图标 / 文案 / 色彩共同表达，不做营销页式大留白。
- 继承现有主题变量与 Vanilla CSS；SkillPort 参考的是结构、克制层级和控件工艺，不逐像素复制其品牌、字体或红色强调色。
- 桌面默认 1200×800 下首屏可看到二级导航、连接状态和核心控制器设置；窄窗口中二级导航可折叠或横向滚动，页面无水平溢出。
- 键盘可到达全部设置导航和操作，保留清晰 `:focus-visible`；密集桌面控件命中区至少 40×40 px。
- 交互只对具体属性做不超过 160 ms 的可中断过渡；尊重 `prefers-reduced-motion`，不得使用 `transition: all`。
- 动态连接数值使用等宽 / `tabular-nums`；标题平衡换行、说明文字避免孤行。

### R5 Compatibility and evidence

- 固定五个顶级 route、Recovery Shell、实时 DTO / Channel、报告、告警、备份、删除与关于命令的产品语义保持兼容。
- 不修改登录自启动的 opt-in 策略，不安装应用、不写真实 Credential Manager、不触碰远程配置。
- 自动化验证覆盖 Rust 生命周期 / collector、TypeScript 状态与交互、i18n、主题、构建与 lint；视觉证据至少包含 1200×800 桌面和窄窗口截图。

## Acceptance Criteria

- [x] **AC1 任务边界**：两个子任务各自拥有可追踪的需求、实施与验证证据；先完成启动契约，再完成设置页消费与整体验收。
- [x] **AC2 启动即监控**：已有有效持久配置时，前台与 `--background` 启动均无需点击设置页按钮即可由唯一 Rust collector 自动开始取帧，Channel / 概览 / 实时页收到权威状态。
- [x] **AC3 状态与用户意图**：未配置、连接中、已连接、鉴权失败、端点缺失、暂停、手动断开和存储故障均有可操作中文状态；窗口保持打开时手动断开不会被重绘自动覆盖，冷启动或重新打开主窗口会恢复自动尝试。
- [x] **AC4 单循环与安全**：代码与测试证明没有第二条采集循环，`test_controller` 未被当作监控；secret 扫描无新增泄漏。
- [x] **AC5 设置架构**：`settings-data` 具有可键盘操作的二级设置导航，五个分组内容完整且危险区域独立；固定顶级导航与 Recovery Shell 不回归。
- [x] **AC6 现代工艺**：主题 / 语言、表单、状态、按钮、帮助文本与危险操作在四套主题下层级一致；40 px 命中区、可见焦点、reduced motion、无 `transition: all` 通过检查。
- [ ] **AC7 视觉与窗口**：1200×800 与窄窗口实拍无截断、重叠或水平溢出；桌面首屏可见连接状态及主要连接配置。
- [x] **AC8 自动门**：前端 typecheck、lint、tests、build 与后端 fmt、clippy、tests 通过；未运行的 Windows 安装态 / Credential Manager 证据单独标为未验证，不以浏览器 fixture 冒充。

## Out of Scope

- 重设计概览、实时连接、分析报告或告警页面。
- 新 UI 框架、远程字体 / 图标、Accent Color、用户自定义字体或新增主题。
- 修改登录自启动的默认值、创建 Windows Service、支持远程控制器或多控制器。
- 自动安装、真实 Credential Manager 写入、NSIS 安装态变更、远程提交或发布。

## Key Decisions

- 仅在存在已保存且通过现有 Rust 校验的控制器地址时自动连接；首次安装不自动尝试 `127.0.0.1:9097`，该值只作为设置提示。
- “打开应用”包括 owner 冷启动、后台启动，以及从托盘或第二实例激活既有 owner 后重新显示主窗口；这些路径统一复用同一个 reconnect / collector seam。
- SkillPort 只作为设置页信息架构与工艺参考；不复制其品牌、功能、字体或红色强调色，也不重做其他业务页面。

## Planning Status

- `in_progress`；父规划已获批准，两个子任务已完成实现与自动化检查。
- 当前剩余门槛：真实 Tauri WebView 1200×800 / 窄窗口截图、键盘走查和四主题实拍仍未验证，因本机已运行安装实例占用 single-instance。
