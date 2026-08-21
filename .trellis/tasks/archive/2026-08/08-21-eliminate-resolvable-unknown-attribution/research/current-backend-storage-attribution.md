# Research: residential-monitor 后端与存储 Unknown 归因链路

- Query: 追踪 controller `/connections` payload → Rust decoder → connection identity/metadata → delta/accounting → SQLite → raw/hourly/daily 聚合，定位 Host / Chain / Process 的 Unknown 产生点、字段优先级、生命周期、信息丢失与风险；用本机热库匿名聚合验证截图数值，并对照 `ref/neko-master`。
- Scope: mixed（当前仓库源码、仓库内 `ref/neko-master` v1.4.0、只读本机 SQLite 热库）
- Date: 2026-08-21

## Findings

### 1. 结论摘要

以下事实已由当前源码和只读热库验证：

1. **截图中的 Chain `Unknown 4.7 GiB` 是确定性的后端语义错误，不是 Clash 缺少链路。** 当前库该窗口没有空 `chain_key`；单跳链共有 `5,011,781,324 B`，内容为 `DIRECT`。但 `last_chain_hop("DIRECT")` 返回 `None`，`RANK_RAW_CHAIN` 再把 `None` 变成 `__unknown__`，恰好得到截图的约 `4.7 GiB`（`residential-monitor/src-tauri/src/c3/rule_name.rs:10-23`, `residential-monitor/src-tauri/src/c3/sql.rs:89-100`）。同一错误也进入小时物化（`residential-monitor/src-tauri/src/c3/retention.rs:269-300`）。
2. **Process `Unknown 13.4 GiB` 与热库完全一致，但不能承诺全部可消除。** last-24h 总流量 `14,348,745,487 B`，其中 `process_id IS NULL` 为 `14,348,735,564 B`，仅 `mihomo` 为 `9,923 B`。用户截图中 Clash Connections 的 Process 列也为空。因此大头是源端未提供进程身份，而不是查询把已有 `process` 错标 Unknown。当前代码仍有两个可修复子集：`processPath` 已解码却不参与持久化/回退；已有 `process_id` 又可被后续空帧覆盖（`residential-monitor/src-tauri/src/controller.rs:144-145`, `residential-monitor/src-tauri/src/storage.rs:639-677`）。
3. **Host `Unknown 4.4 GiB` 是热库中的真实空 identity。** last-24h `coalesce(connection_session.host,'')=''` 为 `4,703,171,170 B`，与截图约 `4.4 GiB` 一致。当前源码本应按 `metadata.host → metadata.sniffHost → metadata.destinationIP` 回退，并在持久化时保留域名优于 IP（`residential-monitor/src-tauri/src/accounting.rs:96-100`, `residential-monitor/src-tauri/src/session_host.rs:18-45`, `residential-monitor/src-tauri/src/storage.rs:570-599`）。然而该热库没有任何 IP 形式的 host，Unknown 仍持续到窗口末端。由于没有读取 secret、没有抓取实时原始 `/connections` 帧，也没有核对正在运行二进制是否包含当前源码，具体原因仍是 **UNVERIFIED**：可能是短连接帧确实同时缺 `host/sniffHost/destinationIP`，也可能是运行二进制与工作树实现漂移。
4. **存在一个高于 Unknown 展示问题的静默写入丢失风险。** `AppFacade::boot` 每次把 `writer_epoch=1, bundle_seq=1`；每个采集 bundle 的 payload 恒为空；SQLite 对相同 `(writer_epoch,bundle_seq)` 与相同 hash 直接返回 `Duplicate`，且返回发生在 `persist_slice` 之前（`residential-monitor/src-tauri/src/c2/facade.rs:286-320`, `residential-monitor/src-tauri/src/c2/facade.rs:685-711`, `residential-monitor/src-tauri/src/storage.rs:321-344`, `residential-monitor/src-tauri/src/storage.rs:383-387`）。本机库只有 epoch 1、seq `1..68211`。因此应用重启后，新帧会从 1 开始与旧 receipt 碰撞，在追平旧最大 seq 前都不会写流量或 metadata。此项应作为归因优化前的 P0 数据完整性门处理。
5. **Process / Chain / host_id 的存储模型是“当前最后一行覆盖整个 session 历史”，没有时间版本。** 每帧先遍历所有 `live_rows`，再把该帧的属性 upsert 到唯一 `connection_session_attr(session_pk)`；空 process/chain 会覆盖先前已知值，之后 raw 查询会把该 session 所有旧 minute 流量一起重新解释为 Unknown（`residential-monitor/src-tauri/src/c3/schema.rs:30-43`, `residential-monitor/src-tauri/src/storage.rs:538-560`, `residential-monitor/src-tauri/src/storage.rs:655-677`, `residential-monitor/src-tauri/src/c3/sql.rs:56-100`）。Host 的 raw 排名使用单独的 `connection_session.host`，所以当前 raw Host 查询避开了这一覆盖；但小时 Host 物化读取可被清空的 `a.host_id`，仍有长期层回归风险（`residential-monitor/src-tauri/src/c3/retention.rs:218-249`）。

