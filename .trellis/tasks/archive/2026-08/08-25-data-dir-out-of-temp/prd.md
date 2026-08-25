# PRD：监控数据目录迁移出 Temp

来源：`08-25-home-ip-traffic-breakdown/research/findings.md` 发现 4。

- `residential-monitor/src-tauri/src/lib.rs:307-309`：默认数据目录为 `std::env::temp_dir().join(IDENTIFIER)`，即 `%TEMP%\io.github.bahayonghang.residential-monitor`。
- 用户决策（2026-08-25）：默认数据目录放**应用安装目录下的 `data\` 子目录**（exe 同级），不放 Temp，也不放 LocalAppData 标识符目录。当前安装位置 `AppData\Local\家宽流量监控\` 为 per-user 安装，目录可写。
- `residential-monitor/docs/data-directory.md` 承诺的默认位置是 `%LOCALAPPDATA%\io.github.bahayonghang.residential-monitor`。代码与文档不一致。
- 现实后果：531 MB 主库（monitor.sqlite3 + WAL/SHM + report-spool + archive-tick）位于 Temp，Windows 磁盘清理 / Storage Sense 可随时删除，采集历史会无提示丢失。日志目录已正确落在 LocalAppData（`app_log.rs:60-70`），只有数据目录错位。

## 需求

- R1 默认数据目录改为 `<安装目录>\data`（`std::env::current_exe()` 同级 `data` 子目录），与安装方式无关地跟随 exe；`RESIDENTIAL_MONITOR_DATA_DIR` 环境变量优先级不变。
- R2 既有安装首次启动时自动迁移：Temp 旧目录 → `<安装目录>\data`，同卷优先 rename；迁移前校验，迁移后核对行数与字节总量守恒；失败回滚到旧目录继续可用（Recovery Shell 语义）。
- R3 迁移对象：`monitor.sqlite3`（含 WAL/SHM）、`report-spool/`、`archive-tick/`；迁移成功后删除 Temp 旧目录。
- R4 迁移过程对用户可见且可重试：启动进度或失败提示，不在静默中丢数据。
- R5 校验 C5 声明清单（`c5/purge.rs` `declared_items`）、备份 / 恢复、VACUUM 路径在新目录下工作（它们均从 `data_dir` 派生，理论上单点改动，需测试证实）。
- R6 卸载保留数据：安装器卸载流程必须跳过 `data\` 子目录，维持 `docs/data-directory.md`「普通卸载保留数据」契约；通过 tauri NSIS 自定义 hook 实现（`tauri.conf.json` bundle.windows.nsis）。

## 设计注意（非阻塞）

- 安装目录不可写时（假想 machine-wide 安装到 Program Files）：启动检测到不可写时回退 Recovery Shell 报错，不静默换目录。当前 per-user 安装不受影响。
- 开发态（cargo/tauri dev）exe 在 `target/debug` 下：继续用 `RESIDENTIAL_MONITOR_DATA_DIR`（现有测试与 bench 已是此模式）；不带环境变量的 dev 运行会把数据写到 `target/debug/data`，可被 `cargo clean` 清掉，属可接受行为。

## 验收标准

- A1 全新安装启动后主库位于 `<安装目录>\data`，Temp 不再产生应用数据。
- A2 本机现场迁移：`connection_minute` 行数（380,394+）与 upload/download 总量迁移前后相等；531 MB 库迁移耗时记录在案。
- A3 迁移中断（模拟 rename 失败 / 目标占用）后旧库完好、应用可启动、可重试。
- A4 「删除全部本地数据」预览清单指向新目录；备份 / 恢复在新目录往返成功。
- A5 卸载后 `<安装目录>\data` 保留（手测一次 NSIS 卸载）。
- A6 `cargo test` 全量通过；`docs/data-directory.md` 与代码行为一致。

## Out of Scope

- Temp 下 `c5-fault-backup-*` / `c5-fault-restore-*` 残留目录清理（soak 测试工件，非应用管理对象）。
- 数据库 schema 变更与保留策略调整。

## Open Questions

- 无阻塞项。迁移提示用启动横幅还是 Recovery Shell 入口，留 `design.md` 决策。
