# 报告不持锁且可取消

## Goal

报告、份额、导出、retention、backup 的长 IO 不持 `Mutex<AppFacade>`。界面取消必须把同一 `AtomicBool` 设为 true，使 SQLite/backup 步骤中断。

## Background

规格：`ReportService::run` 不得持 facade 锁；`archive_tick` 已放锁。证据：`lib.rs:768-776` 持锁调用 `run_report`；`facade.rs:1229-1236` 每次 `AtomicBool::new(false)` 且不登记；`lib.rs:751-760` `cancel` 只打进度夹具。`REPORT_DEADLINE_MS` 为 10s，backup/export 无 deadline。

## Requirements

- R1 `run_report` / `residential_share` / `export_report` / `run_retention` / `create_backup` 短锁取出 path/spool/space，放锁后执行，再短锁写回结果或档案。
- R2 `start_operation` 把 `Arc<AtomicBool>` 登记进 `OperationRegistry`；`cancel_operation` 置位该 flag。
- R3 同一 flag 传入 `ReportService::run`、export、retention、backup（含 protect backup）。
- R4 Recovery-only 仍返回 `recovery_only`，不打开 writer。

## Acceptance Criteria

- [ ] AC1 报告运行期间 `pause_collector` / `query_live_connections` 能在采样间隔内获得锁（测试可用短 stub 或持锁计时断言）。
- [ ] AC2 取消进行中的 report/export/backup：返回 `cancelled`，无半成品覆盖热库。
- [ ] AC3 `archive_tick` 仍放锁后跑，行为不回退。
- [ ] AC4 Recovery-only 下上述写路径仍 `recovery_only`。

## Out of scope

- 拆 `AppFacade` 模块。
- 改 C3 SQL 语义或 Top N。
- 24 小时 soak。
