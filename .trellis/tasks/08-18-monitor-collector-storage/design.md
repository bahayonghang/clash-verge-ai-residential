# C1 技术设计：采集、核算与 core SQLite

## 设计目标

C1 建立单一、可回放的事实流水线。控制器兼容性、核算语义、durable commit 和实时水位各有唯一 owner；任何层都不能绕过前一层自行解释原始 payload 或宣称未持久化数据已经安全保存。

## 技术边界

```text
TCP / verified named pipe
          │
          ▼
ControllerSession
  discovery + transport + protocol + normalization
          │ ControllerInput
          ▼
AccountingEngine
  epoch + baseline + delta + UTC bucket + coverage + policy
          │ AccountingBatch
          ▼
Bounded Commit Queue
          │ CommitBundle
          ▼
StorageCoordinator
  prepared single writer + core migrations + migration backup
          │ CommitReceipt
          ├────────► Durable LiveProjection
          └────────► atomic Channel bootstrap / seq

startup failure ─────────► RecoveryFacade backend
```

C1 只拥有上图模块和 core schema。报告、rollup、retention、alert/outbox、完整 UI 与实际 restore 不属于 C1。

## C0 依赖注入

实现不得硬编码规划阶段的估算值。启动前从已批准 C0 决策冻结：

```text
SqliteBindingDecision
DurabilityProfile
BatchAndQueueLimits
BusyAndOperationDeadlines
CapacityBudget
ControllerCompatibilityProfiles
TransportLimits
CredentialStorePort
StableApplicationIdentity
```

若缺少任何必选决定，构建可以存在，但 C1 运行和验收必须 fail closed。C1 不提供运行时「自动换 binding」或「自动降为 NORMAL」。

## `ControllerSession`

### 接口

```text
connect(ControllerConfig, CancellationToken) -> Stream<ControllerInput>
probe(ControllerCandidate, CancellationToken) -> ControllerCapabilities
execute(ControlCommand, CancellationToken) -> ControlResult
shutdown()
```

本任务只需要采集、probe 和协议基础；控制命令接口可定义，但 C1 不自动调用写操作。

### 输入模型

`ControllerInput` 使用版本化枚举：

```text
Connected { endpoint, capabilities, core_identity }
Snapshot { received_monotonic, received_utc, root_totals, connections }
Restarted { old_identity, new_identity }
Disconnected { reason }
SleepGap { started, ended }
Paused | Resumed | Shutdown
```

normalizer 是原始 mihomo payload 的唯一 owner：

- 忽略未知字段，保留允许的扩展版本信息；
- 缺失字段表达为 unknown/null，不造默认；
- 连接数组只按 ID 合并，不依赖顺序；
- 应用 C0 frame/body/字符串 limits；
- 日志只记录白名单元数据。

### 传输状态

TCP 与 pipe 共享 HTTP/WebSocket 协议层，但鉴权边界不同：

- TCP：loopback + CredentialStore resolver + Authorization header；
- pipe：C0 批准 profile + server PID 验证 + 无 secret。

错误 taxonomy 至少区分 unauthorized、access denied、busy deadline、not found、PID mismatch、protocol incompatible、cancelled 和 core restarted。

## `AccountingEngine`

### 接口与状态

```text
apply(ControllerInput, MonotonicTime, UtcTime) -> AccountingBatch
```

内部状态只包含：

- 当前 controller epoch 和 root totals baseline；
- `(epoch, connection_id)` 的 session/baseline；
- 当前 target set 与版本；
- coverage 状态；
- 尚未封口的 UTC minute accumulator。

它不打开数据库、不访问网络、不读取系统时钟，因此 replay 能完全控制输入。

### 会话与增量

1. 新 core identity 或 root counter reset 建立新 epoch。
2. 新连接首帧写 session metadata/baseline，delta 为未知起点后的 0。
3. 后续 delta 为非负差；counter 回退结束旧 baseline并记录质量事件。
4. 连接消失只以最后观察值结束，不补尾差。
5. 跨分钟按单调时间比例分配；每方向分配和等于原 delta。

### 全局核对

同一 frame/epoch/direction 计算：

```text
attributed_observed = sum(connection_delta)
unattributed_gap    = max(0, controller_delta - attributed_observed)
over_attributed     = max(0, attributed_observed - controller_delta)
```

root reset 首帧三者中依赖差值的值为 unknown。负差不抵扣历史。分类守恒独立于 controller meter：

