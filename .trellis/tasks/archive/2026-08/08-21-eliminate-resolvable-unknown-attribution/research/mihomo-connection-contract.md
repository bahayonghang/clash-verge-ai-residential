# Research: Mihomo/Clash 连接契约、生命周期与元数据继承边界

- Query: 核验仓库内 Mihomo/Clash `/connections` 契约、controller transport、fixtures、连接 ID 与关闭语义；对照 `ref/neko-master`，给出“同连接元数据暂缺时继承”和“旧历史能否回填”的约束与测试矩阵。
- Scope: mixed（当前仓库代码/测试/规格、只读 `ref/neko-master`、仓库内固定提交的一手来源审计；本轮未访问外网）
- Date: 2026-08-21

## Findings

### 结论摘要

1. **`Unknown` 不是一个单一问题。** 当前链路至少有四种不同来源：控制器未连接或没有样本、控制器对该连接从未提供字段、同一活动连接某一帧暂时缺字段但上一帧有、以及旧数据在写入时已经丢失关联。只有第三类以及部分仍保留旁证的旧数据可以安全消除。
2. **Mihomo 的 `id` 只应视为“一个完整快照序列内、持续出现期间”的相关键，不是跨核心重启或跨关闭再出现的永久主键。** 官方固定源码审计确认快照数组无排序保证、以 `id` 合并；关闭后连接从后续完整快照消失；公开契约未承诺 ID 跨重启稳定（`.trellis/tasks/archive/2026-08/08-18-residential-monitor-mvp/research/controller-compatibility-audit.md:40-64,356-360`）。
3. **当前 monitor 对元数据是无状态投影、对数据库属性是 last-frame overwrite。** `project_live` 每帧直接复制可选字段（`accounting.rs:77-115`）；`intern_and_attr` 的 UPSERT 会用本帧的 NULL 覆盖已有 `process_id`/`rule_id`/`network_id`/`chain_key`/`host_id`（`storage.rs:639-677`）。因此，同一连接中间或最后一个可见快照的暂缺字段会把可解释流量重新压回 `Unknown`。
4. **`ref/neko-master` 提供了可借鉴的 active-map 机制，但不能原样复制。** 它以 `Map<id, TrackedConnection>` 计算增量、复用首次保存的元数据、连接从快照消失后删除状态（`gateway.collector.ts:424-464,513-607`）。优点是后续空字段不会降级；缺点是首次为空、后续变丰富时永远无法升级。适合本项目的是“连续生命周期内按字段合并：空值继承、可信非空值升级”，不是“冻结首帧”。
5. **旧历史只能证据驱动回填，不能用当前活动连接或 controller meter 分摊。** 已关闭连接不会再出现在 `/connections`，采样尾差本来就不可恢复（固定审计 `controller-compatibility-audit.md:50-64`）。现有库中：host 可从 `connection_session.host` 修复对应 `host_id`；chain 在 `connection_chain` 仍完整时可重建 `chain_key`；process/rule 若关联 ID 已为 NULL，数据库没有足够映射；`processPath` 根本未落库。已冻结报告 JSON 和已经物化的小时/日桶也不会因 raw 属性更新自动可靠重写。
6. **用户的 Clash 截图不能证明 Process 可回填。** 截图中 Process 列的可见单元格为空；此前同日本机库审计也记录已知主机和未知主机的 `process_id` 几乎/全部为空（`unknown-host-24h.md:28-30`）。Host、Chain 与 Process 必须分别验收，不能以“Clash 当前连接页看起来没有 Unknown”推断所有历史维度均有源字段。

### 数据流与故障点

```text
HTTP GET /connections（约 1 Hz，完整活动快照）
  -> serde_json::Value
  -> controller::normalize_snapshot / normalize_connection
  -> ConnectionFact（每帧可选元数据）
  -> AccountingEngine::project_live（无状态）
  -> LiveConnectionView
  -> StorageCoordinator::persist_live_facts
  -> connection_session + connection_session_attr + connection_chain
  -> raw / hourly / daily 查询
  -> __unknown__
```

关键边界：

