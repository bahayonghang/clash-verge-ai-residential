# 模块与错误

- 稳定 identifier：`io.github.bahayonghang.residential-monitor`。
- 错误对前端只暴露稳定码和中文下一步动作，详情脱敏。
- HTTP 使用成熟实现，不手写完整 HTTP/1.1 解析器。
- TCP 只接受 loopback。named pipe 不发送 secret。
- C0 候选 schema 不得复制为 C1 正式 migration。
- C2 只消费 C1：`ControllerSession`、`AccountingEngine`、`StorageCoordinator`、`LiveProjection`、`RecoveryFacade`。C2 模块不得 `use rusqlite`，不得 `create table`。
- C2 代码位于 `residential-monitor/src-tauri/src/c2/`。
- C3 代码位于 `residential-monitor/src-tauri/src/c3/`。C3 只通过 `StorageCoordinator` / `RecoveryFacade` 访问 SQLite，不得另建 writer 或通用 Repository。
- C4 代码位于 `residential-monitor/src-tauri/src/c4/`。`AlertEngine` 拥有告警状态机；周期用量只调用 `ReportService`；通知只经 `NotificationSink`。C4 不得另建 writer 或第二套小时 / 日 / 月聚合。
- Recovery Shell：`restoreAvailable` 为 `true`。restore 不初始化 `ReportService`；失败必须保留当前可用库。
- C4 前向表：`alert_rule`、`alert_instance`、`alert_event`、`notification_outbox`。不得改写 C1 / C3 已发布 migration。
- 调试：`just tdev`（`tauri dev`）。出包：`just monitor-build`（只生成 NSIS，不安装）。安装：`just tinstall`（会改本机 current-user 安装态）。未再确认前不要执行 `tinstall`、本机 Credential Manager 真机测试或登录自启动写入。

