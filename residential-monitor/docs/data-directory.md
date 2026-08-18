# 数据目录

默认数据目录是用户 LocalAppData 下的 `io.github.bahayonghang.residential-monitor`。

开发态可用环境变量 `RESIDENTIAL_MONITOR_DATA_DIR` 覆盖。

| 对象 | 路径 |
|---|---|
| 主库 | `monitor.sqlite3` |
| WAL / SHM | `monitor.sqlite3-wal` / `monitor.sqlite3-shm` |
| 报告 spool | `report-spool/` |
| 日志 | 应用 log 目录 |

普通卸载保留上述对象和 Credential Manager 项。应用内「删除全部本地数据」才按声明清单分项删除。
