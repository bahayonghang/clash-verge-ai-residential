# Research: neko-master attribution comparison

- Query: 深入审计 `ref/neko-master` 的 Mihomo/Clash 连接采集、Host 选择、metadata 解析、代理链/规则/进程处理、连接消失与统计持久化/回退，并与 `residential-monitor` 的归因链路逐项对照，判断哪些机制可采用、需调整或不可采用。
- Scope: internal
- Date: 2026-08-21

## Findings

### 1. 结论摘要

`neko-master` 值得采用的核心不是某个 `Unknown` 文案，而是“按稳定连接 id 保存累计计数基线，并让后续字节增量复用该连接的元数据”这一数据流。不过它当前实现把**首帧元数据永久冻结**，不能接收后到字段；直接模式又只用裸 `connection.id`、在计数器回退时继续复用旧元数据，因此不能原样移植。

本项目应采用“`epoch:id` 范围内的可信元数据单调合并”：非空 Host/Process/Rule/Chain 可以补全，后续暂时缺失不能清空；连接消失或 epoch 变化后不可继续复用。现有 Host 已实现 `host -> sniffHost -> destinationIP` 和“域名不被 IP 降级”的较强规则，但 Process/Rule/Chain 在 SQLite session attr UPSERT 中仍会被空值覆盖，这是参考机制能够帮助定位、但需要按本项目契约改造的关键点。

不能把 `neko-master` 看起来较少出现 Unknown 当成归因完整性的证据。它至少通过三种方式减少可见未知：空域名不进入 domain 排行、空链被强制解释为 `DIRECT`、空 rule 被解释为 `Match`。这些是过滤或默认分类，不是从控制器获得了更多事实。

### 2. 文件定位

#### 参考树 `ref/neko-master`

- `ref/neko-master/packages/shared/src/index.ts:3-48`：Clash/Mihomo 连接与 metadata 的 TypeScript 接口，声明了 `host`、`sniffHost`、`process`、`processPath`、`chains`、`rule` 等字段。
- `ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts:72-202`：直接模式 `/connections` WebSocket、心跳、重连。
- `ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts:204-694`：直接模式连接基线、字段提取、字节差分、连接消失、批量刷新与 SQLite 回退。
- `ref/neko-master/apps/collector/src/modules/collector/batch-buffer.ts:16-28`：持久化前的 traffic update 契约；没有 process 字段。
- `ref/neko-master/apps/collector/src/modules/collector/batch-buffer.ts:89-248`：按分钟与维度合并，调用唯一批量写路径；ClickHouse 失败时由调用方执行 SQLite 回退。
- `ref/neko-master/apps/collector/src/database/repositories/traffic-writer.repository.ts:45-369`：domain/IP/chain/rule/minute/hour 等维度的聚合键与缺失值处理。
- `ref/neko-master/apps/collector/src/database/repositories/traffic-writer.repository.ts:371-564`：SQLite 聚合表 UPSERT；没有连接生命周期表，也没有 process 维度。
- `ref/neko-master/apps/collector/src/shared/utils/rule-name.ts:1-29`：多跳链的规则聚合键契约。
- `ref/neko-master/apps/collector/src/database/repositories/domain.repository.ts:53-178`：domain 排行查询明确排除空 domain。
- `ref/neko-master/apps/collector/src/database/repositories/proxy.repository.ts:15-34`、`ref/neko-master/apps/collector/src/database/repositories/base.repository.ts:384-412`：先保存全链，查询时按第一跳汇总 proxy。
- `ref/neko-master/apps/agent/internal/gateway/client.go:39-60`、`ref/neko-master/apps/agent/internal/gateway/client.go:170-221`：Agent 模式实际通过 HTTP GET `/connections` 拉取 Clash，而不是 WebSocket。
- `ref/neko-master/apps/agent/internal/agent/runner.go:26-37`、`ref/neko-master/apps/agent/internal/agent/runner.go:448-558`：Agent 的 per-flow 基线、冻结元数据、差分、短时消失保留和队列溢出处理。
- `ref/neko-master/apps/collector/src/modules/app/app.ts:528-562`、`ref/neko-master/apps/collector/src/modules/app/app.ts:681-878`：Agent 上报清洗、请求去重、BatchBuffer 与实时层写入。

