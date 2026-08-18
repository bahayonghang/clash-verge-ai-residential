# 技术设计：residential-monitor（完整 Windows 11 v1）

## 设计目标

本设计把应用视为“本地可观测性产品”，而不是带图表的连接列表。核心目标：

1. 采集窗口隐藏、WebView 重载或无前端订阅者时仍持续运行。
2. 所有实时、历史、告警和导出数字来自同一事实与同一查询实现。
3. 显式记录采集覆盖和数据质量，不把缺口当成零，不把观测值称为权威账单。
4. 复杂性集中在少数深模块内；前端只学习稳定 DTO，不理解 mihomo 原始 payload 或 SQLite schema。
5. 数据库可迁移、备份、恢复、分层清理并支持上一发布版本升级。

## 总体架构

单个普通用户权限的 Tauri 进程承载 Rust 后台服务和 WebView。v1 不安装 Windows Service。

```text
Clash Verge Rev / mihomo
        │
        ▼
ControllerSession
  discovery + TCP / named pipe + HTTP / WebSocket
  reconnect + auth + version/capability normalization
        │ ControllerInput
        ▼
AccountingEngine
  epoch + baselines + deltas + UTC buckets + coverage + policy tags
        │ AccountingBatch
        ▼
AlertEngine
  rules + cooldown + recovery
        │ CommitBundle
        ▼
StorageCoordinator
  SQLite single transaction: facts + coverage + alerts + notification outbox
        │ CommitReceipt
        ├────────────► LiveProjection
        ├────────────► NotificationSink
        └────────────► read-only queries
        │
        ├────────────► ReportService ──► ExportService
        ├────────────► RetentionService
        └────────────► BackupRestoreService
                               │
                               ▼
AppFacade: typed commands + one ordered Channel
                               │
                               ▼
Vanilla TypeScript + Vite UI

OS seams:
CredentialStore | NotificationSink | Autostart | Tray | FileDialog | Clock
```

- collector、accounting、writer、retention 和 alerts 在 Tauri `setup` 后由 Rust 启动，不属于窗口生命周期。
- WebView 不存在时停止生成 UI payload，但采集、落库、告警和托盘继续工作。
- shutdown coordinator 统一执行：停止接帧 → flush writer → 结束 coverage → checkpoint / 关闭数据库 → 删除托盘 → 退出。

## 深模块与接口

### `ControllerSession`

小接口：

```text
connect(ControllerConfig) -> Stream<ControllerInput>
probe(ControllerCandidate) -> ControllerCapabilities
execute(ControlCommand) -> ControlResult
shutdown()
```

隐藏 discovery、TCP / named pipe 差异、HTTP framing、WebSocket、鉴权、重连、超时、取消和 mihomo 版本兼容。测试使用 replay adapter 输入录制帧，不让调用者 mock 内部 HTTP 细节。

### `AccountingEngine`

纯状态机接口：

```text
apply(AccountingInput, MonotonicTime, UtcTime) -> AccountingBatch
```

输入覆盖 snapshot、controllerConnected、controllerRestarted、disconnected、sleepGap、paused、settingsChanged 和 shutdown。输出包含事实写入、覆盖区间变化、实时投影增量和告警评估输入。它不执行 I/O，因此可用录制序列穷举生命周期。

### `StorageCoordinator`

固定使用 SQLite，不定义假想的通用 Repository。接口：

```text
commit(CommitBundle) -> CommitReceipt
query(QuerySpec) -> QueryResult
maintain(MaintenanceCommand) -> MaintenanceResult
health() -> StorageHealth
```

内部拥有单 writer、受控只读连接、迁移、checkpoint、备份、恢复和 retention 串行协调。测试直接使用内存 / 临时 SQLite。

### `ReportService`

```text
run(ReportQuery) -> ReportResult
export(ReportSnapshotToken, ExportSpec) -> ExportArtifact
```

`ReportResult` 是应用内图表、数据表和所有导出的唯一权威投影，并携带短期有效的不可变 `ReportSnapshotToken`。导出消费该 snapshot，不重新查询一个更晚的数据版本，也不得重新实现聚合逻辑。