- 产品采集不是 WebSocket，而是约 1 Hz 的 HTTP `GET /connections`；`fetch_snapshot` 每次走 `ControllerSession::fetch_normalized_snapshot`（`.trellis/spec/residential-monitor/backend/modules-and-errors.md:14`；`c2/collector.rs:78-83`）。
- 传输层为每次请求新建 HTTP/1.1 连接，按需加 `Authorization: Bearer`，收集整个 body；未校验 `Content-Type`（`transport.rs:101-175`）。
- 解析成功后，当前帧被当作**完整活动集合**：hub 用新集合替换旧集合并产生 remove（`c2/hub.rs:276-335`），facade 也按缺失 identity 完成 close confirmation（`c2/facade.rs:594-629`）。
- metadata 在进入 `project_live` 前没有 active-connection cache；同一 `id` 上一帧的字段不会被带入下一帧（`accounting.rs:77-115`）。
- raw 主机排名直接读 `connection_session.host`；进程读 `connection_session_attr.process_id`；链路读 `chain_key`。缺值分别成为 `__unknown__`（`c3/sql.rs:41-100`）。

### 当前 `/connections` 解码契约

#### 帧级字段

| JSON 字段 | 固定 Mihomo 审计 | 当前 monitor 行为 | 风险/建议 |
|---|---|---|---|
| `downloadTotal` | 公开响应字段 | 缺失、类型不对或负数时静默变 `0`（`controller.rs:103-105,171-175`） | 对“完整快照”应区分缺失与真实 0；否则可能伪造 meter reset。 |
| `uploadTotal` | 同上 | 同上 | 同上。 |
| `connections` | 活动连接完整数组；GET 一次快照、WS 首帧立即推送后按 interval 推送 | 缺失或非数组时仍返回成功的空连接集合（`controller.rs:106-120`） | 这是高风险兼容点：畸形/截断对象会被解释成“全部关闭”。应对生产快照要求数组存在；未知附加字段仍宽松忽略。 |
| `memory` | 固定审计确认存在 | 完全忽略 | 与 Unknown 归因无关，可继续忽略。 |
| 未知新增字段 | 可随版本新增 | 忽略，且有回归测试（`controller.rs:202-221`） | 正确的向前兼容策略。 |

#### 连接级字段

| JSON 字段 | 当前类型与可选性 | 当前落点 | 约束 |
|---|---|---|---|
| `id` | 非空 JSON string 才接收；未截断（`controller.rs:123-132`） | `ConnectionFact.id`，live identity 为 `epoch:id` | 仅在连续存在期间稳定。用于 DELETE 的校验另限 1–128 个 ASCII 字母数字/`-_.`（`transport.rs:116-135`），存在“能采集但不能关闭”的契约不对称。 |
| `upload`, `download` | 仅非负 JSON integer；缺失/其它类型为 0（`controller.rs:133-134,171-175`） | 逐连接累计计数器 | 应保留 u64 非负整数约束；计数器下降不是普通 delta，而是 reset/reuse 信号。 |
| `metadata` | 缺失、NULL、非 object 均变空 map（`controller.rs:126-130`） | 见下表 | 允许字段暂缺，但必须在 active merge 层处理，不能直接覆盖已知属性。 |
| `start` | 可选非空 string | live duration，未落库为上游 start | 可参与判断同 ID 是否已换连接；当前未用于 identity/reuse 检测。 |
| `chains` | 可选 string array；缺失/类型错误为空数组 | live、分类、`chain_key`、`connection_chain` | 空数组不能无证据改成 `DIRECT`。官方只承诺数组，不承诺顺序语义（固定审计 `controller-compatibility-audit.md:169-189`）。 |
| `providerChains` | 可选 string array | 解析到 `ConnectionFact.provider_chains`，之后无人消费（`controller.rs:30-31,135-136`；全仓搜索仅定义/fixture） | 不能在未确认语义前把它冒充 `chains`；先作为诊断/兼容字段。 |
| `rule`, `rulePayload` | 可选非空 string | live；只持久化 `rule` 字典 ID | `rulePayload` 未落库，旧数据无法恢复。 |

所有解码字符串最多 4096 个字符，frame 最多 8 MiB（`c0_contract.rs:13-14`；`controller.rs:87-101`）。`truncate_string` 只拒绝精确空串，不 trim；仅空白字符串仍会进入除 host identity 之外的字段，可能形成“看似已知”的空白维度（`controller.rs:87-91`）。