### 2. 本机热库匿名验证

只读打开 `%TEMP%\io.github.bahayonghang.residential-monitor\monitor.sqlite3`，未读取 Credential Manager、controller secret、域名/IP/进程路径明细。查询窗口锚定 `max(utc_minute)=29788343`，起点为 `max-1440`；`user_version=4`。

| 指标（upload + download） | 字节 | 约 GiB | 判定 |
| --- | ---: | ---: | --- |
| 全部 raw minute | 14,348,745,487 | 13.36 | 与 Process 卡片总量一致 |
| `connection_session.host` 为空 | 4,703,171,170 | 4.38 | 与 Host `Unknown 4.4 GiB` 一致 |
| `process_id` 为空 | 14,348,735,564 | 13.36 | 与 Process `Unknown 13.4 GiB` 一致 |
| 非空 process (`mihomo`) | 9,923 | 0.000009 | 与截图 `9.7 KiB` 一致 |
| `chain_key` 真正为空 | 0 | 0 | 没有源端空链造成的该窗口 Unknown |
| 单跳 `chain_key` | 5,011,781,324 | 4.67 | 全部为 `DIRECT`，被当前 SQL 映为 Unknown |
| 多跳 `chain_key` | 9,336,964,163 | 8.70 | 最后一跳产生 `Proxy/Others/AI-家宽/...` |
| 缺整行 `connection_session_attr` | 0 | 0 | 当前窗口 facts 均有 attr 行 |
| `s.host` 已知但 `a.host_id` 为空 | 0 | 0 | 当前热库未观测到该覆盖状态 |

补充只读证据：

- Process 排名原始分布只有 `__unknown__`（`14,348,735,564 B`, 37,127 sessions）与 `mihomo`（`9,923 B`, 1 session）。
- Host Unknown 按链拆分仍有明确链：`DIRECT` 约 `3.25 GB`、`US 07>Proxy` 约 `1.39 GB` 等。这证明 Host Unknown 不等于整个 connection 无 identity；只是 host 三路字段未进入数据库。
- Host 非空流量约 `9.65 GB`，545 个 distinct host；其中没有 IP 字面量。当前源码若实际执行 destination-IP 回退，预期至少部分无域名连接会形成 IP identity；没有抓原始帧/二进制版本前不能据此断言是哪一层失效。
- `committed_bundle` 只有 `(writer_epoch=1, min_seq=1, max_seq=68211, count=68211)`；`data_version=68211`，与源码的固定 boot 值碰撞风险一致。

### 3. 端到端事实链

#### 3.1 生产采集是 1 Hz HTTP GET，不是 WebSocket

- 生产循环每 `SAMPLE_INTERVAL_MS=1000` ms 运行一次（`residential-monitor/src-tauri/src/c2/contract.rs:6`, `residential-monitor/src-tauri/src/lib.rs:1239-1250`）。
- tick 在短锁内规划，然后锁外调用 `fetch_snapshot`，再在锁内应用结果（`residential-monitor/src-tauri/src/lib.rs:173-208`）。
- `fetch_snapshot` 调用 `ControllerSession::fetch_normalized_snapshot`；后者执行 GET `/connections`、解析 JSON，再进入 normalizer（`residential-monitor/src-tauri/src/c2/collector.rs:78-95`, `residential-monitor/src-tauri/src/session.rs:98-117`, `residential-monitor/src-tauri/src/transport.rs:101-105`）。
- 仓库虽有 WebSocket fixture test，但生产 collector 没有使用它（`residential-monitor/src-tauri/src/transport.rs:261-280`）。因此方案不应围绕 WebSocket 增量字段设计，除非另行扩大传输层范围。