### `AlertEngine`

```text
evaluate(AlertInput) -> AlertDecision[]
apply(AlertCommand) -> AlertState
```

冷却、静默、恢复和去重由一个状态机拥有。它把告警状态变化加入与事实、coverage 相同的 `CommitBundle`；SQLite transaction 成功后，`CommitReceipt` 中的 notification outbox 项才交给 `NotificationSink`。发送失败只更新 outbox 状态，不回滚已提交事实或丢失应用内告警。

- 速率规则使用 60 秒滚动平均，连续 3 次 1Hz 评估满足条件才触发；恢复阈值支持滞回。
- 周期用量规则使用 ReportService / rollup 的同源投影，窗口只允许滚动 1 小时、用户本地自然日或自然月。
- 告警证据记录 `data_version`、规则版本、窗口与过滤条件，使告警中心能打开同口径报告。
- notification worker 在应用启动和运行中周期扫描 outbox。它以原子 lease 认领 `pending` / 到期重试项，记录 attempt、`next_attempt_at` 与 `lease_until`；发送成功标记 sent，失败指数退避并保留错误分类，崩溃留下的过期 lease 可被下一实例回收。应用内告警中心显示通知送达状态，保证不存在永久 stuck 且不可见的 outbox。

### `AppFacade`

前端唯一业务 seam：

```text
query(ViewQuery)
command(AppCommand)
subscribe(Channel<MonitorStreamMessage>)
```

前端不调用 SQL、不读取 secret、不直接访问文件系统，不解析 mihomo 原始 JSON。

正常数据库不可用时另有最小 `RecoveryFacade`，只暴露版本 / 诊断、备份列表、候选验证、恢复和打开数据目录；它不依赖正常业务 schema 或 `ReportService`。

## 项目结构

```text
residential-monitor/
├── README.md
├── package.json
├── package-lock.json
├── tsconfig.json
├── vite.config.ts
├── index.html
├── src/
│   ├── main.ts
│   ├── ipc/                 # DTO、decoder、command client、Channel reducer
│   ├── state/               # 前端视图状态；不保存权威业务数据
│   ├── views/               # overview / live / reports / alerts / settings
│   ├── ui/                  # 小型可复用原生控件与图表适配
│   ├── format/              # 字节、速率、时间、覆盖率
│   └── styles/              # tokens / base / layout / views
├── tests/
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── capabilities/
    ├── migrations/
    └── src/
        ├── app/             # setup、facade、shutdown、tray、single instance
        ├── controller/      # discovery、transport、protocol、session
        ├── accounting/      # 状态机、baseline、bucket、coverage、policy
        ├── storage/         # schema、writer、query、migration、backup、retention
        ├── reporting/       # QuerySpec、projection、export
        ├── alerting/        # rules、state machine、notification
        ├── security/        # credential、redaction、capability-owned commands
        └── contract/        # Rust DTO、schema version、错误码
```

根目录现有 `.trellis/spec/frontend/` 只覆盖可粘贴 Clash 扩展，不适用于本子应用。实现前需建立 `residential-monitor` 自有 frontend / backend / storage spec。

## 控制器接入

### 端点模型

```text
Tcp { address, credential_target }
NamedPipe { path, compatibility_profile, verified_server_pid }
```

- TCP External Controller 是受支持路径。secret 从 Credential Manager 读取，只通过 Authorization header 发送。
- TCP v1 只接受 loopback 地址；非本机控制器不在产品范围内。解析后的目标若不是 loopback，probe 和保存设置都拒绝，不通过“风险提示”放行明文远程控制器。
- named pipe 是尽力兼容路径。管道 router 不校验 secret，因此不发送 Authorization，也不把 ACL 错误显示为密钥错误。
- 手动配置优先；自动发现每次核心重启、模式切换或连续失败后重新运行，不永久缓存管道名。

### named pipe 发现

