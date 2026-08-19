# 家宽监控本机日志系统

## Goal

用户或维护者在启动失败、进入 Recovery、采集断线或后台 `--background` 运行时，能在本机日志目录读到脱敏后的生命周期与错误记录，并从设置页或 Recovery 壳打开该目录。日志有容量上限并轮转。secret、Credential Manager 内容、完整域名 / IP、完整进程路径和原始连接 payload 不得进入日志。

本任务补齐父规划 `08-18-residential-monitor-mvp` R10 / AC10 已写明、C0–C5 未落地的本机日志能力。不改采集核算、报告口径或告警状态机。

当前状态：`planning`。实施须在用户批准本规划摘要并 `task.py start` 之后。

## Background

C0–C5 已交付采集、存储、桌面壳、报告、告警诊断与发布硬化。运行时错误目前只进入前端 `AppErrorDto`、告警 `notification_outbox.error_class`，以及 C4 `DiagnosticsSnapshot` 一次性 JSON。三者都不是按时间排列的操作日志。

父 PRD R10 要求日志轮转并脱敏；C2 design 启动顺序第 2 步为「初始化最小日志」；C4 R8 要求「诊断和日志按白名单投影并轮转」。C4 只落地了诊断快照。

## Confirmed Facts

- `residential-monitor/src-tauri/Cargo.toml` 无 `log` / `tracing` / `tauri-plugin-log`。`package.json` 无 `@tauri-apps/plugin-log`。前端 `src/` 无 `console.log`。产品 Rust 无 `log::` / `eprintln!`。`println!` 只在 `monitor-bench` 打 JSON。
- `capabilities/default.json` 只授 `core:default` 与窗口 show/hide/focus，不授 `fs` / opener / log plugin。
- `lib.rs` `boot_facade`（约 217–237 行）：`RESIDENTIAL_MONITOR_DATA_DIR` 缺省为 `temp_dir()/IDENTIFIER`。`StorageCoordinator::open` 失败时 `facade.rs:265` 进入 `RecoveryOnly` 并丢弃 `StorageError`。
- `lib.rs:1160-1161`：`run().expect("启动家宽流量监控失败")`，启动失败无文件。
- 采集 `c2/collector.rs:11`：secret 不写日志或 Channel。`apply_tick_result` 失败只改会话状态。失败可每秒一次，日志必须按状态变迁去重，不能每拍一行。
- C4 `diagnose.rs`：白名单健康 JSON + `notification_outbox.error_class` 最近 8 个 distinct。导出前扫描 `bearer ` / `password=` / `secret=` / `authorization:` / `credential`。`scan_text_for_secrets` 已公开。
- `Secret` 的 `Display` / `redacted()` 为 `<redacted>`（`credential.rs`、`controller.rs:187-188`）。
- `data_directory`（`lib.rs:670-677`）前端未调用。设置页数据区块无路径、无打开目录。`open_releases`（`lib.rs:806-808`）只把 URL 写进界面文案，不启动外部程序。
- Recovery：`main.ts:936-960` 隐藏导航，只渲染 `renderRecovery`（恢复备份）。`action.open_data_dir`（`i18n.rs:227-230`）是错误下一步文案，不是按钮。
- C5 `purge.rs` `declared_items` 无日志目录。普通卸载保留本地对象。
- 文档 `docs/data-directory.md:12` 写「日志 | 应用 log 目录」，未给实际路径或文件名。架构研究建议 `%LOCALAPPDATA%/{identifier}/logs` 与 `tauri-plugin-log`。实现缺省 `data_dir` 是临时目录；日志不得写入该 `data_dir`。
- C4 R1：健康告警不用日志反推状态。
- 稳定 identifier：`io.github.bahayonghang.residential-monitor`（`identity.rs`）。

## Requirements

### R1 本机文件日志

- 在 `AppFacade::boot` 与 Tauri `setup` 之前初始化。`--background`、Recovery、库打开失败、panic hook 能写入已打开的文件。
- 目录：`%LOCALAPPDATA%\io.github.bahayonghang.residential-monitor\logs`。不复用 `data_dir`。`RESIDENTIAL_MONITOR_DATA_DIR` 不改日志目录。测试与开发可用 `RESIDENTIAL_MONITOR_LOG_DIR` 覆盖。
- 文件名 `residential-monitor.log`。开发态 `debug_assertions` 同时写 stderr。正式包默认只写文件。
- 初始化失败不得阻止启动；之后写入失败不得中断采集或告警提交。

### R2 白名单事件

默认落盘 `error` / `warn`，以及下列 `info`（稳定事件码 + 白名单元数据，不记 payload）：

| 域 | 事件 |
|---|---|
| 启动 | launch mode、boot branch、app version、单实例冲突退出 |
| 采集 | 暂停、继续、立即重连；会话码**变迁**（`session_status_name`） |
| 存储 | 打开成功 / 失败类别、进入 Recovery、shutdown checkpoint / 关闭 |
| 维护 | 备份、恢复、保留、VACUUM、删除本地数据的分项结果 |
| 告警 | 规则变更、通知能力不可用、outbox 永久失败（只记 `error_class`） |