#### 3.2 Rust decoder 的接受/丢弃规则

- 根 payload 的 `uploadTotal/downloadTotal` 缺失或类型不对时静默变成 0；`connections` 缺失或非数组时静默变为空列表（`residential-monitor/src-tauri/src/controller.rs:94-120`）。
- connection 只要 object 且有非空字符串 `id` 就被接受；缺 metadata 时使用空 map，缺 upload/download 时变 0（`residential-monitor/src-tauri/src/controller.rs:123-136`）。
- Host 候选：`metadata.host`、`metadata.sniffHost`、`metadata.destinationIP`；Process 候选同时解了 `metadata.process` 和 `metadata.processPath`；Chain 同时解了根 `chains` 与 `providerChains`（`residential-monitor/src-tauri/src/controller.rs:137-160`）。
- 字符串只拒绝长度为 0 的值；纯空白字符串会被保留并进入维度字典。数组只保留字符串元素（`residential-monitor/src-tauri/src/controller.rs:87-92`, `residential-monitor/src-tauri/src/controller.rs:164-187`）。这是 P2 数据清洁缺口。
- `providerChains` 在 DTO 中存在，但除构造测试外全仓没有消费；实际投影只复制 `connection.chains`（`residential-monitor/src-tauri/src/controller.rs:30-32`, `residential-monitor/src-tauri/src/controller.rs:135-136`, `residential-monitor/src-tauri/src/accounting.rs:110-113`）。当 `chains=[]` 但 `providerChains` 有内容时，这是可验证的信息丢失点。

#### 3.3 Live projection 与 metadata 生命周期

- 每帧从当前 `ConnectionFact` 重新构造完整 `LiveConnectionView`；Host 在这里执行三路优先级，Process 只用 `process_name`，Chain 只用 `chains`（`residential-monitor/src-tauri/src/accounting.rs:77-116`）。
- `AccountingEngine.sessions` 只缓存 counter、时间和 seen 标志，不缓存 metadata（`residential-monitor/src-tauri/src/accounting.rs:36-53`）。因此 live 层没有“已知非空值不被临时空值降级”的 merge。
- `MonitorHub::publish` 用本帧 BTreeMap 替换上一帧 rows；它计算 rate，但不合并上一帧 metadata（`residential-monitor/src-tauri/src/c2/hub.rs:276-321`）。
- `connection_session` 的 identity 是 `epoch:controller_connection_id`。`AccountingEngine` 在进程启动时 epoch=0，只在收到 `ControllerInput::Restarted` 时递增（`residential-monitor/src-tauri/src/accounting.rs:45-53`, `residential-monitor/src-tauri/src/accounting.rs:124-150`）。
- 生产 tick 只调用静态 `fetch_normalized_snapshot`，不取 `/version`，也不调用 `detect_restart`；只有设置页 `test_controller` 走 `connect_tcp` 和 restart 检测（`residential-monitor/src-tauri/src/session.rs:59-95`, `residential-monitor/src-tauri/src/c2/collector.rs:78-83`, `residential-monitor/src-tauri/src/lib.rs:488-530`）。因此 core 在后台重启时 epoch 不可靠，connection ID 若复用会合并 session；实际 UUID 复用概率未测，标记 **UNVERIFIED**。

#### 3.4 Delta / sampling 与 facts

- 首次看到 connection 时只建立 baseline，不写 `MinuteFact`；后续按 cumulative counter 做 `saturating_sub`（`residential-monitor/src-tauri/src/accounting.rs:188-244`）。
- counter 下降时当前值被设为新 baseline并整帧跳过，不把当前值当作新 epoch 流量（`residential-monitor/src-tauri/src/accounting.rs:220-230`）。
- `MinuteFact` 只保存 `session_key/minute/upload/download/primary/tags`，没有 host/process/chain 快照（`residential-monitor/src-tauri/src/accounting.rs:6-14`）。归因只能依赖后续可变的 session attr。
- `ingest_snapshot` 从同一帧生成 live rows 和 facts，再将二者放进一个 SQLite commit slice（`residential-monitor/src-tauri/src/c2/facade.rs:594-629`, `residential-monitor/src-tauri/src/c2/facade.rs:635-711`）。正常无 duplicate 时 facts 与当帧 metadata 至少事务一致；但历史 metadata 不是逐分钟一致。