#### `metadata` 字段

当前 monitor 读取以下大小写敏感字段（`controller.rs:137-159`）：

| JSON 字段 | Rust 字段 | 当前使用/持久化 |
|---|---|---|
| `host` | `host` | 与 `sniffHost`/`destinationIP` 合成 host identity；写 `connection_session.host` 与 attr host ID。 |
| `sniffHost` | `sniff_host` | host identity 第二优先级。 |
| `destinationIP` | `destination_ip` | host identity 第三优先级；live destination。没有独立历史 IP 列。 |
| `sourceIP` | `source_ip` | live only。 |
| `sourcePort`, `destinationPort` | string | live only；数值 JSON 不兼容。固定本机样本/共享类型均为 string。 |
| `process` | `process_name` | live；只把该值 intern 为历史 process 维度。 |
| `processPath` | `process_path` | live only；没有历史列（`accounting.rs:103-104`，`storage.rs:645-648`）。 |
| `network` | `network` | live + attr network ID。 |
| `type` | `inbound` | live only。 |

`ref/neko-master` 的共享模型还声明 `sourceGeoIP`、`destinationGeoIP`、ASN、inbound name/user、DNS mode、uid、special proxy/rules、remote destination、DSCP 等字段（`ref/neko-master/packages/shared/src/index.ts:3-29`）；这些是参考树的静态接口，不是本任务消除 Host/Chain/Process Unknown 所必需，也不应无范围扩张地引入。

### 连接 ID、增量与关闭事件语义

#### 官方/固定证据

- `/connections` GET 返回 `DefaultManager.Snapshot()`；WS 首帧立即返回快照，再按正 interval 推送（固定一手审计：`controller-compatibility-audit.md:40-56`）。
- 快照来自并发 map，数组顺序不稳定；以 `id` 合并，不能用数组位置判断同一连接（同上 `:52-64`）。
- tracker 关闭时先从 manager 删除；后续快照不再出现。没有逐连接“final counters/closed”事件（同上 `:50-64,356-360`）。
- `DELETE /connections/{id}` 对存在或不存在 ID 都可返回 204；204 只表示请求已处理，真正 closed 需等下一完整快照中 ID 消失（同上 `:87-103`）。

#### 当前 monitor

- Accounting key 是 `epoch:id`。同一 key 第一次只建立基线；第二帧起才计算 delta（`accounting.rs:188-244,392-421`）。这是观测下界语义，不能改成把首帧累计量全记入当前历史。
- 每帧后，未出现的 id 立即从内存 accounting state 删除（`accounting.rs:201-247`）。连接消失不会“补尾差”，测试明确不发明 tail（`accounting.rs:414-421`）。
- 如果计数器下降，当前实现重置 watermark 并**丢弃该帧当前计数**（`accounting.rs:220-225`）。`ref/neko-master` 则把当前值视为 reset 后的新流量并重新计连接（`gateway.collector.ts:517-530`；`CHANGELOG.en.md:91-93`）。两者都是策略选择，但本项目不能在没有 coverage 设计的情况下直接照搬 Neko。
- 存储对 `(epoch_id, connection_id)` 有唯一索引（`c3/schema.rs:28-32`）。同一个原始 ID 在同一 epoch 内消失后再出现，会复用旧 `session_pk`；这会把两条真实连接的 metadata/分钟事实合并。当前 `/version` body 被当作 `core_identity`，而生产 collector 的周期 tick 只取 `/connections`，所以同版本核心重启未必产生新 epoch（`session.rs:69-95`；`c2/collector.rs:78-83`）。

#### 本任务应冻结的 identity 契约

1. `raw_id` 只用于上游相关和 DELETE。
2. 本地 durable identity 应至少包含：controller generation + 该 raw ID 的 continuous-presence generation。完整成功快照中缺失该 ID时结束 presence generation。
3. 请求失败、解析失败、`connections` 缺失/类型错误不是完整成功快照，**不得**结束全部 generation。
4. 同一 raw ID 的计数器下降、`start` 明确变化、controller generation 改变，均应结束旧 generation并清空旧 metadata cache；不得继承到新连接。
5. 暂停、SleepGap 可按现有规格保留 live rows，但恢复后的第一个完整快照才能确认哪些连接仍持续存在；Disconnected/核心重启必须清 cache（`.trellis/spec/residential-monitor/backend/modules-and-errors.md:15`）。

