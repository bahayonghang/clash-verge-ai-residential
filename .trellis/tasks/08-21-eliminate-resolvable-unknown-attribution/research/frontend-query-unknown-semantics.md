# Research: frontend/query Unknown semantics

- Query: 追踪 `residential-monitor` 的概览、Top Hosts/Chains/Processes、Live Connections 从 Rust IPC DTO、查询聚合到 React 展示的 `Unknown` / connecting / no-sample 语义，解释用户截图中这些状态为何可同时出现，并给出可实施的数据契约、UI 状态机和测试建议。
- Scope: internal
- Date: 2026-08-21

## Findings

### 1. 结论摘要

截图不是一个单一的 “Unknown 问题”，而是两个时间面、至少五种原因被同一个英文单词压扁后的叠加：

1. 顶部五张计量卡与活动连接数来自**当前 hub 快照**。后端启动时明确创建 `health=connecting`、所有字节为 `None`、`active_count=0`、`last_sample_utc=None` 的空快照（`residential-monitor/src-tauri/src/c2/hub.rs:212-224`, `residential-monitor/src-tauri/src/c2/hub.rs:441-461`）。前端把所有 `null` 字节统一渲染为 `Unknown`，同时无条件显示数值 `0`（`residential-monitor/src/components/features/overview/caliber-card.tsx:24-60`, `residential-monitor/src/components/features/overview/caliber-card.tsx:67-106`）。这组 Unknown 的准确含义是“控制器正在连接、当前还没有可计量样本”，不是“已有流量但归因失败”。
2. 趋势和三组 Top 排名来自**选定 24 小时窗口的 SQLite 报告**。Overview 同时发出 host/chain/process 三次 `targetPolicy: "historical"` 查询，趋势复用 host 报告（`residential-monitor/src/components/features/overview/index.tsx:38-68`, `residential-monitor/src/hooks/use-report.ts:83-105`）；Rust `run_report` 直接从 storage path 创建报告（`residential-monitor/src-tauri/src/c2/facade.rs:983-1001`），不要求当前控制器已连接。因此“当前 Connecting + 活动 0”与“过去 24h 有 13.4 GiB 历史事实”完全可以同时成立。
3. 历史 `__unknown__` 是**维度归因守恒桶**，不是空态。Host、Process、Chain 各自独立聚合，相同字节可在三个维度得到三个不同的 Unknown 数量（`residential-monitor/src-tauri/src/c3/sql.rs:41-101`, `residential-monitor/src-tauri/src/c3/service.rs:241-278`）。
4. 本机数据库审计已把截图底部三类 Unknown 精确拆开：Chain 约 4.7 GiB 是合法单跳 `DIRECT` 被查询/物化错误映射为 Unknown；Host 约 4.4 GiB 是旧会话未保存 host/IP，当前库无法无损恢复；Process 约 13.4 GiB 是 37,128 个会话中 37,127 个没有 `process_id`。证据见 `research/local-database-unknown-audit.md:9-38`。
5. 用户提供的 Clash Verge 截图能证明当前部分连接的 Host/Chains/Rule 等列有值，但可见行的 Process 列本身为空。它不能证明 24 小时内每条历史连接都具备每个字段，也不能让当前活动连接反推旧字节。Process Unknown 在现有证据下应被表达为“控制器未报告/维度覆盖极低”，而不是伪造进程。
6. 前端已有一套较好的 Live Hotspot 状态门控，却没有复用于 Overview：热点会在暂停、断连、gap、collector 状态未知或 sample 缺失时隐藏旧值（`residential-monitor/src/format/live-hotspot.ts:42-79`）；Overview 卡只看 `number | null`，完全不知道 `null` 的原因。这说明最小改造应抽出共享的 observation-state 语义，而不是再增加一个 Unknown 文案分支。

### 2. 截图中各区域为什么能够同时出现

