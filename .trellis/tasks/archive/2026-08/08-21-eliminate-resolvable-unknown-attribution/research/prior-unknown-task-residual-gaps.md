# Research: 最近未知主机任务的覆盖范围与残余缺口

- Query: 审计已归档 `08-21-unknown-host-attribution`、相关本地提交证据、当前实现与测试；解释用户截图中 Top Hosts / Chains / Processes 仍出现 `Unknown` 的原因，并识别残余缺口、回归风险和可复用测试。
- Scope: internal（仓库源码、归档 Trellis 产物、本地只读 SQLite、vendored `ref/neko-master`）
- Date: 2026-08-21

## Findings

### 1. 结论摘要

上一任务不是“消除所有 Unknown”的任务，而是一个 **Host-only 修复**：

1. 新增主机 identity `host -> sniffHost -> destinationIP`；
2. 把解析结果写入 `connection_session.host`；
3. 允许同一 session 的空值 / IP 被后到域名升级；
4. 让 Host 的 `__unknown__` 可按规则、链路、进程下钻检查；
5. 改善 IP 标记和长标签显示。

归档 PRD 明确把“修复 Clash 不报进程名”排除在范围外，也明确不回填历史目的 IP、不重跑已物化小时 / 日未知桶（`.trellis/tasks/archive/2026-08/08-21-unknown-host-attribution/prd.md:17-24`, `.trellis/tasks/archive/2026-08/08-21-unknown-host-attribution/prd.md:27-33`）。因此用户截图里的三个 Unknown 不能视为同一种失败：

| 截图项目 | 只读数据库复核 | 结论 |
| --- | ---: | --- |
| Top Hosts `Unknown` 约 4.4 GiB | 20,692 sessions；895,321,222 B up + 3,807,849,948 B down = 4.380 GiB | 修复前历史数据；旧 session 没有存目的 IP，不能安全回填。截图数据库最后样本早于 Host 修复提交，不能据此判定新代码失败。 |
| Top Chains `Unknown` 约 4.7 GiB | 8,628 sessions；69,846,745 B up + 4,941,934,579 B down = 4.667 GiB | **可解析的假 Unknown**。这些行全部是单跳 `chain_key='DIRECT'`，被 `last_chain_hop` 错误折叠为 NULL。 |
| Top Processes `Unknown` 约 13.4 GiB | 37,127 / 37,128 sessions；约 13.363 GiB | 当前历史数据几乎没有 `process_id`。用户 Clash 截图中可见行的 Process 列也是空白，截图没有证明进程字段存在。若原始载荷只有 `processPath`，当前持久化仍会丢失可安全派生的 basename；原始载荷未抓取，故该分支仍是 `UNVERIFIED`。 |

顶部五张实时核算卡的 `Unknown` 又是第四种语义：截图明确为 “Connecting to the controller”、active connections 0、last sample “No sample”。断连 / 无样本时核算 DTO 保持 `None`（`residential-monitor/src-tauri/src/accounting.rs:152-166`），卡片用本地化 Unknown 渲染空 Option（`residential-monitor/src/components/features/overview/caliber-card.tsx:24`, `residential-monitor/src/components/features/overview/caliber-card.tsx:47-59`）。这不是 Host / Chain / Process 归因桶。

### 2. 归档任务与相关提交证据

归档任务 `task.json` 状态为 completed，但 `commit` 字段仍为 null（`.trellis/tasks/archive/2026-08/08-21-unknown-host-attribution/task.json:6-19`）。相关提交只能从 Trellis journal 与本地 reflog 文本重建：

| Commit | 本地记录 | 与本子任务关系 |
| --- | --- | --- |
| `c65daa4` | `chore(residential-monitor): ... 桌面构建去掉 cdylib 与 500kB 警告` | 无关构建清理 |
| `b0dee12` | `feat(residential-monitor): ... 重做英文侧栏品牌与底栏排版` | 兄弟任务 |
| `9dffefe` | `feat(residential-monitor): ... 用嗅探主机或目的 IP 归因空 host` | **Host 产品实现提交** |
| `db8d770` | `docs(residential-monitor): ... 记录主机归因与英文侧栏` | 文档 / spec 同步 |

本地 reflog 文本给出上述顺序和消息（`.git/logs/HEAD:175-178`）；Trellis journal 记录同一提交集合和整套门结果（`.trellis/workspace/lyh/journal-1.md:844-872`）。