### `ref/neko-master` 对照：采用、调整后采用、不采用

参考树版本：根包 `1.4.0`（`ref/neko-master/package.json:3`），shared 包 `1.0.0`（`ref/neko-master/packages/shared/package.json:3`）。

| 机制 | 证据 | 结论 |
|---|---|---|
| 共享 connection contract 集中定义 camelCase 字段 | `packages/shared/src/index.ts:3-48` | **采用思想**：边界统一解码，不让 storage/UI 各自猜字段。Rust 已基本集中在 `controller.rs`。 |
| `metadata.host || metadata.sniffHost || ""` | `gateway.collector.ts:436-443` | **已调整采用**：本项目更完整地用 host → sniffHost → destinationIP，符合现有规格。 |
| `activeConnections: Map<id, TrackedConnection>` | `gateway.collector.ts:204-230` | **采用**：同一连续连接需要状态，才能计算 delta 与继承字段。 |
| 新连接保存 metadata；旧连接产生 delta 时复用 existing metadata | `gateway.collector.ts:445-560` | **调整后采用**：只对当前帧空值复用；当前帧提供更强/更完整非空字段时应升级，而非永远冻结首帧。 |
| Agent 模式显式“first seen 后保持 metadata stable” | `apps/agent/internal/agent/runner.go:448-514` | **不原样采用**：首次缺字段会永久 Unknown。可用作“非空字段不得被空值降级”的证据，而不是拒绝 late enrichment。 |
| 完整快照 currentIds；缺失即删除 active state | `gateway.collector.ts:413-416,601-607` | **采用**，但仅在结构有效的完整成功帧执行。 |
| 计数器回退视为 restart/ID reuse | `gateway.collector.ts:517-530`; `runner.go:482-494` | **采用信号，不直接采用计量结果**：必须切断 metadata/session identity；该帧字节如何记账需遵守本项目观测下界与 meter 守恒。 |
| 空/全空白 chains 归一化为 `DIRECT` | Agent `gateway/client.go:305-323`，direct collector 只在非数组时给 `['DIRECT']`（`gateway.collector.ts:441`） | **不采用为全局规则**：参考实现内部已有漂移，官方未承诺空数组等于 DIRECT；否则只是把 Unknown 政名为 DIRECT。 |
| `rule || "Match"` | `gateway.collector.ts:441-443` | **不用于历史真值**：缺失 rule 改成 Match 会伪造规则类型。UI 可做占位，但数据库不得写成观察事实。 |
| process/processPath | shared 类型有字段，但 tracked state/traffic update 不保存 process（`packages/shared/src/index.ts:19-23,31-41`; `gateway.collector.ts:204-219`） | **不可依赖 Neko 解决 Process Unknown**。本项目需自己做同连接合并，并可在同一 payload 中用 `processPath` 的文件名作为可解释派生值，但必须保留“派生自 path”的来源。 |

### 同连接元数据继承的约束

建议在 controller normalize 之后、`project_live` 与 accounting/storage 之前建立单一 active metadata owner。它接收结构有效的完整快照，输出 enriched `ConnectionFact`，避免 live 与 storage 出现两套规则。

#### 合并规则

| 字段 | incoming 缺失/空白 | incoming 非空 | 生命周期隔离 |
|---|---|---|---|
| `host`, `sniffHost`, `destinationIP` | 继承同 presence generation 的对应 raw 字段 | 保存 raw 字段；projection 仍统一按 host → sniff → destination IP。域名不被 IP 降级，沿用 `prefer_host_identity` 契约（`session_host.rs:18-44`）。 | ID 消失、restart/reset 后全部清空。 |
| `process` | 继承；若 process 空而同帧/缓存 `processPath` 非空，可派生 basename，但标记来源 | 非空 process 是直接事实，优先于派生 basename | 不跨 generation。 |
| `processPath` | 继承用于 live/派生；历史是否保存另行设计，注意隐私 | 非空更新 | 不跨 generation。 |
| `chains` | 空/缺失继承上一非空数组 | 非空数组作为当前事实；保留原顺序，但分类只做集合匹配 | 计数器回退、start 改变或 ID 消失后不得继承。 |
| `providerChains` | 同上，但仅诊断保存 | 非空更新 | 不得自动替代 chains，除非真实版本 fixture 证明语义。 |
| `rule`, `rulePayload`, `network`, `type`, IP/ports, `start` | 空值继承 | 非空更新 | 不跨 generation。 |

