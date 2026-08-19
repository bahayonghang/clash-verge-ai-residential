# 分析报告自动小时与日生成

## Goal

用户打开「分析报告」时能读到已闭合本地小时和已闭合本地自然日的冻结报告。应用在跑时按点生成；关掉期间缺的周期在下次启动分批补跑。手动查询和三种导出仍可用。

## Background

2026-08-19 截图里分析报告四块都是空态（「尚未运行报告」「无数据」）。进页不调用 `run_report`（`residential-monitor/src/main.ts` 871–872、1231–1232）。只有点击「运行报告」才现查（1391–1399）。默认查询是滚动最近 3600 秒（377–392）。`ReportResult` 只留在当前 WebView 会话。

`ReportSnapshotStore` 是 10 分钟 TTL、最多 8 个 token 的进程内快照（`c3/query.rs` 7–8；`c3/snapshot.rs` 45–54、74–117）。启动 `cleanup_orphans` 会清掉磁盘 spool。`report_snapshot_meta` 表存在，但 store 不用它做跨进程档案。

C3 已有小时 / 日维表。`RetentionService` 只在设置页手动触发（`c3/retention.rs` 55–78）。现有 `materialize_hourly` 物化的是即将离开 raw 窗口的数据，不是刚闭合的一小时。30 天内的默认单维查询走 raw 层（`c3/query.rs` 530–542）。

C4 采集路径每 ≥60 秒为告警调用 `evaluate_period_rules` 并立刻 `release` token（`c2/facade.rs` 480–490；`c4/period.rs` 118–129）。那是告警观测，不是报告档案。

无 Windows Service。C3 自动 DELETE 关闭。已发布 C1 / C3 / C4 migration 文本不得改写。当前 `user_version` 上限为 3（`c0_contract.rs` `SCHEMA_VERSION`）。

## Requirements

### R1 自动小时报告

- 应用在跑时，每个已闭合本地小时生成一份默认报告。窗口是该本地小时半开区间 `[hour, hour+1)`。
- 查询走现有 `ReportService`。默认：`granularity=hour`，`grouping=host`，`targetPolicy=historical`，`displayTimezone=local`，`topN=20`，`comparison.previousEqualWindow=true`。
- 30 天窗口内走 raw 能力层，不在每个整点跑一遍全量 `RetentionService`。
- 同一本地小时、同一默认 `query_fingerprint` 只保留一份成功档案。

### R2 自动日报告

- 每个已闭合本地自然日生成一份默认报告。窗口用 `local_day_bounds("local", …)`。
- 默认与 R1 相同，仅 `granularity=day`。日总量来自 `ReportService`，前端不加总小时档案。
- 近 30 日走 raw。更早且仍在 13 个月精确维内的日，走日维；日维未物化或能力不支持时记失败，不写假总量。
- 同一自然日、同一默认 fingerprint 只保留一份成功档案。日档案不要求 24 份小时档案先齐。缺口只出现在该日 `ReportResult.coverage`。

### R3 启动与唤醒补跑

- 正常启动（非 Recovery Shell）后补跑缺档。小时回溯 30 天，日回溯 13 个月（`DIMENSION_RETAIN_DAYS`，396 天）。
- 先补最近的闭合小时和最近的闭合日，再往更早缺档走。每个采集周期最多完成 1 份档案，给 writer 让路。
- 应用关闭形成的采集缺口写入 coverage，不得把缺测区间的总量画成 0 来冒充完整。
- 不写 Windows Service，不写登录自启动。

### R4 手动生成

- 现有预设、粒度、排名维度、「运行报告」和 CSV / JSON / HTML 导出继续可用。
- 手动结果只进当前会话的短期 `reportSnapshotToken`。不覆盖自动档案。

### R5 分析报告页

- 进入页面时加载档案列表。优先展示最新成功日档案；没有则展示最新成功小时档案；都没有则说明尚未有闭合周期或仍在补跑。
- 页上能区分自动小时、自动日、本次手动查询。
- 列表至少能点选最近小时与最近日。点选后 `get` 冻结的 `ReportResult`。前端不聚合。

### R6 口径与失败

- 自动与手动共用 `ReportResult`。空区间总量可以为 0。缺口、未知、能力不支持不得写成 0。
- 生成失败留下可见中文状态；后续 tick 可重试。重试成功后同一周期只留一份成功档案。
- 自动生成不得阻塞采集 HTTP 与 durable commit。Recovery Shell 不跑调度。

### R7 持久化档案

- 每个成功周期把完整 `ReportResult` 写入本机 SQLite。进程退出后仍可 `list` / `get`。
- 首次成功时冻结。补跑只填尚无成功档案的周期。
- 从档案导出时，先把冻结结果水合进现有 snapshot token，再走现有 `export_report`。不得为导出再查更新后的库。
- 小时档案保留 30 天，日档案保留 13 个月。过期档案由调度删除。这不是 C3 raw 自动 DELETE。

## Out of Scope

- Windows Service、登录自启动、定时邮件、云同步、共享链接、PDF / Excel。
- 打开 C3 自动 DELETE 或自动 VACUUM。
- 自动报告默认跑进程 / 规则 / 链路 / 网络 / 分类等其余维度。
- 自然月自动档案。
- 改写已发布的 C1 / C3 / C4 migration 文本。
- 把 C4 告警观测存成报告档案。
- 视觉大改。布局只为列表和类型标记做最小改动。

## Acceptance Criteria

- [ ] **AC1 进页有自动报告**：至少有一份成功档案后，进入分析报告页无需点「运行报告」即可看到总量 / 趋势表 / Top N。
- [ ] **AC2 小时边界**：本地 10:00 生成的是 `[09:00, 10:00)`。同一小时重复调度不产生第二份默认成功档案。
- [ ] **AC3 日边界**：本地次日开始后生成昨日自然日报告，窗口与 `local_day_bounds("local", …)` 一致。日总量来自 C3 查询。
- [ ] **AC4 补跑与缺口**：关掉应用跳过若干闭合小时后再次打开，调度分批补齐缺档（先近后远）。日档案的 coverage 反映缺测，不把缺口写成 0。
- [ ] **AC5 手动仍可用**：自定义查询点「运行报告」得到当前会话 `ReportResult`，导出绑定该 token，不覆盖自动档案。
- [ ] **AC6 口径**：同一档案的图表、表、导出 totals 与 coverage 一致。
- [ ] **AC7 非阻塞**：自动生成进行中，采集与实时页仍可用。Recovery Shell 不初始化调度。
- [ ] **AC8 回归**：`just monitor-check` 通过。不打开自动 DELETE。已发布 migration 文本未改。
- [ ] **AC9 持久化**：结束进程再打开，已成功档案仍在，内容与生成时一致。
- [ ] **AC10 保留**：超过 30 天的小时档案、超过 13 个月的日档案不再出现在列表中。

## Key Decisions

- 产物：跨重启的冻结 `ReportResult` 档案，不是进页现查，也不是定时写导出文件。
- 窗口：已闭合本地小时 / 已闭合本地自然日。`displayTimezone=local`。默认排名维度 `host`。
- 保留：小时档案 30 天，日档案 13 个月。首次升级按同一窗口分批补跑，先近后远，每 tick 最多 1 份。
- 无 Windows Service。关应用期间靠下次启动补跑。
- 手动查询不覆盖自动档案。
- 30 天内默认查询走 raw，不在每个整点跑全量 `RetentionService`。
