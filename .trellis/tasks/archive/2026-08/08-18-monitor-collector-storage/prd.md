# C1：家宽监控采集核算与 core SQLite

## 目标

在 C0 已批准的 binding、性能预算、协议 profile、CredentialStore port 和运行 limits 内，交付无头采集内核：从 mihomo 输入产生可解释、可崩溃恢复、幂等持久化的 core 连接事实和覆盖区间，并向后续桌面层提供原子实时水位与最小恢复后端。

C1 的成功标准不是「能写入 SQLite」，而是进程 kill、存储压力和控制器生命周期变化后仍做到：不重复累计、已确认提交不丢失、所有无法保证完整性的区间显式形成 gap。

## 前置依赖与启动限制

- C0 必须完成并获批：SQLite binding 与实际版本、WAL/FULL 能力、prepared batch 上限、writer 周期、busy deadline、队列与 frame/body limits、固定 `A / L / C / q`、维度基数与每帧变化比例的 `A=250` 设计预算、全部连接计数变化的 10k 峰值结论、控制器 profile、CredentialStore port 和 fallback。
- C1 启动前必须把 C0 的最终决策证据加入实施/检查上下文。只有当前父任务研究的 manifests 不是启动许可。
- 任一 C0 必选决定仍为未批准、`reject` 且无 fallback，或 C0 支持范围无法覆盖 C1 gate 时，C1 保持 `planning`。
- 本轮只授权完善规划，未授权运行 `task.py start` 或编写产品代码。

## 交付范围

### R1. `ControllerSession`

- 通过小接口隐藏发现、TCP/named pipe、HTTP/WebSocket、鉴权、重连、超时、取消和版本兼容。
- TCP 只接受 loopback，按 C0 `CredentialStore` port/fake resolver 注入 secret；secret 不进入 URL、日志、SQLite、Channel、错误或 fixture。
- named pipe 只使用 C0 批准的 best-effort profile，连接前验证 server 身份，不发送 secret；ACL denied、busy timeout、not found、PID mismatch 和协议不兼容保持独立状态。
- 手动配置优先于自动发现；核心身份、运行模式或连续失败变化时重新发现，不永久缓存私有管道名。
- normalizer 对未知字段宽容，对版本相关字段可空，对 frame/body 和字符串长度执行 C0 limits。
- 提供 replay adapter，使核算测试不 mock HTTP 内部细节。

### R2. `AccountingEngine`

- 作为不执行 I/O 的纯状态机消费 snapshot、connected、restart、disconnect、sleep gap、pause、settings change 和 shutdown。
- 使用 `(controller_epoch, connection_id)` 形成内部会话身份；首帧只建立 baseline，不把应用启动前累计计入历史。
- 相邻快照按非负 delta 核算；连接消失不伪造尾部字节；计数器回退、ID 重用和核心重启结束旧 baseline 并产生质量/覆盖事件。
- 速率使用单调时钟，持久时间使用 UTC integer；跨分钟按单调时间比例分配且字节守恒。
- `attributed_observed`、唯一主分类、其他连接、controller meter、正向 gap 和 over-attributed 使用同一 frame/epoch/direction；reset 首帧为未知，不写 0。
- target set 有序且版本化。唯一主分类只由用户优先级决定，全部命中保留为标签；不得依赖 `chains` 首尾。
- 只为非零 delta 或必要质量事件生成稀疏 minute facts；零流量活跃连接不每分钟写零。

### R3. core SQLite schema 与 migration

C1 只创建采集内核需要的逻辑表族：

- migration history、schema/data version 和 committed bundle/durable watermark；
- machine setting/reference；
- controller epoch；
- target set、target item；
- coverage interval；
- connection session、connection chain、sparse connection minute；
- migration backup manifest/primitive。

约束：

- 物理列、索引、页成本和 limits 必须服从 C0 批准结果；
- 优先 STRICT、foreign keys、UTC integer 和参数化 SQL；
- migration 在 collector/writer/query 启动前执行；已发布 migration 不可修改；
- checksum mismatch 和数据库版本高于应用支持上限时 fail closed；
- migration 前使用 SQLite Online Backup primitive，不复制 hot 数据库文件；
- 不创建 hourly/daily/report/retention 表，不创建 alert/notification 表，不预留语义未定的空表。

### R4. prepared writer、幂等与 durable watermark

