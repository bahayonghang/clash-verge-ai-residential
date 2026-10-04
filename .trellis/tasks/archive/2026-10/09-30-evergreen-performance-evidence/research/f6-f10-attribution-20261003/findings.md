# F6–F10 归属排查

日期：2026-10-03，America/Chicago。输入为 `../rebuild-20261002-4f8a8c68/matrix/` 的正式报告，加上本目录的诊断运行。诊断运行使用同一组复制程序和同一组 replay-facade 参数，输出目录在 session scratch；结果只用于归属分析，不作为正式计量。本轮未修改产品源码、runner、语料或门槛。

## 方法

`walattr.py` 运行 replay-facade，每 0.5 秒复制一次 `monitor.sqlite3` 与 `-wal`。脚本取最后一份包含不少于 5 次提交的快照，只使用 salt 与 WAL 头一致的帧，按 commit 帧分组，再从 `sqlite_master` 的根页遍历每棵 b-tree，把页号映射到表或索引。统计窗口为最后 10 次提交。首次运行因脚本未读取 stdout 管道而阻塞；我终止了自己启动的该诊断进程（PID 30452），改为输出到文件后重跑。

## F8–F10：SQLite xWrite

正式报告中，所有 metadata 类场景（a50/a250/a1000-metadata、a250-backlog、a250-failed）的 `writer_rows_changed` 都比 baseline 多 30（每次提交 +1 行）。unchanged 与 counters 场景下 candidate 写入更少。

每次提交写入的 WAL 帧数：

| 场景           | baseline | candidate | 差  |
| -------------- | -------- | --------- | --- |
| a50-metadata   | 12       | 14        | +2  |
| a1000-metadata | 57       | 67        | +10 |

按对象拆分的每次提交页数差：

| 对象                              | a50 | a1000 | 来源                                                     |
| --------------------------------- | --- | ----- | -------------------------------------------------------- |
| `idx_session_attr_chain_identity` | +1  | +5    | v5 新增表达式索引 `chain_identity(chain_key)`            |
| `idx_session_attr_rule_group`     | +1  | +5    | v5 新增表达式索引 `(last_chain_hop(chain_key), rule_id)` |
| `coverage_interval`               | +1  | +1    | `storage.rs:897-913` 每次提交更新观测区间 `ended_utc`    |
| `data_version`                    | −1  | −1    | 09-19 layout3 移除每次提交的 singleton 写入              |

其余对象的页数两边相同。bench 的 metadata 负载让 100% 连接每帧切换 `chain_key`（`bench/facade.rs` 中 `exit-{(stable+variant)%4}`），因此两个 `chain_key` 表达式索引每帧都要更新。

两个索引只被 `c3/retention_day.rs:320-323` 的字典回收存在性检查使用（chain 与 rule_group 值是否仍被 `connection_session_attr` 引用），不被报告查询使用。它们由 `c3/schema.rs:36-37` 的 v5 migration 创建。

F8 另有一项：candidate 在 30 秒窗口内发生一次 checkpoint，主库写入 380928 B（93 页）；baseline 在该窗口内主库写入为 0。只计 WAL 时 F8 比值为 1730432 / 1483200 = 1.1667。checkpoint 是否落入窗口取决于启动后累计帧数，两份 schema 的初始页数不同，所以该位置会随 schema 变化。

## F6/F7：native private p95

| 场景            | baseline min / 中位 / max MB | candidate min / 中位 / max MB |
| --------------- | ---------------------------- | ----------------------------- |
| a1000-unchanged | 9.0 / 9.7 / 11.0             | 8.9 / 10.0 / 12.5             |
| a1000-metadata  | 9.3 / 10.2 / 11.5            | 9.7 / 10.9 / 13.2             |

逐秒序列没有单调增长。句柄数两边均为 114。candidate 的中位数高 0.3–0.7 MB，峰值高 1.5–1.7 MB，序列逐帧交替。p95 由瞬时峰值决定。现有报告没有分配来源字段；09-19 记录的 `ArchiveScheduler` 待处理队列假设只对应 backlog/failed，不覆盖 a1000-unchanged。原因未查明。

## 结论

F8–F10 的增量已精确归属到两个 `chain_key` 表达式索引、`coverage_interval` 每次提交更新和 F8 窗口内的 checkpoint。F6/F7 已排除泄漏，峰值分配来源未查明。

可选方向（均需重新批准范围）：

1. 去掉两个 `chain_key` 表达式索引，改写 chain/rule_group 字典回收检查，使其不依赖这两个索引，同时保持分块时间有界。需要修改 `c3/schema.rs`（v5 checksum 或新 migration）、`c3/retention_day.rs` 及回收测试，并重跑回收容量证据。按页数推算，a250/a1000 的 metadata 场景每次提交回到与 baseline 相同的帧数；F8 仍受 checkpoint 位置影响。
2. 在 bench 二进制中加入分配计数，并对 baseline 同步加入，然后重建两边，用于定位 F6/F7 的峰值来源。需要修改 `bin/monitor-bench.rs` 或新的 bench 文件。
3. 不改代码，把上述归属作为 F6–F10 未通过的已知原因记录。

## 2026-10-04 更正

方向 1 已按复合索引 `idx_session_attr_chain_rule(chain_key,rule_id)` 实施。上文「每次提交回到与 baseline 相同的帧数」的推算不成立：复合索引仍包含完整 `chain_key`，a1000-metadata 每次提交仍多 7 页（原方案 10 页），a250-failed 多 3 页。实测见 `../f6-f10-attribution-20261004/` 与 `../rebuild-20261004-e14ecd88/formal-result-20261004.md`。
