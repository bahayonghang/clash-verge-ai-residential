# 数据目录

默认数据目录是用户 LocalAppData 下的 `io.github.bahayonghang.residential-monitor`。

开发态可用环境变量 `RESIDENTIAL_MONITOR_DATA_DIR` 覆盖。

| 对象 | 路径 |
|---|---|
| 主库 | `monitor.sqlite3` |
| WAL / SHM | `monitor.sqlite3-wal` / `monitor.sqlite3-shm` |
| 报告 spool | `report-spool/` |
| 日志 | `%LOCALAPPDATA%\io.github.bahayonghang.residential-monitor\logs\residential-monitor.log` |

日志目录不跟随 `RESIDENTIAL_MONITOR_DATA_DIR`。测试可用 `RESIDENTIAL_MONITOR_LOG_DIR` 覆盖。当前文件超过 2 MiB 时轮转，最多保留 5 个文件（当前文件与 `.1`–`.4`）。只记录启动、采集生命周期、会话码变迁、存储打开类别、备份/恢复/保留/VACUUM/删除与告警失败，不含 secret、完整域名 / IP / 进程路径或每秒采集帧。

设置 / 数据管理与 Recovery 壳可打开该目录。普通卸载保留上述对象和 Credential Manager 项。应用内「删除全部本地数据」才按声明清单分项删除（含日志目录）。

`retention_preview` 的 `hourlyRows` / `dailyDimRows` 统计 `traffic_hourly_dimension` / `traffic_daily_dimension` 的行数。精确层按 host / process / rule_group / chain / network 五种 `dimension_kind` 各写一行，所以这两项约为仅物化 host 时的五倍。行数口径是表行，不是独立小时或自然日数。