| 截图区域 | 实际数据源 | 截图状态 | 当前代码语义 | 正确解释 |
| --- | --- | --- | --- | --- |
| 顶部 Header | `stream.snapshot.health.session`，无流则回退 bootstrap | `Connecting to the controller` | `app.tsx` 从实时快照选 session（`residential-monitor/src/app.tsx:66-70`） | 当前连接生命周期状态 |
| 五张字节卡 | 同一 `LiveOverview` | Upload/Download 都是 `Unknown` | `None/null` 统一交给 `formatBytes(..., unknown)`（`residential-monitor/src/components/features/overview/caliber-card.tsx:24-60`） | 当前未形成样本，或首帧仅建基线；不是历史归因桶 |
| Active connections | 同一 `LiveOverview.activeCount` | `0`、No sample | 空快照把无观测编码成 `active_count=0`（`residential-monitor/src-tauri/src/c2/hub.rs:441-461`） | “尚未观测”被错误包装成真实零 |
| Traffic trend | host `ReportResult.series/totals` | 24h 有 11.5/1.9 GiB | host 历史报告同时供趋势使用（`residential-monitor/src/components/features/overview/index.tsx:48-63`） | 过去持久化事实，不是当前连接流 |
| Trend coverage | `ReportResult.coverage` | partial/gap | Trend 独立显示报告覆盖（`residential-monitor/src/components/features/overview/trend-card.tsx:27-35`, `residential-monitor/src/components/features/overview/trend-card.tsx:69-84`） | 历史窗口覆盖；不能解释具体维度为什么缺失 |
| Top Hosts/Chains/Processes | 三次独立 historical report | 三个 Unknown 数量不同 | 每个 grouping 单独 query/rank（`residential-monitor/src/components/features/overview/index.tsx:48-50`） | 同一历史字节在不同维度的缺失/误分类程度不同 |

前端目前没有把“当前快照”和“历史报告”标成两个数据面。`app.tsx` 一边向 Overview 传实时 `overview`，一边传全局 `timeRange`（`residential-monitor/src/app.tsx:186-195`）；Overview 又在同一视觉层级紧接着渲染实时卡、历史趋势和历史 Top（`residential-monitor/src/components/features/overview/index.tsx:53-77`）。因此用户自然会把顶部连接失败理解为底部历史 Unknown 的直接原因。

建议在 UI 上明确标注：

- 顶部卡组标题/角标：`实时 · 当前控制器`。
- 趋势与排名标题/角标：`历史 · 已存储数据 · 过去 24 小时`，并显示 `generatedUtc` / coverage status。
- Header 断连只影响实时面；历史面可继续查询，但不得写成“当前连接统计”。

### 3. LiveOverview 把四种状态压成一个 null

#### 3.1 后端至少产生两种合法 null

- **Connecting/no sample**：Hub 构造空 Overview，所有计量字段 `None`、活动数 0、采样时间和 coverage 均空（`residential-monitor/src-tauri/src/c2/hub.rs:441-461`）。
- **Connected/first baseline**：Accounting 首次看到控制器总表时只保存 meter baseline；每个新连接首帧也只保存 per-connection baseline，不生成差分（`residential-monitor/src-tauri/src/accounting.rs:188-219`, `residential-monitor/src-tauri/src/accounting.rs:249-283`）。测试明确断言首帧 attributed 为 None、第二帧才出现值（`residential-monitor/src-tauri/src/accounting.rs:393-410`）。

这两者都应保留“观测下界、非账单”语义，但 UI 文案不同：前者是“等待控制器”，后者是“正在建立差分基线”。当前 DTO 只有 `health.session`、`lastSampleUtc` 和多个 `number | null`，没有显式 `samplePhase` 或每字段 reason（`residential-monitor/src/dto.ts:30-49`）。

#### 3.2 IPC decoder 进一步合并“缺字段”和“有意 null”

`optionalNumber` 同时把 `null` 和 `undefined` 变成 `null`；`coverageKind` / `coverageReason` 也相同（`residential-monitor/src/ipc/decoder.ts:12-20`, `residential-monitor/src/ipc/decoder.ts:36-77`）。因此：

- 后端有意发送 `null`（合法 pending/unavailable）；
- 后端 schema 漂移导致字段漏发；
- 旧版本 payload 不含字段；

都会变成同一种成功解码结果。相比之下，Live Connection page decoder 对 `matchedCount`、`sampleUtc`、`summary` 使用 `hasOwn`，缺字段直接失败（`residential-monitor/src/ipc/live-session.ts:115-141`）。Overview 应采用同样的“字段必须存在，值可为 null”契约。

#### 3.3 Overview 文案与状态自相矛盾