#### 当前产品 `residential-monitor`

- `residential-monitor/src-tauri/src/controller.rs:8-33`、`residential-monitor/src-tauri/src/controller.rs:94-187`：原始连接字段模型及 JSON 规范化。
- `residential-monitor/src-tauri/src/session_host.rs:14-45`：Host 选择与同连接 Host 质量升级规则。
- `residential-monitor/src-tauri/src/accounting.rs:36-53`、`residential-monitor/src-tauri/src/accounting.rs:188-283`：当前连接基线只保存累计计数，不保存归因元数据。
- `residential-monitor/src-tauri/src/accounting.rs:77-116`：每帧直接从当帧 `ConnectionFact` 生成 live metadata。
- `residential-monitor/src-tauri/src/c2/facade.rs:594-629`：先生成当前帧 live rows，再计算差分，并把两者一起提交。
- `residential-monitor/src-tauri/src/storage.rs:538-567`：live rows 更新 session metadata，minute facts 只保存 session 与字节。
- `residential-monitor/src-tauri/src/storage.rs:570-599`：Host 已用质量感知合并，而不是无条件覆盖。
- `residential-monitor/src-tauri/src/storage.rs:639-678`：Process/Rule/Network/Chain 当前采用整行覆盖式 UPSERT。
- `residential-monitor/src-tauri/src/c3/schema.rs:32-43`：每个 session 只有一行可变的归因属性。
- `residential-monitor/src-tauri/src/c3/sql.rs:41-101`：raw 排名通过 session attr 连接 minute facts，缺失值显式汇总到 `__unknown__`。
- `residential-monitor/src-tauri/src/c3/retention.rs:218-300`：小时物化也从 session attr 读取 Host/Process/Chain/Rule；物化后的 dimension id 为历史事实。
- `residential-monitor/src-tauri/src/c3/rule_name.rs:10-59`：本项目已经采用了与 neko 相同的“多跳取最后一跳作为规则组”的机制，但 Chain 排名仍保持独立的 Unknown 语义。

### 3. 连接采集与稳定标识

#### neko 直接模式

- WebSocket 收到的 JSON 被直接断言为 `ConnectionsData`，没有逐字段 runtime decoder；`onData` 只检查 `data.connections` 是否为数组，并跳过无 `id` 项（`gateway.collector.ts:98-109`, `gateway.collector.ts:390-433`）。
- 活动状态用 `Map<conn.id, TrackedConnection>` 保存（`gateway.collector.ts:204-230`）。WebSocket 重连不会主动清空该 map；只有新帧中缺失的 id 才删除（`gateway.collector.ts:601-607`）。
- 计数器回退被解释为新流量，当前累计值会再次写入，同时 `counted=false`（`gateway.collector.ts:517-530`）。元数据却仍沿用旧 `TrackedConnection`，因为增量写入读取 `existing.domain/ip/chains/rule`（`gateway.collector.ts:547-575`）。

结论：**不可原样采用**。本项目已经用 `epoch:id` 作为 session key（`accounting.rs:131-135`, `accounting.rs:204-213`），这比裸 id 更能防止 core 重启或 id 复用串值。对计数器回退，本项目选择更新基线但不发明该段差分（`accounting.rs:220-226`），也符合“无法安全重建则保持未知/缺口”的产品口径。不能为了减少 Unknown 改成 neko 的“把当前累计值当新流量”。

#### neko Agent 模式

