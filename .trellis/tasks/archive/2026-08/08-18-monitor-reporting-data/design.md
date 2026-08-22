# 技术设计：历史报告与数据管理（C3）

## 设计目标

1. 让应用内报告、图表、数据表和三种导出共享同一 `ReportResult`，避免统计口径漂移。
2. 报告快照保持不可变，但不以长期 SQLite read transaction 换取一致性。
3. 在明确能力边界内保存精确高基数数据；过期后返回“不支持”，不返回伪精确 Top N。
4. retention、备份、恢复和 backfill 可取消、可续跑、低空间 fail closed，并让 ingestion 始终优先。
5. 将实际 restore 接入 C2 Recovery Shell，同时保持 Recovery Shell 不依赖正常业务 schema。

## 依赖与模块所有权

### C2 前置交付

C3 开始前必须确认：

- C2 AppFacade 的 command / error / progress DTO 稳定；
- 报告与数据管理导航、文件选择和本地状态 seam 可用；
- C2 Recovery Shell 已能在正常 schema 不可用时独立启动，并只依赖 `RecoveryFacade`；
- C2 的窗口、Channel 和 collector 生命周期隔离已通过 gate。

C3 同时复用 C1 `StorageCoordinator` 的单 writer、只读连接管理、migration、checkpoint、Online Backup 和 health。C3 不创建通用 Repository 或第二个 SQLite owner。

### C3 深模块

```text
ReportQuery
    │ validate / capability plan
    ▼
ReportService ──► StorageCoordinator read snapshot
    │                    │
    │ materialize       └─ short transaction closes
    ▼
ReportSnapshotStore ── token ──► UI projection
    │
    └───────────────────────────► ExportService ── streamed artifact

StorageCoordinator
    ├── RetentionService ── rollup / watermark / delete
    ├── BackupRestoreService ── Online Backup / maintenance restore
    └── MigrationBackfillCoordinator

C2 Recovery Shell ── RecoveryFacade extension ── BackupRestoreService
```

- `ReportService` 拥有 query validation、能力选择、命名 SQL 与权威投影。
- `ReportSnapshotStore` 拥有 token 生命周期、不可变结果 / spool、TTL 和 quota，不拥有 SQLite read transaction。
- `ExportService` 只读取 snapshot projection，不重新实现 SQL 或聚合。
- `RetentionService` 拥有层级 materialize、守恒、watermark 和清理状态。
- `BackupRestoreService` 拥有用户备份、候选验证、maintenance restore 和当前库保护。
- `StorageCoordinator` 统一安排 writer、readers、checkpoint 和维护优先级。

## 统一查询契约

```text
ReportQuery {
  rangeUtc,
  displayTimezone,
  granularity,
  filters,
  grouping,
  targetPolicy,
  comparison,
  sort,
  page
}

ReportResult {
  schemaVersion,
  dataVersion,
  reportSnapshotToken,
  queryEcho,
  totals,
  series,
  rankings,
  coverage,
  drilldownCapability,
  policyMetadata
}
```

Rust 边界把未知输入解码为版本化 query DTO，并校验：

- UTC range 顺序、最大跨度和本地时区标识；
- granularity、dimension、sort 和 grouping allowlist；
- 默认 / 最大页面大小与 Top N 上限；
- keyset cursor 的 query fingerprint、sort tuple 和 identity；
- raw、hourly / daily dimension、daily core 之间的能力组合。

前端只组装 typed `ReportQuery`。SQL、表名、列名和索引提示不能由前端输入。

## 数据层能力规划

### 保留层

| 数据层 | 期限 | 精确能力 |
|---|---|---|
| session / chain / minute raw | 默认 30 天，上限 90 天 | 会话下钻、当前策略重算、受支持维度任意组合 |
| hourly dimension | 最多 13 个月 | 历史主分类 + 一个分析维度的精确趋势 / Top N |
| daily dimension | 最多 13 个月 | 与 hourly 相同，日粒度 |
| daily core | 长期 | attributed total、历史主分类、coverage |
| raw coverage | 最多 13 个月 | 精确区间与原因 |
| daily coverage | 长期 | 每日状态时长与覆盖率 |

`traffic_*_dimension` 保存每个实际 dimension value 的完整精确汇总，不在写入时截断为 Top K。Top N 是查询时对完整保留集合排序得到的结果。v1 不实现 sketch、approximate、bounded candidate set 或误差区间。

当查询范围跨越不同能力层时，planner 先计算能力交集：

- 能由各层共同回答的总量、历史主分类和 coverage 可以合并；
- 需要 raw 会话下钻、当前策略重算或跨维组合，而任一时间片已过 raw 期限时，返回 capability 不支持；
- 需要高基数 Top N，而任一时间片已过 13 个月期限时，不返回截断排名；
- 不把支持区间的局部结果标记成完整范围结果。

## Report Snapshot Token

