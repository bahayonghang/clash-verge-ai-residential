# 消除可解析连接的 Unknown 流量归因

## Goal

让 `residential-monitor` 只把经证据确认无法归因的流量保留在缺失桶中：控制器已提供或同一连接生命周期内曾提供的 Host、Chain、Process 等可信元数据必须被保留并用于历史归因；实时尚未连接/建立样本、可修复的派生错误、源端未报告和旧历史不可恢复必须分别表达。

用户价值：监控结果与 Clash/Mihomo 可观察事实一致，`Unknown` 不再掩盖代码丢字段、重启跳写或 `DIRECT` 误分类，也不会通过猜测、隐藏或改名伪造“已全部归因”。

## Confirmed Facts

### 现场与数据库

- 用户截图中 live Header 为 “Connecting to the controller”，active connections 为 0、last sample 为 No sample；同时 historical 24h 趋势和排名仍有数据。两者来自不同数据面，可以同时成立。
- 对 `%TEMP%/io.github.bahayonghang.residential-monitor/monitor.sqlite3` 的只读 24h 审计按截图精确复现：Host missing 4,703,171,170 B（约 4.4 GiB）、Chain UI Unknown 5,011,781,324 B（约 4.7 GiB）、Process missing 14,348,735,564 B（约 13.4 GiB）。详见 `research/local-database-unknown-audit.md`。
- Host missing 来自 20,692 个旧 session；库中未保存这些 session 的 destination IP/sniff host。数据库最后样本早于上一 Host 修复提交，旧窗口不能证明新 fallback 成功或失败，也无法无损回填。
- Process 37,128 个 session 中只有 1 个拥有 `process_id`（`mihomo`，9,923 B）；用户 Clash 截图的 Process 列可见单元格同样为空。现有证据不支持把其余流量猜成进程。
- Chain 并未缺失：该窗口所有 session 均有非空 `chain_key`；约 4.7 GiB 全部是单跳 `DIRECT`。`last_chain_hop` 为 Rule 分组故意对单跳返回 None，Chain raw/filter/物化错误复用它并产生 `__unknown__`。

### 代码与参考实现

- 产品采集是约 1 Hz HTTP GET `/connections`，不是生产 WebSocket。
- 上一 `08-21-unknown-host-attribution` 只实现 `host -> sniffHost -> destinationIP` 和 Host 检查；明确排除 Process、旧 Host 回填和旧物化重跑。
- `intern_and_attr` 当前每帧整行覆盖 host_id/process_id/rule_id/network_id/chain_key；后帧临时缺字段会把同一 session 曾经已知的属性清空，并回溯性污染该 session 的全部 raw minutes。
- `processPath` 已解码但不参与统计；`providerChains` 已解码但缺乏已验证语义，不得按名称猜测为 chains。
- 每次 `AppFacade::boot` 都设置 `writer_epoch=1,bundle_seq=1`，生产 bundle payload 恒为空。现有库已保存 epoch 1、seq 1..68211；重启后同 key/同空 hash 在 `persist_slice` 前返回 Duplicate，导致新 facts/metadata 长时间不落库。
- `AccountingEngine` epoch 每进程从 0 开始，生产 collector 不持续探测 core restart；跨重启/ID 复用可能命中旧 session。
- `ref/neko-master` 可借鉴同连接 delta 复用已有 metadata；不可原样采用裸 id、first-seen 永久冻结、首帧累计量计入、空 domain 隐藏、空 chain/rule 默认 DIRECT/Match。其 traffic pipeline 没有 Process 归因实现。

## Requirements

### R0. 持久化生命周期完整性

- 每次可写应用生命周期获得不会与旧 receipt 冲突的 durable writer epoch；该 epoch 内 sequence 从 1 开始。
- 每个可信 controller generation 获得 durable controller epoch；跨应用重启、重连或可信 reset 信号不得复用旧连接 session。
- 新启动后的第一份可持久化帧必须立即写入；PayloadMismatch、过期 receipt 和 storage error 不得静默吞掉。

### R1. 严格且兼容的 controller 边界

- 根 frame 的 meter/connections 缺失或类型错误必须成为可观测的不完整/协议错误，不能静默变成零 meter 与完整空列表。
- 可选 metadata 缺失保持兼容；字符串 trim、空 chain 元素过滤；未知附加字段忽略。
- 记录不含敏感值的字段存在性统计，用于区分 source missing 与 runtime drift；不得记录 secret、完整 host/IP/path。

### R2. 同 connection generation 的 canonical metadata

- 由一个 backend owner 在 `epoch:id` 内合并 metadata，再用同一结果生成 live rows、delta 与 storage input。
- Host 来源优先级为 explicit host > sniffHost > destinationIP；空值不降级，域名不被 IP 覆盖，sniff 不覆盖已有 explicit host。
- Process 为 `process > basename(processPath) > existing > missing`；完整 path 不进入历史维度/日志。
- 非空 Chains 作为整组更新，空帧继承；Rule/Network 等非空更新、空帧保留。generation 结束后全部隔离。
- 保持 observed lower bound：首帧只建 baseline、counter reset/连接消失不发明流量。

### R3. SQLite 防御性保真

- host_id/process_id/rule_id/network_id/chain_key 采用维度特定的 non-empty merge；当前空帧不得清除已知值。
- host_id 与 `connection_session.host` 保持一致；raw 与 materialized Host 不得分裂。
- `connection_chain` 与 `chain_key` 对同一 canonical chain 保持单一真相；链变化时整组替换，不拼接不同帧。
- primary category 继续服从当前策略，不套用不可撤销的通用 coalesce。

### R4. Chain identity 与历史派生修复