- Agent 实际用 HTTP GET `/connections` 拉取并解析 Host/SniffHost/DestinationIP/SourceIP/Chains/Rule（`apps/agent/internal/gateway/client.go:46-60`, `apps/agent/internal/gateway/client.go:170-218`）。本地文档却称 Clash Agent 用 WebSocket（`docs/agent/overview.md:27-28`），代码与文档不一致时应以代码为准。
- Agent 的 `trackedFlow` 同样以裸 id 为键并保存首帧 metadata（`runner.go:26-37`, `runner.go:458-478`）。连接从一次拉取中消失后不会立即删除，只有超过 `StaleFlowTimeout` 才删除（`runner.go:539-545`）。

结论：短时保留 baseline 可防一次轮询漏项导致全累计重放，但与本项目明确的连接关闭语义、epoch 边界和不发明尾流量要求不同，属于**需证据后再调整采用**，不应成为本次 Unknown 修复的前置方案。若未来验证 Mihomo `/connections` 存在瞬时漏项，可在 `epoch:id` 内设计有界 tombstone；在没有真实载荷证据前，保持连接消失即结束更稳妥。

### 4. Host 选择与缺失值

#### neko 行为

- 直接模式 Host/domain 只取 `metadata.host || metadata.sniffHost || ""`，目的 IP 单独放在 `ip`，不作为 domain 回退（`gateway.collector.ts:436-443`）。Agent 也只用 Host 后退到 SniffHost（`apps/agent/internal/gateway/client.go:196-218`）。
- 首帧将 domain 固化进 `TrackedConnection`（`gateway.collector.ts:447-464`），后续即使控制器出现更优 Host，也不会更新；增量总用首帧值（`gateway.collector.ts:547-575`）。Agent 代码更明确地注释并执行“first seen 后保持稳定”（`runner.go:469-478`）。
- 空 domain 不写 `domain_stats`（`traffic-writer.repository.ts:96-107`），range domain 查询还显式使用 `domain != ''`（`domain.repository.ts:53-72`, `domain.repository.ts:143-160`）。因此 Top Domains 不显示空值，不代表这些字节已被 Host 归因；同一字节仍进入 minute/hour 总量和 IP/chain/rule 统计。

#### 当前项目对照

- 当前 Host 解析已更完整：`host -> sniffHost -> destinationIP`（`session_host.rs:18-27`），live projection 确实调用该单一实现（`accounting.rs:96-100`）。
- SQLite Host 更新可从空值或 IP 升级为域名，也避免用 IP 覆盖已有域名（`session_host.rs:29-45`, `storage.rs:577-598`）。这已优于 neko 的“首帧永久冻结”。
- raw Host 排名显式把空值归到 `__unknown__`，没有隐藏这部分流量（`c3/sql.rs:41-54`）。

结论：neko 的 Host 字段优先级方向**已采用且当前实现更强**；“空 domain 不进榜”**不可采用**，否则只是隐藏 Unknown 并破坏总量可解释性。真正需要保留的是本项目现有质量感知 Host 合并，并把同样的“非空不被后续空值清除”原则扩展到其他维度。

### 5. Chain、代理与 Rule 不是同一个维度

#### neko 数据契约

- 直接模式只有当 `conn.chains` 不是数组时才回落到 `["DIRECT"]`；合法的空数组仍保持空数组（`gateway.collector.ts:436-443`）。Agent 则把 nil、空数组或全空元素一律规范化为 `["DIRECT"]`（`apps/agent/internal/gateway/client.go:305-323`）。两条采集路径并不完全一致。
- traffic writer 把完整链保存成 `chains.join(' > ') || chain || 'DIRECT'`，并把 `chains[0]` 当 final proxy（`traffic-writer.repository.ts:87-94`）。Proxy 查询再按完整链第一跳汇总（`proxy.repository.ts:15-34`, `base.repository.ts:384-412`）。
- Rule 聚合采用另一套语义：多跳取最后一跳（顶层策略组），单跳才使用 `rule(payload)`，全部为空回落 `DIRECT`（`shared/utils/rule-name.ts:7-29`）。测试明确覆盖 Mihomo 多跳链应按顶层策略组而非 raw rule 类型统计（`traffic-writer.test.ts:271-319`）。

#### 当前项目对照

