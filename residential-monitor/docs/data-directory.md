# 数据目录

默认数据目录是应用安装目录下的 `data` 子目录（exe 同级，跟随安装位置）。
current-user 安装目录是 `%LOCALAPPDATA%\ResiWatch`。不要装到 `%TEMP%`：重启或 Storage Sense 会清掉二进制和数据。

0.2.0 及之前版本默认写在 `%TEMP%\io.github.bahayonghang.residential-monitor`；
升级后首次启动会把旧目录整体迁移到新位置（同卷 rename，逐项搬移时做 size 校验），
迁移失败时沿用旧目录并在下次启动重试，日志事件为
`data_dir_migrated` / `data_dir_skip` / `data_dir_migration_failed`。
若新位置已有主库则跳过迁移并保留旧目录。

开发态可用环境变量 `RESIDENTIAL_MONITOR_DATA_DIR` 覆盖；覆盖生效时不做迁移。

卸载保留 `data` 子目录：NSIS 卸载钩子（`src-tauri/installer.nsh`）在卸载前把它
搬到临时位置、卸载完成后搬回，其余安装文件正常删除。

| 对象 | 路径 |
|---|---|
| 主库 | `<安装目录>\data\monitor.sqlite3` |
| WAL / SHM | `<安装目录>\data\monitor.sqlite3-wal` / `-shm` |
| 报告 spool | `<安装目录>\data\report-spool\` |
| 归档节拍 | `<安装目录>\data\archive-tick\` |
| 日志 | `%LOCALAPPDATA%\io.github.bahayonghang.residential-monitor\logs\residential-monitor.log` |

日志目录不跟随 `RESIDENTIAL_MONITOR_DATA_DIR`。测试可用 `RESIDENTIAL_MONITOR_LOG_DIR` 覆盖。当前文件超过 2 MiB 时轮转，最多保留 5 个文件（当前文件与 `.1`–`.4`）。只记录启动、采集生命周期、会话码变迁、存储打开类别、备份/恢复/保留/VACUUM/删除与告警失败，不含 secret、完整域名 / IP / 进程路径或每秒采集帧。

设置 / 数据管理与 Recovery 壳可打开该目录。普通卸载保留上述对象和 Credential Manager 项。应用内「删除全部本地数据」才按声明清单分项删除（含日志目录）。

`retention_preview` 的 `hourlyRows` / `dailyDimRows` 统计 `traffic_hourly_dimension` / `traffic_daily_dimension` 的行数。精确层按 host / process / rule_group / chain / network 五种 `dimension_kind` 各写一行，所以这两项约为仅物化 host 时的五倍。行数口径是表行，不是独立小时或自然日数。