`CaliberGrid` 只要 `coverageKind` 为 null，就使用 `overview.coverage_ok`，英文是 `Coverage: collecting. Last sample {time}`（`residential-monitor/src/components/features/overview/caliber-grid.tsx:9-19`, `residential-monitor/src/i18n/en.ts:114-115`）。所以截图在 Header 明确 `Connecting` 时仍显示 “Coverage: collecting. Last sample No sample”。`coverageKind=null` 既可能是“尚无样本”，也可能是正常样本无 gap；不能据此声称 collecting。

Health 本身已经能区分 `connecting` 与 `no_data`，并提供不同动作（`residential-monitor/src/i18n/en.ts:46-47`, `residential-monitor/src/i18n/en.ts:88-89`），但五张字节卡完全不读取 health。现有单测只保护“null 不显示成 0”，没有验证 null 原因或 Header/卡片一致性（`residential-monitor/src/components/features/overview/caliber-grid.test.tsx:40-67`）。

#### 3.4 断连后的旧快照也缺少 freshness 语义

Hub 的 `publish_health` 只替换 `snapshot.health`，不清空既有 rows、last sample 或数值（`residential-monitor/src-tauri/src/c2/hub.rs:355-370`）；前端 `healthChanged` 也只替换 `snapshot.health`（`residential-monitor/src/ipc/reducer.ts:73-81`）。这可以保留诊断上下文，但 UI 必须把旧值标为 stale/last-known，不能继续当作当前值。当前 Overview 没有这层门控。

### 4. 历史排名的 Unknown 契约过于贫乏

#### 4.1 后端有守恒桶，DTO 没有原因

Raw Host 把空 `connection_session.host` 收入 `__unknown__`；Process 等 attr 通过字典 join，将无字典值收为同一哨兵；Chain 也用同一哨兵（`residential-monitor/src-tauri/src/c3/sql.rs:41-101`）。`load_rankings` 只把哨兵 label 改成中文“未知”（`residential-monitor/src-tauri/src/c3/service.rs:597-621`）。

`ReportResult.rankings` 最终只有 `identity/label/bytes/count/duration`，没有：

- `unknownReason`；
- metadata source（host/sniffHost/destinationIP/process/chain）；
- known/unknown coverage ratio；
- 是否可由当前代码修复、仅对未来改善、或历史不可恢复。

证据见 `residential-monitor/src/dto.ts:295-342`。`decodeReportResult` 只检查外壳、token、totals、coverage，然后直接 cast；rankings 的形状和未来新增 reason 都没有运行时校验（`residential-monitor/src/dto.ts:495-507`）。

#### 4.2 前端又把 sentinel、空 label 和维度原因合并

`rankDisplayLabel` 对 `identity === "__unknown__"` 或空 label 都返回同一个本地化 Unknown（`residential-monitor/src/format/rank.ts:3-24`）。Overview Top 只读 `result.rankings`，无排名时显示 empty，有行时只显示通用 label 和字节，不展示 `coverage` 或 attribution quality（`residential-monitor/src/components/features/overview/top-columns.tsx:80-137`）。

当前代码仅为 Host 哨兵提供跨维下钻，Process/Chain 哨兵禁止下钻（`residential-monitor/src/components/features/dimension/rank-table.test.tsx:70-104`）。这与 DTO 注释“未知不参与下钻”（`residential-monitor/src/dto.ts:315-323`）以及 spec “filters 无法表达哨兵”（`.trellis/spec/residential-monitor/frontend/dto-and-decoding.md:154-158`）已经漂移；`view-state` 才记录了当前 Host 特例（`.trellis/spec/residential-monitor/frontend/view-state.md:11-14`）。规划应同步契约，不能继续让三处各说一种行为。

### 5. 三个底部 Unknown 的准确原因与 UI 处理

