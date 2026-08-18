# 家宽监控历史报告与数据管理（C3）

## 目标与用户价值

在 C2 已交付稳定桌面外壳、原子实时界面和数据库无关 Recovery Shell 后，完成历史查询、应用内报告、CSV / JSON / 可打印 HTML 导出、分层保留、用户备份与安全恢复。用户应能在明确的 coverage、策略版本和数据层能力边界内得到可复现结果，并在持续采集时执行报告和维护而不破坏 ingestion SLO。

本任务独立交付报告与数据管理能力，不实现告警规则、通知 outbox 或最终发布硬化。

## 前置条件与边界

- C3 严格依赖 `08-18-monitor-desktop-realtime`（C2）。只有 C2 独立验收通过，原子 IPC、报告页面入口、设置 / 数据管理入口和 Recovery Shell seam 稳定后，C3 才能进入实施。
- C3 复用 C1 `StorageCoordinator` 的 single writer、短只读连接、migration 与 Online Backup 原语，并通过 C2 AppFacade / Recovery Shell 暴露能力；不绕过这些接口建立第二套数据库所有者。
- C3 migration 只向前追加并按串行合并顺序分配版本，不修改已交付的 C1 migration，也不预建 C4 alert / notification outbox schema。
- 当前任务保持 `planning`；本轮未授权执行 `task.py start`。

## 范围内需求

### C3-R1 统一报告契约

- `ReportQuery` 是历史与报告的唯一查询输入，至少覆盖 UTC range、展示时区、粒度、分类 / 标签、域名、进程、规则、链路、网络类型、分组、策略、周期对比、排序和分页。
- `ReportResult` 是应用内数字、图表、数据表和导出的唯一权威投影，至少返回 schema / data version、query echo、totals、series、rankings、coverage、策略元数据、可下钻能力和 report snapshot token。
- 时间持久化和查询边界使用 UTC；本地时区只用于自然日 / 月边界和展示。DST、空区间、部分 coverage 和策略变化不得改变字节守恒。
- 30 天 raw 能力期内允许会话下钻、任意受支持维度组合和当前策略重算；超出能力期时必须返回明确 `drilldown_capability`，不得以部分数据伪装完整结果。
- 大列表和明细使用稳定 sort tuple + ID 的 keyset pagination，不使用深 OFFSET。

### C3-R2 不持有长读事务的报告快照

- `run_report` 在一个短期 SQLite read snapshot 中完成一致读取，然后把不可变 `ReportResult` 或有界磁盘 spool 绑定到 snapshot token；返回 token 前必须关闭 read transaction。
- snapshot token 绑定 query、schema / data version、coverage 与不可变结果。UI 和所有导出消费同一 token，不为导出重新查询更晚的数据版本。
- 初始候选 TTL 为 10 分钟；活动 token 数量、单 token 大小和总 spool quota 必须用 C3 基准冻结。过期后要求重新运行报告。
- 活动 token 与 spool 必须有明确释放、过期清理和 quota 错误；不得因 token 阻塞 checkpoint 或导致 WAL 无限增长。

### C3-R3 查询取消、deadline 与计划回归

- 页面查询和完整报告使用独占、可中断的只读连接，并支持 SQLite interrupt / progress handler 或 binding 等价能力。
- 初始候选 deadline：页面 2 秒、报告 10 秒。超时和用户取消必须终止实际 SQLite 工作，不只是丢弃前端结果；最终阈值由真实规模基准冻结。
- 每个公开查询进入命名 SQL corpus。CI 不快照易变的完整 EQP 文本，而检查意外大表全表 SCAN、非预期 TEMP B-TREE、自动索引，以及 FULLSCAN_STEP、SORT、VM_STEP 和冷 / 热延迟。
- 所有过滤值参数化；排序、维度、分页上限和日期范围在 Rust 边界按 allowlist 校验，前端不传 SQL。

### C3-R4 历史与报告界面

