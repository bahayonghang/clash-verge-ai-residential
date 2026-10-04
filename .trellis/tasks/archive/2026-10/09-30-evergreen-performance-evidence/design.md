# T05 设计

## Mechanism

run_uncached -> fill_raw -> load_sessions/fold_window 是证据链。现有 <=3120 分钟窗口投影解决分钟场景；30 天长窗口仍需全量访问。先恢复原始 baseline c278bb7b56603001e32e353d2ee589dccef0bfe9 加相同 bench instrumentation，再测 projection/scan/reader/ingest。替代算法、索引或物化层必须先说明计数/过滤/previous-window 语义及守恒 oracle。若需要改 schema、调度器或其它文件，重新批准，不在本轮预设实现。

## Owned Files

### 2026-09-30 范围内合同修复

新 oracle 在未优化原实现上发现 `network=__unknown__` 回归，原失败收据为 research/projection-cache-20260930/original-semantics.*。强模型追溯确认，既有 SQL、维度层与历史合同仅对 host/process 定义缺失过滤特例；network 使用字典值等值。本任务先在 raw_fold.rs 将 resolve_id 的缺失特例限定为 process，恢复既有合同并单独复测，再实施派生缓存。该修复服务 AC2 的 oracle 一致性，不扩展公开语义、不修改 SQL 对照。详见 research/network-unknown-review.md。

### 文件清单

- residential-monitor/src-tauri/src/c3/raw_fold.rs
- residential-monitor/src-tauri/src/c3/sql.rs
- residential-monitor/src-tauri/src/c3/service.rs
- residential-monitor/src-tauri/src/bench/facade.rs
- residential-monitor/src-tauri/src/bench/process.rs（仅证据归属需要时）
- residential-monitor/src-tauri/src/bench/write_vfs.rs（仅计量验证需要时）
- .trellis/spec/residential-monitor/storage/sqlite-contract.md
- 本任务 research/ 的 baseline/fixture/阶段/回归记录

## Model And Harness

必须强模型规划、阶段归因、SQL/核算审查和最终判定。较低成本模型只整理固定证据或写已确定 oracle；不得自行改阈值、数据规模、缓存条件或算法。 执行工具按父任务五工具矩阵选择；工具原生能力不扩展授权。

## Writeback

storage/sqlite-contract.md、对应 backend 验证说明和本任务 evidence，适用五工具。历史失败记录不覆盖。

## Rollback

任何守恒、deadline、取消或正式性能回归失败即回退候选算法；保持原始 schema、安装库和失败证据。不自动开启删除、维护或 VACUUM。

### 2026-10-01 回退与容量阻断

正式 matrix 的 F2、F6–F10 及 F12 空窗口比值未通过，缓存的因果责任未查明。按上述回退条件，回退本轮按需字典与 chain 派生缓存，保留 Process/Network 合同修复和全部独立 oracle。只恢复 `raw_fold.rs` 已审生产片段，不整体覆盖文件。冻结旧程序的六次 primary 结果保留，不能迁移为回退后源码的性能通过证据。

容量前置核查发现冻结数据库程序、A50/A250 完整合成语料均缺失，原因未查明；用户确认没有备份。实际容量执行为 0/106。原程序/语料 hash 与失败收据保留。重新构建和生成需要先审查新身份、固定输入、隔离路径和资源条件；本轮不以小库、历史结果或更低门槛代替。

### 2026-10-04 范围变更（用户批准）

用户于 2026-10-04 回复「都按照你的深入分析和推荐来优化」，批准 research/f6-f10-attribution-20261003/findings.md 的方向 1 与方向 2。

方向 1（F8–F10）：A250 30 天语料 `connection_session_attr` 为 2160000 行，全表扫描约 3089 ms，不能在回收分块内无索引全扫。改为：v5 DDL 用一个普通列复合索引 `idx_session_attr_chain_rule(chain_key, rule_id)` 替换 `idx_session_attr_chain_identity` 与 `idx_session_attr_rule_group`；`dictionary_referenced` 对 chain 与 rule_group 改为每个字典分块一次的有界跳跃枚举（distinct chain_key，以及 hop 为空的 chain_key 下 distinct rule_id），再用 Rust `chain_identity` / `last_chain_hop` 判定。判定与原三条 `exists` 逐条等价。v5 checksum 改为 `ledger-lifecycle-v5-layout4`；已有 layout3 库在 migrate 中原地删除两个旧索引、创建新索引并更新 checksum。v5 未进入 main 或任何发布。

方向 2（F6/F7）：在 lib 新增 bench 专用计数分配器 `bench/heap.rs`，仅由 `bin/monitor-bench.rs` 注册为全局分配器，仅在 `RESIWATCH_BENCH_HEAP=1` 时计数。SQLite 自身分配走 C malloc，同一文件另用 `sqlite3_memory_highwater` 统计。replay-facade 报告新增 `heap_phases` 字段（未启用时为 null），按 snapshot、ingest、archive、live_query、report 首读与重复读、sample 七个阶段记录 Rust 与 SQLite 峰值增量。baseline 源码副本同步加入同一计数器后重建，用于两边分阶段对比。产品桌面程序与 monitor-db 不注册该分配器。

新增文件范围：`c3/schema.rs`、`c3/retention_day.rs`、`storage.rs`（migration 与 layout 测试）、`c0_contract.rs`（如需）、`bench/heap.rs`（新增，原计划名 `bench/alloc.rs`，为避开 `alloc` crate 名改名）、`bench.rs`、`bin/monitor-bench.rs`、`bench/facade.rs`，以及 bench-data 下 baseline/candidate 隔离源码副本与新构建。门槛、语料与核算语义不变。改动后重新执行 matrix、primary、capacity，并补回收容量证据。

### 2026-10-04 回收不结束修复（用户批准）

用户于 2026-10-04 回复「修复 A50 回收不结束的问题」。原因见 research/rebuild-20261004-e14ecd88/formal-result-20261004.md：字典与覆盖两个辅助游标页数不同（9 与 6），session 删除只重置字典游标，相位差固定后两者永不同块归零。

修改只在 `c3/retention_day.rs` 的 `cleanup_expired`：两个扫描同一轮开始，先结束的一方停在 0，另一方结束后两者一起开始下一轮。`-1` 重扫语义、128 行分页、删除条件与生产 AUTO_DELETE_ENABLED=false 不变。`storage_lifecycle.rs` 与 `bench/corpus.rs` 不改。新增回归测试复现 A50 的 1085/720 行与错位游标。该路径只在删除开启时执行，因此 matrix、primary、capacity 测得的路径不受影响；回收容量用修复后 release lib 测试程序在语料新副本上复测，见 research/rebuild-20261004-e14ecd88/retention-fix/。