| 维度 | 产生点 | 当前持久化/查询 | 截图证据结论 | 是否可安全消除 | 推荐表达 |
| --- | --- | --- | --- | --- | --- |
| Host | 旧采集只写空 host；旧会话没保存目的 IP | Raw host 从 `connection_session.host` 聚合（`residential-monitor/src-tauri/src/c3/sql.rs:41-54`） | 旧 NULL host 约 4.4 GiB | 旧历史不可恢复；新采集已用 `host -> sniffHost -> destinationIP` | `未知主机 · 旧记录未保存可用主机/IP`，并标 `historical_unrecoverable` |
| Chain | `last_chain_hop` 对无 `>` 的单跳返回 None（`residential-monitor/src-tauri/src/c3/rule_name.rs:10-23`） | Chain rank 将 None 映射 `__unknown__`（`residential-monitor/src-tauri/src/c3/sql.rs:89-101`）；hourly 也落 id 0（`residential-monitor/src-tauri/src/c3/retention.rs:269-284`） | 全部 session 有 chain，约 4.7 GiB 是单跳 `DIRECT` | 可以确定性修复 raw/filter/materialization；旧物化需受控重建 | 修复后显示 `DIRECT`；仅真实空 chain 才显示 `未知链路` |
| Process | Mihomo/TUN 当前样本基本未报告 process | 空 `process_id` 经 attr rank 进 `__unknown__`（`residential-monitor/src-tauri/src/c3/sql.rs:56-73`） | 仅 1 个会话/约 9.7 KiB 已知，其余约 13.4 GiB 缺失；Clash 截图 Process 列也空 | 不可猜测；只能保留同 connection 后到/暂失的真实字段 | 主状态 `进程维度不可用（控制器未报告）`；仍允许查看少量已知进程和 `未报告 99.99%` 守恒量 |

上一归档任务已经明确：Host 改进只对重新采集生效，不回填旧 NULL，也不重跑旧 materialization（`.trellis/tasks/archive/2026-08/08-21-unknown-host-attribution/prd.md:17-23`, `.trellis/tasks/archive/2026-08/08-21-unknown-host-attribution/prd.md:29-33`）。因此修复代码已经存在仍看到 24h Host Unknown，不足以证明修复失效；需要先确认安装包版本，再让旧窗口自然过期或明确标注旧历史边界。

### 6. 查询前还有一个会让“曾经已知”退化为 Unknown 的持久化缺陷

`persist_slice` 先用每帧 live rows 更新 session metadata，再写 minute facts（`residential-monitor/src-tauri/src/storage.rs:538-560`）。Host 使用 `prefer_host_identity`，空值不会覆盖已有 host（`residential-monitor/src-tauri/src/storage.rs:570-598`）；但 `intern_and_attr` 对 process/rule/network/chain 使用整行 `excluded.*` 覆盖（`residential-monitor/src-tauri/src/storage.rs:639-678`）。

结果是：同一个 `session_pk` 的后续帧若暂时缺少 process 或 chain，现有非空 attr 会被清成 NULL；而 raw ranking 是 minute facts 在查询时 join **当前唯一 attr 行**，所以一次空帧可以让该 session 之前已经记录的全部字节退化成 Unknown。这直接违反任务 AC3。

仓库内 `neko-master` 可借鉴的性质是“同一 flow 的后续 delta 复用已保存 metadata”：direct collector 的首帧建立 `TrackedConnection`，后续 delta 用 `existing.domain/chains/rule`（`ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts:447-481`, `ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts:513-575`）；Agent 还明确注释 first-seen 后保持 metadata 稳定（`ref/neko-master/apps/agent/internal/agent/runner.go:458-478`, `ref/neko-master/apps/agent/internal/agent/runner.go:524-536`）。

但不能原样照搬：

- neko 首帧有累计量就直接计入（`gateway.collector.ts:447-481`），本项目为了观测下界只建 baseline，这一口径必须保留。
- neko 永久冻结首帧 metadata，无法吸收后到字段；本项目需要 per-`epoch:id` 的**单调可信合并**。
- neko Agent 把空 chain 强制成 `DIRECT`（`ref/neko-master/apps/agent/internal/gateway/client.go:305-323`），这会混淆“缺失”和“已确认直连”，不能作为通用 fallback。
- neko shared schema 声明 process（`ref/neko-master/packages/shared/src/index.ts:3-41`），但 collector 聚合路径没有 process，实现上不能为本项目提供进程归因。

推荐 canonical metadata 规则：只在相同 `epoch:id` 内合并；incoming 非空可以填充/按明确优先级升级，incoming 空仅表示该帧未报告，不清除已知值；连接关闭或 epoch 改变立即终止，绝不跨连接串值。SQLite UPSERT 应再次防御性执行 non-empty preserve。

### 7. Live Connections 已部分区分状态，但仍有两个混淆点

