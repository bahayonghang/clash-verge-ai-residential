# 报告快照配额与一周保留

## Goal

侧栏走完后，规则 / 链路 / 进程与分析报告仍能查出排名或打开档案。进程内 spool 在 8 格上限内复用并淘汰，不再让后续查询全部失败。分析报告、告警跳转与家宽「生成报告」的显式运行结果写入 `report_archive`（`kind=manual`），按生成时间保留 7 天，跨重启可打开。

## User value

用户打开规则 / 链路 / 进程时看到该时间窗的排名或诚实空排名。分析报告能列出自动档案，并能回看一周内自己跑过的报告。配额文案不再把档案列表失败、能力不足与 spool 打满混为一谈。

## Confirmed facts

截图 1：分析报告同时出现「档案列表暂不可用。」「报告快照配额已满。」「尚未运行报告。」截图 2：规则页能力说明与条形图空态都是「报告快照配额已满。」红框为规则、链路、进程。顶栏近 24 小时。

`app.tsx:214-218` 将 `host` / `rule` / `chain` / `process` 交给 `DimensionPage`。`dimension-page.tsx:43-64` 按 `grouping` 调 `useReport`。`08-21-c3-dimension-capability` 已完成。`granularityForTimeRange("24h")` 为 `minute10`，走 raw。空页是 `run_report` 失败。

`c3/query.rs:7-10`：`TOKEN_TTL_SECS=600`、`MAX_ACTIVE_TOKENS=8`、`MAX_TOKEN_BYTES=32MiB`、`MAX_SPOOL_BYTES=128MiB`。`c3/snapshot.rs:86-97` 满额拒绝，fingerprint 只记账不复用。过期清理只在下次 `insert` 开头。文案 `i18n.rs:356`；动作「释放旧报告后再试」，界面无按钮。token 给导出与 `get_report` 用，返回前关闭读事务（`docs/reporting.md:3-4`）。进程退出后 HashMap 清空。

`useReport`（`hooks/use-report.ts:158-185`）不调用 `release_report`。`useReportArchive.release` 存在但页面不调用。`get_report_archive`（`c2/facade.rs:1174-1184`）再 `insert` 一格。C4（`c4/period.rs:129`）与自动档案 tick（`lib.rs:247-265`，目录 `archive-tick`）会释放。`main.tsx:10` StrictMode 开发态双发 effect。

占用：概览 3 次（`overview/index.tsx:48-50`）+ 家宽聚合 1 次 + 四聚合页各 1 次 = 8。下一次水合或下钻即失败。

`use-report-archive.ts:246-285`：列表成功后水合最新档案。水合 `insert` 失败时 `statusZh` 写成 `report.archive.unavailable`，`errorZh` 为配额原文，`TotalsRow` 无结果时 `report.none`。`list_report_archives` 不占 token。`rank-bar-card.tsx:40-44,83,90` 把 `errorZh` 写入能力说明。

自动档案：小时 30 天、日 13 个月，默认 `grouping=host`（`c3/query.rs:514-538`，`docs/reporting.md:26-28`）。唯一键 `(kind, range_start_utc, query_fingerprint)`（`c3/schema.rs:141-142`）。`kind` 为 text。`view-state.md` 现行句「手动运行报告不写 report_archive」，本任务改写。

对照研究：`research/snapshot-quota.md`。

## Requirements

### R1. 侧栏走完后三页与报告仍可查

从概览进入家宽、主机、规则、链路、进程、分析报告，每一页的现查或档案水合不得因「前面几页占满 8 格」失败。`useReport` / `useReportArchive` 在卸载、替换 query、取消或过期响应时 `release_report`；已返回但不再采用的 token 也要释放。同一 `query_fingerprint` 的未过期 token 复用，不新增。

### R2. 满额可回收，失败可理解

`insert` 先清过期，再按 `last_access_utc` 淘汰，然后写入。仍有可回收槽时不得返回 `quota_exceeded`。单 token 超过 32 MiB 或总 spool 超过 128 MiB 仍拒绝。`TOKEN_TTL_SECS` 保持 600。

分析报告：`list_report_archives` 失败才显示 `report.archive.unavailable`。列表成功、水合失败时仍画出列表，结果区用配额/存储原文。规则页配额错误不出现在「能力说明」；能力说明只读 `drilldownCapability.noteZh`。

