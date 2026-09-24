# Rust continuation evidence — 2026-09-20

本次检查没有发现需要再添加的 Rust 产品代码。档案积压的高优先级改动已经在
`src-tauri/src/c3/archive.rs`：`ArchiveDescriptor` 只保留 kind、UTC 范围、时区、
fingerprint 和重试状态，实际 `ReportQuery` 在派发时重建；`ArchiveScheduler` 使用有界
`VecDeque`、单在途作业、失败退避和队尾公平推进。已有测试同时覆盖描述符往返、积压上限、
不重复查询、失败后继续推进、重启恢复、时钟回拨、时区/DST 边界和内部查询不创建 spool。

验证命令：

```text
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml c3::archive::archive_service_tests --lib
22 passed, 0 failed

cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml raw_bucket_series --lib
3 passed, 0 failed
```

报告查询的当前代码也已有逐桶 raw series 优化和等价性/索引/取消回归。隔离大窗口阶段证据
（`research/performance-20260919-query-stages/series-stage-findings.md`）显示 29 日
A=1000 时间序列从 35,604.236ms 降至 26,297.359ms（结果摘要一致），但同一快照的
totals/attribution 为 30,567.243ms，Host ranking 为 53,544.078ms，仍未满足 10 秒报告门。
因此本次不对 totals/ranking 做未经阶段计划和语义等价证明的 SQL 重写；AC7/AC8 仍保留为
未完成证据，不把局部 series 改进宣称为完整大窗口通过。

下一步若继续优化，应先在同一只读快照对 totals 与 ranking 各自取得 plan/阶段耗时和结果
等价性，再选择索引范围或 Top-N seek；当前证据不足以安全确定其中任一方案。