禁止：TCP secret、Credential Manager 内容、完整域名 / IP、完整进程路径、原始 `/connections` JSON、每秒成功采集帧、Channel 全量快照、`StorageError` 原始 Display 字符串（ rusqlite 文案可含路径）。

### R3 轮转

- 当前文件超过 2 MiB 则轮转。最多保留 5 个文件（当前 + `.1` … `.4`），总占用约 10 MiB。
- 日志不是 SQLite 账本，不加 C1–C4 migration。

### R4 脱敏

- 每条写入使用与 C4 诊断同一套禁止子串。命中则该字段改为 `<redacted>`。
- 允许字段：UTC 时间、级别、事件码、已有错误码 / 会话码、数量、布尔、角色化路径（`log_dir` / `data_dir`，不记用户任意导出文件内容）。
- 单测：secret 字节、`password=`、完整 host 不得出现在编码后的日志行。

### R5 打开目录与删除

- `BootstrapDto` 增加 `logDir`（camelCase）。设置 / 数据管理与 Recovery 壳都显示该路径，并提供「打开日志目录」。
- 打开走专用 Rust command，只打开解析后的日志目录。不授 WebView `fs` / `opener:default`。不套用 `open_releases` 的「只回字符串」实现。
- 「删除全部本地数据」预览与执行把日志目录列为声明项。部分失败不得显示「已全部删除」。普通卸载仍保留日志。Recovery 壳不加删除入口。

### R6 文档

更新 `docs/data-directory.md`、`docs/troubleshooting.md`、`docs/privacy.md`：路径、文件名、轮转上限、事件范围、禁止字段、设置页与 Recovery 如何打开目录。

## Out of Scope

- 应用内日志查看器或实时日志页。
- 把日志打进 `export_diagnostics`。
- 远程上报、崩溃分析服务、Sentry、Windows Event Log 作为主存储。
- 用日志反推告警或 coverage。
- 按连接或按帧记录字节。
- 把缺省 `data_dir` 从临时目录改到 LocalAppData。
- 采集 WebView 未捕获异常。
- 给 WebView 授予 log plugin 或文件系统权限。
- 接入 `tauri-plugin-log`（见 design：无法在 `boot_facade` 之前初始化，且不必给 WebView 插件权限）。
- 捕获 Tauri / WebView 内部 `log` 记录进同一文件。
- macOS / Linux 日志路径验收。
- 改 C1–C4 已发布 migration、核算公式、报告口径。

## Acceptance Criteria

- [ ] **AC1 文件存在**：普通启动与 `--background` 后，`%LOCALAPPDATA%\io.github.bahayonghang.residential-monitor\logs\residential-monitor.log` 存在（测试可走 `RESIDENTIAL_MONITOR_LOG_DIR`）。
- [ ] **AC2 启动失败可查**：注入库打开失败后进入 Recovery；日志含启动事件与存储失败类别，不含 secret，不含 rusqlite 原始 Display。
- [ ] **AC3 生命周期**：暂停、继续、立即重连、shutdown 各有对应事件码。同一会话码连续失败不每秒追加。1 Hz 成功帧不产生按连接或按帧的 info 行。
- [ ] **AC4 脱敏**：夹具写入 secret / `password=` / 完整 host 后，文件扫描为零命中。
- [ ] **AC5 轮转**：超过 2 MiB 出现下一份；文件数 ≤ 5。写失败时采集循环仍推进。
- [ ] **AC6 打开目录**：设置页与 Recovery 壳都显示 `logDir`；「打开日志目录」只打开该目录。capability 仍无 `fs` / 通用 opener。
- [ ] **AC7 删除清单**：预览含日志目录；确认后日志文件不在；失败项单独报告；文案不含「已全部删除」。
- [ ] **AC8 质量门**：`cargo fmt --check`、`clippy -D warnings`、`cargo test --workspace`、`npm --prefix residential-monitor` 的 typecheck / lint / test / build、`npm run check:secrets` 通过。

## Key Decisions

- 用户面方案 A（2026-08-19）：文件日志 + 路径展示 + 打开目录 + 删除清单 + 文档。无应用内日志页；诊断导出仍为 C4 JSON。
- Recovery 壳同样显示路径并提供「打开日志目录」（2026-08-19）。不加删除入口。
- 日志与诊断分开。日志不是告警源。
- Rust 拥有写入与打开目录。WebView 不获 log/fs 权限。
- 日志目录固定 LocalAppData `\logs`，不跟临时 `data_dir`。本任务不迁 SQLite。
- 不采集 WebView 未捕获异常。
- 会话失败按变迁记录，不按采集拍记录。

## Technical Notes

实现边界、事件码、轮转与打开目录的系统调用见 `design.md`。执行顺序见 `implement.md`。