### 创建流程

1. 校验 query，确定数据层、命名 SQL 集和 deadline。
2. 从 StorageCoordinator 获取专用可中断 read connection。
3. 在一个 SQLite read snapshot 中执行组成报告的全部查询。
4. 把 `ReportResult` 完整物化到有界内存，或把大 projection 流式写入应用私有 spool。
5. 关闭 statement、结束 read transaction 并归还连接。
6. 生成包含随机 identity 的不透明 token，原子登记 query fingerprint、schema / data version、创建 / 过期时间、字节数和 artifact checksum。
7. 返回 token 与结果摘要。

步骤 5 必须先于步骤 6 / 返回。token 不保存连接、transaction handle 或 WAL end mark。

### 生命周期与 quota

初始 TTL 候选为 10 分钟。C3 基准冻结：

- 每进程最大活动 token 数；
- 单 token 内存 / spool 上限；
- 总 spool quota；
- 清理周期和启动时 orphan spool 清理预算。

显式 release 和 TTL 过期都会删除 artifact。创建新 token 时先清理过期项；仍超 quota 则 fail closed，返回可操作错误，不静默驱逐仍有效 token。token 只可在同一应用实例和用户数据目录中使用，不把路径暴露给前端。

## 查询执行、取消与性能回归

每个页面或报告 operation 拥有：

- 独占 read connection；
- cancellation token；
- SQLite interrupt / progress handler；
- monotonic deadline；
- operation ID 与有界进度；
- statement status 和脱敏性能诊断。

前端取消、窗口关闭、deadline 到期或 shutdown 都触发实际 interrupt。错误明确区分 cancelled、deadline exceeded、capability unsupported、storage busy 和 invalid query。

初始 deadline 候选为页面 2 秒、报告 10 秒。常用查询发布预算为 30 天 raw p95 小于 2 秒、13 个月 hourly p95 小于 3 秒。C0 / C3 若用实测调整数值，必须同步 PRD / gate 后再发布。

### 命名 SQL corpus

每个公开 query shape 有稳定名称、参数 fixture、预期数据层和规模标签。CI 对 `A=50 / 250 / 1000` 真实数据库采集：

- EQP 是否意外 SCAN 指定大表；
- 非预期 TEMP B-TREE；
- 自动索引；
- FULLSCAN_STEP、SORT、VM_STEP；
- 冷 / 热 p50、p95、p99、max；
- 返回行数和 peak memory。

不 snapshot 完整 EQP 文本。索引增删只能由 corpus 与真实规模结果驱动。

列表 cursor 绑定 query fingerprint；以稳定 sort tuple + unique ID 做 keyset。排序、筛选、data tier 或策略变化后旧 cursor 失效。

## 报告 UI 与流式导出

报告页面持有 query draft、operation 状态、`ReportResult` 和 token，不持有第二套聚合。图表与数据表读取相同 totals / series / rankings / coverage。

导出流程：

1. 验证 token 未过期，锁定不可变 artifact reader。
2. 根据 ExportSpec 选择字段、格式和脱敏策略。
3. 返回脱敏预览与 metadata 摘要。
4. 用户确认目标路径后创建同目录临时文件。
5. CSV、JSON 或 HTML encoder 逐块消费 projection；额外内存受固定 buffer 上限约束。
6. flush / close，执行格式 smoke 与适用 checksum 后原子 rename。
7. 取消或失败时关闭并删除 partial，保留已有目标。

三种 encoder 共享一个 typed projection iterator 和 metadata builder，不能各自查询或重算。HTML 只包含本地静态样式，不加载远程资源。

## Retention 与 migration

### C3 前向 migration

C3 只追加：

- 稳定 dimension ID 与必要字典；
- hourly / daily dimension；
- daily core 与 daily coverage；
- retention state / watermark；
- report spool / backup manifest 所需的版本元数据（若 C1 尚未提供通用字段）。

不修改已发布 C1 migration，不创建告警表。新表优先 STRICT，索引来自 query corpus。

大型变化采用 expand → checkpointed backfill → contract：

1. expand 添加兼容结构；
2. backfill 以封口 UTC bucket 和稳定 chunk identity 执行；
3. 每个 chunk 原子登记 checksum、范围和完成状态；
4. 跨启动从最后 verified chunk 继续；
5. 所有读写切换并验证后，后续独立 migration 才 contract 旧结构。

### Retention 状态机

对每个 UTC chunk：

1. 按 cutoff 与 watermark 选择已封口范围。
2. 从下层事实确定性物化完整上层 chunk。
3. 在同一 transaction 以 replacement / set 语义写入，并核对上传、下载、连接数、活跃时长和 coverage。
4. 记录 verified chunk 并推进 watermark。
5. 另一个有界 transaction 删除已覆盖下层事实。

