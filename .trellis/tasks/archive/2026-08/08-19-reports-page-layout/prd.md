# 分析报告页档案后置与结果可视化

## Goal

用户进入「分析报告」后，先读当前报告的总量、趋势和 Top N（含按下行之和与总量对齐的扇形图），再在页尾用短列表点选自动档案。默认 1200×800 首屏能看到结果，不必先滚过几十行档案。

## Background

2026-08-19 用户截图（分析报告页 Mocha）：

- 首屏被「自动档案」占满。`renderReports`（`residential-monitor/src/main.ts` 491–494）把档案表放在总量 / 趋势 / Top N 之前。`list_report_archives` 一次最多 50 条（`c3/archive.rs` `ARCHIVE_LIST_MAX = 50`，`main.ts` 1083），全部渲染。`ReportArchiveSummary` 已有 `totalsDownload`，表未用。
- 档案说明、`drilldownCapability.noteZh`、策略说明、覆盖句叠在控件下方。
- 「趋势图对应数据表」只有表。`view-state.md` 要求图表配对应数据表。
- Top N 占比条相对 `rankings[0].download` 缩放（`main.ts` 431–433），最小宽 `0.4rem`。空 `label` 输出空单元格。
- 总量面板只有一行文案。进页加载自动日档案后，预设仍显示「最近一小时」：`<select>` 每次重绘回到第一项（`main.ts` 459–482）。

沿用已有约束：同一 `ReportResult`；前端不分类、不守恒、不重算 Top N、不传 SQL；禁止 UI 框架 / CDN / 远程 URL；缺口、未知、能力不支持不得画成 0；点选档案走 `get_report_archive`；手动查询与导出不覆盖自动档案；Catppuccin 四口味，不换壳。

## Requirements

### R1 阅读顺序

主区顺序：查询工具条 → 一行状态与压缩覆盖 → 当前结果（总量、趋势图+表、精确 Top N 扇形图+表）→ 页尾「自动档案」。

### R2 工具条与状态

- 工具条一行可扫。运行报告为主按钮，三种导出为次要按钮。
- 状态继续区分自动日 / 自动小时 / 本次手动查询 / 补跑 / 失败 / 空。
- 覆盖一句：覆盖状态、缺口秒、单位。`drilldownCapability.noteZh` 与策略说明默认收起。
- 视图状态保存预设 / 粒度 / 维度，重绘时回填 `selected`。
- 载入档案或手动结果后，粒度、维度回填 `queryEcho`。时间跨度能对应现有预设则选中该预设；否则工具条标明当前窗口来自档案，不把日历日显示成「最近一小时」。

### R3 总量

- 与导出同一 `reportSnapshotToken`。
- 指标格：上行、下行、连接数。`null` 显示「未知」。空报告显示「尚未运行报告。」

### R4 趋势图与对应表

- 用本地 SVG 画 `series` 的上行、下行。
- 同一 `series` 必须有对应表：时间、上行、下行。表体可内部滚动。
- 单桶也保留图+表。空 `series` 显示「无数据」，不画假零线。

### R5 精确 Top N：表 + 扇形图

- 只画下行扇形图。分母 = `totals.download`。各扇区 = 该行 `download`。
- 「其余」= `totals.download - sum(rankings.download)`。差值 > 0 时，扇形图与排名表都出现「其余」行。该行是展示差额，不写入 `ReportResult`，不进导出。
- 表列：名称、上行、下行、份额（相对 `totals.download` 的数字百分比 + 条）。空或空白 `label` 显示「未知」。其余行上行显示「—」，不填 0。
- 不画扇形图的情况：`drilldownCapability.exactTopN === false`；`totals.download === 0`；排名下行之和大于 `totals.download`。此时表仍按 `totals.download` 列份额（分母为 0 则份额显示「未知」），能力不支持走现有「无数据或能力不支持」。
- 禁止相对第一名缩放占比条。

### R6 自动档案后置并缩短

- 档案块在结果区之后。可见约 8 行，超出块内滚动。仍最多持有后端 50 条，不改 `ARCHIVE_LIST_MAX`。
- 行点选与 `aria-current="true"` 保留。失败行可点，不覆盖已成功结果。
- 列：时间、类型、下行、状态。下行取 `totalsDownload`，`null` 显示「未知」。
- 类型筛选：全部 / 自动日 / 自动小时，走 `list_report_archives.kind`。筛选只改列表，不加总档案。

### R7 口径、主题、语言

- 图和表只读当前 `ReportResult`。百分比、弧度、条宽、其余差额属于展示比。禁止重算排名或跨档案汇总。
- 四口味 token。禁止 CDN、chart 库、远程字体。
- 中英文 key 成对。

## Out of Scope

- 改 `ReportResult` / `report_archive` schema，或打开 C3 自动 DELETE。
- 改 HTML / CSV / JSON 导出内容，或给导出加图。
- 上环比卡（`previousUpload` / `previousDownload`）。
- 会话下钻、跨维下钻、改默认 `topN`。
- 档案无限滚动或提高 `ARCHIVE_LIST_MAX`。
- 换壳、改侧栏、改概览 / 实时 / 告警 / 设置页。
- Windows Service、云同步、邮件推送。
- 上行扇形图。

## Acceptance Criteria

- [ ] **AC1 首屏是结果**：默认 1200×800、已有成功日档案时，进入分析报告页无需滚动即可看到总量，以及趋势或 Top N 至少一块。自动档案在这两者之下。
- [ ] **AC2 档案短列表**：档案块可见约 8 行；50 条不把结果顶出首屏。点选成功档仍加载冻结 `ReportResult`。类型筛选只改列表，不加总。
- [ ] **AC3 图表明细同源**：趋势 SVG 与趋势表来自同一 `series`；扇形图与 Top N 表来自同一 `rankings`（其余行除外，其余行只来自总量减排名下行）。导出仍用当前 token，屏上总量与导出一致。
- [ ] **AC4 份额诚实**：Top N 份额与扇形图分母均为 `totals.download`。其余仅在差值 > 0 时同时出现在图和表。空名称显示「未知」。`exactTopN` 不支持、分母为 0、或排名下行之和大于总量时无扇形图。
- [ ] **AC5 控件与档案一致**：加载自动档后，粒度 / 维度与 `queryEcho` 一致。日历日窗口不显示成「最近一小时」。
- [ ] **AC6 门禁**：`npm --prefix residential-monitor run typecheck`、`lint`、`test`、`build` 通过。

## Key Decisions

- 扇形图分母取 `totals.download`，其余为总量减 Top N 下行（用户 2026-08-19 选 A）。
- 其余只存在于当前页展示，不改 DTO、不改导出。
- 单任务交付阅读顺序、档案高度和结果图。
- 视觉世界沿用 Catppuccin，只改分析报告主区。