语义差异可由归档前研究与当前源码交叉验证：修复前只读 `metadata.host`、`ensure_session_on` 不更新 host（`.trellis/tasks/archive/2026-08/08-21-unknown-host-en-sidebar/research/unknown-host-24h.md:32-37`）；当前代码已经读取 `sniffHost`（`residential-monitor/src-tauri/src/controller.rs:123-160`）、统一解析 Host（`residential-monitor/src-tauri/src/session_host.rs:18-43`）、投影到 live row（`residential-monitor/src-tauri/src/accounting.rs:77-113`）并在持久化时升级 `connection_session.host`（`residential-monitor/src-tauri/src/storage.rs:570-598`）。

归档工件有两个可追踪性缺口：

- PRD 的 AC1-AC6 在归档后仍全部是未勾选状态（`.trellis/tasks/archive/2026-08/08-21-unknown-host-attribution/prd.md:35-42`）。
- `task.json.commit` 未记录产品提交；后续审计必须依赖父任务 journal / reflog，而不能从子任务元数据直接定位。

### 3. 上一任务实际修复的端到端路径

#### 3.1 控制器解码与实时投影

- `ConnectionMeta` 新增 `sniff_host`，`normalize_connection` 从 `metadata.sniffHost` 读取（`residential-monitor/src-tauri/src/controller.rs:8-23`, `residential-monitor/src-tauri/src/controller.rs:123-160`）。
- `resolve_host_identity` 对 trim 后非空值按 `host -> sniffHost -> destinationIP` 选择（`residential-monitor/src-tauri/src/session_host.rs:14-27`）。
- `AccountingEngine::project_live` 把解析 identity 写入 `LiveConnectionView.host`，同时保留独立 `destination_ip`（`residential-monitor/src-tauri/src/accounting.rs:77-113`）。
- 前端实时表再以 `row.host ?? row.destinationIp` 做显示兜底（`residential-monitor/src/format/live-row.ts:52-78`）。

#### 3.2 持久化与查询

- 每帧先遍历 live rows 建 session、写 attr，再写分钟 fact，因而同一帧的 fact 可以复用当前 live 元数据（`residential-monitor/src-tauri/src/storage.rs:538-560`）。
- 已存在 session 使用 `prefer_host_identity` 更新 `connection_session.host`；空值不覆盖，IP 不覆盖域名（`residential-monitor/src-tauri/src/storage.rs:570-598`）。
- raw Host 排名直接读取 `s.host`，空值映射 `__unknown__`（`residential-monitor/src-tauri/src/c3/sql.rs:41-54`）。
- Host sentinel 的 raw 过滤匹配 `coalesce(s.host,'')=''`，精确层匹配 `dimension_id=0`，不把哨兵绑定为字典值（`residential-monitor/src-tauri/src/c3/sql.rs:310-345`, `residential-monitor/src-tauri/src/c3/sql.rs:388-409`）。

#### 3.3 前端检查能力

- Host `__unknown__` 可生成 host filter；非 Host Unknown 不生成过滤器（`residential-monitor/src/format/rank.test.ts:37-40`）。
- Host Unknown 在 cross-dimension 能力下出现下钻按钮；Process 等其它维度 Unknown 仍不可下钻（`residential-monitor/src/components/features/dimension/rank-table.test.tsx:70-100`）。
- IP identity、标签省略和动态 Y 轴宽度有纯函数与组件接线（`residential-monitor/src/format/rank.ts:26-58`, `residential-monitor/src/components/charts/rank-bar.tsx:62-71`）。

### 4. 为什么当前截图仍然显示这些 Unknown

#### 4.1 Host：截图数据库没有任何修复提交后的新事实

只读打开 `%TEMP%/io.github.bahayonghang.residential-monitor/monitor.sqlite3`，以数据库最大 `utc_minute` 为终点复核最后 24 小时：

- 窗口为 `2026-08-20T16:24:00+08:00 .. 2026-08-21T16:24:00+08:00`；
- 数据库最大样本终点为 `2026-08-21T16:24:00+08:00`；
- Host 产品提交 `9dffefe` 的本地 reflog 时间为 `2026-08-21T17:10:43+08:00`（`.git/logs/HEAD:177`）；
- `connection_session.started_utc >= 1787303443` 的 session / fact 数量均为 0。

