# SQLite 读写性能与容量预算

> 状态：规划基线，尚未运行基准。C0 必须以真实 binding、目标 Windows 11 机器和生成数据冻结最终数值；不得把本文估算当成已验证结果。

## 目标

完整 v1 会保存全部连接、完整元数据和分钟级 delta。数据库性能必须在 schema 冻结前解决，而不是在发布阶段靠调 PRAGMA 补救。本预算约束 C0、C1、C3 与 C5。

## 容量模型

设：

- `A`：30 天平均活跃连接数；
- `L`：平均会话时长（分钟）；
- `C`：每个会话平均 chain 节点数；
- `q`：活跃会话分钟中生成非零 `connection_minute` 的比例。

估算：

```text
minute_rows = A × 43,200 × q
sessions    = A × 43,200 ÷ L
chain_rows  = sessions × C
```

示例：

| A | L | C | q | 30 天 minute rows | sessions | chain rows |
|---:|---:|---:|---:|---:|---:|---:|
| 50 | 5 | 3 | 1 | 216 万 | 43.2 万 | 129.6 万 |
| 250 | 5 | 3 | 1 | 1,080 万 | 216 万 | 648 万 |
| 1,000 | 5 | 3 | 1 | 4,320 万 | 864 万 | 2,592 万 |
| 10,000 | 5 | 3 | 1 | 4.32 亿 | 8,640 万 | 2.592 亿 |

10,000 是短时实时峰值测试，不是 30 天持续容量承诺。若一条分钟事实连同索引实际占 128–256 B，`A=250` 的 minute 表就约 1.38–2.76 GB，尚未包含 session、chain、元数据、rollup、WAL、freelist 和备份。

C0 必须用 `dbstat` / `sqlite3_analyzer` 或 binding 等价能力测量每表和每索引真实 B/row。

## v1 容量边界

- raw 默认 30 天；配置上限 90 天。提高期限前显示基于当前 B/row、增长率和备份空间的预测。
- 精确高基数维度（域名、进程、规则、链路、网络类型）最多保留 13 个月。
- 长期 daily 只保留 attributed total、历史主分类和 coverage；不无限保存所有高基数维度。
- v1 不采用 approximate / bounded Top K。若未来引入，UI、查询和导出必须明确标记，不能称为任意范围精确 Top N。
- `connection_minute` 保持稀疏：仅非零 delta 或必要质量事件产生行。零流量连接的存在和时长来自 session / 紧凑活动区间，不每分钟写零行。
- 列表分页默认 200、最大 1,000；Top N 默认 20、最大 100。

基准档：

| 档位 | 用途 |
|---|---|
| `A=50` | 小型库与开发快速回归 |
| `A=250` | 30 天完整发布设计点 |
| `A=1,000` | 压力库 |
| 10,000 活跃、1Hz、30 分钟 | 短时采集与实时峰值 |

## 写路径

### prepared 与批处理

- 单 writer 连接缓存并复用 prepared statements；逐行只做 bind → step → reset。
- 禁止每行重新 prepare，禁止拼接无界多值 SQL。
- Accounting 在写入前按 `(session_id, utc_minute)` 合并，同一 transaction 中每个键最多一次写。
- 不在原始 snapshot 层直接丢帧或盲目 latest-only；合并只能发生在已完成生命周期与 delta 核算之后。

### 幂等提交

- 每个 `CommitBundle` 有稳定唯一 `bundle_id`。
- 同一 SQLite transaction：
  1. 登记 `bundle_id`；
  2. 写 facts、coverage、alerts 与 notification outbox；
  3. 推进 `data_version` / durable watermark；
  4. commit。
- commit 结果不确定时重试同一 `bundle_id`。重复 bundle 返回已有 `CommitReceipt`，不得再次累加字节。
- `bundle_id` 使用连续 `(writer_epoch, bundle_seq)`。recent receipt ledger 初始至少覆盖最近 24 小时与 100,000 条中较大的集合；只有最高连续 sequence 已覆盖、越过时间 / 数量安全线且无进程引用时才能小批量 prune。epoch summary / watermark / rolling hash 保留，窗口外 duplicate 必须拒绝而不是重新执行，避免每秒一行永久增长。
- rollup 使用确定性 `SET value = excluded.value` 或基于已物化 chunk 的替换，不使用重试会重复累计的无条件 `value = value + excluded.value`。

### durable 与背压

“不丢账”定义：

- 已返回 `CommitReceipt` 的 FULL transaction 具有断电 durability；
- 未提交 frame 不宣称 durable；
- crash 后从最后 durable watermark 到新 baseline 建立显式 gap；
- 存储阻塞时不静默丢 batch，也不无限堆内存。

队列有界。若 oldest batch 超过 SLO 或队列饱和：

1. health 进入 `storage_backpressure`；
2. 停止宣称完整 coverage；
3. 允许在核算后按 minute key 合并待写 delta；
4. 若仍无法恢复，丢弃前先记录内存中的 gap 起点，恢复后持久化 degraded interval；
5. 不允许 UI 继续显示“已完整保存”。

### WAL 与 checkpoint

- WAL 仍只有一个 writer；长 read transaction 会阻止 checkpoint。
- 初始保留 SQLite 默认约 1,000 页 auto-checkpoint，先测量后调。
- 运行期优先 PASSIVE checkpoint；FULL / RESTART / TRUNCATE 只用于受控维护或退出。
- checkpoint starvation 时取消 / 过期长读者，绝不删除 `-wal`。
- `busy_timeout` 由 C0 按操作 deadline 冻结；它不是背压策略，不能超过正常 durable commit SLO 后仍把写入称为健康。
- `cache_size`、`mmap_size`、`page_size`、`temp_store`、`auto_vacuum` 保持基线默认，只有独立基准证明收益才改变。