另外，空白字符串和全空白 chain element 应在边界 trim 后当缺失；这比当前只检查 `value.is_empty()` 更准确（`controller.rs:87-91,164-186`）。

#### 存储写入必须“monotonic enrichment”

- 只在 incoming dimension ID 非 NULL 时更新 `process_id`/`rule_id`/`network_id`/`host_id`；空快照不能覆盖已知 ID。当前 UPSERT 无条件覆盖，是主要缺口（`storage.rs:655-666`）。
- `chain_key` 同样只允许非空 incoming 更新；`connection_chain` 需要与最终 chain 策略一致，不能一边 `insert or ignore` 保留首帧节点、一边 attr 保存末帧 chain（`storage.rs:543-548,650-666`）。
- 一旦引入 presence generation，durable session key必须能区分 raw ID reuse；否则 active cache 虽安全，SQLite 唯一索引仍会跨连接串值（`c3/schema.rs:30-32`）。
- 元数据合并不能改变 `upload/download` delta、controller meter 或 coverage；它只改变有来源证据的维度关联。

### 旧历史回填边界

#### 可安全回填

1. **同一尚活跃连接的 earlier minutes**：如果稍后快照提供 metadata，现有 schema 的每 session 单一 attr 会使该 session 的 raw minutes 在查询时一起获得维度。这是 session-level retroactive enrichment，不是把流量分摊给别的连接。必须确认 presence generation 未跨 ID reuse。
2. **Host attr 修复**：`connection_session.host` 非空而 `connection_session_attr.host_id` 为空时，可以 intern 同一个 host 并回填 host_id。raw host 排名本来直接使用 `s.host`（`c3/sql.rs:41-53`）。
3. **Chain attr 修复**：同一 `session_pk` 的 `connection_chain(position,node)` 行完整、无冲突时，可按 position 重建 `chain_key`。表结构保留了该旁证（`storage.rs:165-169`）。
4. **仍保留 raw 的派生层重建**：可以针对明确时间窗事务性删除旧的 hourly/daily rows 后，从 raw 重新物化，并做上传/下载守恒校验。不能只运行当前 `INSERT OR REPLACE`：重分类后新 known key 会新增一行，而旧 dimension_id=0 Unknown 行未必被删除（`c3/retention.rs:218-300`）。

#### 不可安全回填

1. `connection_session.host IS NULL` 且没有同 session 的其它 host/IP 旁证：`destinationIP`/`sniffHost` 未独立存储，不能从现在的连接或域名字典猜。此前实测旧 unknown 会话就是这种情况（`unknown-host-24h.md:39-41`）。
2. `process_id IS NULL`：`processPath` 未落库，`dimension_dict` 里存在某个进程值也不能证明属于该 session。
3. `rule_id IS NULL` 且没有原始 `rule`/`rulePayload`：chain 最后一跳是代理链维度，不等价于原始匹配规则。
4. 已关闭连接在最后采样之后产生的尾部字节：Mihomo 没有 final event，无法恢复。
5. controller meter 与 attributed 之间的 gap：只能保持 gap，不能按当前连接比例分摊。
6. 已冻结 `report_archive.result_json`：成功档案首次写入后 `existing_ok` 阻止覆盖，读取直接反序列化 frozen JSON（`c3/archive.rs:219-241,260-330,473-484`）。若产品决定重建，必须是显式版本化操作，不能被普通查询悄悄改变。
7. raw 已删除或只剩 hourly/daily/core 的区间：没有 session 级证据时无法把 dimension_id=0 拆分成已知维度。当前自动 raw DELETE 虽关闭，但兼容设计不能假设所有用户库永远保留 raw（`.trellis/spec/residential-monitor/storage/sqlite-contract.md:21`）。

