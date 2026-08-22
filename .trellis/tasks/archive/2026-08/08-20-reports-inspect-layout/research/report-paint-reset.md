# 分析报告页：Channel 重绘冲掉滚动 / details，图表无命中区

日期：2026-08-20

## 现象

用户在 Mocha、已加载自动小时报告时：

1. 「精确 Top N」表体滚轮下滑后回到顶端。
2. 「数据层与能力」展开后立即合上。
3. 趋势柱和扇形图不能悬停或点击查看 host。

## 机制

`renderApp`（`residential-monitor/src/main.ts:1180`）每次 `paint()` 执行 `root.innerHTML = …`。重绘前只保存 `.live-table-wrap` 滚动和 `activeElement.id`（`1152–1209`）。

`handleMonitorRaw`（`1427–1444`）对每条已解码 Monitor 消息无条件 `paint()`。采集运行时 `connectionDelta` 持续到达。`route === "reports"` 时报告 DTO 未变，DOM 仍被整页替换。

因此：

- `.report-table-wrap`（趋势 `755`、Top N `766`）`scrollTop` 归零。
- `.report-archive-wrap`（`784`）同样归零。
- `.workspace`（`styles.css:384–389`）在报告页可滚，也被重建。
- `<details class="report-notes">`（`666`）输出时无 `open`，原生展开态随节点销毁。

实时表不受同一症状，是因为 `1153–1209` 专门写回 `.live-table-wrap`。报告页没有等价捕获。

## 图表

`PieSlice`（`format/report-svg.ts:3–5`）只有 `kind` + `value`。`renderReports` `687–690` 丢掉 `label` / `identity` / `share`。扇区与柱无 `data-*`、无焦点、无 hover CSS。`role="img"`。即使停止无关键重绘，指针也读不出 host。

展示层已有 `ShareRow.label`、`upload`、`download`、`share` 和 `TrendPoint.bucketUtc`。探查接到这些字段即可，不必重算 Top N。

## 复用

- 捕获 / 写回：扩展现有 live-table 滚动恢复，不要引入虚拟 DOM。
- 无关键 `paint`：`route === "reports"` 且消息不改变报告视图时直接返回。
- 命中区：纯函数仍放 `format/report-svg.ts`；tooltip 用 `position: fixed` 逃出 `overflow: auto`。

## 非原因

- 不是 CSS `overflow` 把滚轮锁死。
- 不是 `<details>` 缺少 click 监听；原生 summary 有效，失败在重绘。
- 不是 `ReportResult` 在 Channel 上被替换；报告仍走 `get_report_archive` / `run_report`。