1. 读取已知稳定版运行配置，得到候选而非权威端点。
2. 枚举受限的 `verge-mihomo*` 候选时验证 server PID，并核对为当前 mihomo 核心进程。
3. 先发送不含敏感信息的 `GET /version`，再验证所需字段。
4. 未知布局、ACL 拒绝、PID 不符或协议不兼容时停止猜测，引导启用 TCP。
5. `ERROR_PIPE_BUSY` 使用可取消退避和总时限；区分不存在、拒绝、忙超时。

### 协议实现

- REST 使用成熟 HTTP/1.1 实现作用于统一异步流，支持 Content-Length、chunked、connection-close framing、超时、取消、响应体上限和参数编码；不手写完整 HTTP parser。
- `/connections` 使用 WebSocket，正 interval 默认 1000ms。数组按 ID 合并，不依赖返回顺序。
- `/proxies` 用于目标配置；模型忽略未知字段，版本相关字段可选。
- `/traffic` 不作为必需的第二条长连接。全局 meter 从 `/connections` 根 totals 的相邻差值获得；如未来启用 `/traffic`，必须按流式 HTTP / WebSocket 处理且不得与 connections totals 相加。
- `DELETE /connections/{id}` 的 204 只表示请求处理完成。UI 等待连接从后续 snapshot 消失。

## 核算、时间与数据质量

### epoch 与连接生命周期

- 每次核心进程身份或全局计数器重置建立新 `controller_epoch`。
- 内部连接键为 `(epoch_id, connection_id)`；不假设 ID 跨核心重启唯一。
- 第一次看到连接时记录 metadata 和累计值为 baseline，增量为 0；避免把应用启动前流量计入。
- 后续增量为 `max(0, current - previous)`。计数器下降结束旧 baseline，并形成 reset / restart 质量事件。
- 连接从 snapshot 消失时以最后可见值结束；不伪造最终字节。

### 时间桶

- 速率使用单调时钟计算，避免系统时间回拨。
- 持久时间统一 UTC integer epoch；本地时区仅用于查询和显示。
- 一个采样间隔跨 UTC 分钟边界时，按单调时间比例分配增量，保证总字节守恒，同时标记为采样估算。
- 每秒完整 frame 不持久化；保存连接 session、链和分钟级 delta。

### 两类全局数字

- `attributed_observed`：全部连接 delta 之和，严格满足 `重点主分类 + 其他 = attributed_observed`，是历史报告主口径。
- `controller_meter`：mihomo 根 uploadTotal / downloadTotal 差值，用于全局速率与核对采样遗漏，不尝试把差额分配到域名、进程或分类。
- 两者按同一 epoch、同一相邻 frame、同一方向计算；核心重启 / counter reset 的第一帧都只建 baseline，差额为未知而不是 0。
- `unattributed_gap = max(0, controller_delta - attributed_delta)`；`over_attributed = max(0, attributed_delta - controller_delta)`。负差不抵扣历史，而记录为 over-attributed 质量异常。
- 跨分钟时 controller、attributed、gap 与 over-attributed 使用同一单调时间比例分桶；每个 bucket 分方向保存。
- UI 同时显示 coverage、未归因 gap 与 over-attributed 异常，避免把 controller meter 与逐连接事实混为一套守恒账本。

### 覆盖区间

`coverage_interval` 至少记录：

```text
running | disconnected | tcp_unauthorized | pipe_access_denied |
protocol_incompatible | sleeping_or_clock_gap | paused |
storage_failure | app_exit
```

报告按 bucket 合并 coverage；无覆盖的 bucket 显示缺口，不显示为 0。

## 分类与策略版本

- `target_set` 是有序、版本化配置；每项包含精确名称、启用状态和优先级。
- 每条连接保存原始 `chains` 和全部目标命中标签。
- 唯一主分类由 target priority 决定，与 chain 位置无关。
- raw 明细保留期内，ReportService 可按当前 target set 重算。
- raw 删除前生成带 `target_set_version` 的小时汇总；raw 过期后不承诺用任意新策略完整回算，UI 明确显示“基于历史策略”。