#### 3.5 SQLite 写入与 merge 优先级

- C1 表 `connection_session` 保存唯一 `(epoch_id, connection_id)` 的 host，`connection_minute` 保存分钟 delta（`residential-monitor/src-tauri/src/storage.rs:158-177`）；C3 `connection_session_attr` 每 session 只有一行 process/rule/network/chain/host_id（`residential-monitor/src-tauri/src/c3/schema.rs:30-43`）。
- 写入时先遍历所有 live rows，调用 `ensure_session_on` 与 `intern_and_attr`，再写 facts（`residential-monitor/src-tauri/src/storage.rs:538-560`）。
- Host 的 `connection_session.host` 是唯一实现了保真升级的字段：空→incoming、IP→domain、domain 不被 incoming IP 降级；新 domain 可以替换旧 domain（`residential-monitor/src-tauri/src/session_host.rs:29-45`, `residential-monitor/src-tauri/src/storage.rs:570-599`）。
- `intern_and_attr` 对 host_id/process_id/rule_id/network_id/chain_key/primary_category_id 全部执行 `column = excluded.column`；没有 `coalesce`、quality rank 或 source provenance（`residential-monitor/src-tauri/src/storage.rs:639-677`）。因此临时空 process/chain 会清掉已知值。
- `connection_chain` 对 `(session_pk,position)` 使用 `insert or ignore`，保留首次写入节点；而查询使用可变的 `connection_session_attr.chain_key`（`residential-monitor/src-tauri/src/storage.rs:539-548`, `residential-monitor/src-tauri/src/c3/sql.rs:89-100`）。链发生变化时，同一库内会同时存在“first observed nodes”和“last overwritten chain_key”两套真相。
- `ended_utc` 字段从未在生产写路径更新；attr 也没有时间区间版本。schema 看似支持 lifecycle，实际是永久开放且全历史共用（`residential-monitor/src-tauri/src/c3/schema.rs:32-43`, `residential-monitor/src-tauri/src/storage.rs:655-677`）。

#### 3.6 Raw 与物化查询如何制造 Unknown

- 最近 30 天默认走 Raw tier（`residential-monitor/src-tauri/src/c3/query.rs:13-15`, `residential-monitor/src-tauri/src/c3/query.rs:628-675`），截图 Last 24 hours 因而使用 raw ranking。
- Host raw：`s.host` 空才变 `__unknown__`（`residential-monitor/src-tauri/src/c3/sql.rs:41-54`）。
- Process raw：`a.process_id` LEFT JOIN `dimension_dict(process)`，空或查不到时变 `__unknown__`（`residential-monitor/src-tauri/src/c3/sql.rs:56-73`, `residential-monitor/src-tauri/src/c3/service.rs:241-278`）。
- Chain raw：调用 `last_chain_hop(a.chain_key)`；该 helper 只为“多跳规则组”语义设计，遇到单跳会返回 None；查询随后把它映成 `__unknown__`（`residential-monitor/src-tauri/src/c3/rule_name.rs:10-23`, `residential-monitor/src-tauri/src/c3/sql.rs:89-100`）。因此不仅 `DIRECT`，任何单元素链都会错误落入 Unknown。
- Service 把 identity `__unknown__` 的 label 固定为“未知”；前端收到的不是空字符串（`residential-monitor/src-tauri/src/c3/service.rs:597-621`）。
- 小时物化 Host/Process 对 NULL 使用 dimension_id 0；Chain 同样调用只接受多跳的 `last_chain_hop`，单跳也变 0（`residential-monitor/src-tauri/src/c3/retention.rs:218-300`）。小时/日排名再因 dimension_id 0 无字典项而输出 `__unknown__`（`residential-monitor/src-tauri/src/c3/sql.rs:153-164`, `residential-monitor/src-tauri/src/c3/sql.rs:202-213`）。

### 4. 三个 Unknown 的产生点与可解性

