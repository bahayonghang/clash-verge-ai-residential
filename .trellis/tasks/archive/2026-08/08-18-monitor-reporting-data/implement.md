# 实施计划：历史报告与数据管理（C3）

## 启动前门禁

- [x] 任务已由用户授权启动；`task.py start` 于 2026-08-18 将状态改为 `in_progress`（会话身份缺失时降级写状态，退出码 0）。
- [x] 用户在独立消息中授权执行 `task.py start` 并实施到 Gate 完成或暂停条件。
- [x] C2 已完成并通过独立验收；AppFacade、报告 / 数据管理入口、文件选择、operation progress 和 Recovery Shell seam 已冻结。
- [x] C1 StorageCoordinator 的 single writer、短 reader、migration、checkpoint、Online Backup、health 和 ingestion SLO 已冻结。
- [x] C0 真实规模数据生成器已存在于归档证据；本轮不重跑完整 30 天三档库。
- [x] C1 / C2 gate 已通过；C3 未另建 writer，未复制热库。

## 执行顺序

### 1. 冻结 ReportQuery / ReportResult 与能力矩阵

- [ ] 定义版本化 `ReportQuery`、`ReportResult`、coverage、policy metadata、rankings 和 `drilldownCapability` DTO。
- [ ] 建立 raw、hourly / daily dimension、daily core 的查询能力矩阵和跨层能力交集算法。
- [ ] 定义 query fingerprint、稳定 sort tuple + ID keyset cursor、默认 / 最大页和 Top N 上限。
- [ ] 在 Rust 边界实现 range、timezone、dimension、sort、grouping、page 和 cursor 校验。
- [ ] 用 contract fixtures 证明前端不传 SQL，不自行聚合，不在 capability 过期时展示伪结果。

**Gate 1**：通过。每种页面 / 报告 shape 都能映射到明确数据层、命名 query 和可观察 capability；没有模糊降级。

**回滚点**：仅有 DTO 与 planner；移除 C3 facade 不影响 C2。

### 2. 追加 C3 schema 与可续跑 backfill

- [ ] 只追加 C3 migration：稳定 dimension ID、hourly / daily dimension、daily core、daily coverage 和 retention state。
- [ ] 新表优先 STRICT；索引从初始命名 SQL corpus 与三档规模基准产生。
- [ ] 为大型变化实现 expand → checkpointed backfill → contract，不在单个长事务中回填。
- [ ] 为每个 backfill chunk 记录范围、checksum、状态和 resume cursor。
- [ ] 注入进程退出、I/O error、busy、空间不足和重复启动，验证从 verified chunk 续跑。
- [ ] 检查 migration diff，确认未修改 C1 migration、未创建 C4 告警 schema。

**Gate 2**：通过。空库、C1 库、backfill 中断、future schema、checksum mismatch 和低空间 fixtures fail safe；C2 实时能力保持可用或显示明确维护状态。

**回滚点**：contract 前保留旧读路径；停止 backfill 不删除 verified 数据。已应用 migration 不 down migrate。

### 3. 实现统一 ReportService 与命名 SQL corpus

- [ ] 实现 totals、series、精确单维 Top N、连接数、活跃时长、coverage、策略元数据和周期对比查询。
- [ ] 30 天 raw 路径支持会话下钻、受支持组合过滤和当前策略重算。
- [ ] 13 个月 dimension 路径只支持历史主分类 + 单一维度；长期 core 只支持总量 / 历史主分类 / coverage。
- [ ] 所有列表使用 keyset，不使用深 OFFSET；所有值参数化。
- [ ] 每个公开 query shape 加入命名 SQL corpus、规模 fixture、EQP 与 statement-status gate。
- [ ] 使用独占 read connection、progress handler / interrupt 和 monotonic deadline。
- [ ] 验证用户取消、页面 2 秒和报告 10 秒候选 deadline 会停止实际 SQLite 工作。

**Gate 3**：fixture 通过。golden fixtures 覆盖时区、DST、空区间、coverage、策略变化、层级边界和 keyset；三档真实规模库未重跑。

**回滚点**：按 query family feature-disable，保留已写 rollup；不改变 C1 ingestion。

### 4. 实现不持有长读事务的 SnapshotStore

