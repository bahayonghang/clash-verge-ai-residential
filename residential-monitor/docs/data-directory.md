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
| 日志 | `%LOCALAPPDATA%\io.github.bahayonghang.residential-monitor\logs\residential-monitor.log` |

日志目录不跟随 `RESIDENTIAL_MONITOR_DATA_DIR`。测试可用 `RESIDENTIAL_MONITOR_LOG_DIR` 覆盖。当前文件超过 2 MiB 时轮转，最多保留 5 个文件（当前文件与 `.1`–`.4`）。只记录启动、采集生命周期、会话码变迁、存储打开类别、备份/恢复/保留/VACUUM/删除与告警失败，不含 secret、完整域名 / IP / 进程路径或每秒采集帧。

设置 / 数据管理与 Recovery 壳可打开该目录。普通卸载保留上述对象和 Credential Manager 项。应用内「删除全部本地数据」才按声明清单分项删除（含日志目录）。

`retention_preview` 的 `hourlyRows` / `dailyDimRows` 统计 `traffic_hourly_dimension` / `traffic_daily_dimension` 的行数。精确层按 host / process / rule_group / chain / network 五种 `dimension_kind` 各写一行，所以这两项约为仅物化 host 时的五倍。行数口径是表行，不是独立小时或自然日数。

保留维护先分块构建和复核一个完整 UTC 日的精确汇总；该日字节、维度、去重数、时长和覆盖全部核验后，才允许分块删除明细。进度与当前块同事务保存，中断后可继续；取消、失败或汇总损坏不能推进删除水位。构建期间会占用临时的数据库页。

无明细引用且已关闭或有持久退役证明的会话才能回收，活跃、待重试及证据不足的旧会话保留。提交回执保留最近 24 小时与最新 100000 条的并集，持久过期边界继续拒绝旧回执重放。自动明细删除目前仍关闭，等待完整守恒与容量验收。

DELETE 释放的是 SQLite 可复用页，主库文件不一定变小；WAL、有效页和可复用页应分别查看。应用不会自动 VACUUM。需要物理缩小文件时使用已有手动维护入口，并满足空间和采集停用条件。内部档案与周期告警不再创建 `archive-tick` 临时 spool，旧目录不因此自动删除。