- 支持小时、指定日期、近 7 / 30 日、自然月和自定义区间，以及总量、上下行、趋势、重点分类占比、Top 域名 / 进程 / 规则 / 链路、连接数、活跃时长和周期对比。
- 图表与数据表消费同一个 `ReportResult`，明确单位、时区、策略版本、coverage、缺口原因和可下钻边界。
- 查询取消、deadline、token 过期、能力过期和存储故障有独立中文状态与恢复动作。
- 大结果不经实时 Channel；报告状态通过 Commands 和有界进度消息交互。

### C3-R5 一致且流式的导出

- 按当前 report snapshot token 导出 CSV、JSON 和可打印 HTML；v1 不输出 PDF 或 Excel。
- 导出 header / metadata 包含 UTC range、本地时区、单位、筛选、策略、schema / data version、生成时间、coverage 和缺口说明。
- 字段选择、域名 / IP / 进程路径脱敏和预览在 Rust 完成；secret 永不进入预览、导出或错误。
- 导出采用有界内存流式写入，先写临时文件，成功关闭并验证后原子完成。取消或失败清理 partial，不覆盖已有目标。

### C3-R6 精确分层保留

- session / chain / minute raw 默认保留 30 天，v1 可配置上限 90 天；提高期限前显示基于真实占用、增长率、备份和临时空间的预测。
- 域名、进程、规则、链路和网络类型的精确高基数 hourly / daily 数据最多保留 13 个月；只支持历史主分类 + 单一分析维度，不承诺跨维笛卡尔组合或会话下钻。
- 长期 daily 只保留 attributed total、历史主分类和 coverage，不保留高基数 Top N。
- v1 不实现 approximate Top K，也不保存 bounded Top K 后将其称为精确结果。保留期内的 Top N 必须从完整精确维度汇总计算；能力过期后明确返回不支持。
- raw coverage 最多保留 13 个月，daily coverage 长期保留。
- retention 先幂等生成并验证上层汇总，再推进 watermark，最后删除已覆盖下层事实；中断后可区分状态并续跑。
- 自动与手动清理共用同一服务。DELETE 产生的 freelist 不得宣称为立即释放磁盘；v1 不自动 VACUUM。

### C3-R7 Online Backup 与安全恢复

- 用户手动备份复用 C1 Online Backup 原语，分页 step、显示进度、可取消、可节流并在开始前检查目标空间；不得复制 hot SQLite 文件。
- 备份先写 `.partial`，完成后执行 checksum、manifest 和适用的 integrity 验证，再原子 rename。
- 恢复进入 maintenance mode：停止 collector / writer / query，先保护当前可用数据库，再验证候选、受控 swap、前向迁移和 smoke check；失败不得覆盖或丢失当前可用数据库。
- 实际 restore command 接入 C2 Recovery Shell。Recovery Shell 在正常 schema 不可用时仍可调用恢复 seam，不依赖 ReportService。
- Credential Manager secret 不随数据库备份；跨机恢复必须重新输入。

### C3-R8 真实规模并发性能门

- 使用 C0 冻结的 `A=50 / 250 / 1000` 完整 30 天真实规模数据库、13 个月精确高基数 rollup 与长期 core daily；不得用小库线性外推。
- 在 writer、report、export、backup、retention 和 checkpoint 并发时，仍满足 C1 冻结的 ingestion SLO：frame receipt → durable commit p95 小于 1.5 秒、正常最大值小于 3 秒，collector 输入队列不持续超过 2 帧；若 C0 后续以实测调整 SLO，则使用已审阅的新基线。
- 30 天 raw 常用报告 p95 小于 2 秒、13 个月 hourly 常用报告 p95 小于 3 秒，并同时执行页面 2 秒 / 报告 10 秒候选 deadline gate。
- token TTL / 数量 / spool quota、查询取消、WAL / checkpoint、DB / WAL / freelist、冷 / 热 p50 / p95 / p99 / max 和 statement status 必须纳入证据。
- checkpoint starvation 时取消或过期长读者，不删除 WAL；任何并发场景不得让 WAL 无限增长。
- backup、restore、migration 和用户主动 VACUUM 在低空间下 fail closed，不先损坏当前可用库。大 backfill 必须 checkpointed、可取消、跨启动续跑。