重启后能区分 `pending | materialized | verified | delete_pending | done`。重复执行不会双加。ingestion 优先；每批有行数、时间和 I/O budget，队列或 commit latency 接近门限时主动 yield。

raw 到期前生成精确 hourly / daily dimension 与 core。13 个月高基数到期前确认 daily core / coverage 已存在，再删除 dimension。DELETE 后只更新真实逻辑占用、freelist 和预计可复用空间，不声称文件已缩小。

应用不自动 VACUUM。用户主动 VACUUM 是独立 maintenance operation：暂停 writer、先备份、检查约两倍数据库临时空间、可取消边界和明确风险；不满足即 fail closed。

## Online Backup 与恢复

### 用户备份

Backup 使用 C1 Online Backup API：

- 从 live DB 分页 step 到 `.partial`；
- 每步后报告进度并让 writer / ingestion 优先；
- 支持取消和整体 deadline / 收敛监测；
- 开始前按源库、WAL、目标文件和验证临时空间预检；
- 完成后关闭目标库，生成 checksum / manifest，运行适用 integrity / smoke，再原子 rename。

若持续写入下无法满足 ingestion SLO 或备份无法收敛，可请求短暂停采完成；必须通过 C1 生命周期接口形成 coverage gap。

### 恢复

`RecoveryFacade` 新增候选验证与 restore operation，不暴露普通 SQL：

1. 进入 maintenance mode，拒绝新报告 / 导出 / retention。
2. 停止 collector、flush / 关闭 writer 和 readers。
3. 对当前可用库创建受保护备份；失败则停止恢复。
4. 独立打开候选，验证 manifest、checksum、integrity、schema 支持范围和空间。
5. 在同卷准备 swap，保留当前库及相关 WAL / SHM 的受控恢复点。
6. 原子切换候选，执行前向 migration 与 smoke query。
7. 成功后重建 StorageCoordinator 并恢复采集；失败则恢复原库并保持 Recovery Shell。

Credential Manager 不属于数据库备份。候选来自其他机器时，恢复后状态为“需要重新输入 secret”。

## 并发调度与 WAL 边界

StorageCoordinator 使用统一 operation registry 和优先级：

1. writer durable commit；
2. 短页面查询；
3. 报告物化；
4. export spool 读取；
5. PASSIVE checkpoint；
6. retention / backfill / backup chunk。

低优先级任务按 chunk yield，不以长 busy timeout 抢占 writer。监控 oldest reader、WAL pages / bytes、checkpoint progress、writer queue、commit latency 和 operation memory。

WAL 超过 C0 冻结软阈值时：

- 拒绝新长报告；
- 取消已过 deadline 的 reader；
- 使 retention / backup yield；
- 执行 PASSIVE checkpoint 并观察进展。

超过硬阈值且无法回落时进入明确 degraded health，取消剩余非关键 readers；不得删除 WAL 或继续无限增长。由于 token 不持有 read transaction，token TTL 本身不会阻止 checkpoint。

## 低空间策略

所有可能增加磁盘占用的 operation 在开始和每个大阶段前检查：

- 当前 DB / WAL、目标 artifact、验证副本和安全余量；
- spool 总 quota；
- backup / restore swap 空间；
- migration backfill 临时空间；
- 用户主动 VACUUM 的约两倍数据库预算。

空间不足时不开始或在安全 chunk 边界停止，保留当前数据库、verified watermark 和可续跑状态。不得通过提前删除未验证 raw、跳过备份或缩短用户保留期自动“解决”空间不足。

## 验证策略

- Golden reports：时区 / DST、策略变化、coverage、raw / dimension / core 边界和周期对比。
- Token：事务关闭断言、TTL、显式释放、并发 quota、orphan spool、过期导出和 checksum。
- SQL corpus：三档真实规模、keyset 深分页、EQP / statement status、取消和 deadline。
- Retention crash points：materialize、verify、watermark、delete 和 13 个月高基数淘汰。
- Export：大结果三种格式、流式内存、取消、I/O error、原子覆盖与脱敏。
- Backup / restore：持续写、坏候选、旧 schema、WAL / SHM、低空间、swap / migration / smoke 中断。
- 并发矩阵：writer、report、export、backup、retention、checkpoint 同时运行，记录 ingestion、WAL、CPU、RSS、DB / freelist 与 operation latency。

## 独立回滚

- ReportService、报告 UI 与 ExportService 可 feature-disable，不影响 C2 实时监控。
- retention 在守恒与 crash gate 通过前只允许 dry-run / materialize，不启用自动 DELETE。
- restore 全矩阵通过前只允许创建和验证备份，不允许 swap。
- C3 migration 只前进，不提供 down migration；二进制回滚必须检查 schema compatibility，数据回滚只使用验证备份。
- 大 backfill 可停在 verified checkpoint，旧读路径在 contract migration 前保持兼容。
- 回滚不得删除 C1 raw、C2 设置或当前可用数据库。
