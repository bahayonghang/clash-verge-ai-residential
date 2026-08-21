# 设计：消除可解析连接的 Unknown 流量归因

## 1. 问题分类与设计目标

当前截图把四种不同事实都显示为 `Unknown`。本设计不承诺“让 Unknown 数值归零”，而是只消除可证明的误归因，并让不可恢复或源端未报告的部分保持守恒且可解释。

| 类别 | 现场证据 | 根因 | 设计处理 |
| --- | --- | --- | --- |
| 实时五张计量卡 Unknown | Header 为 connecting、无 last sample | 当前样本尚未建立，不是维度归因失败 | 显式 `observationPhase`，显示等待连接/建立基线，不再用通用 Unknown |
| Host 约 4.4 GiB | 旧库 20,692 个 session 的 `s.host` 为空 | 修复前没有持久化 host/sniff/IP；数据库最后样本早于 Host 修复提交 | 旧历史保持缺失；新会话按可信来源合并；UI 标明历史未归因 |
| Chain 约 4.7 GiB | 所有对应 `chain_key` 均为单跳 `DIRECT` | 规则分组 helper `last_chain_hop` 被错误复用于 Chain identity | 独立 `chain_identity`，修 raw/filter/物化并受控重建派生层 |
| Process 约 13.4 GiB | 37,128 个 session 仅 1 个有 process；Clash 截图 Process 列也为空 | 绝大部分源端未报告；另有 path-only 与空帧擦除的可修子集 | `process -> basename(processPath)`、同连接保留、覆盖度 DTO；两者皆空仍保持缺失 |
| 新代码无法改善旧库 | `committed_bundle` 只有 epoch 1、seq 1..68211 | 每次 boot 重置 epoch/seq 且空 payload hash 相同，重启帧被判 Duplicate | P0 先分配 durable writer/controller epoch，第一帧立即 Applied |

相关证据：`research/local-database-unknown-audit.md`、`research/current-backend-storage-attribution.md`、`research/prior-unknown-task-residual-gaps.md`、`research/neko-master-attribution-comparison.md`、`research/mihomo-connection-contract.md`、`research/frontend-query-unknown-semantics.md`。

## 2. 总体数据流与所有权

```text
HTTP GET /connections (约 1 Hz)
  -> controller.rs: 严格 frame 解码、trim、字段存在性诊断
  -> AccountingEngine: controller generation 内 canonical metadata + counter baseline
  -> 同一 canonical snapshot 同时生成 LiveConnectionView 与 MinuteFact
  -> storage.rs: durable bundle id + 防御性字段级 merge + raw facts
  -> c3: raw query / chain identity / hourly-daily materialization / attribution quality
  -> IPC decoder: 字段存在与显式 null 分开
  -> React: live observation plane 与 historical report plane 分开
```

边界职责：

- `controller.rs` 只拥有外部 JSON 契约和清洗，不做跨帧继承。
- `AccountingEngine` 是活动连接 canonical metadata 的唯一内存 owner；不得让 live、delta、storage 各自重新猜 fallback。
- `storage.rs` 拥有 durable epoch、幂等 receipt 与 SQLite 防御性 merge；即使上层回归，空帧也不能擦除已知维度。
- `c3/rule_name.rs` 分开 Rule group 和 Chain identity；查询、过滤与物化只调用各自契约。
- 后端报告返回守恒的维度覆盖度；前端不从截断 Top N 反算完整度。
- React 明确显示数据面和状态，不把 live 连接失败解释成 historical report 失败。

## 3. P0：durable writer 与 controller generation

### 3.1 Writer epoch

利用既有 `bundle_epoch`，不升 schema：