- 本项目已把整条链写入 `connection_chain`/`chain_key`（`storage.rs:539-548`, `storage.rs:650-654`），并用最后一跳作为 Chain 排名与多跳 Rule group（`c3/sql.rs:75-101`, `c3/rule_name.rs:10-29`）。
- 这与 neko 的“Top Proxies 按第一跳最终节点”并非同一产品概念。不能把 `chains[0]` 直接替换当前 Top Chains 的最后一跳，否则会悄然改变页面语义。
- 当前空 Chain 显式落入 `__unknown__`（`c3/sql.rs:89-101`）。把缺失链强制解释为 `DIRECT` 会把“控制器没给链”与“控制器确认直连”合并，违反本任务 R3/R5。

结论：完整链保留、代理最终节点与顶层策略组分离的思想**可采用/现已部分采用**；neko 的 `DIRECT` 和 `Match` 缺省值对本项目的 Unknown 消除**不可采用**。优化方案必须先过滤空链元素，然后仅在控制器明确给出非空链时更新 canonical chain；空链保留 Unknown。Rule 可继续用已采用的 `build_rule_name` 语义，但 raw `rule` 缺失不能仅靠默认 `Match` 伪装成已知。

### 6. Process：参考树没有可借用的归因实现

- shared 接口声明 `metadata.process` 和 `processPath`（`packages/shared/src/index.ts:19-29`），但 `TrackedConnection` 没有 process 字段（`gateway.collector.ts:204-219`）。
- direct collector 提取 metadata 时完全没有读取 process（`gateway.collector.ts:436-464`）。
- `TrafficUpdate`、BatchBuffer key、SQLite writer schema/聚合 map 也没有 process（`batch-buffer.ts:16-28`, `traffic-writer.repository.ts:11-23`, `traffic-writer.repository.ts:51-73`）。Agent 的 Clash 响应结构和 `FlowSnapshot` 同样没有 process（`apps/agent/internal/gateway/client.go:46-60`, `apps/agent/internal/domain/types.go:17-28`）。

结论：对 Process 维度，neko **不可采用/没有实现**。本项目已经从 `metadata.process`/`processPath` 解码并投影（`controller.rs:137-160`, `accounting.rs:101-105`），需要修的是同连接生命周期内的保留与持久化，而不是从 neko 复制字段解析。真实 Mihomo 在某些 TUN/平台连接上可能确实不给 process；这种真实缺失必须继续显示 Unknown，不能用可执行文件名猜测，也不能从其他连接串值。

### 7. 元数据后到、暂时缺失与当前覆盖缺陷

#### neko 能防什么

首帧有值、后帧暂时缺失时，neko 的增量仍使用 `existing.*`，因此不会退化为 Unknown（`gateway.collector.ts:513-575`；Agent 同理见 `runner.go:469-478`, `runner.go:524-536`）。这是值得借鉴的性质。

#### neko 不能防什么

首帧为空、后帧补全时，neko 仍坚持首帧空值，后续所有增量继续无 Host/Chain/Rule。它没有“可信元数据后到”的测试或合并函数。计数器 reset/id reuse 时还会把旧连接 metadata 带入新累计值。

#### 当前项目的具体风险

- `AccountingEngine::SessionAcc` 只存 last counters/timestamps，不存 metadata（`accounting.rs:36-53`）。`project_live` 每次直接使用当帧字段（`accounting.rs:77-116`），所以后帧临时缺失会进入 live rows。
- `AppFacade` 先由当前帧生成 live rows，然后让 engine 计算差分，并把当前 rows 与 minute facts 同批提交（`c2/facade.rs:594-629`, `c2/facade.rs:701-711`）。
- `intern_and_attr` 的 UPSERT 对 `process_id`、`rule_id`、`network_id`、`chain_key` 全部写 `excluded.*`，当本帧缺失时会把已有非空值覆盖为 NULL（`storage.rs:639-678`）。Host 因为走 `prefer_host_identity` 不受这一具体问题影响。
- raw bytes 本身只存在 `connection_minute(session_pk, upload, download)`，查询时再 join 当前唯一 attr 行（`storage.rs:550-559`, `c3/sql.rs:56-73`）。因此一次空帧不仅影响新差分，还能让同一 session 之前的所有 raw 流量在 Process/Chain 排行中变成 Unknown。
- 小时物化同样 join 当前 attr，并以 NULL -> dimension id 0 固化（`c3/retention.rs:218-300`）。一旦物化，后续 metadata 到达也不能无损改写已经固化的历史桶。

