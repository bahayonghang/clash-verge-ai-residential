# 托盘图标状态与左右键

## Goal

Windows 通知区里的家宽监控托盘图标能扫到当前运行状态。左键打开主窗口，右键打开菜单。窗口关闭后仍能从托盘判断采集是否在跑、控制器是否健康。

## Background

C2 已要求托盘提供打开窗口、暂停 / 继续采集、立即重连、权威健康摘要和明确退出（`08-18-monitor-desktop-realtime` C2-R1）。当前实现只交付了静态菜单，没有把健康摘要画到图标或 tooltip 上，也没有把左键绑定到打开窗口。

本任务补齐这块外壳行为。不改采集、核算、报告或告警语义。

## Confirmed Facts

- 托盘在 `residential-monitor/src-tauri/src/lib.rs` `build_tray`（约 868–937 行）创建：id `main`，菜单固定为打开窗口 / 暂停采集 / 继续采集 / 立即重连 / 退出。暂停与继续始终同时可点。
- Tauri 2.11.5 默认 `show_menu_on_left_click = true`。当前没有 `on_tray_icon_event`，左键会弹出菜单。菜单 `"open"` 会 `desktop.open_window()` 并 `show` + `set_focus`。
- 图标使用 `app.default_window_icon()`，与窗口图标 `src-tauri/icons/icon.png` 相同：白底圆角方块 + 海军蓝房子与 wifi 标记。
- tooltip 只设为 `product.display_name`（「家宽流量监控」），在创建托盘和 `apply_locale_chrome` 时各写一次，不随采集或健康变化更新。
- `TraySummary`（`c2/desktop.rs` 41–45 行）已有 `collectorRunning`、`health`、`windowVisible`。`tray_summary` command 把 `health` 设为 `hub.overview().health.session`。前端只用它读 `collectorRunning`（`src/main.ts` 1019–1020 行）。
- 采集是否在跑看 `desktop.collector_running`，不是 `health.session`。暂停权威来源见 `.trellis/spec/residential-monitor/frontend/view-state.md`。
- `session_status_name` 产出：`connecting`、`connected`、`tcp_unauthorized`、`pipe_access_denied`、`pipe_busy_timeout`、`endpoint_missing`、`protocol_incompatible`、`pid_mismatch`、`core_restarted`、`cancelled`、`non_loopback`。健康标题已在 `i18n.rs` 与前端 i18n 表。
- `HealthView` 另有 `storage_ok`。采集循环 `collector_loop_tick` 约 1 Hz 推进会话健康，但不刷新托盘。
- 产品范围是 Windows 11。关闭主窗口只隐藏到托盘；明确「退出」才 shutdown。
- 截图中的菜单五项与当前 `build_tray_menu` 一致。

## Requirements

### R1 左键打开窗口，右键打开菜单

- 托盘图标单击左键：显示并聚焦主窗口，行为与菜单「打开窗口」相同。窗口已可见时仍聚焦。
- 左键不得弹出菜单。
- 右键弹出既有托盘菜单。菜单项仍为：打开窗口、暂停采集、继续采集、立即重连、退出。不新增项，不改这些项的既有生命周期语义。
- 双击左键若到达，同样打开窗口，不另做动作。

### R2 图标用右下角色点显示四种聚合状态

- 保留现有白底房子 + wifi 产品标记。右下角加实心色点。禁止只改 tooltip、图标外观不变。
- 权威输入：`collector_running` + `health.session` + `health.storage_ok`。前端不维护第二套托盘状态。
- 映射：

  | 视觉 | 色点 | 条件 |
  |---|---|---|
  | 采集中 | 绿 | `collector_running` 且 `storage_ok` 且会话 `connected` |
  | 连接中 | 蓝 | `collector_running` 且 `storage_ok` 且会话为 `connecting` 或 `core_restarted` |
  | 已暂停 | 黄 | `collector_running == false`（优先于会话码与存储健康） |
  | 故障 | 红 | 其余情况（其余会话码，或 `storage_ok == false`） |