Live page 的 query envelope 比 Overview 严格：`rows/matchedCount/sampleUtc/summary` 来自同一 hub snapshot（`residential-monitor/src-tauri/src/c2/facade.rs:734-742`, `residential-monitor/src-tauri/src/c2/query.rs:374-435`），前端 decoder 要求字段存在（`residential-monitor/src/ipc/live-session.ts:115-141`）。页面把 unconfigured/paused/disconnected/connectedEmpty/resync/hasRows 分开（`residential-monitor/src/ipc/live-empty.ts:23-63`），热点又进一步区分 ready/noMatch/gap/unknown（`residential-monitor/src/format/live-hotspot.ts:42-79`）。

残余问题：

1. `connecting` 被放进 `DISCONNECTED_SESSIONS`，所以 Live empty state 无法表达“正在连接”（`residential-monitor/src/ipc/live-empty.ts:23-35`, `residential-monitor/src/ipc/live-empty.ts:47-63`）。
2. `rowCount > 0` 的判断早于 session/paused 判断；Hub health-only 更新又不清 rows。代码允许旧 rows 在断连时被判为 `hasRows`，尽管热点值会被正确隐藏。这至少需要一个回归测试；若确实保留旧 rows，应显示 `stale` 而非 current。

此外，表格单元格仍把空 chains、rule、process、time/source/destination/type 全部渲染为同一个 Unknown（`residential-monitor/src/format/live-row.ts:52-79`）。页面级状态较清楚，字段级 provenance 仍丢失。

## Recommended UI / Data-contract State Machine

### 8. 不再用一个 Unknown enum；至少分三条正交轴

#### 8.1 数据面 `SourcePlane`

```ts
type SourcePlane =
  | { kind: "live"; controllerEpoch: number | null }
  | { kind: "historical"; rangeStartUtc: number; rangeEndUtc: number; dataVersion: number; generatedUtc: number };
```

历史 `ready` 不能推导 live `connected`；live `connecting` 也不能让历史报告变 empty。组件必须显式接收数据面，不再靠页面位置暗示。

#### 8.2 实时观测阶段 `LiveObservationPhase`

```ts
type LiveObservationPhase =
  | { kind: "unconfigured" }
  | { kind: "connecting" }
  | { kind: "baselinePending"; sampleUtc: number | null }
  | { kind: "current"; sampleUtc: number }
  | { kind: "paused"; lastSampleUtc: number | null }
  | { kind: "disconnected"; lastSampleUtc: number | null; reason: string }
  | { kind: "resyncRequired"; lastSampleUtc: number | null }
  | { kind: "decodeFailed"; reason: string };
```

推荐转换：

```text
unconfigured -> connecting -> baselinePending -> current
current -> paused | disconnected | resyncRequired | decodeFailed
paused | disconnected -> connecting
```

`activeCount=0` 只有在 `current` 样本中才是“真实无活动连接”；在 `connecting/baselinePending` 中应显示 `—` 和原因。断连后可保留 last-known 值，但必须带 lastSampleUtc 和 stale 标签。

#### 8.3 值与归因状态 `ObservedValue<T>` / `AttributionQuality`

```ts
type ObservedValue<T> =
  | { kind: "known"; value: T }
  | { kind: "pending"; reason: "controller_connecting" | "first_baseline" | "no_sample" }
  | { kind: "unavailable"; reason: "paused" | "disconnected" | "protocol_error" | "storage_error" }
  | { kind: "missing"; reason: "not_observed" | "coverage_gap" };

type AttributionQuality = {
  status: "complete" | "partial" | "unavailable" | "legacy_unknown";
  knownBytes: number;
  missingBytes: number;
  knownSessions: number;
  missingSessions: number;
  reason: "missing_host" | "missing_process" | "missing_chain" | "legacy_missing_metadata" | "none";
  recoverability: "resolved" | "future_only" | "irrecoverable";
};
```

排名行继续保留 `__unknown__` 守恒字节，但增加结构化 `identityKind: "known" | "missing"` 和 `unknownReason`。不能只换 label 后隐藏 missing bytes。Process 维即使 `status=unavailable`，也应保留唯一已知 `mihomo` 行和 missing bytes 明细。

### 9. 最小数据契约落点

