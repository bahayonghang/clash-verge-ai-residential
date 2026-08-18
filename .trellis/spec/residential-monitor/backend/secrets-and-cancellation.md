# 密钥与取消

- secret 只存在于 Credential Manager 或当前进程内存。
- 日志、SQLite、Channel、错误、诊断和导出不得包含 secret。
- C1 使用 FakeCredentialStore。C2 `SettingsWorkflow` 实现补偿：先写 pending、读回验证、probe、再写稳定 target；失败删除 pending 并保留旧引用。
- Windows Credential Manager adapter 已存在于 `credential::windows_cm`。`credential_windows_generic_crud` 保持 `#[ignore]`，未获本机写入授权不得跑。
- Credential Manager 不可用时只允许 `ProcessLocalStore` 会话 secret，退出或替换后必须 `clear`。v1 无 DPAPI fallback。
- 长操作必须可取消。SQLite 使用 interrupt / progress。
- C2 `FileDialogPort` 只返回预声明用途的用户选择路径。
- C3 真实 operation：`run_report`、`export_report`、`create_backup`、`restore_backup`、`run_retention`。取消必须 interrupt 实际 SQLite / 备份 step，不只丢弃前端结果。
- rusqlite `progress_handler` 返回 `true` 表示中断，`false` 表示继续。
- secret 不得进入 URL、日志、SQLite、Channel、预览、导出、诊断或 Release 资产说明。导出与诊断前扫描 `bearer ` / `password=` / `secret=`。
- C4 诊断只含白名单字段；完整域名、IP、进程路径和 Credential Manager 内容不得进入诊断包。
- C5 `confirm_delete_local_data` 只清除当前进程凭据引用。未再确认前不调用 Windows Credential Manager 真机删除。