## SQLite 设计

### 位置与连接策略

- `monitor.sqlite3`、备份和机器专属设置位于 `app_local_data_dir`；日志位于 `app_log_dir`。
- bundled SQLite 版本必须为已修复 WAL-reset race 的版本（实现时验证 `sqlite_version()`，基线 3.51.3+ 或官方 backport）。
- 一个 writer actor；查询使用短只读事务。
- 每个连接显式设置 WAL、`synchronous=FULL`、foreign keys、busy timeout。v1 用一次约 1 秒的批量 fsync 换取事实、coverage、告警和 outbox 的断电持久性；C0 必须用目标负载验证性能，任何降级到 NORMAL 或拆分耐久性策略都需要重新评审本设计。
- writer 按约 1 秒或有限行数批量提交。磁盘满 / I/O error 进入 storage failure coverage 和健康告警。
- `busy_timeout` 必须由 C0 按操作 deadline 冻结，不能长于正常 durable commit SLO，也不能充当背压策略。
- WAL 初始保留 SQLite 默认 auto-checkpoint，运行期优先 PASSIVE；FULL / RESTART / TRUNCATE 只用于受控维护或退出。长读造成 checkpoint starvation 时取消 / 过期读者，不删除 WAL。
- `cache_size`、`mmap_size`、`page_size`、`temp_store` 和 `auto_vacuum` 先使用 binding / SQLite 基线；只有独立基准证明收益后才改变。

### 容量模型与稀疏事实

设 30 天平均活跃连接数为 `A`、平均会话时长分钟为 `L`、每会话平均链节点为 `C`、生成非零分钟事实的比例为 `q`：

```text
minute_rows = A × 43,200 × q
sessions    = A × 43,200 ÷ L
chain_rows  = sessions × C
```

- C0 基准档为 `A=50 / 250 / 1000` 的完整 30 天库；每档同时冻结 `L / C / q`、维度基数和每帧变化比例，不能只用 `A` 命名后更换分布。10,000 活跃、1Hz、全部连接计数每帧变化只作为至少 30 分钟短时峰值。
- `connection_minute` 只为非零 delta 或必要质量事件写行；零流量活跃连接由 session / 紧凑活动区间表达，不每分钟写零。
- raw 默认 30 天、v1 上限 90 天；提高期限前按实测 B/row、增长率、备份和临时空间预测。
- C0 用 `dbstat` / `sqlite3_analyzer` 或等价能力记录每表 / 索引真实页成本；不以估算字节替代发布预算。

