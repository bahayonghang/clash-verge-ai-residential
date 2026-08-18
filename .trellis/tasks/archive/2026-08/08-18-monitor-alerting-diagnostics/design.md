# C4 技术设计：告警、通知 outbox 与脱敏诊断

## 设计目标

1. 告警与采集事实共享事务边界，任何失败都不会产生「通知已发但账本无记录」或「事实已提交但告警状态丢失」。
2. 速率和周期规则复用既有核算与 C3 报告能力，不形成第二套统计口径。
3. 通知允许失败、重试和崩溃恢复；应用内告警记录始终权威。
4. 规则规模、抖动和通知故障不会把单 writer、SQLite 或内存队列拖出 C1 ingestion SLO。
5. 诊断足以解释故障，同时不暴露 secret 或完整敏感连接元数据。

## 依赖与模块边界

本任务只能在 C3 独立验收后实施。边界如下：

```text
Controller / Storage / Maintenance health
AccountingBatch + shared live projection
                │
                ▼
AlertEngine
  health reducer
  shared 60-second rate windows
  rule matcher + state machine
                │
                ├──────────────┐
                ▼              │
C3 ReportService / rollup      │
  batched usage projection     │
                │              │
                └──────┬───────┘
                       ▼
CommitBundle
  facts + coverage + alert changes + outbox intents
                       │
                       ▼
StorageCoordinator single transaction
                       │ CommitReceipt
              ┌────────┴────────┐
              ▼                 ▼
AlertProjection        NotificationWorker
                              │
                              ▼
                    Windows NotificationSink
```

- `AccountingEngine` 仍拥有 epoch、baseline、delta、UTC bucket 和 coverage，不把告警状态塞回核算状态机。
- `AlertEngine` 是活动、恢复、冷却、静默、滞回、去重和规则版本迁移的唯一所有者。
- `ReportService` 是周期用量的唯一查询 seam。C4 可以增加面向规则批量评估的稳定请求，但实现必须使用 C3 同一 SQL corpus / rollup / coverage 投影。
- `StorageCoordinator` 继续拥有单 writer、migration、事务和只读查询；`NotificationWorker` 不直接持有任意业务写连接。
- `NotificationSink` 只负责系统送达，不决定告警是否成立。
- 前端只消费版本化 DTO，不计算滚动窗口、周期累计或状态迁移。

## 规则与证据契约

### 规则

```text
AlertRule {
  id, version, enabled,
  kind: health | rate | period_usage,
  selector: health_kind | primary_category | domain | process,
  direction: upload | download | combined,
  threshold_bytes_or_bps,
  recovery_threshold,
  period: rolling_1h | local_day | local_month | null,
  timezone,
  cooldown,
  quiet_schedule,
  created_at, updated_at
}
```

- `recovery_threshold` 必须与触发阈值形成有效滞回；无效组合在后端拒绝。
- 本地自然日 / 月规则保存明确 IANA / Windows 可转换的时区标识，不从运行机器当前时区静默漂移。
- 编辑规则生成新版本。旧版本活动实例先记录 `superseded` 或恢复 / 结束原因，新版本重新建立连续命中基线。

### 证据

```text
AlertEvidence {
  rule_id, rule_version, data_version,
  evaluated_at_utc,
  window_start_utc, window_end_utc, display_timezone,
  selector, direction,
  observed_value, trigger_threshold, recovery_threshold,
  coverage_summary, policy_metadata,
  report_query_reference
}
```

证据保存打开同口径报告所需的过滤条件与版本信息，不复制整份报告，不持有长期 SQLite read transaction。敏感 selector 的展示值使用受控投影；诊断包默认进一步脱敏。

## 评估设计

### 健康规则

health reducer 消费稳定、类型化的 health / coverage / maintenance 事件。它按根因键合并状态：

```text
healthy → pending → active → recovering → resolved
                    │
                    └─ quiet / cooldown 仅影响通知资格
```

- 瞬态内部重试不自动等于用户可见故障；达到既有 health 状态阈值后才产生 pending / active。
- 同一根因保持同一个活动实例；严重度或上下文变化追加事件，不新建轰炸式实例。
- 恢复由明确健康状态或成功维护结果驱动，不靠超时猜测。

