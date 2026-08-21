# 报告快照配额：代码对照

2026-08-21 分析报告与规则页截图对照仓库现状。供 `08-21-report-snapshot-quota` 实施引用。

## Spool 上限

`residential-monitor/src-tauri/src/c3/query.rs:7-10`：

- `TOKEN_TTL_SECS = 600`
- `MAX_ACTIVE_TOKENS = 8`
- `MAX_TOKEN_BYTES = 32 MiB`
- `MAX_SPOOL_BYTES = 128 MiB`

`c3/snapshot.rs:86-97`：读事务仍开、单 token 过大、总字节超、或活跃数 ≥ 8 时失败。满额拒绝，不淘汰。`insert` 记录 `query_fingerprint` 但不复用。`cleanup_expired` 只在下一次 `insert` 开头执行。

文案：`i18n.rs:356` `report.quota_exceeded` = 「报告快照配额已满。」动作「释放旧报告后再试」。界面无释放按钮。

## 谁释放

| 路径 | 释放 |
|---|---|
| `useReport` → `run_report` | 否。卸载、换 query、StrictMode 取消后的成功响应都不 `release_report` |
| `useReportArchive.release` | 函数存在，报告页与家宽报告段不调用 |
| `get_report_archive` | `insert` 再占一格 |
| C4 `c4/period.rs:129` | 是 |
| 自动档案 `lib.rs:247-265` | 是；独立目录 `data_dir/archive-tick`，避免 `cleanup_orphans` 删门面 token |

## 打满 8 格的页面组合

`app.tsx` 除实时页外按 route 卸载。`useReport` 仍把 token 留在进程内 store。

| 页面 | 次数 | 位置 |
|---|---|---|
| 概览 | 3 | `overview/index.tsx:48-50` host / chain / process |
| 家宽聚合 | 1 | `residential/aggregate-section.tsx:30-35` category |
| 主机 / 规则 / 链路 / 进程 | 各 1，下钻 +1 | `dimension-page.tsx:43-64` |
| 分析报告进页 | `get_report_archive` +1 | `use-report-archive.ts:266-268` |

概览 3 + 家宽 1 + 四聚合页 4 = 8。下一次水合或下钻即 `quota_exceeded`。`main.tsx:10` StrictMode 在开发态把 effect 跑两遍。

## 截图 1 文案对应

`use-report-archive.ts:246-285`：列表成功后水合最新档案。`insert` 失败时 `statusZh` = `report.archive.unavailable`，「档案列表暂不可用」；`errorZh` = 配额原文。`TotalsRow` 无结果时 `report.none` = 「尚未运行报告。」`list_report_archives` 不占 token。

## 截图 2

`rank-bar-card.tsx:40-44,83,90` 把 `errorZh` 写入「能力说明」和条形图空态。规则 / 链路 / 进程是已实现的 `DimensionPage`（`app.tsx:214-218`）。

## 与一周快照的现有分层

- spool token：10 分钟，进程内。
- 自动档案：`kind=hour|day`，默认 `grouping=host`，小时 30 天、日 13 个月。`report_archive_period_uniq` = `(kind, range_start_utc, query_fingerprint)`。已有 `ok` 不覆盖。
- 手动「运行报告」：只进 spool。`view-state.md` 写明不写 `report_archive`，本任务将改写该句。
- 不升 schema：`kind` 已是 `text`，新增 `manual` 不改 `C3_ARCHIVE_DDL`。