### 逻辑表

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
traffic_hourly_dimension
traffic_daily_dimension
traffic_daily_core
coverage_daily
retention_state
alert_rule
alert_instance
alert_event
notification_outbox
```

- 新表优先 SQLite STRICT。
- `connection_session` 保存 normalized metadata 与 payload / schema version；可空字段不填伪默认。
- `connection_chain` 保存原始位置仅供诊断，分类不使用位置。
- `connection_minute` 是稀疏的 30 天 raw 事实；小时、日表是经守恒验证的 rollup。
- 索引围绕时间、session、host、process、rule、chain 和 retention cutoff；不为所有隐私字段盲目建索引。
- 域名、进程、规则、链路和网络类型先归一为稳定整数 dimension ID，避免在多个索引重复长 TEXT；大列表使用 keyset pagination，不使用深 OFFSET。
- 表所有权按交付拆分：C1 只创建 migration / data version、bundle epoch / recent receipt、machine、controller epoch、target、coverage、session、chain、minute 与 migration backup 原语；C3 通过前向 migration 增加 dimension rollup、daily coverage 与 retention；C4 再增加 alert 与 notification outbox。不得在 C1 预建语义尚未由 C3 / C4 定稿的空表。

### 保留层与查询能力

为避免“日汇总长期保留”演变成所有维度的无限笛卡尔积，rollup 使用“主分类 + 单一分析维度”，不保存域名 × 进程 × 规则 × 链路的交叉组合：

| 数据层 | 默认期限 | 可查询能力 |
|---|---:|---|
| session / chain / minute raw | 30 天 | 完整会话下钻；分类、标签、域名、进程、规则、链路、网络类型任意组合过滤；当前策略重算 |
| hourly dimension | 13 个月 | 总量趋势；按历史主分类筛选；分别查询分类、域名、进程、规则、链路、网络类型单维 Top N；无会话下钻 |
| daily dimension | 13 个月 | 与 hourly 相同的精确单维趋势 / Top N，粒度为日；无任意跨维组合 |
| daily core | 长期 | attributed total、历史主分类与 coverage；不保留高基数 Top N |
| raw coverage | 13 个月 | 原始缺口原因与精确区间 |
| daily coverage | 长期 | 每日各 coverage 状态的持续时长与覆盖率 |

`traffic_*_dimension` 的逻辑键至少包含 bucket、target_set_version、primary_category、dimension_kind、dimension_value，并保存 upload / download、connection_count 与 observed_active_ms。查询结果必须通过 `drilldown_capability` 告知 UI 当前时间范围可用的过滤和下钻能力；UI 不得对老数据展示不可实现的跨维筛选。

### 写入幂等与背压

- writer 缓存并复用 prepared statements；逐行只做 bind → step → reset，禁止每行重新 prepare 或拼接无界多值 SQL。
- Accounting 在入队前按 `(session_id, utc_minute)` 合并，同一 transaction 每个键最多一次写；不能在生命周期 / delta 核算前盲目丢 snapshot。
- 每个 `CommitBundle` 使用连续 `(writer_epoch, bundle_seq)` 作为稳定 `bundle_id`。同一事务先登记 bundle，再写 facts / coverage / alerts / outbox，最后推进 `data_version`、durable watermark 与 epoch highest contiguous sequence。
- commit 结果不确定时重试相同 `bundle_id`。初始 retry window 至少覆盖最近 24 小时与 100,000 receipts 中较大的集合；窗口内已提交 bundle 返回原 `CommitReceipt`，不得重复累加。
- recent receipt 只有在最高连续 sequence 已覆盖、同时越过时间 / 数量安全线且无进程内引用时才能小批量删除；`bundle_epoch` 保留水位和滚动 hash。窗口外 duplicate 返回 `RetryWindowExpired`，不得重新执行，避免每秒一行永久累积。
- rollup chunk 使用确定性 replacement / set 语义，不使用重试会双加的无条件 `bytes = bytes + excluded.bytes`。
- 队列和内存有界。oldest batch 超时或队列饱和时进入 `storage_backpressure`，停止宣称完整 coverage；仅允许在完成核算后合并同 minute key。恢复后持久化 degraded interval，不静默丢账或无限堆积。

### 迁移

- `PRAGMA user_version` 记录当前整数版本，`schema_migration` 记录 version、checksum、应用时间和 app version。
- migration 在 collector、writer、query 启动前执行。小型 DDL / metadata 变更在单个 `BEGIN IMMEDIATE` transaction 原子应用；大型数据变化使用 expand → checkpointed backfill → contract，可跨启动续跑，不把全部 backfill 塞进一个长事务。
- 已发布 migration 不可修改；checksum 不同或数据库版本高于 binary 支持上限时 fail closed。
- migration 前 Online Backup；成功后执行 foreign key check、必要的 integrity / smoke query。
- migration backup / backfill 延迟采集启动时，Recovery Shell 显示进度并建立未采集 coverage，不把该时段写为零。
- 不把 down migration 当回滚。回滚是恢复升级前备份并安装兼容 binary。
- 子任务按 C1 → C2 → C3 → C4 串行合并，migration ID 在合并时单调分配；后续子任务只追加自己的 migration，不修改前序 migration，也不并行占用版本号区间。

## 查询、报告与导出

### 统一查询

```text
ReportQuery {
  range_utc, display_timezone, granularity,
  filters, grouping, target_policy, comparison,
  sort, page
}