| 维度 | 当前字段优先级 | Unknown 的直接条件 | 可解子集 | 当前不可安全解决的部分 |
| --- | --- | --- | --- | --- |
| Host | `host → sniffHost → destinationIP`；持久化 `domain > IP > empty` | raw `s.host` NULL/空 | 当前帧有 destinationIP 但旧 binary/decoder 未用；已有 session 后续出现 host/sniff/IP；运行态 drift 可修 | 已关闭 session 的三路字段均未持久化，库内无法反推准确 host；不能用 chain 猜 host |
| Chain | 当前只用 `chains`；排名想取最后一跳 | 空 chain；**以及所有单跳 chain（确定性 bug）** | `DIRECT`/任意单跳返回自身；`providerChains` 受控 fallback；已保留 raw chain_key 可重算 | `chains/providerChains/rule` 都缺时不能声称真实链 |
| Process | 仅 `metadata.process` | `process_id` NULL/字典缺失 | `process` 晚到时保留；`processPath` 非空时只提 basename/受控 label；阻止空帧降级 | 用户 controller/TUN 若没有 process 与 processPath，库内无法发明应用名；不能把所有 TUN 流量标成 `mihomo` |

### 5. `ref/neko-master` 可复用模式与不可照搬点

仓库内 ref 版本为 `neko-master` 1.4.0（`ref/neko-master/package.json:1-5`）。其 shared connection contract 同样包含 `host/sniffHost/destinationIP/process/processPath/chains/providerChains/rule/rulePayload`（`ref/neko-master/packages/shared/src/index.ts:4-40`）。

可复用模式：

- Gateway collector 明确把 `metadata` 缺失视为空 object，并为非数组 chain 回退 `DIRECT`、rule 回退 `Match`（`ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts:424-456`）。
- 新 connection 建立 `activeConnections` 状态后，后续 delta 使用 `existing.domain/chains/rule/sourceIP`，而不是每次用可能缺失的当前 metadata 重写；这避免“已知→空”降级（`ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts:445-464`, `ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts:513-570`）。
- Neko 对首次看到且已有 cumulative traffic 的 connection 会立即记入初始 traffic；当前 residential-monitor 把首次观察只当 baseline。两者统计口径不同，若要改必须先确认“观测下界”产品口径，不能把它当无风险机械移植（`ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts:447-496`, `residential-monitor/src-tauri/src/accounting.rs:207-219`）。

不可照搬：

- Neko 的 gateway collector 把 domain 与 destination IP 分开，domain 只取 `host || sniffHost || ""`；residential-monitor 已定义 Host identity 可回退 IP，不能倒退（`ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts:436-443`, `residential-monitor/src-tauri/src/session_host.rs:18-27`）。
- Neko 的 active state 固定 first-seen metadata；residential-monitor 更适合“单调 enrichment”：空→IP→domain、空 process→process、空 chain→chain，同时禁止非空被空覆盖。纯 first-seen 会把初始缺字段永久固化。
- Neko gateway 写入链/域名/IP流量，但没有给本任务提供可直接复用的 Process 聚合实现；不能声称参考项目已经解决 Process Unknown。

### 6. 风险分类

| 级别 | 风险 | 证据与影响 |
| --- | --- | --- |
| P0 | writer epoch/seq 每次 boot 重置，重复 receipt 跳过新 slice | `facade.rs:286-320,685-711`; `storage.rs:321-344,383-387`。重启后静默丢帧，且 metadata enrichment 也不落库 |
| P1 | 单跳 Chain 被错误映成 Unknown | `rule_name.rs:10-23`; `sql.rs:89-100`; 热库 `DIRECT=5,011,781,324 B` 精确复现截图 |
| P1 | attr 最后值覆盖整个历史、可由空值降级 | `schema.rs:30-43`; `storage.rs:639-677`; raw 与物化聚合都 retroactive relabel |
| P1 | 生产 collector 不持续检测 core restart/epoch | `collector.rs:78-83`; `session.rs:59-95`; ID 复用时合并会话，meter reset 也只 saturating/drop；实际复用频率 UNVERIFIED |
| P1 | Host Unknown 的 live source/runtime 原因不明 | 当前源码应回退 destinationIP，但热库 4.70 GB 空 host 且无 IP host；若不先抓字段存在性计数，直接改 SQL 只能掩盖问题 |
| P1 | Process 几乎全部没有源 identity | 热库 13.348 GiB Unknown、截图 Process 列为空；“消除全部 Unknown”在当前 source contract 下不可达 |
| P2 | `processPath` 与 `providerChains` 解码后未消费 | 可修复有信息但未归因的子集；需脱敏与明确 fallback 优先级 |
| P2 | `connection_chain` first-observed 与 attr chain_key last-overwritten 分叉 | 同一 session 两套链真相；查询只读后者 |
| P2 | 首样本与 counter reset 流量被丢弃 | 保持 observed-lower-bound 可能是有意设计，但与 Neko 口径不同，需独立产品决策 |
| P2 | whitespace identity 可进字典 | 空白 process/host/chain 可能表现为视觉空行而非 Unknown |

