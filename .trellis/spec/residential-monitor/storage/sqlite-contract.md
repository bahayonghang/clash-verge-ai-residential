# SQLite 契约

启动后每个连接显式设置：

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = FULL;
PRAGMA foreign_keys = ON;
```

- 单 writer。逐行只做 bind → step → reset。
- 缺口不得写成零。
- Online Backup 必须分页。不得复制热库文件并丢掉 WAL。
- 未来 schema 或 checksum mismatch 必须 fail closed。
- `busy_timeout` 由 C0 测量后冻结，不能超过 durable commit SLO 仍称为健康。
- SQLite `user_version`：C1 = 1 / checksum `c1-core-v1`；C3 = 2 / checksum `c3-report-v2`；C4 = 3 / checksum `c4-alert-v3`；C3 档案 = 4 / checksum `c3-archive-v4`。不得改写已发布 C1 / C3 / C4 migration 文本。`C3_DDL` 不得出现 `report_archive`。
- C3 追加表：`dimension_dict`、`connection_session_attr`、`traffic_hourly_dimension`、`traffic_daily_dimension`、`traffic_daily_core`、`coverage_daily`、`retention_state`、`retention_watermark`、`report_snapshot_meta`。
- C3 档案表（v4 `C3_ARCHIVE_DDL`）：`report_archive`。过期删除只针对该表，与 `AUTO_DELETE_ENABLED` 无关。
- C4 追加表：`alert_rule`、`alert_instance`、`alert_event`、`notification_outbox`。facts、coverage、alert 与 outbox 必须在同一 writer 事务中提交。
- `report_snapshot_token` 返回前必须关闭 SQLite read transaction。token 不持有连接或 WAL end mark。
- 自动 DELETE 保持关闭（`AUTO_DELETE_ENABLED=false`），直到守恒门通过。不自动 VACUUM。freelist 不得显示为已释放文件空间。
- 低空间 backup / restore / spool / VACUUM 必须 fail closed，不得覆盖当前可用库。
