# 密钥与取消

- secret 只存在于 Credential Manager 或当前进程内存。
- 日志、SQLite、Channel、错误、诊断和导出不得包含 secret。日志与诊断共用 `redact::scan_text_for_secrets`（`bearer ` / `password=` / `secret=` / `authorization:` / `credential`）。
- 设置页可通过 `get_controller_secret` 把密钥写进密码框的 `input.value`。默认 `type=password`，由「显示密钥」按钮改成明文。密钥不得插进 HTML 模板、日志或 Channel。
- 保存与测试连接默认 `session_only=false`，写入本机凭据。Credential Manager 不可用时才退回当前进程会话 secret。
- C1 使用 FakeCredentialStore。C2 `SettingsWorkflow` 实现补偿：先写 pending、读回验证、probe、再写稳定 target；失败删除 pending 并保留旧引用。
- Windows Credential Manager adapter 已存在于 `credential::windows_cm`。`credential_windows_generic_crud` 保持 `#[ignore]`，未获本机写入授权不得跑。
- Credential Manager 不可用时只允许 `ProcessLocalStore` 会话 secret，退出或替换后必须 `clear`。v1 无 DPAPI fallback。
- 长操作必须可取消。SQLite 使用 interrupt / progress。
- C2 `FileDialogPort` 只返回预声明用途的用户选择路径。
- C3 真实 operation：`run_report`、`export_report`、`create_backup`、`restore_backup`、`run_retention`、`list_report_archives`、`get_report_archive`。取消必须 interrupt 实际 SQLite / 备份 step，不只丢弃前端结果。自动档案生成走同一 `ReportService` 取消 / deadline 路径。
- rusqlite `progress_handler` 返回 `true` 表示中断，`false` 表示继续。
- secret 不得进入 URL、日志、SQLite、Channel、预览、导出、诊断或 Release 资产说明。导出与诊断前扫描 `bearer ` / `password=` / `secret=`。
- C4 诊断只含白名单字段；完整域名、IP、进程路径和 Credential Manager 内容不得进入诊断包。
- C5 `confirm_delete_local_data` 只清除当前进程凭据引用。未再确认前不调用 Windows Credential Manager 真机删除。
- `monitor-db` 查询路径必须把取消标志接到 SQLite `progress_handler`（`attach_cancel`）。`vacuum` 与 `purge` 不可中断，运行时输出必须写明失败后状态与人工恢复步骤。
- `--redact` 下 host 与进程 identity 只输出 sha256 前 8 位加长度。完整进程路径任何模式都不进入 CLI 输出。

## Scenario: Display operation ownership

### 1. Scope / Trigger

Display queries can overlap background reports and other views. Each invocation owns its cancellation flag and registry lifetime.

### 2. Signatures

- `start_operation(operation_id, kind) -> Result<OperationProgress, AppErrorDto>`
- `cancel_operation(operation_id)` sets only that operation's flag.
- `finish_operation(operation_id) -> Result<Option<OperationProgress>, AppErrorDto>` removes the registry entry after the command settles.
- `OperationRegistry::resolve_cancel(operation_id, kind) -> Arc<AtomicBool>` resolves only an explicit matching ID. Missing/unspecified IDs receive independent flags.

### 3. Contracts

Use a fresh ID per invocation. Await start before invoking the query; attach abort to that ID; finish in `finally` after query settlement. Registry removal does not invalidate an `Arc` already held by SQLite. An internal query without an ID must never borrow an arbitrary operation with the same kind.

`OperationRegistry` clones share one registry mutex independently of the facade/writer mutex. Tauri manages a clone for `cancel_operation`; cancellation must not acquire the facade lock held by retention. Settings retention/backup/restore commands must pass the exact registered `operationId` as well as display queries.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Cancel one display ID | Other views and internal work remain uncancelled |
| Finish while a consumer retains the flag | Registry entry disappears; consumer flag remains valid |
| Query fails or is cancelled | Finish still runs; cleanup error does not replace the query error |
| Unknown ID | No other operation is selected by kind |

### 5. Good/Base/Bad Cases

Good: unique `display-${crypto.randomUUID()}` with balanced start/finish. Base: internal report uses an independent cancellation flag. Bad: selecting the first running `report` operation when no ID was supplied.

### 6. Tests Required

Assert exact-ID cancellation isolation, retained flag lifetime after removal, and zero retained entries after 3,600 balanced invocations. Frontend tests must cover abort before start resolves and query rejection cleanup.

### 7. Wrong vs Correct

Wrong: keep every completed operation in the registry, or reuse another query's cancellation flag by `kind`.

Correct: `try { return await run(id); } finally { await finish(id); }`, with one ID per invocation.