结论：应**调整后采用** neko 的“连接内复用”而非“首帧冻结”。推荐 canonical metadata 合并规则：

1. key 必须是 `epoch:id`，连接移除或 epoch 变化即终止，禁止跨连接复用。
2. 每帧先做字符串 trim/空数组过滤，再与该 session canonical metadata 合并，之后 live rows、差分事实与持久化都使用同一 canonical 结果。
3. Host 继续使用现有 `resolve_host_identity` + `prefer_host_identity` 的质量顺序。
4. Process name/path、Rule/payload、Network、Chain：incoming 非空可填充或更新，incoming 为空只表示“本帧未提供”，不得清除已有值；Chain 必须整组更新，不能逐 hop 拼接不同帧。
5. SQLite UPSERT 再做一次防御性非空合并，避免未来调用方绕过 engine 后又把 attr 清空。
6. 同一 session 内后到 metadata 回填该 session 先前 raw minute bytes 是可证明的同连接归因；已经物化的旧桶或无 session 证据的旧 Unknown 不自动重写。

### 8. 首帧、连接关闭与“尾流量”

- neko direct 把新连接首帧已有累计值立即当作本次流量写入（`gateway.collector.ts:447-481`）；Agent 也以当前累计值作为无 previous 时的 delta（`runner.go:480-495`, `runner.go:524-536`）。
- 当前项目把第一帧仅当 baseline，不生成 facts（`accounting.rs:207-219`），测试明确断言 first frame unknown then delta（`accounting.rs:392-412`）。这避免把 monitor 启动前的累计值塞进当前采样时段。
- neko direct 在连接从列表消失时只删除状态，并注释“剩余流量已经计数”，不推测最终尾差（`gateway.collector.ts:601-607`）。当前项目同样不发明消失尾流量，并有回归测试（`accounting.rs:415-421`）。

结论：首帧累计值策略**不可采用**，连接消失不造尾流量的策略**保留现状**。Unknown 优化不能通过把首帧历史累计强行分摊到当前 minute 完成。metadata canonical cache 应在当前帧 delta 计算前使用，但仍只对相邻可比样本的差分负责。

### 9. 持久化、实时层与失败回退

#### neko 的持久化形态

- 每个可观测 delta 在写前带上 domain/ip/full chain/rule/sourceIP，然后按 minute/hour 直接物化为维度事实（`traffic-writer.repository.ts:234-274`）。后续 connection metadata 变化不会反向改写已落库 delta。
- 代价是首帧空 metadata 的历史不会在后到 metadata 时获得补全；而本项目的“同 session 后到字段可回填”要求更适合 session-level canonical metadata。
- BatchBuffer 以 minute + 全部维度为 key 合并（`batch-buffer.ts:95-123`），统一调用 `batchUpdateTrafficStats`（`batch-buffer.ts:142-187`）。这解决吞吐和写放大，不负责可信 metadata fallback。
- ClickHouse 失败且 SQLite 被跳过时，direct collector 明确把同一 updates snapshot 写回 SQLite（`gateway.collector.ts:290-311`）。成功后才按 detail/agg 状态清 realtime store（`gateway.collector.ts:314-323`）。Agent ingest 也经同一 BatchBuffer，requestId 防重复（`app.ts:702-739`, `app.ts:789-823`）。

#### 当前项目对照

