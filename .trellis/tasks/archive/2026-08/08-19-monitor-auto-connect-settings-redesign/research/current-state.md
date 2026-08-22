# residential-monitor 当前状态研究

## 研究范围

本文件只记录规划阶段对仓库现状的证据，供父任务及两个子任务共享。它不是产品代码，也不授权启动任务。

## 启动、单实例与采集

- `residential-monitor/src-tauri/src/c2/facade.rs`：`AppFacade::boot` 正常分支从 `controller` 设置恢复地址、凭据引用和 `wizard_complete`；正常分支初始 `session_status` 为 `Connecting`，`ControllerSettings::default().address` 为空。
- `residential-monitor/src-tauri/src/c2/collector.rs`：`plan_tick` 是每次采集节拍的纯计划函数。只有 `NormalReady`、collector running、shutdown idle、非 `Cancelled`、地址非空且能解析为 loopback 时才 `should_fetch`；`fetch_snapshot` 调用既有 `ControllerSession`，`apply_tick_result` 将结果交给 facade。
- `residential-monitor/src-tauri/src/lib.rs`：Tauri setup 只启动一条以 `SAMPLE_INTERVAL_MS` 睡眠的 `collector_loop_tick` 循环；循环结束后同步托盘，不受 WebView 是否存在影响。`reconnect_now` / `resume_collector` 已存在并发布生命周期消息。
- `residential-monitor/src-tauri/src/c2/desktop.rs`：关闭窗口只隐藏，`open_window` 只恢复可见状态；`FocusExisting` 由 Windows named mutex / 非 Windows 进程锁识别。当前 `lib.rs` 在 `FocusExisting` 分支记录日志后直接返回，规划中的“第二实例重新打开”若需落地必须补 owner 激活信号，且不得新建 collector。
- `residential-monitor/src-tauri/src/lib.rs`：托盘左键与菜单打开路径最终调用 `open_main_window`；托盘 `reconnect` 菜单调用 facade 的 `reconnect_now`。当前 `open_main_window` 尚未自动恢复 `Cancelled`。

## IPC 与前端状态

- `residential-monitor/src/ipc/live-session.ts`：`subscribeMonitor` 建立并保留 Tauri `Channel`，`resyncMonitor` 更换 Channel；`queryLiveConnections` 与 `fetchTraySummary` 走 typed command。
- `residential-monitor/src/main.ts`：启动时先 `get_bootstrap`、应用 locale/theme，初始 route 为 `overview`，末尾订阅 Monitor Channel；bootstrap / connection delta 后刷新实时查询页。`test-controller` 调 `test_controller`，仅处理 probe 结果；`disconnect-controller` 调 `disconnect_controller`。
- `residential-monitor/src/main.ts:597` 附近：`renderSettings` 目前一次输出四个纵向 `.panel`，首块包含五步 wizard、locale、theme、address、secret、targets、保存 / 测试 / 断开；其后是数据、关于、删除。
- `residential-monitor/src/main.ts` 的事件委托在 `app` 上处理 `input`、`change`、`click`。动态 `renderApp` 会重写 `#app`，但会按 id 恢复 focus；新增设置二级导航必须另行保留草稿和 secret 的进程内状态，不能把 secret 插入 HTML。

## 现有契约与限制

- `.trellis/spec/residential-monitor/frontend/dto-and-decoding.md`：Rust 是 DTO 权威校验者；Channel 首帧必须 bootstrap，后续 seq 单调；列表走 query command；未知 / gap fail closed。
- `.trellis/spec/residential-monitor/frontend/view-state.md`：前端只保留导航、筛选、分页和 DTO cache；主题 / locale 走独立本机设置键；删除短语固定中文；Recovery 不增加删除入口。
- `.trellis/spec/residential-monitor/backend/modules-and-errors.md`：错误只暴露稳定码与当前语言下一步；secret 不进日志、SQLite、Channel、诊断或导出；collector 为约 1 Hz HTTP `/connections`；`test_controller` 不是循环。
- `.trellis/spec/residential-monitor/backend/secrets-and-cancellation.md`：secret 只能在凭据库或进程内存，设置页通过 `input.value` 回填；默认保存持久凭据，Credential Manager 不可用才退回 session secret。
- `PRODUCT.md`：Windows 11 Tauri WebView、Vanilla TypeScript + Vite、固定五个顶级页面、数据只留本机、操作型高密度界面、无远程资源 / CDN / UI 框架。
- 当前用户决策：首次无持久地址时不自动访问 `127.0.0.1:9097`；SkillPort 截图只提供二级导航、分组与控件层级参考。

## 相关并行任务风险

- `08-19-monitor-tray-status` 也修改 `residential-monitor/src-tauri/src/lib.rs` 的托盘与 `open_main_window` 接线。实现前必须重新读取其当前状态并基于最新文件调整，不得覆盖或回滚其改动。
- 当前工作区已有其他未提交修改；实施与提交只选择本任务拥有的文件和语义边界。
