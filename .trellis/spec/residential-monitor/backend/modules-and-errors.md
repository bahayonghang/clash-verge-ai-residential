# 模块与错误

- 稳定 identifier：`io.github.bahayonghang.residential-monitor`。
- 错误对前端只暴露稳定码和当前语言的下一步动作，详情脱敏。JSON 字段仍叫 `messageZh`。
- 界面语言键 `ui_locale`（`zh`/`en`）走 `put_setting`，不进控制器 JSON。`identity::PRODUCT_NAME` 与删除确认短语不随语言改。
- 外观键 `ui_theme` 与实时表列布局键 `live_table_layout` 同样走 `put_setting`，不进控制器 JSON。非法值回落默认。
- HTTP 使用成熟实现，不手写完整 HTTP/1.1 解析器。
- TCP 只接受 loopback。named pipe 不发送 secret。
- C0 候选 schema 不得复制为 C1 正式 migration。
- C2 只消费 C1：`ControllerSession`、`AccountingEngine`、`StorageCoordinator`、`LiveProjection`、`RecoveryFacade`。C2 模块不得 `use rusqlite`，不得 `create table`。
- C2 代码位于 `residential-monitor/src-tauri/src/c2/`。
- 产品进程用 `c2/collector.rs` 约 1 Hz HTTP GET `/connections`。`test_controller` 只取一帧，不能代替循环。HTTP 期间不得持 `Mutex<AppFacade>`。
- `Paused` / `Resumed` / `SleepGap` 发布时保留 `hub.rows()`。`Disconnected` 才允许清空。`session_status == Cancelled` 时跳过取帧；`reconnect_now` / `resume_collector` 必须离开 `Cancelled`，不得新开第二条循环。
- Tauri `Channel` 只放在 `lib.rs` 订阅表，不进入 `AppFacade`。
- 托盘 id `main`。Tauri 2 默认左键弹菜单，必须 `show_menu_on_left_click(false)`。左键 Up 与左键双击打开窗口，右键才是菜单。四态由 `c2::desktop::tray_chrome(collector_running, session, storage_ok)` 决定，资源是 `icons/tray-*.png`。窗口 `icon.png` 不随状态变。`just tdev` 重启后通知区才换图标。
- C3 代码位于 `residential-monitor/src-tauri/src/c3/`。C3 只通过 `StorageCoordinator` / `RecoveryFacade` 访问 SQLite，不得另建 writer 或通用 Repository。`ReportArchiveService` 拥有 `report_archive` 读写与过期删除。
- `collector_loop_tick` 在 `apply_tick_result` 之后调用 `archive_tick`。`ReportService::run` 不得持 `Mutex<AppFacade>`。每 tick 最多 1 份档案。临时 snapshot 必须打开独立目录（`data_dir/archive-tick`），不得 `ReportSnapshotStore::open(data_dir)`，否则 `cleanup_orphans` 会删掉门面仍有效的 spool token。
- Recovery Shell 与 shutdown 跳过档案调度，不初始化 `ReportArchiveService` 循环。
- C4 代码位于 `residential-monitor/src-tauri/src/c4/`。`AlertEngine` 拥有告警状态机；周期用量只调用 `ReportService`；通知只经 `NotificationSink`。C4 不得另建 writer 或第二套小时 / 日 / 月聚合。
- C5 代码位于 `residential-monitor/src-tauri/src/c5/`。只做发布硬化：关于页、删除、VACUUM、故障矩阵、并发 fixture、供应链与 C0 基线核验。不得改写 C1 核算、C3 报告 / retention / backup 或 C4 告警语义。
- Recovery Shell：`restoreAvailable` 为 `true`。restore 不初始化 `ReportService`；失败必须保留当前可用库。
- C4 前向表：`alert_rule`、`alert_instance`、`alert_event`、`notification_outbox`。不得改写 C1 / C3 已发布 migration。
- AUMID 与 identifier 相同：`io.github.bahayonghang.residential-monitor`。About 固定 Releases URL，不注册 updater plugin，不新增 Windows Service。
- 调试：`just tdev`（`tauri dev`）。出包：`just monitor-build`（只生成 NSIS，不安装）。安装：`just tinstall`（会改本机 current-user 安装态）。C5 自动门：`just monitor-c5-auto`。未再确认前不要执行 `tinstall`、本机 Credential Manager 真机测试或登录自启动写入。
- C5 完整 30 天库、24 小时 soak、安装态通知 / 签名 / GitHub Release 不得由 fixture 或 smoke 冒充完成。C0 升级基线缺失时 `monitor-bench c5-baseline` 退出码 2。