- 16×16 与高 DPI 下四种色点可区分。色点颜色固定，不跟随 `ui_theme`。
- 图标资源生成本地文件，放在 `src-tauri/icons/`。任务栏 / 窗口图标保持无色点的产品标记。

### R3 tooltip 写产品名与精确状态

- tooltip 格式：`{产品名} — {状态标题}`。状态标题复用已有 `health.*` 文案。
- `collector_running == false` 时标题键为 `paused`。
- `collector_running == true` 且 `storage_ok == false` 时标题键为 `storage_failure`。
- 其余用当前 `health.session` 对应的 `health.{session}`。未知键回落到会话码本身。
- 语言跟随 `ui_locale`。切换语言后图标状态与 tooltip 同步更新。

### R4 菜单项反映可否暂停 / 继续

- 「暂停采集」仅在 `collector_running == true` 时可用。
- 「继续采集」仅在 `collector_running == false` 时可用。
- 打开窗口、立即重连、退出保持始终可用。两项都保留在菜单里，不可用的一项禁用。不增加状态只读行。

### R5 托盘外壳随权威状态刷新

- 暂停、继续、立即重连、采集 tick 导致的会话或存储健康变化、语言切换后，图标、tooltip 与暂停/继续可用性与映射表一致。
- WebView 隐藏或销毁时托盘仍更新。不把托盘刷新放到前端。
- 视觉未变时不要每秒重设图标，避免通知区闪烁。

## Acceptance Criteria

- [ ] **AC1** 单击托盘左键显示并聚焦主窗口；左键不弹出菜单。
- [ ] **AC2** 右键弹出菜单，五项文案与现网一致；暂停与继续不会同时可点。
- [ ] **AC3** 采集中 / 连接中 / 已暂停 / 故障四种右下角色点在通知区可区分；窗口任务栏图标不随状态切换。
- [ ] **AC4** 暂停采集后图标与 tooltip 在该次操作内变为已暂停；继续采集且会话 `connected`、存储健康后回到采集中。
- [ ] **AC5** tooltip 为「家宽流量监控 — {状态标题}」（英文 locale 用对应产品名与 health 文案）。`collector_running == false` 时状态标题是采集已暂停。
- [ ] **AC6** 鉴权失败、端点不存在、已取消等故障会话显示红点；存储不健康且采集仍在跑时同样红点，tooltip 为存储故障。
- [ ] **AC7** 映射函数有单元测试：`(collector_running, session, storage_ok) -> 视觉状态 + tooltip 键`。`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace` 与 `npm --prefix residential-monitor` 的 typecheck / lint / test 通过。
- [ ] **AC8** 关闭窗口隐藏到托盘后，托盘仍按 R5 更新；明确退出仍走现有 shutdown。

## Out of Scope

- 不改采集循环、核算、报告、告警、备份、恢复。
- 不改菜单项动作语义（暂停 / 继续 / 重连 / 退出）。
- 不在菜单里增加状态只读行、流量数字或连接数。
- 不把状态画到任务栏窗口图标上。
- 不支持 macOS / Linux 托盘。
- 不引入系统通知气泡作为状态载体。
- 不跟随 `ui_theme` 更换托盘标记底色或色点。
- 不为每个 `health.session` 码各做一枚独立图标。

## Key Decisions

- 左键打开、右键菜单：用户指定。
- 视觉编码：保留房子标记，右下角四态色点（采集中绿 / 连接中蓝 / 已暂停黄 / 故障红）。用户已确认建议方案。
- 暂停优先于会话码与存储健康：与现有前端空态契约一致。
- 托盘权威在 Rust：与 C2「实时摘要与托盘摘要来自同一份 Rust 投影」一致。
- 窗口图标不随状态变：任务栏保持产品标记。
- 暂停与继续都留在菜单，不可用项禁用。

## Notes

- 本任务保持 `planning`，直到用户批准本规划摘要后才能 `task.py start`。
- `08-19-ai-route-domain-audit` 仍为 `in_progress`。当前 Trellis 指针在本任务。