参考：

- [SQLite WAL](https://www.sqlite.org/wal.html)
- [PRAGMA synchronous](https://www.sqlite.org/pragma.html#pragma_synchronous)

## 读路径

### 索引候选

最终索引必须由真实 Query Corpus 与基准决定，初始候选：

```text
connection_minute(utc_minute, session_id)
connection_minute(session_id, utc_minute)
connection_session(epoch_id, connection_id) UNIQUE
connection_chain(chain_dimension_id, session_id)
rollup(bucket_utc, policy_id, category_id, dimension_kind, dimension_id)
rollup(dimension_kind, dimension_id, bucket_utc)
```

- 域名、进程、规则和链路使用稳定整数 dimension ID，避免在多个索引重复长 TEXT。
- 多值 chain 报告不能把所有 chain 节点流量相加后冒充守恒总量。
- 不为所有隐私字段盲目建索引。

### 查询约束

- 大列表全部使用 keyset pagination，不使用深 OFFSET。
- 总量和趋势优先 finalized hourly / daily rollup；只有 30 天内任意跨维过滤和 session 下钻扫描 raw。
- report snapshot token 绑定已物化的有界 `ReportResult` 或磁盘 spool，不持有长期 SQLite read transaction。
- 初始候选：token TTL 10 分钟、每进程有限活动 token、总 spool quota；C3 以基准冻结。
- 交互查询可取消：独占可中断连接，使用 `sqlite3_interrupt()` / progress handler 或 binding 等价能力。
- 初始 deadline：页面查询 2 秒、报告 10 秒；导出 / 备份可长跑但必须显示进度并可取消。

参考：

- [Keyset scrolling with row values](https://www.sqlite.org/rowvalue.html#scrolling_window_queries)
- [EXPLAIN QUERY PLAN](https://www.sqlite.org/eqp.html)
- [sqlite3_interrupt](https://www.sqlite.org/c3ref/interrupt.html)
- [progress handler](https://www.sqlite.org/c3ref/progress_handler.html)

### Query Plan 回归

每个公开查询维护命名 SQL corpus。CI 不 snapshot 易变的完整 EQP 文本，只检查：

- 指定大表没有意外全表 SCAN；
- 没有非预期 `USE TEMP B-TREE`；
- 没有意外自动临时索引；
- statement status 的 FULLSCAN_STEP、SORT、VM_STEP；
- 冷 / 热 p50、p95、p99、max。

## Retention、backup 与 migration

### Retention

- 只处理已封口 UTC bucket；
- 小批次、可取消，ingestion 优先；
- 先确定性 materialize rollup chunk，再推进 watermark，最后删除 raw；
- 中断后能区分“已汇总未删除”和“未汇总”；
- DELETE 只增加 freelist，不宣称立即释放等量磁盘；
- 不自动 VACUUM。VACUUM 需约额外数据库大小空间并持有写锁，只能用户主动维护且先做空间检查。

参考：[SQLite VACUUM](https://www.sqlite.org/lang_vacuum.html)

### Backup

- Online Backup 分页 step、显示进度、可取消和节流；
- 在 writer、报告、retention 并发负载下验证是否能完成；
- 必要时允许短暂停采，必须形成 coverage gap；
- 开始前检查目标空间；完成 `.partial` 后再 checksum / integrity / rename；
- 大库完整 integrity check 的 I/O 影响纳入 gate。

参考：[SQLite Backup API](https://sqlite.org/backup.html)

### Migration

- 小型 DDL / metadata migration 在单个 `BEGIN IMMEDIATE` 原子执行。
- 大型 backfill 使用 expand → checkpointed backfill → contract，多次启动可续跑，不用一个长 transaction 阻塞首次窗口。
- migration 前备份和 backfill 延迟 collector 启动时，UI 显示进度并把该时段记为未采集，不写零。

## 性能验收

### C0

- 生成 `A=50/250/1000` 的完整 30 天库及 10k 峰值 fixture；
- 使用真实 / 高基数分布、长进程路径和多 chain；
- 输出表 / 索引 B/row、DB/WAL 峰值、写放大、冷 / 热查询；
- 证明 binding 暴露 interrupt、progress、backup step、checkpoint / statement stats；
- 比较 FULL / prepared batch 大小，但 FULL 是默认；降级需重新批准 durability。

### C1

- 10k 活跃、1Hz、全部计数变化时满足 CommitBundle / durable commit SLO；
- prepared statement、per-key merge 和 `bundle_id` 幂等有自动测试；
- recent receipt 行数在 retry window 后达到平台；recent duplicate、payload mismatch 与过期 duplicate 三条路径都有验证；
- kill 点覆盖 commit 前、commit 结果不确定、commit 后未回执；
- queue saturation 产生显式 gap，内存有界；
- checkpoint starvation、WAL 回落、磁盘满和 busy deadline 有 gate。

### C3

- 每个公开查询有 SQL corpus、规模 fixture、EQP / statement status 与延迟预算；
- 全部深分页为 keyset；
- token 不持有 read transaction，TTL、spool quota、取消 / 超时通过；
- retention、backup、export 并行时仍满足 ingestion SLO；
- 高基数维度只保留有限期限；
- 大 backfill 可续跑。

### C5

- 使用真实 30 天设计点库和 13 个月 rollup，不以小库线性外推；
- writer、report、export、backup、retention、checkpoint 并发；
- 记录 p50 / p95 / p99 / max、DB/WAL、queue、freelist、CPU、RSS；
- 24 小时 nominal soak + 独立 10k 峰值；
- 零重复 bundle、零静默 gap、零无限 WAL、零不可取消查询；
- backup / migration / VACUUM 低磁盘空间 fail closed。
