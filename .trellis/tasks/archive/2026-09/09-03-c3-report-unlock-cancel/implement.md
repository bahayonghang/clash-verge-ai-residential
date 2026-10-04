# 实现：报告放锁与取消

1. 给 `OperationRegistry` 增加 `cancel_flag(id) -> Option<Arc<AtomicBool>>`；`start_operation` 创建 flag。
2. `run_report` / `residential_share` / `export_report` / `run_retention` / `create_backup`：短锁检查 branch 与 path，clone 需要的 path/space，放锁，执行，再短锁 persist。
3. `archive_tick` 的 cancel 改为 registry 或独立 tick flag，禁止写死 `false`。
4. 测试：假 ReportService 睡眠期间 `query_live_connections` 获锁；cancel 后返回 `cancelled`。
5. 验证：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c2::` 与 `c3::`。