- [ ] 在单一短 read snapshot 中构造完整 `ReportResult`。
- [ ] 小结果有界内存物化，大结果写应用私有有界 spool。
- [ ] 在 token 登记和返回前强制关闭 statement / read transaction，并建立自动断言。
- [ ] token 绑定 query fingerprint、schema / data version、coverage、artifact checksum、创建 / 过期时间和字节数。
- [ ] 实现初始 10 分钟 TTL、显式 release、启动 orphan cleanup、活动 token 数量、单 token 和总 spool quota。
- [ ] 基准冻结最终 TTL / quota；quota 满时只清理过期项，仍超限则 fail closed，不静默驱逐有效 token。
- [ ] 证明 token 生命周期不阻止 PASSIVE checkpoint 或造成 WAL 增长。

**Gate 4**：通过。token 事务关闭、TTL、quota、过期导出 fixtures 通过，资源始终有界。

**回滚点**：禁用大结果 spool，只允许有界小报告；不得回退为长期 read transaction。

### 5. 实现报告界面

- [ ] 实现小时、日期、近 7 / 30 日、自然月和自定义区间 query builder。
- [ ] 实现总量、上下行、趋势、分类占比、精确 Top N、连接数、活跃时长、周期对比和会话下钻。
- [ ] 图表与数据表只读取同一个 `ReportResult`，并展示单位、时区、策略、coverage、缺口和 capability。
- [ ] 实现 loading、cancelled、deadline、token expired、capability unsupported、storage busy 和 failure 状态。
- [ ] 大结果和进度通过 Commands / 有界 operation 状态交互，不经实时 Channel。

**Gate 5**：通过。UI 数字、数据表和 query echo 来自同一 `ReportResult`；能力不支持时返回明确错误。

**回滚点**：隐藏报告导航和命令，C2 实时页面继续运行。

### 6. 实现流式 ExportService

- [ ] 三种 encoder 共享 typed projection iterator 和 metadata builder。
- [ ] 实现 CSV、JSON、可打印 HTML 的字段选择与 Rust 侧脱敏预览。
- [ ] 导出只消费 snapshot token，不重新运行 SQL。
- [ ] 以固定 buffer 流式写同目录临时文件，关闭 / smoke 后原子 rename。
- [ ] 实现取消、过期 token、quota、目标已存在、I/O error 和 partial 清理。
- [ ] 对大报告记录额外内存、吞吐和 artifact checksum；扫描 secret 和敏感字段。

**Gate 6**：通过。同一 token 的 UI / CSV / JSON / HTML totals 和 metadata 一致；固定 64KiB buffer 流式写入。

**回滚点**：按格式分别关闭 encoder；保留应用内报告，不留下 partial 或覆盖已有目标。

### 7. 实现精确 RetentionService

- [ ] 实现 raw → hourly dimension / daily dimension / daily core / daily coverage 的确定性 materialize。
- [ ] 每个 UTC chunk 核对 upload、download、connection count、active duration 和 coverage 后再推进 watermark。
- [ ] 以 replacement / set 语义保证重试不双加；watermark 后才分批删除已覆盖下层事实。
- [ ] 实现 `pending | materialized | verified | delete_pending | done` 恢复状态和所有 crash point。
- [ ] raw 默认 30 天、可配置上限 90 天；期限变更显示真实占用、增长、备份与临时空间预测。
- [ ] 精确高基数 hourly / daily 最多 13 个月；到期前确认 core / coverage，再删除 dimension。
- [ ] 长期只保留 attributed total、历史主分类和 coverage。
- [ ] 增加断言：没有 bounded / approximate Top K 表或写入截断；Top N 从完整精确集合查询。
- [ ] manual / scheduled clean 共用服务，按 ingestion health yield；不自动 VACUUM，不把 freelist 显示成已释放文件空间。

**Gate 7**：通过。中断后续跑，自动 DELETE 保持关闭；能力过期返回不支持。

**回滚点**：切换到 dry-run / materialize-only，停止 DELETE；verified rollup 保留，raw 不提前删除。

### 8. 实现 Online Backup 与 C2 Recovery Shell 恢复

