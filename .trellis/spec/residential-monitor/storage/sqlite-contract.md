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