### 版本兼容结论

- 当前 monitor 没有 Mihomo semver feature negotiation。`connect_tcp` 只要求 `/version` 2xx，并把整个 body string 当 `core_identity`；没有解析 `version`/`meta` 字段，也没有按版本选择连接 schema（`session.rs:69-95`）。
- TCP loopback 是 supported profile；Clash Verge `2.5.2` 固定 pipe 与动态 sidecar/service pipe 都只是 best effort（`transport.rs:34-60`；`docs/controller.md:3-7`）。Unknown 修复不应绑定私有 pipe 名。
- 仓库内固定审计基线为本机 Clash Verge Rev 2.5.2 + Mihomo v1.19.29、稳定 Mihomo v1.19.30、源码快照 `fe22fdd...`；相关 controller 文件在这些 Mihomo 基线间无行为变更（`controller-compatibility-audit.md:21-36`）。
- 安全兼容策略应是：要求 frame object + `connections` array + 每条非空 string `id`；对新增字段宽松忽略；对 metadata 子字段缺失宽松；对类型错误作可观测兼容诊断，而不是静默制造空快照。端口是否兼容 number、旧 Clash 分支是否缺 `sniffHost`/`processPath`，仓库当前没有版本矩阵 fixture，仍为 UNVERIFIED。

### Fixtures 与现有测试缺口

当前 fixture 覆盖不足以证明真实 Mihomo 元数据不会丢失：

- controller model 只有 unknown-field/order 与 `sniffHost`/destination IP 两个小 fixture；没有 full canonical connection、process/path、rule/payload、ports、providerChains、metadata late-arrival（`controller.rs:197-249`）。
- HTTP fixture `/connections` 永远返回空数组；WS transport 测试也只发送空数组（`transport.rs:226-243,261-280`）。
- collector scripted fixture 仅带 `metadata.host`/`network`，并用完全不同 id 验证两 tick 替换；没有同 id 的元数据合并（`c2/collector.rs:207-240`）。
- storage 只测试 host 的 None → IP → domain → IP 不降级；没有 process/rule/network/chain 防 NULL 覆盖（`storage.rs:1460-1507`）。
- Neko `gateway.collector.test.ts` 只覆盖 heartbeat watchdog，不覆盖 active metadata/state（`ref/neko-master/apps/collector/src/modules/collector/gateway.collector.test.ts:1-98`）。参考实现的元数据行为必须由本项目自己的回归测试锁定。

### 建议测试矩阵