- 当前事务把 bundle receipt、minute facts、session attrs 与 alerts 一起提交，失败 rollback（`storage.rs:321-427`）；已经有自己的原子和幂等边界，不需要引入 neko 的 BatchBuffer/ClickHouse 双写架构。
- Unknown 的主要持久化问题不是“写失败”，而是单行 session attr 的空值覆盖和 materialization 时间边界。

结论：neko 的“同一 updates snapshot 失败回退、成功后清 realtime”是良好的可靠性模式，但对本任务只属于**概念可采用、实现不可搬用**。不要把 durability fallback 与 attribution fallback 混为一谈，也不要为解决 Unknown 引入 ClickHouse、额外 writer 或新的聚合表。

### 10. 逐项采用矩阵

| 机制 | neko 行为 | 当前项目状态 | 结论 |
| --- | --- | --- | --- |
| Clash/Mihomo 采集 | direct WebSocket；Agent 实际 HTTP GET | 约 1 Hz HTTP GET `/connections` | **不迁移传输层**；Unknown 与 WS/HTTP 本身无直接因果证据 |
| 稳定连接 key | 裸 `connection.id` | `epoch:id` | **保留当前**；不可退化 |
| 累计计数 baseline | per-id map | per-`epoch:id` map | **采用方向/当前已有** |
| Host 选择 | `host -> sniffHost` | `host -> sniffHost -> destinationIP` | **当前更强** |
| metadata 暂时缺失 | 复用首帧值 | Host 可保留；Process/Rule/Chain 可被清空 | **调整后采用连接内 canonical merge** |
| metadata 后到 | 永不吸收 | Host 能升级；其他维度无统一合并 | **不能照搬首帧冻结** |
| 空 Host | domain 排行过滤掉 | 显式 `__unknown__` | **不可采用隐藏策略** |
| 空 Chain | direct 路径不一致；Agent 强制 DIRECT | 显式 Unknown | **不可把缺失伪装 DIRECT** |
| Rule key | 多跳最后一跳；单跳 `rule(payload)` | 已部分采用同契约 | **保留并统一测试** |
| Proxy/Chain label | proxy 按第一跳最终节点 | chain 按最后一跳策略组 | **概念不同，不互换** |
| Process | 接口声明但采集/存储未实现 | 已解码、投影、存储 | **无可借用实现** |
| 新连接首帧 | 计入全累计值 | 只建 baseline | **不可采用** |
| 连接消失 | direct 立即删除，不造尾差 | 立即结束，不造尾差 | **保留当前** |
| counter reset | 当前值当新流量，可能复用旧 metadata | 更新 baseline、不造差分 | **不可采用** |
| 维度持久化 | 每个 delta 直接带维度物化 | raw minute + 可变 session attr | **保留当前 schema，修 canonical/UPSERT；旧桶不伪回填** |
| 写失败回退 | CH 失败回写 SQLite | 单 SQLite 事务/receipt | **只借鉴原则，不引入架构** |

### 11. 对规划文档的直接输入

建议 `design.md` 把 Unknown 优化拆成以下可验证边界：

1. **解析层**：继续保留 `host/sniffHost/destinationIP/process/processPath/rule/rulePayload/chains` 原始可选性；trim 后空字符串仍是缺失，不能默认成成功分类。
2. **连接 canonical 层**：将 metadata 并入 `AccountingEngine` 的 `SessionAcc` 或等价单一状态机；确保 project/live、delta 与 storage 使用同一份合并结果，避免目前 project 与 apply 的两条平行路径。
3. **持久化层**：Host 使用现有质量升级；其余 attr 使用“non-empty incoming wins, missing incoming preserves”而非无条件 `excluded.*`。若 incoming 与已有都非空且不同，记录/测试确定性更新，不跨 session。
4. **查询层**：继续保留 `__unknown__` 桶和总量守恒。Unknown 减少只能来自 canonical metadata 变为已知；不得加 `WHERE value != ''` 或把 NULL 变成 DIRECT/Match。
5. **历史边界**：只对当前 schema 中同一 `session_pk` 有证据的 raw facts使用后到 metadata；旧库中已经物化到 id 0 的小时/日桶及无法定位 session 的历史 Unknown 保持不变并说明限制。
6. **测试层**：Host、Process、Chain 分别覆盖 `缺失 -> 非空 -> 暂时缺失 -> 连接关闭`；再覆盖同 id 跨 epoch、counter reset、第一帧已有累计值、真实全程缺失、旧 DB id 0 读取。断言每个时刻的总字节守恒，且不会通过隐藏 Unknown 达标。
7. **真实验收**：同时保存一段 Mihomo `/connections` 原始 JSON（脱敏）和 monitor 新写入 session/attr/rank 结果；Clash Verge 活动连接页只能作为人工对照，不能替代原始 payload 与历史 DB 证据。

