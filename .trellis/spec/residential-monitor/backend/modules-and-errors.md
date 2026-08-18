# 模块与错误

- 稳定 identifier：`io.github.bahayonghang.residential-monitor`。
- 错误对前端只暴露稳定码和中文下一步动作，详情脱敏。
- HTTP 使用成熟实现，不手写完整 HTTP/1.1 解析器。
- TCP 只接受 loopback。named pipe 不发送 secret。
- C0 候选 schema 不得复制为 C1 正式 migration。
- C2 只消费 C1：`ControllerSession`、`AccountingEngine`、`StorageCoordinator`、`LiveProjection`、`RecoveryFacade`。C2 模块不得 `use rusqlite`，不得 `create table`。
- C2 代码位于 `residential-monitor/src-tauri/src/c2/`。
- Recovery Shell：`restoreAvailable` 保持 `false`，直到 C3 接入实际 restore。
- 调试：`just tdev`（`tauri dev`）。出包：`just monitor-build`（只生成 NSIS，不安装）。安装：`just tinstall`（会改本机 current-user 安装态）。未再确认前不要执行 `tinstall`、本机 Credential Manager 真机测试或登录自启动写入。