### 60 秒滚动速率

- 从 C1 已核算的单调时间增量构造共享、有界的 60 秒 ring buffer，按可匹配 selector 和方向维护必要投影。
- 同一 selector / direction 的多条规则共享窗口结果；规则 matcher 先按对象索引候选，再批量比较阈值。
- 每个正常评估 tick 产生一个结果。连续满足计数达到 3 才触发；缺口、epoch reset 或不可用帧清除连续计数并产生 `not_evaluable`，不注入零。
- 恢复使用独立阈值与连续恢复判定。冷却只抑制重复通知，不阻止状态记录。
- ring buffer、selector cache 和候选规则索引均设置上限；超限进入可见 degraded health，不退化为逐规则 SQL。

### 周期用量

周期规则按边界或受控调度批量评估：

```text
AlertUsageQueryBatch
  -> C3 ReportService
  -> C3 rollup / coverage / capability
  -> UsageEvaluation[]
```

- 滚动 1 小时使用 C3 同一时间范围查询；本地自然日 / 月先通过 C3 的时区边界函数得到 UTC 范围。
- 多条规则在一次调度中按兼容查询形状分组，复用 snapshot / rollup 读取，禁止 N 条规则产生 N 次彼此独立的 raw SQL。
- `drilldown_capability` 不支持、coverage 不完整、deadline / interrupt 或 data version 失效时返回可解释的 `not_evaluable`。
- C4 不持久化独立用量累计。持久化内容仅限规则、状态、证据引用和事件。

## 状态机语义

每个 `(rule_id, rule_version, selector_identity)` 最多有一个活动实例：

```text
inactive
  ├─ condition false ───────────────► inactive
  ├─ condition unknown ─────────────► not_evaluable
  └─ condition true × required ─────► active

active
  ├─ still true ────────────────────► active
  ├─ inside hysteresis band ────────► active
  ├─ recovery condition met ────────► resolved
  └─ data unknown ──────────────────► active + evaluation_gap event
```

- 静默时段内仍创建活动 / 恢复事件，但 outbox intent 标记为 suppressed，不进入系统发送。
- 冷却期内继续更新证据和活动状态，只抑制同一实例的重复外部通知。
- 恢复可产生一条恢复通知；是否处于静默仍按恢复发生时刻计算。
- 进程重启从持久实例恢复状态，连续 3 次计数等瞬态窗口重新基线化，避免把不可观察时段拼接成连续命中。

## SQLite 追加设计

C4 通过新的前向 migration 增加以下逻辑表：

```text
alert_rule
alert_instance
alert_event
notification_outbox
```

核心约束：

- 规则版本不可原地覆盖历史语义。
- 活动实例有唯一性约束，阻止同一规则版本和对象并发创建多个活动实例。
- alert event 与 outbox intent 带稳定幂等键，并引用产生它们的 `bundle_id`。
- outbox 状态至少为 `pending | leased | retry | sent | failed | suppressed`。
- 错误只保存稳定分类和脱敏摘要，不保存原始系统错误 payload。

候选索引围绕真实查询设计：

- 告警中心：状态 + 最近事件时间 + 稳定 ID 的 keyset 顺序。
- 活动实例：规则 / 版本 / selector 唯一查找。
- 待发送扫描：`status + next_attempt_at + id` 的有界范围。
- stale lease 回收：`status + lease_until + id` 的有界范围。
- retention：已结束时间 + 稳定 ID。

索引最终由规模 fixture、写放大和查询计划门冻结，不为所有证据字段盲目建索引。

## 事务与 outbox 协议

### 写入

1. `AlertEngine` 对一个 `AccountingBatch` 和相应健康输入生成 alert changes 与 outbox intents。
2. `StorageCoordinator` 使用已有稳定 `bundle_id` 登记提交。
3. 同一 SQLite transaction 写入 facts、coverage、alert instance / event、outbox，并推进 data version / durable watermark。
4. commit 成功返回 `CommitReceipt`；只有 receipt 中的 outbox ID 可以触发即时唤醒。
5. commit 结果不确定时重试相同 bundle；已存在的幂等键返回已有 receipt。