1. `StorageCoordinator::reserve_writer_epoch()` 在 `BEGIN IMMEDIATE` 内读取 `max(writer_epoch)`，安全加一，并插入 `(new_epoch, highest_contiguous_seq=0, durable_watermark=current data_version)`。
2. `AppFacade::boot` 在 NormalReady 分支取得该 epoch，`bundle_seq` 在新 epoch 内从 1 开始；RecoveryOnly 不伪造可写 epoch。
3. `commit_alert_bundle` 的 receipt key 保持 `(writer_epoch,bundle_seq)`。同一应用生命周期的合法 retry 仍可返回 Duplicate；跨启动永不复用 writer epoch。
4. 告警 slice 的 hash 必须覆盖本次实际提交内容或至少覆盖不可碰撞的 frame fingerprint，不能继续让所有生产 bundle 对空字符串求同一 hash。复用现有 `sha2`，不新增依赖。
5. `PayloadMismatch`、`RetryWindowExpired` 与 storage error 必须进入稳定错误/health，不可被 `.ok()` 静默吞掉。

### 3.2 Controller epoch 与 session identity

利用既有 `controller_epoch` 表，不升 schema：

- 首次成功采样前原子分配新的 `epoch_id`，并以不含 secret 的 core/run identity 写入表；`AccountingEngine` 使用该 durable epoch，而不是每次进程启动从 0 开始。
- collector 从非 Connected 转为成功、显式 controller restart、全局 meter 回退或可信 start/reset 信号时，结束旧 generation、清空 canonical metadata/cache、分配新 controller epoch；下一帧只建 baseline。
- HTTP/JSON 失败或结构不完整不是“完整空快照”，不得结束全部连接或清 cache。
- 同一 generation 内 connection id 消失后再出现、计数器下降或 `start` 明确变化时，必须切断旧 session；可以安全地提升整个 controller epoch，宁可丢一个不可比较区间，也不得跨连接继承 metadata。
- `connection_session(epoch_id,connection_id)` 继续作为 durable identity；不修改用户可见 raw controller id 与关闭连接命令。

这两种 epoch 含义独立：writer epoch 解决 commit 幂等，controller epoch 解决连接生命周期隔离。禁止用当前 `bundle_seq` 代替 session generation。

## 4. Canonical metadata 单调合并

### 4.1 完整成功帧

`normalize_snapshot` 必须要求根对象包含可用 `connections` 数组以及数值型 meter 字段；缺字段与类型错误返回可观测的 `ProtocolIncompatible` / incomplete 状态，而不是静默变成“0 meter + 0 connections”并关闭全部现有连接。未知附加字段继续宽松忽略。

所有字符串先 trim；空白等同缺失。Chains 过滤空白元素但保留原顺序。`providerChains` 只记录字段覆盖诊断，在真实 fixture 证明等价前不得自动替代 `chains`。

### 4.2 活动连接 canonical state

把 `AccountingEngine::SessionAcc` 扩展为 counter + canonical metadata，或在 accounting 模块内建立等价的单一 tracker。`apply_snapshot` 先按当前 controller epoch 合并 metadata，再由同一结果产生 live rows 和 delta facts。

字段规则：

| 字段 | 优先级 / 合并 | 禁止行为 |
| --- | --- | --- |
| Host | 显式 `host` > `sniffHost` > `destinationIP`；保留来源等级，空不降级，域名不被 IP 覆盖 | 只比较“像不像 IP”后让 sniff 覆盖曾经的 explicit host |
| Process | 非空 `process` > 安全的 `basename(processPath)` > 既有值 > 缺失；直接 process 可升级 path-derived 值 | 保存完整进程路径到字典、日志或报告；把空进程猜成 `mihomo` |
| Chains | 当前非空、清洗后的整组 chains 更新 canonical；空组继承；连接代际变化清空 | 跨帧逐 hop 拼接；把未证明的空链强制改成 DIRECT |
| Rule / Network / ports / IP / start | incoming 非空更新，当前空继承；代际变化清空 | 用 `Match` 等默认值写成观察事实 |
| providerChains | 仅诊断覆盖与 fixture | 未验证时冒充 chains |

首次 snapshot 仍只建 counter baseline，不采用 Neko 的“把启动前累计量算进当前窗口”。连接消失不猜尾流量。Unknown 优化不得改变 controller meter、observed lower bound 或 gap/over 口径。

### 4.3 SQLite 防御性 merge

`intern_and_attr` 使用字段级更新：