| 层级 | 场景 | 预期硬断言 |
|---|---|---|
| parser | 固定审计形状的 full connection：全部 metadata、start、chains、providerChains、rule/payload | 每个 camelCase 字段进入正确 Rust 字段；未知附加字段不影响。 |
| parser | frame 缺 `connections` / 为 object / null | 返回 ProtocolIncompatible 或明确 incomplete frame；不得发布空集合、不得产生 removes。 |
| parser | connection 无 id、空 id、非 string id | 该 row 被隔离并产生诊断计数；其它合法 row 保留。 |
| parser | u64 最大值、负数、小数、numeric string；port string/number | 固定支持矩阵；类型不支持时不静默当 0 后继续计量。 |
| normalization | 空串、空白串、chains 含空白元素 | trim 后缺失；不产生空白 dimension。 |
| active merge | 同 id：rich → metadata missing，计数增长 | live 和本次 delta 继续归于 rich Host/Chain/Process；DB attr 不变为 NULL。 |
| active merge | 同 id：missing → rich，计数增长/不增长各一次 | rich 字段都能升级；即使该帧 delta=0，session attr 也能更新；earlier raw minutes 只在同 session 内回填。 |
| active merge | 同 id：host IP → sniff domain → host domain | 按 host → sniff → IP 与 domain-over-IP 规则单调升级。 |
| process | `process` 缺、`processPath` 有；随后 process 到达 | 首先可显示/归因可追溯的 basename 派生值，后被直接 process 事实升级；path 不意外进入导出/热点 DTO。 |
| chain | chains rich → empty → rich；providerChains 单独出现 | empty 继承；providerChains 不自动冒充 chains；非空 chains 的分类对数组顺序不敏感。 |
| lifecycle | 合法完整帧中 id 消失 | presence generation 结束，cache 清除，close mark 由 remove 确认。 |
| lifecycle | HTTP/JSON 失败或 incomplete frame | 保留上一状态但标记 coverage gap；不能确认 close/清空 cache。 |
| identity | id 消失后同 raw id 再出现 | 新 durable session/generation；旧 metadata 不继承，SQLite 不复用旧 session_pk。 |
| identity | 同 id 计数器下降、start 改变 | 结束旧 generation，建立新基线；不跨代继承；meter/attributed/gap 按既定观测下界守恒。 |
| restart | 相同 `/version` 文本的核心重启 | 不能靠 version string 假装检测到 restart；由计数器/global meter/reset 或 transport generation 隔离。 |
| close | DELETE existing/missing 都返回 204 | UI 先 Accepted；只有后续完整快照缺 id 才 Closed；超时是 Unconfirmed。 |
| storage | rich attr 后写全 NULL row | host/process/rule/network/chain IDs 均不降级。 |
| storage | `connection_session.host` 有值、host_id NULL | repair 后 raw/derived host 一致，上传/下载总量不变。 |
| storage | `connection_chain` 完整、chain_key NULL | 可重建；节点缺失/位置冲突则拒绝自动回填并报告 caveat。 |
| backfill | process_id NULL、字典只有一个 process 值 | 仍保持 Unknown，证明不做全局猜测。 |
| materialization | Unknown raw session 修复为 known 后重建窗口 | 旧 dimension_id=0 行被移除；各维上传/下载重建前后守恒；不能双计。 |
| archive | raw 修复后读取既有成功 archive | 仍为 frozen 旧结果，除非执行显式版本化 rebuild。 |
| compatibility | Mihomo v1.19.29/v1.19.30 固定 payload + 至少一个旧 Clash payload + additive future field | 规范化结果一致；缺失可选字段不降级已有 active metadata。 |
| manual | 真实 controller 同一窗口抓取原始 `/connections` 与 monitor live/排行 | 按 raw id 对照 Host/Chain/Process；分别记录源字段是否存在。真实 controller、现有用户库修复与 installed WebView 均标记为独立证据 gate。 |

### Files found

| Path | Description |
|---|---|
| `residential-monitor/src-tauri/src/controller.rs` | `/connections` JSON 的唯一规范化边界与现有小型 fixtures。 |
| `residential-monitor/src-tauri/src/transport.rs` | HTTP GET/DELETE、Bearer、连接 ID 校验与空 fixture server。 |
| `residential-monitor/src-tauri/src/session.rs` | `/version` 探测、raw body core identity 与 snapshot normalize。 |
| `residential-monitor/src-tauri/src/c2/collector.rs` | 生产约 1 Hz GET tick 与 scripted HTTP fixtures。 |
| `residential-monitor/src-tauri/src/accounting.rs` | stateless live projection、`epoch:id` delta state、first-frame/closure/reset 语义。 |
| `residential-monitor/src-tauri/src/c2/facade.rs` | 完整帧替换、remove/close confirmation、commit 顺序。 |
| `residential-monitor/src-tauri/src/c2/hub.rs` | live rows 的全量替换与前端 ConnectionDelta remove/upsert。 |
| `residential-monitor/src-tauri/src/storage.rs` | session/attr/chain 写入；当前 NULL overwrite 与 host-only upgrade。 |
| `residential-monitor/src-tauri/src/c3/schema.rs` | `(epoch_id, connection_id)` 唯一索引与每 session 单一 attr。 |
| `residential-monitor/src-tauri/src/c3/sql.rs` | Host/Process/Chain Unknown 的 raw/hourly/daily 查询语义。 |
| `residential-monitor/src-tauri/src/c3/retention.rs` | raw 到五维 hourly/daily 物化；回填时需显式清理旧 Unknown 桶。 |
| `residential-monitor/src-tauri/src/c3/archive.rs` | 成功报告冻结为 `result_json`，普通流程不覆盖。 |
| `ref/neko-master/packages/shared/src/index.ts` | Neko 的静态 ConnectionMetadata/Connection/ConnectionsData 契约。 |
| `ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts` | Direct WS active map、增量、metadata reuse、reset 与 missing-id close。 |
| `ref/neko-master/apps/agent/internal/gateway/client.go` | Clash HTTP payload、host/sniff 选择、chains/rule 默认。 |
| `ref/neko-master/apps/agent/internal/agent/runner.go` | Agent active flow、first-seen metadata freeze、reset 与 stale cleanup。 |
| `.trellis/tasks/archive/2026-08/08-18-residential-monitor-mvp/research/controller-compatibility-audit.md` | 固定版本的一手 Mihomo/Verge API、ID、关闭与 chains 审计。 |
| `.trellis/tasks/archive/2026-08/08-21-unknown-host-en-sidebar/research/unknown-host-24h.md` | 本机旧库 Unknown Host/Process 的实测组成与不可回填边界。 |

