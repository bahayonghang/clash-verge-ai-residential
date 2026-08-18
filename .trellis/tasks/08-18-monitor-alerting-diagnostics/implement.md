# C4 实施计划：告警与诊断

## 启动前 Gate

- [ ] C3 `08-18-monitor-reporting-data` 已完成独立验收，`ReportService`、rollup、coverage、时区边界、可取消查询和 retention 接口稳定。
- [ ] C1 ingestion SLO 与 C0 性能 harness 的实际冻结值可用；若冻结值与父任务预算不同，先回到规划评审，不在 C4 临时改口径。
- [ ] C4 的 PRD、design、implement 和 manifests 已由用户审阅。
- [ ] 用户在审阅后另行明确授权启动；在此之前保持 `planning`，不得运行 `task.py start`。

## 实施顺序

### 1. 冻结 C3 复用契约

- [ ] 盘点周期用量所需的 `ReportQuery`、rollup、coverage、策略版本、data version、时区边界和 capability 返回。
- [ ] 定义批量 `AlertUsageQuery` seam；保证它复用 C3 SQL corpus / 投影，不新增私有 raw 扫描或日 / 月累计器。
- [ ] 建立规则类型、状态、证据、outbox 和脱敏错误的版本化跨层 DTO。
- [ ] 写下 C3 不可用、查询超时、coverage 不足和能力过期时的 `not_evaluable` 映射。

Gate：评审数据流，证明 UI、告警证据和 C3 报告可由同一查询参数往返；发现第二套聚合即停止实施。

### 2. 追加 C4 schema 与 migration

- [ ] 只追加 alert rule、instance、event 和 notification outbox 所需前向 migration。
- [ ] 加入规则版本、活动实例唯一性、event / outbox 幂等键、`bundle_id` 引用和稳定状态约束。
- [ ] 为告警中心分页、活动实例查找、pending / retry 扫描、stale lease 和 retention 建立最小候选索引。
- [ ] 把新增 SQL 纳入命名 corpus、prepared statement cache、EQP / statement-status 与 B/row 测量。
- [ ] 验证空库、C3 schema 升级、重复启动、migration 中断、checksum mismatch 和 future schema fail closed。

Gate：migration 前备份和失败恢复通过；不得修改 C1 / C3 已有 migration，不得以增加无依据索引掩盖查询问题。

### 3. 实现 AlertEngine 状态机

- [ ] 实现规则创建、编辑、启停、版本化、阈值与静默输入校验。
- [ ] 实现 health reducer 与根因去重。
- [ ] 实现活动、恢复、冷却、静默、滞回、不可评估和 superseded 迁移。
- [ ] 对规则变更、重启、时间回拨、缺口、epoch reset 和 DST 使用确定性时钟测试。
- [ ] 保证每个规则版本和对象最多一个活动实例，事件重放可还原状态。

Gate：状态转换表和属性测试通过；不得由前端或 NotificationSink 补充业务状态。

### 4. 实现共享速率窗口

- [ ] 从已核算增量构造有界 60 秒 ring buffer，按 selector / direction 共享窗口结果。
- [ ] 建立规则候选内存索引，一次 frame 批量匹配和评估。
- [ ] 实现连续 3 次满足、恢复滞回、缺口重置和不可评估语义。
- [ ] 对大量规则和高 selector 基数设置明确支持上限与 degraded health。
- [ ] 加入 instrumentation，确认热路径没有逐 frame 逐规则 SQL、重新 prepare 或独立 transaction。

Gate：10,000 活跃短峰叠加大量规则时，AlertEngine 增量开销和整体 frame / durable commit 延迟仍满足 C1 SLO。

### 5. 接入周期用量规则

- [ ] 实现滚动 1 小时、本地自然日和本地自然月的批量调度。
- [ ] 复用 C3 时区边界、rollup、coverage、data version 和 policy metadata。
- [ ] 保存可打开同口径报告的证据引用，不复制整份报告或长期持有 read transaction。
- [ ] 覆盖 DST、跨午夜、跨月、月份长度、部分 coverage、策略变化和 capability 不支持。
- [ ] 对 deadline、interrupt、token / data version 失效返回稳定不可评估状态。

Gate：同一窗口与过滤条件的告警观测值和 C3 报告逐项一致；代码 / schema 搜索确认没有第二套周期累计。

### 6. 扩展 CommitBundle 原子写入

- [ ] 把 alert instance / event 和 outbox intents 纳入 facts、coverage 所在的同一 writer transaction。
- [ ] 延用稳定 `bundle_id` 和 `CommitReceipt`，为 alert event / outbox 加幂等键。
- [ ] 在 transaction 各写入点、commit 前、结果不确定和 commit 后未回执位置注入故障。
- [ ] 验证重试不重复事实、告警事件或通知意图。
- [ ] 保持批次与 prepared statements 有界，量化新增写放大和 transaction 延迟。