- [ ] 用户备份复用 C1 Online Backup API，分页 step、进度、取消、节流和空间预检。
- [ ] `.partial` 完成后生成 manifest / checksum，执行 integrity / smoke，再原子 rename。
- [ ] 实现 maintenance mode 和 operation 排他，停止 collector / writer / readers 后再恢复。
- [ ] restore 前创建当前库保护备份；验证候选 checksum、integrity、schema 和空间。
- [ ] 实现受控 swap、forward migration、smoke、失败恢复原库和成功重建 StorageCoordinator。
- [ ] 扩展 C2 `RecoveryFacade`，让正常 schema 不可用时也能执行 restore，不初始化 ReportService。
- [ ] 验证跨机恢复不携带 Credential Manager secret，并引导重新输入。

**Gate 8**：通过。持续写、取消、坏候选、未来 schema、低空间 fail closed；失败不覆盖当前库。

**回滚点**：禁用 restore，只保留创建 / 验证备份；Recovery Shell 继续提供 C2 只读能力。

### 9. 冻结并发与容量性能门

- [ ] 运行 `A=50 / 250 / 1000` 完整 30 天真实规模库、13 个月精确维度和长期 core daily，不以小库线性外推。
- [ ] 并发启动 writer、report、export、backup、retention 和 PASSIVE checkpoint。
- [ ] 验证 frame → durable commit p95 小于 1.5 秒、正常最大值小于 3 秒、collector 队列不持续超过 2 帧；若 C0 已审阅调整，则使用新冻结值。
- [ ] 验证 30 天 raw 常用报告 p95 小于 2 秒、13 个月 hourly 常用报告 p95 小于 3 秒，以及页面 2 秒 / 报告 10 秒 deadline。
- [ ] 记录 query / commit p50 / p95 / p99 / max、FULLSCAN_STEP / SORT / VM_STEP、CPU、RSS、DB / WAL / freelist、队列和 checkpoint。
- [ ] 压测 token TTL / 活动数量 / spool quota、取消与 orphan cleanup。
- [ ] 制造长报告与取消，验证 WAL 回落；不得删除 WAL 或无限增长。
- [ ] 制造低空间，验证 backup、restore、migration、spool 和主动 VACUUM fail closed。
- [ ] 对大 backfill 跨多次启动，验证 checkpoint、取消、resume 和 ingestion 优先。

**Gate 9**：部分通过。fixture 并发与候选 quota 已冻结。完整 30 天 `A=50/250/1000` 重跑按暂停条件未执行。

## 计划验证命令

实施完成后，以 C0 冻结的实际脚本为准，至少运行：

```text
npm --prefix residential-monitor ci
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check
cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace
npm --prefix residential-monitor run tauri:build
just monitor-check
just ci
npm run check:secrets
```

另需运行命名 SQL corpus、三档容量、并发操作、retention crash、流式导出和 backup / restore 故障矩阵。

## 独立验收

- [ ] 逐项映射 C3 PRD 的十一项验收标准，不以父任务或后续 C5 验收替代。
- [ ] 审查所有公开查询的命名 corpus、keyset、deadline、cancel 和 capability。
- [ ] 审查 token 返回路径，确认没有长期 read transaction。
- [ ] 审查 schema 与 retention，确认 raw 30 / 90 天、高基数最多 13 个月、长期 core / coverage 和零 approximate Top K。
- [ ] 审查低空间、WAL、backfill、backup / restore 和 C2 Recovery Shell 的 fail-closed 证据。
- [ ] 用户审阅独立验收证据后，C3 才可归档并允许 C4 进入启动审查。

## 整体回滚方案

1. feature-disable 报告 UI、ReportService 和 ExportService，保留 C2 实时监控。
2. retention 未通过守恒 gate 时保持 dry-run / materialize-only，禁止自动 DELETE。
3. restore 未通过全矩阵时仅允许创建和验证备份，禁止数据库 swap。
4. 停止大 backfill 于 verified checkpoint；contract 前继续使用兼容读路径。
5. 不执行 down migration，不删除 C1 raw，不通过缩短保留期或自动 VACUUM处理故障。
6. 若已发布 schema 需要数据回滚，只恢复验证备份并使用兼容 binary；失败时回到 C2 Recovery Shell。