也就是说，截图的历史库在修复提交前约 46 分钟已经停止新增数据；截图本身也显示控制器仍在连接。该库的 37,128 sessions、Host Unknown 20,692 sessions 及字节值与归档前研究完全一致（`.trellis/tasks/archive/2026-08/08-21-unknown-host-en-sidebar/research/unknown-host-24h.md:3-15`）。

上一任务明确不把旧 NULL host 猜成未存储的目的 IP（`.trellis/tasks/archive/2026-08/08-21-unknown-host-attribution/prd.md:23`, `.trellis/tasks/archive/2026-08/08-21-unknown-host-attribution/design.md:27-29`）。因此这一桶应保留为 legacy unknown；只有成功连接并采到 **新 snapshot** 才能验证新 Host 归因是否下降。`ingest_snapshot` 也只有在 Snapshot 分支才 project live、apply delta 并 commit（`residential-monitor/src-tauri/src/c2/facade.rs:594-632`）。

#### 4.2 Chain：单跳 DIRECT 被错误解释成 Unknown

`connection_session_attr.chain_key` 对上述 8,628 sessions 都是非空 `DIRECT`，而 raw chain 排名却调用 `last_chain_hop(a.chain_key)`（`residential-monitor/src-tauri/src/c3/sql.rs:89-101`）。`last_chain_hop` 的既定规则是：没有 `>` 的单跳字符串一律返回 None（`residential-monitor/src-tauri/src/c3/rule_name.rs:10-23`），并且单测显式固定了 `last_chain_hop("DIRECT") == None`（`residential-monitor/src-tauri/src/c3/rule_name.rs:104-113`）。所以这些完全可解释的 DIRECT 流量被稳定地产生为 `__unknown__`。

该函数最初是“规则 / 顶层策略组”派生函数：单跳时需要回退到 rule；它不适合作为 Chain identity。**不能直接把 `last_chain_hop` 改成单跳原值**，否则 `RULE_KEY_SQL` 会优先拿到 `DIRECT`，压掉 IPCIDR / DomainSuffix 等当前规则分组（`residential-monitor/src-tauri/src/c3/sql.rs:12-14`, `residential-monitor/src-tauri/src/c3/sql.rs:75-87`）。最小安全方向是为 Chain 新增独立语义：多跳取末跳，单跳取 trim 后原值，真正空数组才按已批准语义决定 `DIRECT` 或 Unknown；raw SQL、过滤和物化层必须共用它。

vendored Neko 也没有把单跳 direct 当未知：collector 为每条 batch 写 `chain: chains[0] || "DIRECT"`（`ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts:466-480`, `ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts:547-559`）。但 Neko 取第一项，本项目展示顶层组时取末项；只能借用“单跳 / 空值有明确 identity”的机制，不能直接复制数组方向。

#### 4.3 Process：截图与历史库都没有证明存在可用 process name

当前控制器模型会读取 `metadata.process` 与 `metadata.processPath`（`residential-monitor/src-tauri/src/controller.rs:137-160`），实时 DTO 两者都保留（`residential-monitor/src-tauri/src/accounting.rs:96-112`），但持久化只 intern `row.process_name`，完全不使用 `process_path`（`residential-monitor/src-tauri/src/storage.rs:639-677`）。因此：

- 原始 `process` 非空：可以归因；
- `process` 空、`processPath` 非空：当前仍归为 Unknown，存在可安全派生 basename 的潜在信息损失；
- 两者都空：只能保留 Unknown，不能从 Host / Source / 当前活动连接猜进程。

本次历史库只证明几乎所有 `process_id` 为 NULL；库中不保存原始 processPath，无法回溯判断是哪一分支。用户 Clash 截图的可见行 Process 列为空，故“Clash 当前连接没有 Unknown”至少不成立于 Process 维度。vendored Neko 的共享类型虽声明 `process` / `processPath`（`ref/neko-master/packages/shared/src/index.ts:3-29`），Gateway collector 的 traffic update 并未采集这两个字段（`ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts:436-480`），所以 Neko 也不能作为“Process 一定可归因”的证据。

### 5. 残余实现缺口与回归风险

#### P0：Chain 的单跳语义错误（已由截图字节闭环）