1. `LiveOverview` 增加显式 `observationPhase`（或 `sampleState`），不要让前端从 `health + lastSample + null` 猜。所有现有可空字段在 JSON 中仍必须出现。
2. `decodeOverview` 对每个字段先 `hasOwn`；字段存在且值为 null 才是合法 unknown/pending。schema 漏字段应冻结流并提示协议不兼容。
3. `activeCount` 可保持 number，但 UI 只在 current phase 使用；更严格的 contract 可改成 `number | null` 并带 reason。
4. `ReportResult` 增加 dimension-level `attributionQuality`，ranking missing row 增加 reason/recoverability；decoder 完整验证 rankings/quality，而不是 unsafe cast。
5. Chain 使用独立 `chain_identity(chain_key)`：空字符串/空数组 -> missing；单跳 trim 后原值（含 `DIRECT`）；多跳保持当前产品约定的最后一跳。不要修改 Rule grouping 的 `last_chain_hop -> rule -> DIRECT` 语义。
6. Overview Top 使用 `ReportResult.coverage` 标记 historical partial/empty，同时使用 `attributionQuality` 解释维度缺失；二者不可互相替代。
7. Live rows 若暂时保留旧快照，应在 page DTO 增加 snapshot freshness/phase，避免仅凭 `rowCount` 判断 current。

### 10. 推荐 UI 文案和行为

| 状态 | 计量卡 | Active connections | 历史趋势/排行 |
| --- | --- | --- | --- |
| connecting | `等待控制器连接` | `—` | 继续显示并标 `历史 · 已存储数据` |
| baselinePending | `正在建立差分基线` | 可显示当前 rows，但标 `首帧` | 独立显示历史 |
| current + value | 格式化数值；真实 0 显示 0 | 真实数字 | 独立显示历史 |
| paused/disconnected | `当前不可用` + last sample；可折叠显示 last-known | `—` 或 `上次 N`，不得冒充当前 | 继续显示历史 |
| historical host missing | 不适用 | 不适用 | `未知主机（旧记录缺少主机/IP）` |
| historical chain resolvable | 不适用 | 不适用 | 修复后显示 `DIRECT`，不再 Unknown |
| historical process missing | 不适用 | 不适用 | Banner `控制器未报告进程；已知 1 / 37128 会话`；已知行照常展示，missing bytes 单独守恒 |

这套文案不需要把现有 `Unknown` 全部删掉；它把 Unknown 限定为**已有观测中确实缺少某个维度**，并为 pending/unavailable/stale 使用不同词。

## Test Gaps

### 11. 现有保护

- Overview 单测只证明 null 不伪装成 0（`residential-monitor/src/components/features/overview/caliber-grid.test.tsx:40-67`）。
- Accounting 单测证明首帧不计入累计量、第二帧才形成 delta（`residential-monitor/src-tauri/src/accounting.rs:393-410`）。
- Live empty state 覆盖 unconfigured/disconnected/paused/connectedEmpty/resync，但把 connecting 纳入 disconnected，且明确测试有 rows 时优先 `hasRows`（`residential-monitor/src/ipc/live-empty.test.ts:19-60`）。
- Live hotspot 覆盖 paused/gap/disconnected/unknown 时隐藏旧事实（`residential-monitor/src/format/live-hotspot.test.ts:42-70`）。
- Rank table 覆盖 Host Unknown 可下钻、Process Unknown 不可下钻（`residential-monitor/src/components/features/dimension/rank-table.test.tsx:70-104`）。
- C3 service 有 orphan attr 汇总为 Unknown、总量守恒且 sentinel 不进入字典的测试（`residential-monitor/src-tauri/src/c3/service.rs:1271-1308`）。

### 12. 必须补充