## External References

- 未进行外部网络检索；本结论只基于仓库内只读参考快照和当前产品代码。
- 参考快照自报 `neko-master` 版本 `1.4.0`（`ref/neko-master/package.json:3`），collector 包自报 `1.0.0`，依赖 `ws ^8.21.0` 与 `better-sqlite3 ^12.10.0`（`ref/neko-master/apps/collector/package.json:3`, `ref/neko-master/apps/collector/package.json:19-23`）；Agent 使用 Go 1.22（`ref/neko-master/apps/agent/go.mod:3`）。这些版本信息只说明审计快照环境，不证明 Mihomo API 的跨版本字段稳定性。

## Related Specs

- `.trellis/spec/residential-monitor/backend/index.md`：要求 `session_host::resolve_host_identity` 为 Host 单一实现，collector 约 1 Hz GET `/connections`，C2 不直接访问 SQLite。
- `.trellis/spec/residential-monitor/backend/modules-and-errors.md`：明确 `host -> sniffHost -> destination IP`、Host 质量升级、暂停/断线和 snapshot 查询契约。
- `.trellis/spec/residential-monitor/storage/sqlite-contract.md`：SQLite 单 writer、事务与兼容边界。
- `.trellis/spec/residential-monitor/frontend/dto-and-decoding.md`：`null`/unknown 不能被前端解码成 0 或成功态。
- `.trellis/spec/residential-monitor/frontend/view-state.md`：无样本、断开、暂停、缺口与数据 Unknown 应保持不同状态。
- `.trellis/tasks/08-21-eliminate-resolvable-unknown-attribution/prd.md`：R1-R7、AC1-AC7，尤其同连接内可信 metadata 后到不退化、旧 Unknown 不伪回填、真实控制器验收保持 `UNVERIFIED`。

## Caveats / Not Found

- `ref/neko-master` 没有 direct collector 的字段后到/暂时缺失/连接关闭归因单测；现有 `gateway.collector.test.ts` 只覆盖 WebSocket heartbeat。Agent runner 测试覆盖差分与 counter reset，但未覆盖 metadata 后到。
- 参考树 shared 类型声明 process，不代表 collector 真正处理 process；全树检索没有找到 `metadata.process` 进入 TrafficUpdate、writer 或查询的路径。
- 参考文档声称 Clash Agent 使用 WebSocket，但实际 Go client 使用 HTTP GET `/connections`；不能仅凭 README/architecture 图设计当前产品。
- 截图只能证明某一时刻 Clash Verge 能展示部分活动连接字段。截图中的 Process 列亦存在空白；它不能证明历史窗口内每条连接都提供 Host/Chain/Process，也不能证明旧数据库 Unknown 可无损回填。
- 尚未取得真实 Mihomo `/connections` 脱敏原始载荷、当前 `monitor.sqlite3` 中 Unknown session 的字段分布、core restart/id reuse 样本。因此“现场 Unknown 中有多少可由 canonical merge 消除”仍为 **UNVERIFIED**。
- 未发现可以安全推导旧 Unknown 的外部事实表。对旧的 dimension id 0、被过滤掉的空 domain 或已物化 bucket，不应依据当前活动连接、同名进程、IP/域名相似性或现有流量占比进行回填/分摊。