- `DIRECT` 是已知 chain，却被 raw / hourly chain 两层都映射成 Unknown（`residential-monitor/src-tauri/src/c3/sql.rs:89-101`, `residential-monitor/src-tauri/src/c3/retention.rs:269-284`）。
- 现有 `five_kinds_materialize_and_keys_match_raw` 只验证 raw 与 dimension 两层 keys 相同；两层复用同一错误函数时仍会通过（`residential-monitor/src-tauri/src/c3/service.rs:1149-1198`）。这是“一致地错误”的测试盲区。
- 直接改全局 `last_chain_hop` 会回归 Rule grouping；必须拆出 Chain 专用 identity。

#### P1：session attr 会被后续空值整体擦除

`intern_and_attr` 的 upsert 无条件令 `host_id/process_id/rule_id/network_id/chain_key/category_id = excluded.*`（`residential-monitor/src-tauri/src/storage.rs:639-677`）。如果同一连接某帧提供 Chain / Process、后一帧暂时缺字段，后一帧会把已知 attr 覆盖成 NULL；raw 查询再把该 session 的 **全部历史分钟** 归为 Unknown。上一任务只为 `connection_session.host` 加了单独的非空升级逻辑，没有为 attr 做同等合并。

Host 也存在 raw / 物化层分裂风险：raw Host 排名读 `s.host`（`residential-monitor/src-tauri/src/c3/sql.rs:41-54`），小时 Host 物化读 `a.host_id`（`residential-monitor/src-tauri/src/c3/retention.rs:218-233`）。若后续空帧保住 `s.host` 却擦除 `a.host_id`，同一 session 在 raw 层已知、在物化层 Unknown。本次库未出现该不一致，但代码路径允许发生。

#### P1：Host 强度只有“IP / 非 IP”，没有来源 provenance

设计声称显式 host 强于 sniffHost，sniff domain 强于目的 IP（`.trellis/tasks/archive/2026-08/08-21-unknown-host-attribution/design.md:19-27`），但持久层只收到最终字符串。`prefer_host_identity` 对两个不同的非 IP 字符串总是采用 incoming（`residential-monitor/src-tauri/src/session_host.rs:29-43`）；因此某帧显式 host、后一帧 host 缺失但 sniffHost 不同，会把显式 host 替换成 sniffHost。现有测试甚至把 `old.test -> new.test` 固定为允许行为（`residential-monitor/src-tauri/src/session_host.rs:77-98`），没有 provenance 场景。

#### P1：Process 只有 name 路径，且没有同 session 继承

- `processPath` 已解码但不参与持久化；若仅路径存在，可能丢失可用 basename。
- 后到 process name 或暂时缺失 process name 没有端到端回归。
- 不能把当前活动连接的 process 分摊给旧历史；旧 Unknown 必须保留。

#### P1：一个 session 只有一份可变 attr，历史分钟会随“最终 attr”重分类

分钟 fact 只存 `session_pk + bytes`，所有维度通过单行 `connection_session_attr` 联接（`residential-monitor/src-tauri/src/storage.rs:550-559`, `residential-monitor/src-tauri/src/c3/sql.rs:56-73`）。这依赖“同一 controller connection id 的 Host / Chain / Process 在生命周期内稳定，变化只属于 late fill”这一假设。若 proxy chain 确实在同一连接中变化，保留 last-known / strongest 会把之前分钟一起重分类；现有 schema 无法表达 per-minute attribute version。新设计必须明确此边界，不能暗示字节级时序精度。

#### P2：控制器未连通状态与历史 Unknown 混在同一视觉词汇

实时卡片、历史排名、字段缺失、legacy 不可回填都显示 “Unknown”，但它们的可操作性不同。当前 spec 只要求 unknown / gap 不画成零（`.trellis/spec/residential-monitor/frontend/view-state.md:12-13`）。新任务应在 DTO / UI 层区分 `no sample / connecting`、`missing metadata`、`legacy unresolved` 和真正的 rank sentinel，而不是把历史桶隐藏或改名成已知。

### 6. 现有测试审计与可复用测试

#### 已有且可复用

