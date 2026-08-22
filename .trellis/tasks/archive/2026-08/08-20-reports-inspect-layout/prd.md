# 分析报告页探查与排版

## Goal

操作员在「分析报告」阅读当前窗口的占用构成时，Top N 表滚动位置和「数据层与能力」展开状态保持住；柱状图 / 扇形图可按切片查看名称、流量和份额，点击后钉住。同一页在默认 1200×800 下按专家工具密度排开总量、趋势、Top N 和档案，不因实时 Channel 增量把结果区打回初始态。

## User value

分析报告是冻结快照页。实时连接仍在后台更新，不能把这份快照的阅读状态冲掉。操作员需要在 Top N 里向下看低份额行、对照扇区看是谁占用最高，而不是反复从第一行或收起的说明重新开始。

## Background

2026-08-20 用户截图（分析报告 / Mocha，最近一小时自动档案已加载）：红框「精确 Top N」表体滚轮下滑后回到顶端；「数据层与能力」点开后立即合上；趋势柱和 Top N 扇形图不能悬停或点击查看占用最高的 host。

沿用归档任务 `08-19-reports-page-layout`：同一 `ReportResult`；前端不分类、不守恒、不重算 Top N、不传 SQL；禁止 UI 框架 / CDN / 远程 URL；缺口、未知、能力不支持不得画成 0；点选档案走 `get_report_archive`；手动查询与导出不覆盖自动档案；Catppuccin 四口味，不换壳。

视觉世界以现网 `styles.css` 的 Catppuccin token 和 `.impeccable/surfaces/residential-monitor-src-main-ts.md` 的 Operate 方向为准。根目录 `DESIGN.md` 仍写浅主区，与现网 Mocha 主区不一致；本任务不修复该文档漂移。

机制证据见 `research/report-paint-reset.md`。摘要：`renderApp` 每次 `paint()` 都 `innerHTML` 重建整页，只恢复 `.live-table-wrap`；`handleMonitorRaw` 对每条 `connectionDelta` 无条件 `paint()`。`<details class="report-notes">` 输出无 `open`。`PieSlice` 只有 `kind`/`value`，扇区与柱无命中区。

排版对照同一截图：总量是独立全宽三格，右侧「连接」和 Top N 卡顶沿抢视线；`.report-visuals` 两列等权，单桶趋势图区空；扇形图叠在表上，表体写死 `max-height: 14rem` 内滚；报告页 `.workspace` 可滚，和两处内滚并存。密度按专家工具收紧，不加大留白。

## Requirements

### R1 报告阅读状态不被实时增量冲掉

- `connectionDelta` 以及仅为刷新实时快照的 Channel 消息不得把分析报告 DOM 打回初始态。
- 必要重绘（运行报告、加载档案、改预设、改档案筛选、切语言/主题/字号/密度、切路由回来）之后，仍恢复：趋势表、Top N 表、档案表的纵向滚动；报告页 `.workspace` 纵向滚动；「数据层与能力」的展开/收起；已钉住的探查项。
- 展开状态和钉住只留当前会话。离开 `reports` 再回来可以重置。合法新结果允许把表滚回顶端；已展开的 details 保持，除非新结果没有该块。钉住 key 在新结果中不存在则清除。
- 焦点恢复继续走现有元素 `id`。不得引入 UI 框架或虚拟 DOM 库。

### R2 图表探查同一 `ReportResult`

- 扇形图每个可见扇区、趋势图每个可见柱（单桶）或每个可命中桶（多桶）能用指针和键盘查出对应展示行：名称（空则「未知」）、上行/下行、份额（扇形/Top N）或时间桶（趋势）。
- 悬停或 `:focus-visible` 高亮图上命中切片，并同步高亮对应表行。点击切片、柱或对应表行钉住；再点同一项、点图表空白、或 Escape 取消。已钉住时悬停其他项为临时预览，移开回到钉住项。
- 「其余」按现有展示模型处理，不写回 `ReportResult`，不进导出。
- `exactTopN === false`、分母为 0、排名下行大于总量而不画饼时，表仍可扫读；无图则无图上探查。
- 禁止 npm 图表包、Canvas 图表库、CDN、远程字体。本地 SVG + 现有 i18n。
- 禁止按探查结果再算 Top N、改 grouping、或自动 `run_report`。

### R3 分析报告主区排版（Operate 精修）