- 单 writer 缓存 prepared statements；逐行只执行 bind → step → reset。
- Accounting 完成生命周期与 delta 核算后，才允许按 `(session_id, utc_minute)` 合并；同一 transaction 每个 minute key 最多一次写。
- 每个 `CommitBundle` 有稳定唯一 `bundle_id` 和可核验 payload identity。
- 同一 `FULL` transaction 原子登记 bundle、写 core facts/coverage、推进 `data_version` 与 durable watermark。
- producer 为每个 writer epoch 生成连续 bundle sequence，并承诺只在冻结的 retry window 内重试；commit 结果不确定或 receipt 未送达的 bundle 必然位于窗口内。
- retry window 初始至少保留最近 24 小时与 100,000 个 receipt 中覆盖更大的集合，最终值由 C0 / C1 页成本与故障测试冻结。窗口内相同 bundle 返回原 `CommitReceipt`，不得再次累加；相同 `bundle_id` 携带不同 payload 时 fail closed。
- 超出窗口的历史 receipt 只有在 epoch 的 highest contiguous sequence / durable watermark 已覆盖、超过时间与数量安全线且没有进程内引用时才可小批量删除；epoch summary 与 hash 证据保留。旧 bundle 重试返回 `RetryWindowExpired`，永不重新执行。
- commit 结果不确定或 commit 后 receipt 未送达时，重试同一 `bundle_id`；不得生成新 ID 猜测重放。
- 返回 `CommitReceipt` 只发生在 durable commit 成功后。已确认提交在重启后必须可查询。

### R5. 有界队列、backpressure 与 gap

- collector 输入、待核算 frame 和待写 batch 均有 C0 冻结的显式上限；内存不得随存储阻塞无界增长。
- 原始 snapshot 在生命周期/delta 核算前不得通过 latest-only 静默丢弃。
- 只有核算后才可按相同 minute key 合并待写事实。
- oldest batch 超时、队列饱和、busy 超过 deadline、WAL 不可用、checkpoint starvation、磁盘满或 I/O failure 时，health 进入明确 degraded/storage 状态，并停止宣称完整 coverage。
- 无法继续保留事实时，先捕获 gap 起点；恢复后持久化 degraded interval。允许显式 gap，不允许静默 gap。
- 正常 shutdown 执行停接帧、flush、结束 coverage、受控 checkpoint 和关闭数据库。

### R6. 原子 live projection 与 Channel 水位

- 基础 live projection 只在 `CommitReceipt` 后推进 durable 数据版本，不实现完整 UI。
- `seq` 是当前 Rust 进程内全局单调序号。
- 订阅者登记、snapshot 捕获和 `baseSeq` 读取在同一 state lock/原子临界区完成；第一条消息必须是 `bootstrap { snapshot, baseSeq }`。
- 后续消息只允许 `seq > baseSeq`。前端发现 gap 后停止应用 delta，并通过 `resync_monitor` 获取新的原子 bootstrap/watermark。
- Channel 可 coalesce/latest-only 的仅是可重建投影通知，不得借此丢弃采集或持久化业务事实。

### R7. `RecoveryFacade` backend

- 正常 schema 不可打开、future schema、checksum mismatch 或 migration失败时，不启动 collector/writer/query。
- 独立的最小 backend 可读取应用/SQLite/schema 版本和脱敏诊断，列出 migration backup，并验证候选 checksum、integrity 和兼容版本。
- C1 不执行用户恢复、不 swap 当前库、不实现恢复 UI；C2/C3 在该 backend 上分别交付 Recovery Shell 和实际恢复命令。
- RecoveryFacade 不依赖正常业务查询、报告表或告警表。

### R8. 自动验证与性能 gate

- replay 覆盖首帧、数组乱序、连接消失、负增量、重启、ID 重用、断线、休眠、跨分钟/小时、DST 边界和 target 变化。
- 属性/守恒测试证明分类 + 其他 = attributed observed，跨桶和跨方向总量守恒，缺口不产生零事实。
- migration 覆盖空库、重复启动、上一 schema、迁移中断、checksum mismatch、future schema 和 migration backup。
- kill 测试必须覆盖：commit 前、commit 结果不确定、commit 后未回执。
- 性能 gate 必须实际运行 10,000 活跃、1 Hz、全部连接计数每帧变化、至少 30 分钟峰值，以及 C0 固定完整 workload tuple 生成的 `A=250` 完整 30 天设计库。
- fault gate 必须覆盖 checkpoint starvation、WAL 回落/不可保持 WAL、busy deadline、磁盘满和 I/O error。
- 所有 gate 使用 C0 批准的 SLO；验收要求零重复 bundle、零重复字节、零已确认漏账、零静默 gap。故障造成的显式 coverage gap 可以存在，但必须可查询且原因正确。