| 测试 | 已证明 | 未证明 |
| --- | --- | --- |
| `controller_model_reads_sniff_host_when_host_empty`（`controller.rs:223-249`） | `sniffHost` / destinationIP 解码 | 多帧、后到字段、连接关闭、真实 Mihomo payload |
| `project_live_resolves_sniff_host_and_destination_ip`（`accounting.rs:330-359`） | stateless live fallback | fact 写入、SQLite round-trip、排行 |
| `resolve_prefers_host_then_sniff_then_ip` / `prefer_upgrades...`（`session_host.rs:51-99`） | 字符串优先级、IP 检测 | 显式 host 与 sniffHost provenance |
| `persist_upgrades_empty_and_ip_host_but_not_domain`（`storage.rs:1486-1507`） | 直接调用 storage 时 `s.host` 升级 | 同一 snapshot 的 fact、attr 保留、report 查询、重启 |
| `unknown_host_filter_matches_empty_host_without_binding_sentinel`（`c3/sql.rs:468-487`） | SQL fragment 与参数形状 | 真库执行、raw / dimension 一致性 |
| `five_kinds_materialize_and_keys_match_raw`（`c3/service.rs:1149-1214`） | raw / materialized key parity | key 本身语义正确；DIRECT 正被两层共同误判 |
| `rank.test.ts` / `rank-table.test.tsx` | IP 显示、Host Unknown 下钻 | Chain / Process Unknown 的语义与来源 |

Trellis journal 记录当时 `cargo test --workspace` 267 passed、Vitest 187、typecheck/lint/build 与 clippy 通过（`.trellis/workspace/lyh/journal-1.md:870-872`）。这些是已有执行证据，本次研究角色没有重跑。

#### 下一任务应新增的回归

1. **Chain semantic oracle**：`chain_key='DIRECT'` 在 raw 与 hourly rank 都必须是 `DIRECT`，不是 `__unknown__`；`node>Proxy` 为 `Proxy`；NULL / empty 的策略单独断言。不要只比较两层互相相等。
2. **Rule non-regression**：修 Chain 后，单跳 `DIRECT + rule=IPCIDR` 的 Rule rank 仍为 `IPCIDR`，不得被 Chain helper 改成 `DIRECT`。
3. **同 session 单调元数据**：known Host / Chain / Process -> 下一帧字段空 -> 产生 delta；已知 attr 不得退化为 Unknown。
4. **late metadata**：首帧 / 首个 delta 缺元数据，后帧得到可信元数据；在现有“session-wide attr”语义下验证之前与之后分钟的明确预期。
5. **Host provenance**：显式 host -> 仅 sniffHost；IP -> sniff domain -> explicit host。断言来源强度而非仅字符串类型。
6. **Process path policy**：若批准 basename fallback，覆盖 Windows 路径、Unix 路径、空 basename、超长值与隐私边界；若不批准，则明确 `processPath only -> Unknown`。
7. **epoch / connection isolation**：同 connection id 在 core restart / app restart边界不得继承另一生命周期的元数据。
8. **legacy database**：旧 NULL Host / Process 不猜测回填；旧数据仍可读并显示 legacy unresolved 说明。
9. **端到端 fixture**：从 Mihomo JSON -> normalize -> accounting delta -> SQLite -> raw / hourly report -> frontend decoder，至少覆盖 sniff host、single-hop DIRECT、process missing 和暂时空字段。
10. **真实控制器 gate**：成功连接后采新窗口，对齐 Clash Verge 当前连接与 monitor 新增 session；旧 24h 历史不得拿来证明新采集失败或成功。

### 7. 对 `ref/neko-master` 的可复用边界

vendored `neko-master` v1.4.0（`ref/neko-master/package.json:2-3`）提供三个可复用点：

- Host 候选明确为 `host || sniffHost`，IP 独立保存（`gateway.collector.ts:436-440`）；本项目把 IP 合入 Host identity 是自己的 schema 权衡，不是 Neko 原样行为。
- batch chain 对缺少首项明确回退 `DIRECT`（`gateway.collector.ts:466-480`, `gateway.collector.ts:547-559`），证明单跳 direct 不应展示为未知。
- Neko 为 existing connection 的 delta 复用 `existing.domain/ip/chains/rule`（`gateway.collector.ts:513-559`），避免暂时缺字段擦除已知值。

但不能直接复制：

- Neko 在首次看到连接时冻结 metadata，后续没有 stronger-host merge（`gateway.collector.ts:445-464`, `gateway.collector.ts:513-559`）；本项目需要 late metadata，必须做带来源强度的单调合并。
- Neko chain 用 `chains[0]`，本项目当前 Top Chain 是顶层末跳语义；数组方向必须通过 Mihomo fixture 与 UI 口径确认。
- Neko Gateway traffic collector 不写 Process，不能拿它证明 Process 的优化方案。

## Files Found