- Chain identity 独立于 Rule helper：空 -> missing，单跳 -> 原值（含 `DIRECT`），多跳 -> 当前产品约定的末级策略组。
- raw rank、filter、字典 intern、hourly/daily materialization 共用该语义；Rule 单跳仍回退 raw rule，不得回归。
- 对仍保留 raw 的时间窗事务性 delete+rebuild Chain hourly/daily 并验证守恒；raw 已删除和 frozen report archive 不猜测改写。

### R5. Host/Process 可恢复边界

- 新 Host/Process 只从同一连接的可信字段或 path basename 补全；不得从当前活动连接、DNS、全局字典或唯一已知进程推测旧历史。
- 旧 Host NULL 与旧 Process NULL 无旁证时保持 missing；真实 controller/binary 未验证前不得承诺可消除比例。
- 不自动修改 Clash/Mihomo process discovery 配置。

### R6. Historical attribution quality

- 后端按当前 grouping 返回 exact known/missing upload/download/connections 与 complete/partial/unavailable；Top N 不影响该统计。
- `known + missing == totals` 为硬守恒；时间 coverage 与维度 attribution quality 分开。
- Missing ranking row 保留字节和 `__unknown__` identity；UI 使用 Host/Chain/Process 维度化文案，不隐藏守恒桶。

### R7. Live observation 与双数据面 UI

- LiveOverview 显式区分 unconfigured、connecting、baselinePending、current、paused、disconnected、resyncRequired、decodeFailed。
- 只有 current 状态的 0 才显示真实 0；pending/unavailable/stale 显示原因和 last sample，不使用通用 Unknown。
- Overview 明示 `实时 · 当前控制器` 与 `历史 · 已存储数据 · 时间窗`；live 断连不清空或误述历史报告。
- IPC decoder 区分 present-null 与 missing field；缺约定字段解码失败。
- Live page 中 connecting 与 disconnected 分开，断连保留旧 rows 时标 stale。

### R8. 验证与证据边界

- 自动化覆盖重启 receipt、controller epoch、metadata missing→known→temporary missing、DIRECT/单跳/多跳/空 Chain、Rule non-regression、processPath basename、raw/materialized守恒、DTO/UI 状态矩阵。
- 真实 controller、当前安装包、新样本写入、Mihomo 版本/模式字段覆盖与 native WebView 状态必须独立人工验收；未执行时标 `UNVERIFIED`。

## Acceptance Criteria

- [x] AC0 (R0)：给定已存在 `(writer_epoch=1,bundle_seq=1..N)` 和旧 `(epoch_id,connection_id)` 的库，新实例首份有效帧返回 Applied、data_version 前进并创建新 session；不得先返回 N 次 Duplicate或写进旧 session。
- [x] AC1 (R1)：缺 `connections`、meter 类型错误或 incomplete frame 不发布完整空快照、不关闭 active sessions，并产生稳定脱敏诊断；合法可选 metadata 缺失仍兼容。
- [x] AC2 (R2/R3)：同 generation 的 rich→empty 不降级，empty→rich 即使 delta=0 也补全；new generation 不继承。Host/Process/Chain 在 live、attr、raw query 中一致。
- [x] AC3 (R2/R5)：`process=None, processPath=Some(path)` 只归因安全 basename；两者皆空继续 missing。Windows/Unix/空 basename/路径隐私有测试。
- [x] AC4 (R4)：`chains=["DIRECT"]` 在 raw rank、filter、hourly、daily 都为 DIRECT；单跳代理为自身、多跳保持末跳、空链才 missing；`DIRECT + rule=IPCIDR` 的 Rule rank 仍为 IPCIDR。
- [x] AC5 (R4)：旧 Chain 派生重建在临时数据库副本中前后 upload/download 完全守恒，旧 dimension_id=0 DIRECT 行不残留、不双计；失败自动 rollback。
- [x] AC6 (R5/R6)：旧 Host 4.4 GiB 与无旁证 Process missing 不被伪回填；报告精确返回 known/missing，TopN 改变不影响 quality，known+missing 等于 totals。
- [x] AC7 (R7)：connecting 时计量卡写等待连接、active 显示 `—`，历史趋势/Top 继续显示且标为历史；baseline/current/disconnected/stale/协议错误均有独立组件与 decoder 测试。
- [x] AC8 (R6/R7)：Process 覆盖极低时 UI 显示“控制器未报告/部分覆盖”与精确守恒量，少量已知进程仍可查看；不得把整个窗口包装成普通 Unknown 第一名或隐藏 missing bytes。
- [x] AC9 (R8)：`just monitor-check`、`just ci`、`git diff --check` 通过；真实 controller/安装态检查逐项给出 PASS/FAIL/UNVERIFIED，不用 fixture 冒充。
- [x] AC10：任务包含收敛后的 `prd.md`、`design.md`、`implement.md`、真实 implement/check manifests 与研究记录；未经用户后续明确批准，不运行 `task.py start`，不修改产品代码。

## Out of Scope

- 修改只读 `ref/neko-master`、迁移到 WebSocket或引入其 ClickHouse/BatchBuffer 架构。
- 用启发式、当前连接、DNS、唯一进程值或比例分摊重写无旁证旧历史。
- 自动改 Clash/Mihomo 配置、清空用户数据库、VACUUM、重建成功 frozen report archive。
- 新增 GeoIP/ASN/独立 IP 页，或把 monitor 变成 Clash Verge 连接页克隆。
- 在本轮规划中启动实现、提交、归档或推送。

## Planning Decision

采用“真实性优先”方案：确定修复 writer/epoch、metadata 保留和 DIRECT 误分类；对不可恢复 Host 与源端未报告 Process 提供结构化缺失/覆盖表达，不承诺 Unknown 全部归零。该决策需用户在最新规划摘要之后明确批准，才进入实现。