### 7. 对优化方案的约束与建议机制

以下是由证据直接推出的方案边界，不是已实施变更：

1. **先修 writer lifecycle，再评估 Unknown 指标。** boot 时必须获得不会与历史 receipt 冲突的 writer epoch（例如持久化原子分配新 epoch），bundle seq 在该 epoch 内从 1 开始；增加“关闭→重启→第一帧立即 Applied 且 facts/attrs 落库”的真实 SQLite 回归。仅把 seq 设置为旧 max+1 仍会把不同进程生命周期混在同一 epoch，优先级低于新 epoch。
2. **拆分 Rule helper 与 Chain identity helper。** Rule 仍需“多跳取最后一跳、单跳回退 raw rule”；Chain 应为“空→Unknown，单跳→自身，多跳→最后一跳”。同步修改 raw rank、filter、intern、hourly materialization，覆盖 `DIRECT`、单跳代理、多跳、空链四组测试。
3. **metadata 采用带质量等级的单调 enrichment。** Host 复用现有 `prefer_host_identity`；Process 为 `process > basename(processPath) > existing > empty`；Chain 为 `chains > vetted providerChains > existing > empty`。任何当前空值不得覆盖已知值。若业务允许链在 session 中真实变化，则单行 attr 不够，必须先做时间版本设计，不能靠 `coalesce` 假装精确。
4. **不要把 Process Unknown 伪装成 `mihomo` 或 TUN。** 增加不含敏感值的 source coverage 计数（例如 `process present / path-only / absent`、`host/sniff/ip/absent`、`chains/provider/absent`），再决定是否提示用户检查 Mihomo process discovery 配置。自动改 Clash 配置属于额外授权范围。
5. **数据修复分层处理。** Chain 单跳错误可从现存 raw `chain_key` 精确重算；小时/日层需要新物化水位/向前 migration，不能改写已发布 C1/C3 migration。Process/Host 的已关闭 Unknown 若原始 path/IP 从未持久化，则保持 Unknown 或标“历史源字段缺失”，不得猜测回填。
6. **诊断先验证 running binary 与 source 一致。** 在不输出 secret/host/IP/path 的前提下记录单帧字段存在性计数和 binary version。只有确认当前 binary 收到 destinationIP 但仍写空 host，才能把 Host 问题收敛到 decoder/storage；否则它是 source coverage 或部署漂移问题。

建议验收测试矩阵：

- raw Chain：`["DIRECT"] → DIRECT`、`["ProxyA"] → ProxyA`、`["node","group"] → group`、`[] → __unknown__`。
- hourly/daily Chain 与 raw identity 集合一致；旧 `dimension_id=0` 的单跳流量经受控重物化归入真实维度。
- 同 session 序列：process/chain/host 由空到已知可升级；后续空帧不降级；domain 不被 IP 覆盖。
- `process=None, processPath=Some(...)` 只持久化安全 basename，不落完整路径；两者都空保持 Unknown。
- `chains=[]`, `providerChains!=[]` 的 fallback 行为有明确 fixture；两者都空不制造 DIRECT，除非产品明确规定该语义。
- 两次 `AppFacade::boot` 使用同一 DB，第二次第一帧不是 Duplicate；data_version 增长且 minute/attr 均可见。
- controller restart/total reset 有新的 epoch 或显式 coverage gap，不把新 core 的 connection 与旧 session 合并。

### 8. Files found