### External references

本轮未访问外网；以下均来自仓库中 2026-08-18 已完成的固定提交审计，可复现且不会随分支漂移：

- Mihomo API `/connections` 与 DELETE 文档：`MetaCubeX/Meta-Docs@e848aefb...`，固定链接登记于 `controller-compatibility-audit.md:308-315`。
- Mihomo `/connections` route/snapshot/WS ticker/DELETE：`MetaCubeX/mihomo@fe22fdd.../hub/route/connections.go`，固定链接登记于 `controller-compatibility-audit.md:317-327`。
- Mihomo active manager/tracker/计数与关闭删除：同固定提交的 `manager.go`、`tracker.go`，登记于 `controller-compatibility-audit.md:325-327`。
- 本机兼容基线：Clash Verge Rev 2.5.2 + Mihomo v1.19.29；对照稳定 v1.19.30，登记于 `controller-compatibility-audit.md:21-36`。

### Related specs

- `.trellis/spec/residential-monitor/backend/modules-and-errors.md:13-16`：host identity、1 Hz HTTP collector、lifecycle row retention、query snapshot 契约。
- `.trellis/spec/residential-monitor/storage/sqlite-contract.md:12,21,24-25`：gap 不写 0、raw DELETE 关闭、rule/chain 派生与 `__unknown__` 维度语义。
- `.trellis/spec/guides/cross-layer-thinking-guide.md:39,118-121`：缺失字段边界、round-trip 与派生状态必须指向源 identifier。
- `.trellis/tasks/08-21-eliminate-resolvable-unknown-attribution/prd.md`：R1-R7 与 AC1-AC7；本研究直接支撑稳定 ID、可信 fallback、旧历史兼容和真实 controller gate。

## Caveats / Not Found

- 本轮没有连接用户正在运行的 controller，也没有读取用户真实 `/connections` payload 或 SQLite；用户截图只证明 Clash UI 当前存在大量 active/closed 连接，不能证明每个历史 session 的 Host/Chain/Process 字段都曾被 controller 发出。
- 当前输入截图的 Process 列可见单元格为空，因此 Process Unknown 可能主要是上游真实缺失，而不是 monitor 丢字段。只有抓取脱敏 raw payload 才能定论。
- Mihomo 公开文档未承诺连接 ID 跨重启稳定、没有逐连接 final/close event，也未规定 chains 顺序；这些不能提升为产品保证。
- 仓库没有旧 Clash/Mihomo payload corpus，也没有按 v1.19.29/v1.19.30 自动跑的 contract fixture；“不同版本兼容”目前主要来自宽松 decoder 与固定源码审计，真实矩阵仍 UNVERIFIED。
- `providerChains` 的确切业务语义、数值端口 payload、process/processPath 在不同 find-process 配置/权限下的行为未由当前本地 fixture 证明；不得据名称猜测。
- 现有 schema 每 session 只有一组 attr，无法表示同一长连接内真正随时间变化的 rule/chain/process。采用 session-level enrichment 会把 later-known metadata 关联到该 session 的全部 raw minutes；这是有来源的同连接回填，但不是逐分钟“当时已知状态”的重建。
- 对旧 derived rows 的修复必须单独设计事务、窗口删除和守恒验证；直接更新 attr 或重复调用当前 materializer 可能留下旧 Unknown 桶并造成双计。
