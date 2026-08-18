# C2 Gate 状态

生成时间：2026-08-18。未跑的本机写入项不得标完成。

## Gate

| Gate | 结果 | 证据 |
|---|---|---|
| 1 C1 契约 | 通过 | `c2::facade::c2_only_names_frozen_c1_owners`；`normal_boot_does_not_create_report_tables`；C2 目录无 `use rusqlite`、无 `create table` |
| 2 生命周期 | 自动化通过；安装态未跑 | `desktop_lifecycle_tests`；未执行本机 NSIS 安装、未写登录自启动 |
| 3 凭据与向导 | Fake / 进程内通过；本机 CM 未写 | `settings_workflow_tests`；`credential_windows_generic_crud` 仍为 ignored |
| 4 原子订阅 | 通过 | Rust `channel_contract_tests`；TS `reducer.test.ts` |
| 5 实时口径与关闭 | 通过 | `missing_id_204_is_accepted_until_remove`；`controller_session_close_missing_id_is_accepted`；概览字段分开展示 |
| 6 Recovery Shell | 通过 | `future_schema_enters_recovery_without_writer`；`restore_available=false` |
| 7 性能 / 安全 / 可用性 | 短时 UI 峰值见 `c2-peak-ui.json`；安装态 smoke 未跑 | 10k 连接 map 有界；筛选 p95 门限在单元测试中检查 |

## 暂停项

- 未向本机写入 NSIS 安装、登录自启动、Credential Manager 条目或系统通知。
- 未连接用户正在使用的 Clash Verge 控制器，未发送真实 DELETE。
- 未跑 24 小时 soak、代码签名、GitHub Release。