| 层 | 缺口 | 建议断言 |
| --- | --- | --- |
| DTO decoder | Overview 缺字段与显式 null 被同样接受 | 缺任一约定字段 -> decode fail；字段 present/null -> 合法 pending |
| Live observation | connecting、first baseline、current zero、disconnected last-known 无组件矩阵 | 四态文案、active count、数值 freshness 分别正确 |
| Overview composition | 没有“live connecting + historical report ready”集成测试 | Header/卡片显示 connecting；趋势/Top 保留且带 historical 标识 |
| Coverage | `coverageKind=null` 被写成 collecting | connecting/no sample 不显示 collecting；current/no gap 才显示 covered/collecting |
| Live empty | connecting 被压成 disconnected；stale rows 可优先 hasRows | connecting 独立；断连有旧 rows 时显示 stale/disconnected，不显示 current table |
| Report decoder | rankings/quality 没有 runtime validation | 缺 identity/reason/quality、非法 bytes/enum -> fail |
| Chain raw | 单跳 `DIRECT` 误归 Unknown | raw rank/filter 对 DIRECT 精确命中；空 chain 才 Unknown；多跳不变 |
| Chain materialization | hourly chain id 0 错误 | intern/materialize/rebuild 后 DIRECT 使用非零 dimension id；daily 同步 |
| Process capability | 99.99% missing 仍显示普通 Unknown 第一名 | 页面显示 unavailable/coverage，known row 可见，known+missing=total |
| Metadata merge | process/chain known 后遇空帧被清除 | 同 `epoch:id`: missing -> known -> temporary missing，attr 保持 known；新 epoch 不继承 |
| Historical compatibility | 旧 host NULL / old dimension id 0 无 reason | 旧数据保持守恒并标 legacy/irrecoverable，不由当前连接回填 |
| Native acceptance | 没有真实 controller 对照 | 保存同窗脱敏 `/connections` payload + monitor session/attr/rank；安装包版本可识别；未执行前标 UNVERIFIED |

## Files Found

- `residential-monitor/src/app.tsx` — 同一 Overview 页面组合实时健康/快照与历史 time range。
- `residential-monitor/src/components/features/overview/index.tsx` — 实时卡组以及 host/chain/process 三次历史报告的组合入口。
- `residential-monitor/src/components/features/overview/caliber-grid.tsx` / `caliber-card.tsx` — 把所有 null 数值渲染为通用 Unknown，并将 null coverage 写成 collecting。
- `residential-monitor/src/components/features/overview/trend-card.tsx` / `top-columns.tsx` — 历史 trend 显示 coverage，而 Top ranking 不显示 coverage/attribution quality。
- `residential-monitor/src/hooks/use-report.ts` — 历史 ReportQuery 构造及 IPC 调用。
- `residential-monitor/src/dto.ts` / `residential-monitor/src/ipc/decoder.ts` — LiveOverview 与 ReportResult 的当前贫信息 DTO/decoder。
- `residential-monitor/src/ipc/live-session.ts` / `live-empty.ts` — Live page 的严格信封 decoder 和页面状态分类。
- `residential-monitor/src/format/live-hotspot.ts` / `live-row.ts` — 热点 freshness 门控与表格单元格通用 Unknown 回退。
- `residential-monitor/src-tauri/src/c2/hub.rs` / `accounting.rs` — connecting 空快照、first-baseline 和实时 sample 生成。
- `residential-monitor/src-tauri/src/c2/query.rs` / `c2/facade.rs` — Live 同快照查询与 historical SQLite report 的独立路径。
- `residential-monitor/src-tauri/src/c3/sql.rs` / `service.rs` / `rule_name.rs` / `retention.rs` — Unknown sentinel、Chain 单跳缺陷、报告覆盖和物化语义。
- `residential-monitor/src-tauri/src/storage.rs` — session metadata 的 Host 单调合并与其它 attr 空值覆盖风险。
- `ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts` / `apps/agent/internal/agent/runner.go` — 连接内 metadata 复用参考及不适用的首帧冻结/累计量策略。
- `.trellis/tasks/archive/2026-08/08-21-unknown-host-attribution/` — 已完成 Host fallback 的范围与明确不回填旧历史的边界。
- `.trellis/tasks/08-21-eliminate-resolvable-unknown-attribution/research/local-database-unknown-audit.md` — 截图三类 Unknown 的本机 SQLite 只读复现。
- `.trellis/tasks/08-21-eliminate-resolvable-unknown-attribution/research/neko-master-attribution-comparison.md` — 参考实现的完整采用/调整/拒绝矩阵。

## Code Patterns