- 总量改为结果区顶部的紧凑指标条，不再单独占一张全宽空卡。
- 默认 1200×800 保持趋势 | Top N 两列；容器变窄时改一列，DOM 顺序与视觉顺序一致。
- Top N：扇形图作为表的色例，与表同一视觉组。表仍是精确数字的权威面。表体使用结果区剩余高度，不再与外壳滚动对抢。
- 趋势：单桶柱和图下表仍配对；空 `series` 继续「无数据」，不画假零线。
- 查询工具条一行可扫；运行报告为主按钮，导出为次要。状态、覆盖、可展开的数据层说明仍在结果之前。
- 档案块仍在结果之后，约 8 行内滚，点选与 `aria-current` 保留。
- 四口味 token。切片色继续 `--chart-n` / `--chart-remainder`。控件具备默认 / hover / focus / disabled；探查命中有可见 focus。尊重 `prefers-reduced-motion` 和 `prefers-contrast: more`。
- 中英文 key 成对。不在主区重复「分析报告」标题。

### R4 口径与壳层边界

- 图、表、探查文案只读当前 `ReportResult`。百分比、弧度、条宽、其余差额仍是展示比。
- Command 形状不变：`run_report`、`get_report`、`list_report_archives`、`get_report_archive`、`export_report`。
- 不改 C3 档案生成、保留、导出字节内容、`ARCHIVE_LIST_MAX`。
- 不改侧栏、概览、实时连接、告警、设置页的信息架构。实时页现有滚动恢复不得回退。
- secret、原始流量内容、账单口径不得进入探查文案。

## Acceptance Criteria

- [ ] **AC1 滚动保持**：采集运行、分析报告已加载且 Top N 超过一屏时，把 Top N 表滚到非顶端后等待至少一次 `connectionDelta`（或等价的无关键更新）。表仍停在操作员离开的位置。趋势表与档案表同样成立。
- [ ] **AC2 details 保持**：展开「数据层与能力」后等待同上的无关键更新，块保持展开；`summary` 再点一次才收起。
- [ ] **AC3 图表探查**：悬停或键盘聚焦扇区 / 单桶柱时，能读到对应名称（或「未知」「其余」）和下行/份额（趋势则时间与上下行），对应表行同步高亮。点击钉住后移开指针仍保持；Escape 或再点同一项取消。
- [ ] **AC4 同源**：探查数字与当前表、当前 `reportSnapshotToken` 一致。导出仍无「其余」行。空名称显示「未知」。不画饼的三种条件保持 `08-19-reports-page-layout` R5。
- [ ] **AC5 首屏结构**：默认 1200×800、已有成功小时或日档案时，无需滚动即可看到总量数字，以及趋势或 Top N 至少一块。总量不再是一张几乎空白的全宽卡。自动档案在结果之下。
- [ ] **AC6 门禁**：`npm --prefix residential-monitor run typecheck`、`lint`、`test`、`build` 通过。新增展示纯函数测试覆盖：切片带标签、其余行、不画饼。抽出可测的「是否跳过 paint」谓词和捕获/恢复辅助。

## Out of scope

- 改 `ReportResult` / `report_archive` schema，或打开 C3 自动 DELETE。
- 改 HTML / CSV / JSON 导出内容，或给导出加图。
- 上环比卡（`previousUpload` / `previousDownload`）。
- 会话下钻、跨维下钻、改默认 `topN`。
- 档案无限滚动或提高 `ARCHIVE_LIST_MAX`。
- 换壳、改侧栏、改概览 / 实时 / 告警 / 设置页。
- 引入图表库、UI 框架、CDN、远程字体。
- 上行扇形图。
- 修复根目录 `DESIGN.md` 与 Catppuccin 现网的文档漂移。

## Key decisions

- 点击钉住当前探查项，不按 host 再跑报告（2026-08-20）。
- `route === "reports"` 时跳过无关键 `paint`，必要重绘再写回滚动 / details / 钉住。对齐实时表的捕获-写回，不引入虚拟 DOM。
- 扇形图分母与「其余」沿用 `08-19-reports-page-layout`：`totals.download`，正差额只存在于展示层。
- 排版是 Operate 精修：指标条并入结果区，扇形图作色例，报告页锁外壳滚动。不换视觉世界。

## Risks

- `handleMonitorRaw` 的跳过条件若写反，实时表会停更。谓词必须单测，并保留 `route === "live"` 时现有刷新。
- `:has(.reports)` 锁外壳滚动后，结果区必须 `min-height: 0` 且表体可伸展，否则 Top N 无法内滚。

## Artifact status

- `prd.md`：已过收敛。无阻塞开放问题。
- `design.md`：paint 策略、探查契约、页面结构。
- `implement.md`：有序清单与门禁。
- `research/report-paint-reset.md`：根因。
