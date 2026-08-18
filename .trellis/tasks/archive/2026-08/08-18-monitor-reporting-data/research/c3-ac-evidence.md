# C3 验收证据

| 项 | 命令或文件 | 结果 |
|---|---|---|
| C3-AC1 依赖门 | `c3_reuses_c1_storage_and_c2_recovery`；C3 表追加；无 C4 alert/outbox | 通过。C2 无 rusqlite 产品代码路径；restore 走 RecoveryFacade |
| C3-AC2 报告正确性 | `golden_totals_and_token_close_transaction`；`restart_same_query_same_totals`；DST / 空区间 / 能力过期 | 通过。空区间为 0，缺口不写零 |
| C3-AC3 快照一致性 | `same_token_export_matches_ui_totals`；token 关闭事务后 checkpoint | 通过 |
| C3-AC4 查询控制 | 命名 SQL corpus；无深 OFFSET；progress interrupt；cancel fixture | 通过。页面 2s / 报告 10s 为候选 deadline |
| C3-AC5 精确保留 | raw 默认 30 / 上限 90；dimension 396 天；core 长期；无 Top K 截断表 | 通过。能力过期明确不支持 |
| C3-AC6 retention 可恢复 | `crash_after_materialize_can_resume`；`materialize_is_idempotent_and_auto_delete_stays_off` | 通过。自动 DELETE 关闭 |
| C3-AC7 导出有界 | 三种格式流式写；取消清理 partial；secret 扫描拒绝 Bearer | 通过 |
| C3-AC8 备份恢复 | 持续写 backup、坏候选、未来 schema、低空间、取消 | 通过。失败保留当前库 |
| C3-AC9 Recovery Shell | `restore_available=true`；future schema 进入 RecoveryOnly 仍可 restore | 通过。restore 不初始化 ReportService |
| C3-AC10 并发性能 | `writer_report_checkpoint_can_run_together` | fixture 通过。完整三档 30 天库未重跑 |
| C3-AC11 独立回滚 | `AUTO_DELETE_ENABLED=false`；报告可停用路由；migration 只前进 | 通过 |

## 未执行

- 完整 30 天 `A=50 / 250 / 1000` 库重跑
- 24 小时 soak、代码签名、GitHub Release
- 本机 NSIS 安装、登录自启动、Credential Manager 真机写入