- **双数据面**：`stream.snapshot ?? boot.overview` 只代表 live；`useReport(... targetPolicy: historical)` 只代表历史，UI 不应隐式互相解释（`residential-monitor/src/app.tsx:66-70`, `residential-monitor/src/components/features/overview/index.tsx:48-68`, `residential-monitor/src/hooks/use-report.ts:91-105`）。
- **合法 null 不等于缺字段**：Live page 已使用 `hasOwn` 严格区分，可作为 Overview decoder 的项目内先例（`residential-monitor/src/ipc/live-session.ts:115-141`）。
- **先门控 freshness，再显示事实**：`liveHotspotStatus` 是可复用模式；Overview 目前绕过了它（`residential-monitor/src/format/live-hotspot.ts:42-79`）。
- **Unknown 是守恒行而非 empty**：C3 测试保留 orphan attr bytes，且 sentinel 不写入字典（`residential-monitor/src-tauri/src/c3/service.rs:1271-1308`）。
- **单调连接内 metadata**：Host 已有 `prefer_host_identity`；其它维度应采用同类但维度特定的 merge，不能用整行空值覆盖（`residential-monitor/src-tauri/src/storage.rs:570-598`, `residential-monitor/src-tauri/src/storage.rs:639-678`）。
- **Chain 与 Rule identity 分离**：`last_chain_hop` 的单跳 None 对 Rule 是有意回退，对 Chain 是 bug；不可修改一个共享函数同时改变两种产品语义（`residential-monitor/src-tauri/src/c3/rule_name.rs:10-31`, `residential-monitor/src-tauri/src/c3/sql.rs:75-101`）。

## External References

- 未进行外部网络检索。本研究只使用仓库当前代码、任务内只读数据库审计和仓库内 `ref/neko-master` 快照。
- `ref/neko-master` 是实现参考，不是 Mihomo API 的规范证明。字段是否在当前 Windows/TUN 模式稳定提供，仍须用真实控制器脱敏 payload 验证。

## Related Specs

- `.trellis/spec/residential-monitor/frontend/dto-and-decoding.md:82-84` — 规定 null 不得伪装成 0，但尚未区分 null 原因。
- `.trellis/spec/residential-monitor/frontend/dto-and-decoding.md:89-120` — Live query envelope、同快照 summary 和严格 decoder，可作为 Overview 新契约模板。
- `.trellis/spec/residential-monitor/frontend/dto-and-decoding.md:154-158` — ranking sentinel 说明已与 Host Unknown 可下钻实现漂移，需要同步。
- `.trellis/spec/residential-monitor/frontend/view-state.md:11-18` — Host 哨兵、热点 freshness 和 Live 空态必须分开的现有约束。
- `.trellis/spec/residential-monitor/backend/index.md` / `backend/modules-and-errors.md` — Controller/Accounting/C2/C3 分层、Host fallback 和连接生命周期约束。
- `.trellis/spec/residential-monitor/storage/sqlite-contract.md` — SQLite writer、兼容与事务边界；旧 Unknown 迁移不得绕过。
- `.trellis/tasks/08-21-eliminate-resolvable-unknown-attribution/prd.md:20-42` — R1-R7 与 AC1-AC7，尤其数据契约区分、Chain identity、Process capability 和真实控制器验收。

## Caveats / Not Found

- 数据库审计验证了截图对应开发态库里的字节和字段分布，但没有验证正在运行的安装包是否包含最新 Host fallback；安装包/前端版本仍为 **UNVERIFIED**。
- 尚未取得用户现场 Mihomo `/connections` 的脱敏原始 JSON。Clash Verge UI 只能作为人工线索，不能证明字段在整个 24h 窗口持续存在，也不能证明后端原始 key/empty/null 形态。
- Process 列在用户提供的 Clash Verge 截图可见行中为空；“当前 Clash 不存在 Unknown”对 Host/Chains 更有支持，对 Process 没有同等证据。
- 代码路径允许 health-only 断连保留 hub rows/overview；是否还有上层生命周期路径在现场断连时主动清空，需要实现阶段补充 runtime trace。研究阶段只将其列为 freshness 风险，不把它当作已复现故障。
- `ref/neko-master` 没有可借用的 Process 归因实现，也没有 metadata 后到的测试。它减少可见空值的部分行为是过滤/默认 `DIRECT`/`Match`，不能当作字段完整性的证据。
- 旧 Host NULL、已经物化为 dimension id 0 的历史桶以及缺少 session 证据的旧 Unknown 都不能依据当前活动连接无损恢复；任何重建必须限定在 SQL 可证明的确定性修复（例如单跳 `DIRECT`）。
- 真实 controller、安装态 WebView、断连/重连以及 24h 窗口滚动后的人工验收尚未执行，均保持 **UNVERIFIED**。