```text
sum(primary_target_categories) + other = attributed_observed
```

### 稀疏 minute facts

minute accumulator 只在以下情况输出：

- upload/download 非零；
- gap/over-attributed 非零；
- 必须持久化的质量或 coverage 边界。

零流量活跃连接通过 session 生命周期表达，不写周期性零行。

## `CommitBundle` 与队列

### bundle 契约

```text
CommitBundle {
  bundle_id,
  payload_hash,
  previous_durable_watermark,
  facts,
  coverage_changes,
  projection_delta
}
```

- `bundle_id` 在首次形成待提交工作时生成，重试保持不变。
- `payload_hash` 防止调用者复用 ID 携带不同内容。
- `previous_durable_watermark` 用于检测错误顺序或跨 epoch 重放。
- projection delta 只有在 commit 成功后可见。

### 有界队列

队列上限来自 C0。分层策略：

1. 完成 Accounting 后，先按相同 `(session_id, utc_minute)` 合并。
2. 容量接近上限时降低非持久化投影通知频率。
3. oldest batch 超过 deadline 或队列饱和时开启 `storage_backpressure` gap。
4. 仍无法恢复时可丢弃尚未 durable 的事实，但必须保留内存 gap 起点，恢复后的第一个可提交 bundle 先写 degraded interval。

不得在 Accounting 前丢 snapshot；不得把丢弃事实记录为 0。

## core SQLite

### 连接策略

- 使用 C0 adopt 的 bundled binding 和实际 SQLite 版本。
- 主库只位于本地 `app_local_data_dir`。
- 单 writer；只读连接使用短 transaction。
- 每连接显式应用 WAL、`synchronous=FULL`、foreign keys 和 C0 busy deadline。
- 运行期优先 PASSIVE checkpoint；受控退出可使用 C0 批准的更强 checkpoint。

### C1 逻辑表

```text
schema_migration
data_version
bundle_epoch
committed_bundle
machine_setting
controller_epoch
target_set
target_item
coverage_interval
connection_session
connection_chain
connection_minute
backup_manifest
```

说明：

- `bundle_epoch` 保存 writer epoch、highest contiguous sequence、durable watermark 与滚动 hash；增长与 writer epoch 数量一致，不与每秒 commit 数量一致。
- `committed_bundle` 是有界的 recent idempotency ledger，`bundle_id = (writer_epoch, bundle_seq)` 唯一，保存 payload hash、结果 data version、durable watermark 和重建 `CommitReceipt` 所需最小字段。
- `data_version` 保存当前 durable watermark；只在同一 commit transaction 最后推进。
- `connection_session` 保存 normalized metadata 与 payload/schema version；unknown 使用 NULL。
- `connection_chain` 保存原始顺序用于诊断，分类算法不读取位置语义。
- `connection_minute` 以 session/minute 为核心唯一键，保存稀疏双向 delta 和必要质量量。
- `backup_manifest` 仅支持 migration 前 backup 的清单与候选验证，不包含用户备份工作流。

C0 可以基于页成本批准额外 core lookup/helper table；新增表必须直接服务以上 core 语义并在 C1 schema 清单中解释。禁止预建 C3/C4 表。

### prepared transaction

writer 启动时准备固定 statement set，逐 bundle：

```text
BEGIN IMMEDIATE
  lookup committed_bundle(bundle_id)
  if exists:
    verify payload_hash
    return stored receipt
  insert committed_bundle pending identity
  upsert core facts with deterministic set semantics
  apply coverage transitions
  advance data_version + durable watermark
  finalize committed_bundle receipt fields
COMMIT
return CommitReceipt
```

SQLite transaction 回滚时不返回 receipt。commit 调用返回错误但结果未知时，调用方以同一 bundle 重试；在已冻结 retry window 与 producer 协议内，唯一键和 payload hash 使结果 exactly-once。

### bundle retry window 与有界账本