- host_id 从 `connection_session.host` 的 canonical 最终值 intern，确保 raw Host 与 hourly Host 一致。
- process_id、rule_id、network_id、chain_key 仅在 incoming non-null/non-empty 时更新；当前空值保留旧值。
- primary_category_id 仍服从当前分类策略，不把“维度保留”误用于可撤销的策略分类。
- canonical chain 改变时，`connection_chain` 作为整组替换；不再 `insert or ignore` 留下 first-seen nodes、同时 attr 保存 last-seen chain 的双重真相。
- 同一 session 的 late metadata 会让该 session 先前 raw minutes 获得归因，这是当前单行 attr schema 的明确语义；不能宣传为逐分钟当时状态。跨 generation 严禁回填。

## 5. Chain identity 与派生层修复

### 5.1 两个独立纯函数

保留现有 `last_chain_hop` 给 Rule group：

- 多跳取末跳；
- 单跳返回 None，让 Rule 回退 raw rule；
- 皆空时 Rule 可按现有契约回退 DIRECT。

新增 `chain_identity` 给 Chain 维：

- 空 / 全空白 -> None / `__unknown__`；
- 单跳 -> trim 后原值，例如 `DIRECT`、`ProxyA`；
- 多跳 -> 末个非空 hop，保持当前产品“顶层策略组”口径。

注册确定性、innocuous SQLite scalar，并在每个 writer/reader connection 上注册。不得直接改变 `last_chain_hop("DIRECT") == None`，否则 Rule 页会从 IPCIDR 等退化成 DIRECT。

### 5.2 全路径替换

以下路径必须共同使用 `chain_identity`：

- raw Chain rank；
- `filters.chain`；
- chain 字典 intern；
- hourly chain materialization；
- raw/hourly key parity tests；
- dimension coverage 的 known/missing 判定。

### 5.3 旧派生层

当前 raw DELETE 关闭，因此当前库可从 `connection_minute + connection_session_attr.chain_key` 精确重建 Chain 派生层。使用新的版本化 retention watermark/marker：

1. 找出仍有 raw 的时间窗；
2. 在一个事务内删除该窗 `traffic_hourly_dimension.dimension_kind='chain'` 的旧行；
3. intern 新 chain identities 并重新物化；
4. 删除并从 hourly 重建受影响的 daily chain 行；
5. 验证重建前后 upload/download 总量守恒、旧 dimension_id=0 DIRECT 消失，再提交 marker。

只做 `INSERT OR REPLACE` 不够，因为分类变化会新增 DIRECT 行而遗留旧 0 行，造成双计。raw 已删除的区间保持旧 Unknown；已成功冻结的 `report_archive.result_json` 不由普通启动悄悄改写。

## 6. Process 与 Host 的可恢复边界

### 6.1 Process

- `processPath` 只取 Windows/Unix 两种分隔符后的 basename；空 basename、目录结尾和超限值保持缺失。
- live row 可保留完整 path 的既有短期语义，但统计、字典、报告、诊断和日志只使用 name/basename，防止路径泄露。
- 若本帧 process 缺失但同 generation 曾有可信值，沿用；若随后直接 process 到达，升级 path-derived identity。
- 两者始终缺失时，保持守恒 missing bucket，并通过维度覆盖度告诉用户“控制器未报告”，不修改 Clash 配置。自动调整 `find-process-mode` 属于额外授权范围。

### 6.2 Host

- 保留 `host -> sniffHost -> destinationIP`，新增来源等级以阻止 explicit host 被后续仅 sniffHost 降级。
- 修复 `connection_session.host` 与 `connection_session_attr.host_id` 一致性。
- 旧 `s.host IS NULL` 且无同 session IP/sniff 旁证的 4.4 GiB 保持未归因；不得用当前活动连接、全局 DNS 或字典猜测。
- 真实 controller 验收必须先确认运行 binary 含当前提交，再以新时间窗验证；旧 24 小时窗不能证明新 Host fallback 成功或失败。

## 7. Historical attribution quality 契约

`ReportResult` 增加后端计算的 `attributionQuality`，不由前端从 Top N 反算：