### R3. 看档案与导出共用冻结结果

打开成功自动或手动档案，展示冻结 `ReportResult`。导出绑定该次水合 token，不重查更新后的库（`docs/reporting.md:29`）。水合仍走 `get_report_archive` 的 `insert`，依赖 R1/R2 的复用与 LRU，不得再把进页水合报成列表不可用。

### R4. 规则 / 链路 / 进程继续现查

三页保持 `DimensionPage` + `grouping` 现查。24 小时与 7 天窗在 raw 期内返回该维排名。不得换成占位页，不得只读自动档案。`useReport` 现查不写 `report_archive`。

### R5. 手动报告保留 7 天

分析报告「运行报告」、告警跳转 `runQuery`、家宽页「生成报告」在查询成功后写入 `report_archive`，`kind=manual`。失败查询不写行。保留按 `generated_utc` 起算 7 天；`purge_expired` 只删本表，与 raw 自动 DELETE 无关。

同一 `(kind, range_start_utc, query_fingerprint)` 再跑则覆盖结果（自动 hour/day 仍是已有 ok 不覆盖）。不升 schema、不改 `C3_ARCHIVE_DDL`。自动 hour/day 默认 grouping 仍为 host，调度不产生 manual 行。

档案列表可筛选并点选手动行。进入分析报告页仍优先最新成功日档案，否则最新成功小时档案，不自动选手动行。列表 kind 列为「手动」；当前会话来源行仍为「本次手动查询」。

## Out of scope

- 打开 C3 自动 DELETE 或自动 VACUUM。
- 改 raw 30 天 / 精确维 13 个月 / 自动小时 30 天 / 自动日 13 个月。
- 把自动档案默认 grouping 扩到规则 / 链路 / 进程。
- 把 `TOKEN_TTL_SECS` 改成 7 天。
- 手动档案删除按钮、数量上限、云同步、邮件、共享链接。
- 重做分析报告或三页视觉布局。
- 改采集、告警规则引擎、家宽判定。
- 改写已发布 C1 / C3 / C4 / `C3_ARCHIVE_DDL` migration 文本。

## Acceptance Criteria

- [ ] AC1 (R1)：冷启动后按 概览 → 家宽 → 主机 → 规则 → 链路 → 进程 → 分析报告 走一遍，规则 / 链路 / 进程在 raw 窗内显示排名或诚实空排名，不得出现 `report.quota_exceeded`。
- [ ] AC2 (R1)：`useReport` 卸载、换 query、取消响应时释放 token；有测试覆盖「第二次查询不增加活跃 token」和「取消后释放」。
- [ ] AC3 (R1)：同一 `query_fingerprint` 未过期则复用 token；Rust 单测覆盖。
- [ ] AC4 (R2)：8 个未过期不同 fingerprint 后再插入第 9 个，store 淘汰最旧并成功；32 MiB / 128 MiB 硬上限仍拒绝。
- [ ] AC5 (R2)：水合失败时档案列表仍可见；配额错误不显示 `report.archive.unavailable`。规则页配额错误不出现在「能力说明」。
- [ ] AC6 (R3)：点开一份成功自动档案能看到冻结 totals / series / rankings；随后导出仍走该结果，不重查。
- [ ] AC7 (R4)：规则 / 链路 / 进程仍是 `DimensionPage` 现查；24 小时 raw 窗下三页 `grouping` 各不相同。
- [ ] AC8 (R5)：显式运行成功后列表出现 `kind=manual` 行；结束进程再打开仍能点选同一冻结结果；`generated_utc` 超过 7 天的手动行不再出现。现查页面不新增 manual 行。同窗同 query 再跑覆盖，不第二行。
- [ ] AC9 (R5)：家宽「生成报告」与告警跳转同样落 manual 行。自动 hour/day 补跑不受 manual 行影响。
- [ ] AC10：`just monitor-check` 通过。不打开自动 DELETE。已发布 migration 文本不改。

## Key decisions

- Q1 一周快照落点：**B**。显式运行写入 `kind=manual` 保留 7 天。三页继续现查。不改 spool TTL。不为规则 / 链路 / 进程做自动小时档案。
- 唯一键复用现表，避免 schema 升级。手动覆盖、自动不覆盖。
- 导出继续要 token，档案水合仍 `insert`，用复用+LRU 避免抢槽失败。