- producer 在单个 writer epoch 内使用连续 `bundle_seq`，不跳号复用；进程内只允许重试当前有引用的 pending / uncertain bundle。
- 初始窗口至少保留最近 24 小时与最近 100,000 个 receipt 中覆盖更大的集合，最终值由页成本和故障基准冻结。
- 每次 commit 原子推进 `bundle_epoch.highest_contiguous_seq`、durable watermark 和滚动 hash。只有 sequence 已被该连续水位覆盖、同时越过时间 / 数量安全线且没有进程内引用的 receipt 才可删除。
- pruning 使用小批次并服从 ingestion 优先级；失败只留下可重试旧行，不影响事实或 watermark。
- 窗口内 duplicate 返回已存 receipt；相同 ID / 不同 hash fail closed；窗口外 duplicate 返回 `RetryWindowExpired`，永不把旧 payload 当成新 bundle 执行。
- 重启后唯一合法的不确定重试是 durable watermark 附近的 recent bundle，因此必在窗口内。更老输入代表调用方协议错误，不是恢复途径。

### migration 与 backup primitive

启动序列：

1. 打开并读取 `user_version`/history；
2. future schema 或 checksum mismatch → RecoveryFacade；
3. 对需要迁移的数据库执行分页 Online Backup 至 `.partial`；
4. 完成后校验并原子 rename，登记 manifest；
5. 在 collector 前执行前向 migration；
6. 执行 foreign key/integrity smoke；
7. 成功后启动 writer 和 collector。

C1 不实现候选 swap 或 down migration。

## kill 点与 durable 语义

### commit 前 kill

- 没有 `CommitReceipt`，bundle 可以不存在。
- 重启以最后 durable watermark 为起点，新的采集 baseline 之间形成显式 app/storage gap。

### commit 结果不确定时 kill

- transaction 可能提交也可能回滚。
- 重启后重放相同 `bundle_id`；存在则返回原 receipt，不存在则执行一次。

### commit 后、receipt 未送达时 kill

- `committed_bundle` 和 watermark 已 durable。
- 相同 bundle 重试只读取原 receipt；不得再次写 minute delta 或推进 data version。

测试同时断言 bundle 行数、事实字节、watermark 和 coverage，不能只断言「进程能重启」。

## checkpoint、WAL 与存储故障

- 长读阻止 checkpoint 时，先让读者过期/interrupt，再重试 PASSIVE。
- 不删除或截断活跃 WAL 来恢复空间。
- `journal_mode` 未返回/保持 `wal` 视为 WAL 回落。startup 时 fail closed；运行期转 storage failure/degraded，不继续报告 healthy durability。
- busy 超过 C0 deadline 不是普通慢请求，必须触发 backpressure health。
- disk full/I/O error 后停止返回新的 receipt，队列保持有界，打开 gap；空间恢复并成功 commit 后关闭 gap。

## durable live projection 与原子 Channel

`DurableProjectionState` 由单锁保护：

```text
{ snapshot, current_seq, durable_watermark, subscribers }
```

commit receipt 到达后，在同一临界区：

1. 应用 projection delta；
2. 增加全局 `seq`；
3. 捕获消息；
4. 发布给订阅者。

subscribe 在同一锁下登记订阅者并捕获 snapshot/current_seq，随后第一条发送：

```text
bootstrap { snapshot, baseSeq: current_seq, durable_watermark }
```

因此在 bootstrap 之后收到的消息必有 `seq > baseSeq`。发送队列可以合并可重建 projection，但一旦客户端观察到 seq gap，必须 resync，不能继续应用 delta。

## `RecoveryFacade` backend

RecoveryFacade 使用独立打开路径和最小 DTO：

```text
get_recovery_status()
list_migration_backups()
validate_backup_candidate(path)
get_redacted_diagnostics()
```

- 只读版本 header、backup manifest/sidecar metadata 和候选文件；
- validation 使用独立连接检查 checksum、integrity 和支持的 schema range；
- 不启动普通 migration、collector、writer、ReportService 或 alert；
- 不执行 restore/swap。

## 安全与隐私

- secret resolver 只在 TCP request 构造边界使用，类型不能被序列化。
- 所有 SQL 参数化；payload limits 在 normalizer 边界执行。
- 诊断不包含完整域名、IP、进程路径或 raw payload。
- fault/kill/performance 测试只使用合成临时数据库和隔离子进程。

## 回滚与演进

- 候选发布前，开发数据库可以按 migration 重建。
- 一旦 C1 schema 进入候选 Release，后续只追加 migration；不得修改已发布 migration 或复用 checksum。
- C1 功能回滚可以停止 collector/writer 并保留数据库；不能安装旧 binary 猜测 schema 兼容。
- 报告/retention 由 C3 前向扩展，alert/outbox 由 C4 前向扩展。
- C1 失败时可回滚到 C0 基础骨架，不涉及用户正式数据。