ReportResult {
  schema_version, data_version, report_snapshot_token, query_echo,
  totals, series, rankings, coverage,
  drilldown_capability, policy_metadata
}
```

- filters 覆盖主分类 / 标签、域名、进程、规则、链路、网络类型。
- 所有参数在 Rust 校验并参数化查询；前端不得传 SQL。
- 查询连接短事务；列表使用稳定 sort key + ID 的 keyset pagination，默认 200、最大 1,000，不使用深 OFFSET。Top N 默认 20、最大 100。
- 每个交互查询使用可取消连接，binding 必须暴露 interrupt / progress handler 等价能力。初始 deadline：页面 2 秒、报告 10 秒；导出 / 备份可长跑但必须显示进度并可取消。
- 大结果不经实时 Channel。
- `run_report` 在一个 SQLite read snapshot 中构造结果，并把不可变投影或有界磁盘 spool 绑定到短期 `report_snapshot_token`。token 不持有长期 read transaction；初始 TTL 10 分钟、活动数量与总 spool quota 由 C3 基准冻结。过期后要求重新运行报告。
- 每个公开查询进入命名 SQL corpus。CI 不 snapshot 易变的完整 EQP 文本，而检查意外全表 SCAN、非预期 TEMP B-TREE / 自动索引，并记录 FULLSCAN_STEP、SORT、VM_STEP 与冷 / 热延迟。

### 导出

- CSV、JSON、可打印 HTML 均消费 `report_snapshot_token` 对应的同一个 `ReportResult` / 流式 projection；持续采集期间也不会因二次查询而与屏幕总计漂移。
- 导出 header / metadata 包含 UTC range、本地时区、单位、筛选、策略、schema / data version、生成时间、coverage 和缺口。
- 字段选择与脱敏在 Rust 完成；先生成预览摘要，用户确认路径后写入临时文件并原子完成。

## 保留、备份与恢复

### 分层保留

1. 以 UTC cutoff 和 retention watermark 选择批次。
2. 同一事务先幂等 UPSERT 上层汇总并核对字节 / 行数 / coverage。
3. 成功后推进 watermark，再删除已覆盖下层事实。
4. 每批限制行数和时间，ingestion 优先；中断后能区分“已汇总未删除”和“未汇总”并续跑。
5. manual clean 和 scheduled clean 共用同一服务。
6. DELETE 只增加可复用 freelist，不把逻辑删除量显示为立即释放的磁盘空间；不自动 VACUUM。用户主动 VACUUM 前检查约 2 倍数据库临时空间并暂停写入。

默认：

- session / chain / minute 明细 30 天；
- 精确高基数 hourly / daily 13 个月；
- attributed total / 历史主分类 / coverage daily 长期；
- alert history 180 天；
- raw coverage 13 个月，daily coverage 长期；
- migration 与 backup manifest 长期。

### 备份恢复

- migration 前和用户手动备份使用 SQLite Online Backup API，不复制 hot DB 文件。
- Online Backup 分页 step、报告进度、可取消和节流；开始前检查目标空间。在持续写入下无法满足 commit SLO 或无法收敛时，可短暂停采完成备份，但必须形成 coverage gap。
- 目标先写 `.partial`，关闭并 integrity check 成功后原子 rename。
- 恢复进入 maintenance mode：停 collector / writer / query → 备份当前库 → 验证候选 checksum、integrity、schema → 受控 swap → forward migrate → smoke check → 恢复采集。
- Credential Manager secret 不随数据库备份；跨机恢复要求重新输入。

## Rust ↔ 前端契约

### Commands

包括但不限于：

```text
get_bootstrap
subscribe_monitor
resync_monitor
query_connections
run_report
preview_export
export_report
get_settings
save_settings
probe_controller
list_targets
close_connection
list_alerts
save_alert_rule
test_notification
get_storage_status
run_retention
create_backup
restore_backup
get_recovery_status
validate_backup
open_releases
shutdown_app
```

统一返回版本化成功 DTO 或稳定错误：

```text
AppError { code, message_zh, retryable, action, details_redacted }
```

### Live Channel

单个有序 `Channel<MonitorStreamMessage>`：

```text
snapshot | connectionDelta | healthChanged | alertChanged | dataVersionChanged
```

`seq` 是当前 Rust 进程内全局单调序号。`subscribe_monitor` 在同一 state lock 下登记订阅者、捕获最新 projection 与当前 seq，并保证 Channel 第一条消息是 `bootstrap { snapshot, baseSeq }`；后续消息只允许 `seq > baseSeq`。因此不存在“先取 snapshot、后订阅”之间的丢事件窗口。

每条后续消息含 `schemaVersion`、`seq` 和后端时间。前端发现 seq gap 时停止应用 delta，调用 `resync_monitor`；后端以同一原子流程返回新的 bootstrap / watermark。实时摘要约 1Hz；连接列表使用 delta 或有界 snapshot；Rust 侧 latest-only / coalescing，不能依赖业务级 backpressure。窗口重建后重新建立订阅。

## Windows 生命周期与系统能力

- 稳定 identifier：`io.github.bahayonghang.residential-monitor`，一经发布不得修改。
- NSIS current-user 安装到用户范围，不要求管理员；正式通知只在安装态验收。
- autostart 默认不静默开启，在 onboarding 由用户确认；实际状态以 OS plugin 为准，启动参数 `--background`。
- single-instance 在其他后台服务前注册；第二实例只聚焦现有窗口。
- 点击窗口 X：prevent close + hide；首次提示“仍在托盘采集”。
- Windows 通知先持久化告警，再 best-effort 发送；Focus Assist 或系统禁用不会删除应用内告警。
- v1 不注册 updater plugin。About 页打开固定 GitHub Releases URL。
- 普通卸载默认保留 LocalAppData 数据库、备份和 Credential Manager 凭据，避免误删历史；应用内提供需要二次确认的“删除全部本地数据与凭据”操作。Release 文档同时给出卸载后手动清理路径。

## 安全与隐私

- TCP secret 存 Windows Credential Manager `CRED_TYPE_GENERIC`，target 使用稳定 identifier；SQLite 只存引用。
- `CredentialStore` seam 在 C0 冻结；C1 使用不含真实 secret 的 fake resolver，C2 实现 Credential Manager adapter。v1 不实现 DPAPI fallback；Credential Manager 不可用时拒绝持久化 secret，可允许用户仅在当前进程内临时输入并明确提示退出后失效。
- `withGlobalTauri: false`；生产只加载 bundle 内资源。
- 显式 CSP；不使用 inline handler、远程 URL、CDN 或宽泛 opener / fs 权限。
- capability 只匹配主窗口和 Windows，显式列出必要 app commands；数据库、凭据、通知、autostart 和文件 dialog 由 Rust 拥有。
- command 参数视为不可信；日期、分页、排序、路径、连接 ID 和设置长度在 Rust 校验。
- 日志轮转并按白名单记录；secret、完整元数据和导出内容不进入普通日志。
- named pipe 没有只读角色；设置页说明应用获得完整 mihomo 控制器能力，后台绝不自动发送控制命令。

## 前端与视觉

- Vanilla TypeScript + Vite，不引入 React / Vue / Svelte；锁文件提交，CI frozen install。
- 导航：概览、实时连接、分析报告、告警、设置 / 数据管理。
- 统一 design tokens；深蓝灰基调、等宽数字、明确层级、固定分类色板。
- 图表与表格共享 QueryResult；每个图表有数据表替代、单位、时间范围和缺口标记。
- 状态不是单一红黄绿：明确连接、鉴权、ACL、协议、覆盖、存储、迁移和通知状态，并提供下一步动作。
- 前端 store 只管理视图选择和后端 DTO cache；不持有第二套账本或分类算法。

## 性能与运行预算

C0 在 Windows 11、4 核 x64、8GB RAM、当前 WebView2 的基准机上冻结可复现 harness。v1 初始发布门：

- 完整 30 天库档位 `A=50 / 250 / 1000`；`A=250` 是初始发布设计点，最终支持范围由 C0 实测冻结。记录每表 / 索引 B/row、DB / WAL 峰值、写放大、freelist、冷 / 热查询和 p50 / p95 / p99 / max。
- 10,000 活跃连接、1Hz snapshot、全部连接计数每帧变化：frame receipt → Accounting / Alert 产生 CommitBundle 的计算 p95 < 500ms（不含最多 1 秒 batch wait）；frame receipt → durable commit 的端到端 p95 < 1.5s、最大正常值 < 3s。collector 输入队列不持续超过 2 帧，不发生 seq gap 或内存无界增长。
- 稳态应用总 CPU 平均 < 15%，RSS < 500MB；预热 1 小时后 24 小时 RSS 净增长 < 10%。
- 活跃连接筛选 / 排序的可见交互 p95 < 150ms；30 天 raw 常用报告 p95 < 2s；13 个月 hourly 常用报告 p95 < 3s。
- writer 1 秒批量 transaction；历史查询分页；导出流式写，单次导出额外内存不随输出文件线性增长。
- 监控数据库大小、WAL 大小、最后 commit / checkpoint / backup、retention watermark、队列深度、重连次数和最后帧龄。
- 至少 24 小时真实或回放 soak 使用 C0 批准发布设计点的完整 `A / L / C / q`、维度基数、每帧变化比例，并按运行前冻结的报告 / 导出 / backup / retention / checkpoint / 告警日程执行：零崩溃、零守恒失败、零未解释 coverage gap、零永久 stuck alert / outbox。
- C3 在 writer、report、export、backup、retention 与 checkpoint 并发时仍需满足 ingestion SLO；所有查询可取消，WAL 不无限增长。
- backup、migration 和用户主动 VACUUM 在低磁盘空间下 fail closed，不先破坏当前可用库。

若 C0 基准证明某数值在合理硬件上不可达，必须以测量结果回改预算和验收，不得在 C5 临时忽略。

## 重要取舍

- 采用 SQLite，拒绝 JSON 小时账本。
- 采用 Vanilla TypeScript + Vite，拒绝无构建全局 JS；根扩展的零依赖约束不跨越到独立桌面子应用。
- 采用 Commands + 单 Channel，拒绝高频全量 event snapshot。
- 采用成熟 HTTP 实现，拒绝自写完整 HTTP/1.1 parser。
- 采用后端权威查询，拒绝前端重复汇总、历史筛选和导出计算。
- 采用登录后托盘常驻，拒绝 Windows Service 和关闭窗口即退出。
- 采用 GitHub Releases 手动升级，延后 updater plugin。
- 采用 TCP 作为稳定路径、pipe 尽力兼容，拒绝把 Verge 私有 IPC 当永久契约。

## 回滚与故障边界

- 产品代码仍位于新增 `residential-monitor/` 子目录；可独立停止构建，不改变根扩展运行时。
- 发布后不通过删除目录回滚用户数据。二进制回滚必须检查 schema compatibility；可靠数据回滚使用升级前备份。
- 迁移、恢复或 integrity check 失败时不启动 collector，以只读恢复界面提供诊断和恢复入口。
- 存储运行时失败时停止把新数据标为已持久化，建立缺口并告警；内存不得无界积压。
- 已发布版本不得原地改 migration 或替换同名 Release 资产。发布事故通过新版本或撤回 Release 处理；数据回滚使用已验证备份，不能以安装旧 binary 猜测兼容。

## 尚需实施前验证的技术选型

1. Rust SQLite binding：证明 bundled SQLite 版本、WAL、Backup API、STRICT、license 和错误映射。
2. Windows Credential Manager binding：证明 generic credential CRUD、升级 / 卸载语义和敏感内存处理。
3. HTTP over named pipe 的成熟实现组合及取消 / framing / 大响应压力测试。
4. 最终图表实现的高 DPI、无障碍数据表与打印 HTML 兼容。
5. Windows 代码签名证书是否可用于正式 Release；无证书时必须标明 SmartScreen 风险。