Gate：hard reset 后只能观察到全部提交或全部未提交；任何通知发送都发生在成功 receipt 之后。

### 7. 实现 NotificationWorker 与 Windows adapter

- [ ] 应用启动后立即扫描，运行中周期扫描。
- [ ] 实现按稳定顺序和 `LIMIT` 的 pending / retry 扫描、原子 lease、lease token 与 stale reclaim。
- [ ] 实现 attempt、`next_attempt_at`、`lease_until`、错误分类、有上限指数退避和抖动。
- [ ] 实现 `sent | failed | suppressed` 终态与告警中心可见性。
- [ ] 实现 Windows 通知发送、测试通知和点击后打开 / 聚焦应用。
- [ ] 在 worker 认领、发送前、发送结果不确定和写回前终止进程，验证重启恢复。

Gate：outbox SQL 命中预期索引、扫描批次和耗时有界；持续失败不抢占 ingestion，且没有永久 stuck 项。

### 8. 交付告警中心与诊断

- [ ] 交付活动 / 历史列表、keyset 分页、规则编辑、证据链接、恢复时间和通知状态。
- [ ] 为单位、时区、静默、冷却和恢复滞回提供明确中文校验与预览。
- [ ] 通过有序 Channel 发布小型告警状态变化；历史查询不走实时 Channel。
- [ ] 汇总版本、transport、last frame、coverage、reconnect、queue、DB / WAL、commit、checkpoint、backup、retention、alert / outbox health。
- [ ] 对日志和诊断应用统一白名单与脱敏；实现预览、临时文件和原子导出。

Gate：键盘、焦点、通知不可用、空状态和错误恢复走查通过；secret 和完整敏感连接字段扫描为零。

### 9. 性能、故障与独立验收

- [ ] 运行 health flapping、大量启用规则、10,000 活跃短峰、通知持续失败、积压 outbox 和 stale lease 组合压力。
- [ ] 记录 alert evaluation、frame → `CommitBundle`、frame → durable commit、outbox scan / send 的 p50 / p95 / p99 / max。
- [ ] 记录 CPU、RSS、writer queue、outbox backlog、DB / WAL、每表 / 索引 B/row 和写放大。
- [ ] 证明输入队列不持续超过 2 帧，内存和重试有界，没有逐 frame 逐规则独立 SQL。
- [ ] 在真实 NSIS current-user 安装态验证普通用户通知、系统禁用 / Focus Assist 说明与点击行为。
- [ ] 汇总命令、环境、fixture、原始指标、门限判定、脱敏扫描和回滚演练，形成 C4 独立验收证据。

Gate：C4 PRD 全部 AC 逐项有证据；任一 ingestion SLO、原子性、outbox 可恢复性或脱敏 gate 失败均不得交给 C5。

## 验证命令

以 C0 冻结的实际 package scripts 和 harness 名称为准。实施后至少执行：

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

任务专项命令必须由实现时纳入 package / Cargo test 入口，并可分别重放：

- AlertEngine 确定性时钟与状态机测试。
- 周期用量与 C3 golden report 一致性测试。
- transaction / kill-point / bundle 幂等测试。
- outbox lease、stale reclaim、退避和通知故障测试。
- 告警性能与安装态通知 smoke。
- 诊断与日志敏感信息扫描。

## 验收证据

每个 gate 至少记录：

- 执行命令、提交 / schema / fixture 版本和基准机信息；
- 原始输出或机器可读指标位置；
- 预期门限、实际结果和通过 / 失败结论；
- 故障注入点与重启后状态；
- 对应 PRD AC；
- 已知限制和回滚演练结果。

仅有截图或口头结论不构成数据库原子性、性能或脱敏验收证据。

## 回滚计划

- **未发布候选**：禁用新增告警入口并前向修复；开发数据库可从 migration 前备份恢复，但不得修改已提交的历史 migration。
- **通知故障**：关闭 `NotificationSink` / worker，保留 AlertEngine、应用内告警、历史和 outbox；修复后从 pending / retry / stale 状态续跑。
- **周期评估故障**：暂停周期规则并显示不可评估，不得切换到第二套聚合；修复 C3 seam 后恢复。
- **性能回退**：首先降低通知 worker 批次 / 并发并暂停非关键扫描；不得丢事实、缩短保留、拆分原子事务或把 `synchronous=FULL` 静默降级。
- **已发布 schema**：只允许前向 migration 和修复版本，不执行 down migration，不删除告警历史。

## 完成条件

- [ ] C4-AC1 至 C4-AC10 全部通过并有可重复证据。
- [ ] 周期用量只复用 C3 报告 / rollup 的设计与代码审查通过。
- [ ] facts、coverage、alerts、outbox 同事务和崩溃恢复通过。
- [ ] C1 ingestion SLO、outbox 有界扫描和敏感信息零泄露通过。
- [ ] 回滚演练通过，C5 可以依赖稳定的告警、通知和诊断契约。