- `.trellis/tasks/archive/2026-08/08-21-unknown-host-attribution/{task.json,prd.md,design.md,implement.md}` — 上一 Host-only 子任务的范围、设计、验收与风险。
- `.trellis/tasks/archive/2026-08/08-21-unknown-host-en-sidebar/research/unknown-host-24h.md` — 修复前本机 24h SQLite 统计与不可回填边界。
- `.trellis/workspace/lyh/journal-1.md` — 相关提交集合与历史门结果。
- `.git/logs/HEAD` — 未调用 Git 命令时用于确认本地提交顺序、消息与时间的 reflog 文本。
- `residential-monitor/src-tauri/src/{controller.rs,session_host.rs,accounting.rs,storage.rs}` — 原始字段、Host fallback、delta 与 session attr 持久化链路。
- `residential-monitor/src-tauri/src/c2/facade.rs` — snapshot 到 live / facts / commit 的编排。
- `residential-monitor/src-tauri/src/c3/{sql.rs,rule_name.rs,retention.rs,service.rs}` — raw / materialized ranking、Chain identity 与相关测试。
- `residential-monitor/src/{format,components}/**` — Unknown / IP / live fallback 与排名展示测试。
- `ref/neko-master/apps/collector/src/modules/collector/gateway.collector.ts` — Neko 的 Host、IP、Chain fallback 与 connection-state reuse。
- `ref/neko-master/packages/shared/src/index.ts` — Neko Gateway connection metadata 类型，含 process / processPath。

## Code Patterns

- 数据流应按 `Source -> Transform -> Store -> Retrieve -> Transform -> Display` 全链路验证；本问题同时跨 DTO、session state、SQLite 与 React（`.trellis/spec/guides/cross-layer-thinking-guide.md:21-33`）。
- `connection_session.host` 与 `connection_session_attr.host_id` 当前有两个主机权威投影；任何 merge 方案都必须验证两者 round-trip 一致。
- Chain 的“规则分组末跳”和“链路 identity”是两个不同概念；共享一个 `last_chain_hop` 会把单跳 DIRECT 错判为缺失。
- nullable metadata 的 upsert 应采用字段级单调 merge，而不是整行 `excluded.*` 覆盖；否则暂时缺字段会回溯性污染所有 session minutes。
- `Unknown` 必须表示真实缺失，不能用于已有值的派生失败，也不能用当前连接猜测不可重建历史。

## External References

- 未访问网络资料。本次只使用仓库 vendored `neko-master` 源码快照；根 package 声明版本 `1.4.0`、pnpm `9.15.9`（`ref/neko-master/package.json:2-10`）。
- 当前 `residential-monitor` package 版本为 `0.2.0`（`residential-monitor/package.json:2-3`）；SQLite 总 schema version 为 4（`residential-monitor/src-tauri/src/c0_contract.rs:23-25`）。

## Related Specs

- `.trellis/spec/residential-monitor/backend/modules-and-errors.md:13` — 上一任务固化的 Host identity / sentinel 契约。
- `.trellis/spec/residential-monitor/storage/sqlite-contract.md:25` — 五维物化、dimension 0 与 `__unknown__` 契约。
- `.trellis/spec/residential-monitor/frontend/view-state.md:12-13` — unknown / gap 不得画成零，Host Unknown 下钻与 IP 标记。
- `.trellis/spec/guides/cross-layer-thinking-guide.md:21-50` — 跨层格式、NULL 与责任边界检查。

## Caveats / Not Found

- Trellis research 角色禁止任何 Git 操作，因此本次没有运行 `git log`、`git show`、`git diff` 或测试命令。提交信息来自只读 reflog 文件与 journal；“9dffefe 的精确文件 diff”未用 Git 对象独立复核。
- 当前工作树可能包含提交后的变化；本报告以现有源码、归档前研究与提交消息三方交叉验证语义，不把现有每一行都断言为只来自 `9dffefe`。
- SQLite 使用 `mode=ro` 读取开发态 `%TEMP%` 数据库；该 snapshot 最后样本早于 Host 修复提交，不是修复后真实控制器验收。
- 未取得新的原始 `/connections` JSON，无法判断 Process Unknown 中有多少是 `processPath-only`、多少是两字段都缺失；该分解保持 `UNVERIFIED`。
- 未验证安装态 / 真实 WebView 当前是否加载包含 `9dffefe` 的 binary。截图“Connecting”只证明当时未完成采样。
- Neko vendored snapshot 的来源 commit / tag 未通过 Git 操作核对，仅按 `package.json` 的 1.4.0 记录版本。