## 非目标

- 不实现告警规则、Windows 通知、告警中心或 notification outbox。
- 不提供 approximate / bounded Top K，不把不完整排名标为精确。
- 不保留 13 个月之后的高基数 Top N，不为老数据伪造任意跨维筛选或会话下钻。
- 不输出 PDF / Excel，不做定时邮件、云同步或共享链接。
- 不自动 VACUUM；用户主动 VACUUM 也只有在独立空间与暂停写入 gate 通过后才可用。
- 不修改 C1 migration，不以 down migration 作为回滚。

## 验收标准

- [ ] **C3-AC1 依赖门**：C2 已完成并独立验收；C3 通过 C1 StorageCoordinator 和 C2 AppFacade / Recovery Shell 接入，没有第二个 writer、直接热库复制或 C4 schema。
- [ ] **C3-AC2 报告正确性**：golden fixtures 覆盖 UTC / 本地时区、DST、空区间、部分 coverage、策略变化、raw 过期和周期对比；应用重启前后总量、趋势、排名和 coverage 一致。
- [ ] **C3-AC3 快照一致性**：同一 token 的 UI、CSV、JSON、HTML 总计和 metadata 逐项一致；token 返回时没有活跃 read transaction，过期 / 释放 / quota 场景有界且可恢复。
- [ ] **C3-AC4 查询控制**：所有公开查询有命名 SQL corpus、keyset、参数校验、interrupt / progress、页面 2 秒 / 报告 10 秒候选 deadline、EQP 和 statement-status 证据；取消后 SQLite 工作实际停止。
- [ ] **C3-AC5 精确保留**：raw 默认 30 天 / 上限 90 天，精确高基数最多 13 个月，长期 daily 只有总量 / 历史主分类 / coverage；保留前后守恒，能力过期明确不支持，测试证明没有 bounded 或 approximate Top K。
- [ ] **C3-AC6 retention 可恢复**：在汇总前、已汇总未推进 watermark、已推进未删除和删除中断点重启后均可幂等续跑；ingestion 优先，不自动 VACUUM，不把 freelist 当作已释放空间。
- [ ] **C3-AC7 导出有界**：大报告的三种导出流式完成，额外内存不随输出线性增长；取消 / I/O 失败不覆盖目标且清理 partial；脱敏与 secret 扫描通过。
- [ ] **C3-AC8 备份恢复**：持续写入 Online Backup、取消、节流、坏 checksum、坏 integrity、旧 schema、残留 WAL / SHM、空间不足和恢复中断矩阵通过；失败始终保留当前可用数据库。
- [ ] **C3-AC9 Recovery Shell 接入**：正常 schema 不可用时可从 C2 Recovery Shell 验证并执行 C3 restore；流程不初始化普通 ReportService，恢复成功后才进入正常应用。
- [ ] **C3-AC10 并发性能**：`A=50 / 250 / 1000` 真实规模下 writer / report / export / backup / retention / checkpoint 并发仍满足 ingestion 与常用报告 SLO；WAL 有界、查询可取消、token quota 有界、低空间 fail closed、大 backfill 可续跑。
- [ ] **C3-AC11 独立回滚**：报告 / 导出可 feature-disable；retention 守恒 gate 前不启用自动删除；恢复矩阵通过前只允许创建备份；已追加 migration 不回退，当前数据库和 C2 实时能力保持可用。

## 开放问题

无阻塞性产品开放问题。token 数量、spool quota、最终 deadline 和支持的 `A` 上限必须由 C0 / C3 基准冻结；这些数值未冻结前不得扩大容量承诺或省略 fail-closed 行为。