```text
AttributionQuality {
  knownUpload, knownDownload,
  missingUpload, missingDownload,
  knownConnections, missingConnections,
  status: complete | partial | unavailable
}
```

- `known + missing == totals` 对 upload/download 是硬守恒断言。
- `unavailable` 只用于该窗口有流量且 known bytes/connections 为 0；少量已知时为 `partial` 并显示精确覆盖，不设置任意百分比阈值。
- Ranking missing row 仍保留 `identity='__unknown__'` 和字节，不隐藏；前端按 grouping 使用“未归因主机 / 未报告链路 / 控制器未报告进程”等维度化 label。
- coverage（时间覆盖/gap）与 attributionQuality（维度字段覆盖）是两个独立轴，不能互相替代。
- 现有历史无法可靠区分“legacy”与“当时源端确实缺失”时，DTO 不伪造原因；UI 使用“未保存或未报告”组合文案。

## 8. Live observation 与双数据面 UI

`LiveOverview` 增加显式 `observationPhase`：

```text
unconfigured | connecting | baselinePending | current |
paused | disconnected | resyncRequired | decodeFailed
```

- connecting：计量卡显示“等待控制器连接”，active count 显示 `—`。
- baselinePending：显示“正在建立差分基线”；可显示当前 rows，但不把 null bytes 写成 Unknown。
- current：真实 0 才显示 0。
- paused/disconnected/resync/decodeFailed：当前值不可用；如保留 last-known，必须显示 stale 与 last sample。

IPC decoder 必须区分“字段存在且值为 null”和“字段缺失”：所有约定字段缺失均解码失败；显式 null 才是合法 pending/unavailable。

Overview 加清晰数据面标签：

- 顶部卡组：`实时 · 当前控制器`；
- 趋势/Top：`历史 · 已存储数据 · <时间窗>`，继续展示 report coverage 与 generated time。

因此 live connecting 与 historical 24h ready 可以同时出现而不矛盾。Live page 同步把 connecting 从 disconnected 中拆出，并在断连但保留 rows 时标 stale，不让 rowCount 优先冒充 current。

## 9. 兼容、安全与回滚

### 兼容

- 不修改已发布 C1/C3/C4 DDL 与 checksum；durable epoch、session attr、dimension 表均复用现有 schema。
- DTO 是同包 Rust/React 同步升级，decoder 严格拒绝旧/缺字段 payload；无需支持新前端连接旧 backend 的混装。
- 旧 raw 数据可读；只对有原始旁证的 Chain 派生层重建。Host/Process 旧 Unknown 保持守恒。

### 安全与隐私

- secret 不进日志、数据库、研究输出或 fixture。
- 真实 controller fixture 必须脱敏 host/IP/path，只保留字段存在性和结构。
- `processPath` basename 之外不进入历史维度；完整路径不得进入 attributionQuality 或错误文本。
- `providerChains` 未经版本 fixture 不参与真值。

### 回滚点

1. Durable epoch 修复可独立提交；新增 epoch 行无破坏性，回滚代码不会删除历史收据。
2. Canonical metadata 与 storage merge 在 Chain 重建前可直接回滚；已安全补全的 session attr 是同连接观测值，不主动抹除。
3. Chain raw/helper 可在物化重建前回滚。重建事务失败自动回滚；成功后数据来自现有 raw，可用相同工具按旧规则重建，但正常回滚不应把已知 DIRECT 再伪装 Unknown。
4. DTO/UI 可独立回滚，但不得回滚到“pending 显示 0/Unknown”而仍声称状态已修复。

## 10. 明确不采用的 Neko 行为

- 不迁移到 WebSocket；当前生产路径是 1 Hz HTTP GET，Unknown 与传输形式无直接证据。
- 不采用裸 `connection.id` 跨重连复用。
- 不采用首帧累计量直接计费。
- 不永久冻结 first-seen metadata；允许同 generation 内可信 late enrichment。
- 不把空 domain 从排名中删除；missing bytes 必须守恒。
- 不把空 chain/rule 无条件写成 DIRECT/Match。
- 不复制其 Process 实现，因为 Neko traffic pipeline 实际没有 Process 归因。