### 认领与发送

worker 启动时立即运行一次，此后周期运行：

1. 按 `next_attempt_at` 和稳定 ID 用 `LIMIT` 读取一小批 eligible 项。
2. 在短事务中以唯一 lease token 原子更新为 `leased`，写入 `lease_until`。
3. 在事务外调用 `NotificationSink`。
4. 成功时以 lease token 条件更新为 `sent`；可重试失败增加 attempt 并计算带抖动的有上限指数退避；永久失败更新为 `failed`。
5. 独立的有界 stale 扫描把过期 `leased` 项恢复为可重试状态。

所有扫描都有批次、单轮耗时和并发上限。writer 忙或 ingestion backlog 上升时，通知工作让步；告警提交不等待系统通知。

## 性能与背压

- frame 热路径不执行逐规则 SQL。规则和必要状态在启动 / 变更时批量加载，使用内存索引匹配。
- 速率窗口只保留 60 秒所需数据，按共享 selector 投影复用；规则规模和 selector 基数必须由 C0 / C4 harness 设置支持上限。
- 周期规则使用低频、有 deadline、可取消的 C3 批量查询。查询与 outbox 扫描均受 ingestion 优先调度。
- alert / outbox 语句纳入 writer 的 prepared statement cache，与 facts 同批提交，不额外创建每条规则 transaction。
- 记录 alert evaluation、transaction 增量、outbox backlog / scan / lease / attempt、DB / WAL、CPU、RSS 和 ingestion latency。
- 压测至少组合 10,000 活跃短峰、health flapping、大量规则、通知持续失败和 stale lease；任何支持上限触发都必须产生显式 degraded health。

## Windows 通知与前端

- Windows adapter 通过稳定应用标识发送通知；正式结果只以 NSIS current-user 安装态为准。
- 系统禁用、Focus Assist 和 API 错误映射为稳定送达状态，不宣称用户一定看到通知。
- 点击通知仅打开或聚焦主窗口，并导航到告警中心；不在通知回调执行危险操作。
- 告警中心通过分页 query 和有序 Channel 更新活动摘要；大历史不经实时 Channel。
- 图表或证据链接直接构造 C3 `ReportQuery`，前端不重算周期用量。

## 脱敏诊断设计

诊断服务从各模块的稳定 health snapshot 生成白名单结构：

```text
versions
controller_transport_status
coverage_summary
writer_and_queue_health
database_wal_checkpoint_health
backup_retention_health
alert_outbox_health
recent_redacted_error_classes
```

- 连接元数据默认只保留计数、分类数量和散列 / 截断后的可选样本；完整域名、IP 和进程路径不进入诊断包。
- secret、Authorization、credential target 的敏感部分和原始异常链在生成边界统一过滤。
- 导出使用临时文件和原子完成；失败只产生脱敏错误，不影响 collector、writer 或 AlertEngine。

## 故障处理与回滚

- SQLite transaction 失败：不发送通知；沿用 C1 storage failure coverage 与健康告警路径，内存有界。
- `NotificationSink` 失败：保留应用内告警并重试；永久失败可见。
- C3 周期查询失败：规则标记不可评估，不回退私有聚合。
- outbox 积压：通知 worker 降速并告警，不抢占 ingestion。
- C4 候选未发布时可禁用 AlertEngine / NotificationSink 并修复；已执行 migration 后只允许前向修复。
- 发布后若系统通知异常，可关闭 `NotificationSink`，继续保留应用内告警、历史与 outbox；不得删除历史表或回退 schema。

## 设计验收

- 数据流和事务故障注入证明 facts、coverage、alerts 与 outbox 原子一致。
- SQL corpus 证明 outbox 扫描命中预期索引且每轮有界。
- instrumentation 证明评估和通知压力不破坏 C1 ingestion SLO。
- 代码审查确认周期规则只依赖 C3 报告 / rollup，前端和 C4 均无第二套聚合。
- 安装态、崩溃恢复、flapping、大量规则、通知失败和脱敏扫描均有可重复证据。