- `residential-monitor/src-tauri/src/transport.rs` — HTTP GET `/connections` 与 WebSocket 仅测试 fixture。
- `residential-monitor/src-tauri/src/session.rs` — controller session、JSON parse、仅手工 probe 的 version/restart 检测。
- `residential-monitor/src-tauri/src/controller.rs` — 原始 JSON normalizer 与可选字段 DTO。
- `residential-monitor/src-tauri/src/session_host.rs` — Host 三路解析与空/IP/domain 质量优先级。
- `residential-monitor/src-tauri/src/accounting.rs` — live projection、counter baseline/delta、无 metadata 的 minute facts。
- `residential-monitor/src-tauri/src/c2/collector.rs` — 生产 tick 规划与静态 snapshot fetch。
- `residential-monitor/src-tauri/src/c2/facade.rs` — snapshot→live/batch→commit slice，固定 writer epoch/seq 初始化。
- `residential-monitor/src-tauri/src/c2/hub.rs` — live rows 全量替换，无 metadata merge。
- `residential-monitor/src-tauri/src/storage.rs` — C1 schema、receipt 去重、session/attr/fact 写入与维度 intern。
- `residential-monitor/src-tauri/src/c3/schema.rs` — 单行 `connection_session_attr` 与物化表结构。
- `residential-monitor/src-tauri/src/c3/rule_name.rs` — `last_chain_hop` 的“必须多跳”语义。
- `residential-monitor/src-tauri/src/c3/sql.rs` — raw/hourly/daily ranking 的 Unknown sentinel 产生点。
- `residential-monitor/src-tauri/src/c3/query.rs` — 30 天 raw tier 选择与报告 DTO。
- `residential-monitor/src-tauri/src/c3/service.rs` — ranking SQL 路由与 `__unknown__ → 未知` 映射。
- `residential-monitor/src-tauri/src/c3/retention.rs` — 五维小时/日物化及 dimension_id 0 Unknown。
- `ref/neko-master/packages/shared/src/index.ts` — 参考 controller connection 字段 contract。
- `ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts` — active connection metadata reuse、默认 chain/rule 与 first-sample 口径。

### 9. Related specs

- `.trellis/spec/residential-monitor/backend/index.md` — backend scope 与真实 controller 手工证据边界。
- `.trellis/spec/residential-monitor/backend/modules-and-errors.md` — Host identity 单一实现、1 Hz `/connections`、C2/C3 边界。
- `.trellis/spec/residential-monitor/backend/secrets-and-cancellation.md` — secret 不得进入日志/SQLite/诊断；本次未读取 secret。
- `.trellis/spec/residential-monitor/storage/index.md` — SQLite 是权威账本，性能/容量需实测。
- `.trellis/spec/residential-monitor/storage/sqlite-contract.md` — 单 writer、Unknown sentinel、维度物化与已发布 migration 不可改写。
- `.trellis/spec/residential-monitor/frontend/dto-and-decoding.md` — `__unknown__` 展示与不可下钻契约；不允许把缺失显示为 0。

### 10. External references / versions

- 仓库内参考实现：`neko-master` v1.4.0，`pnpm@9.15.9`（`ref/neko-master/package.json:1-10`）。
- 未调用外部网络文档。本任务所需 controller 字段由当前 normalizer、用户截图和仓库内 Neko shared contract 交叉核对。

## Caveats / Not Found

- **UNVERIFIED：实时 controller 原始字段覆盖。** 未读取 Credential Manager secret，未抓取带 host/IP/process/path 的真实 `/connections` body；因此 Host 4.70 GB 的源端缺失比例与 decoder/runtime drift 尚未分离。
- **UNVERIFIED：运行二进制与当前工作树是否同版。** 当前源码有 destination-IP fallback，但热库没有任何 IP host。必须用 binary version/构建 SHA 或同版重新采样确认。
- **UNVERIFIED：Mihomo process discovery 配置。** 用户截图的 Process 列为空，热库也几乎全空，但未读取或修改真实 Clash 配置；不能断言是 `find-process-mode`、TUN 限制还是平台权限。
- **UNVERIFIED：core connection ID 实际复用率。** 生产 collector 的 epoch 检测缺口已由代码确认，但它造成多少 session 合并需要专门 restart fixture/真实控制器测试。
- 本机热库查询是 2026-08-21 的动态只读快照；数值会随 collector 继续运行而变化。未复制、修复、VACUUM 或修改该数据库。
- 旧 Host/Process Unknown 的原始 destinationIP/processPath 没有保存在现有 schema；关闭后无法从账本恢复，任何“全量历史清零 Unknown”承诺都不成立。
- 研究未修改产品代码、spec、task planning 文件，也未执行 git 操作。