## 可观察验收标准

- [ ] AC1：录制帧通过 `ControllerSession` TCP 和批准的 pipe profile 产生同一版本化输入；鉴权、ACL、busy、not found、PID mismatch、协议错误和取消状态可区分，secret 扫描为零。
- [ ] AC2：replay/属性测试覆盖 R8 生命周期；每批满足分类守恒与跨桶字节守恒，首帧/reset 未知，连接消失不伪造尾差，gap 不写 0。
- [ ] AC3：从空库启动只创建 R3 core 表族；schema 清单中没有 report/hourly/daily/retention/alert/notification 表。
- [ ] AC4：空库、重复启动、上一 schema、中断、checksum mismatch 和 future schema 行为可重复；失败时 collector 未启动，migration backup 可列出并验证。
- [ ] AC5：prepared statement 复用和 transaction 内 per-key merge 有自动计数/断言；测试证明没有逐行 prepare 或无界 SQL。
- [ ] AC6：commit 前 kill 后没有伪造 receipt；恢复时从最后 durable watermark 建立显式 gap，不重复已有事实。
- [ ] AC7：commit 结果不确定和 commit 后未回执 kill 后，以相同 `bundle_id` 重试只得到一个 committed bundle/receipt，字节与 data version 只推进一次。
- [ ] AC8：10k/1Hz/30m 且全部连接计数每帧变化的峰值满足 C0 批准的计算、durable commit、CPU、RSS 和队列 SLO；没有无界增长、重复 bundle 或静默 gap。
- [ ] AC9：`A=250` 完整 30 天设计库上的写入、重开、核心查询 smoke 和 DB/WAL 指标不超过 C0 批准预算；不得用小库外推替代。
- [ ] AC10：checkpoint starvation 会取消/过期长读并恢复 PASSIVE checkpoint；不删除 WAL。WAL 回落或不可保持 WAL 时 fail closed/degraded，不继续声称 durable healthy。
- [ ] AC11：busy 超过 C0 deadline、磁盘满和 I/O error 均使队列保持有界、health 可见并产生明确 gap；恢复后 gap 可查询，无静默丢账。
- [ ] AC12：并发 subscribe 与 projection 更新测试证明第一条 bootstrap 的 `baseSeq` 原子，后续 `seq` 严格更大；注入 seq gap 后 resync 不重放旧 delta。
- [ ] AC13：正常 schema 不可用时，RecoveryFacade backend 仍可返回脱敏版本/诊断、backup 列表和候选验证结果，且不访问普通报告/告警 schema。
- [ ] AC14：Rust fmt/clippy/test、子项目质量 gate、根 `just ci` 和 secret scan 全部通过；性能与 fault 原始证据关联到验收项。
- [ ] AC15：持续生成超过 retry window 的 bundle 后 receipt 行数按冻结窗口达到平台；recent duplicate 返回原 receipt，payload mismatch fail closed，过期 duplicate 返回 `RetryWindowExpired`，没有旧 bundle 被重新累计。

## 非目标

- 不实现完整 Tauri 窗口、tray、autostart、single-instance、onboarding、实时连接页面或连接关闭 UI。
- 不交付 Windows Credential Manager 产品 adapter；C1 使用 C0 冻结的 port 和 fake resolver。
- 不实现历史 ReportService、CSV/JSON/HTML 导出、hourly/daily rollup、retention、用户备份/恢复或 VACUUM。
- 不创建告警规则、告警状态、notification outbox 或 Windows 通知表。
- 不实现完整 Recovery Shell 或实际 restore/swap 命令。
- 不支持非 loopback TCP、多个控制器并行、Windows Service、macOS 或 Linux。
- 不把观测数据称为严格账单，也不补造应用未运行期间的流量。

## 后续依赖与风险

- C2 依赖 C1 的 ControllerSession、AccountingEngine、core schema、health、原子 Channel 和 RecoveryFacade backend。
- C3 只能以前向 migration 增加报告、rollup、retention 与用户备份恢复；C4 再增加 alert/outbox。后续任务不得修改 C1 已发布 migration。
- kill、磁盘满和 WAL fault injection 需要隔离临时数据库和受控子进程；不得对用户数据库执行。
- C0 决策发生变化时，先回到 C0 重新批准，再更新 C1 设计和 manifests。不得在实现中自行换 binding 或降低 FULL。
