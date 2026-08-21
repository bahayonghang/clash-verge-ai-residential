# 概览页与四个聚合页（前端）

## Goal

按 neko 视觉重做概览页（成对口径卡 + 分钟级流量趋势图 + Top 三列），并新增主机 / 规则 / 链路 / 进程四个聚合页。本子任务**只写前端**：全部 Rust 查询能力由 `08-21-c3-dimension-capability` 交付。

## 父任务

`.trellis/tasks/08-21-neko-ui-refactor`。目录约定、主题映射、图表封装边界与跨维降级表见父任务 `design.md` 第 1 / 3 / 5 / 6c 节。

## 依赖

- `08-21-neko-shell-foundation`：复用 `components/ui/`、`components/common/`、`useBootstrap`、`useMonitorStream`、`usePreferences` 与顶栏时间范围选择器。本子任务负责把时间范围选择器接到报告查询上。
- `08-21-c3-dimension-capability`：分钟档枚举值、`filters` 在 totals/series/rankings 三处生效、`exact_top_n` 的诚实语义、`__unknown__` 哨兵。本子任务的 AC2 / AC5 / AC6 / AC7 在 C3 子任务落地前无法通过。

本子任务**不改任何 Rust**。

## Confirmed facts

### 概览页现状与口径字段

- 现有概览在 `residential-monitor/src/main.ts:330-369`，五对口径由 `caliberPair`（`:309-328`）渲染，加一张活跃连接卡与一张「重点分类」表。
- 口径字段在 `residential-monitor/src/dto.ts:20-39` 的 `LiveOverview`：`meterUpload/Download`、`attributedUpload/Download`、`otherUpload/Download`、`gapUpload/Download`、`overUpload/Download` 全部是 `number | null`，`activeCount: number`，另有 `categoryUpload/Download: Record<string, number>`、`lastSampleUtc`、`coverageKind`、`coverageReason`、`health`。
- 「可归因观测」的定义是**每条连接 delta 之和，包含未分类连接**：`residential-monitor/src-tauri/src/accounting.rs:229-231` 对每条连接无条件 `attributed_up += delta_up`，`MinuteFact` 的 `primary` 可为 None 仍写入。它对照 controller meter 的全局计数，不是「已分类流量」。
- 分类行由 `residential-monitor/src/format/overview.ts:7-19` 的 `categoryRows` 生成：取 upload / download 两个 map 的键并集，缺失键给 `null`（不是 0），按 `localeCompare(zh)` 排序。这个「缺失即 null」的语义必须保留。
- 覆盖文案两分支在 `main.ts:342-344`：有 `coverageKind` 走 `overview.coverage_gap`，否则走 `overview.coverage_ok` 带最后采样时间。

### 报告查询契约

- `run_report` 入参 `ReportQuery`（`src/dto.ts:270-283`），返回 `ReportResult`（`:285-331`）含 `totals`（含 `previousUpload/Download`）、`series`、`rankings`、`coverage`、`drilldownCapability`、`policyMetadata`、`dataTier`、`namedSql`。
- `DimensionKind` 六值在 `residential-monitor/src-tauri/src/c3/query.rs:116-124`。
- `comparison.previousEqualWindow` 已支持等长前窗对比（`c3/service.rs:201-220`），可支撑环比标签。
- `drilldownCapability` 三档（`c3/query.rs:615-681`）：30 天 raw 期内三个 flag 全 true；超出后 `cross_dimension` / `sessions` 转 false；daily core 层 `exact_top_n` 也转 false。

### 目标视觉

- 统计卡 `ref/neko-master/apps/web/components/features/stats/stats-cards.tsx:68-108`（正常）、`:126-163`（占位 / 不可用）。
- 趋势图 `ref/neko-master/apps/web/components/features/stats/charts/trend-chart.tsx:249-435`；分档切换 `:52-63`；loading 骨架与空态 `:177-247`。
- 分钟 bucket 按区间自适应的思路见 `ref/neko-master/apps/web/components/overview/index.tsx:52-57` 的 `getMinuteBucket`（≤2h→1min，≤6h→2min，≤12h→5min，其余 10min）。
- Top 三列 `ref/neko-master/apps/web/components/overview/index.tsx:302-329`，列表项 `ref/neko-master/apps/web/components/common/top-list-item.tsx:46-98`。
- 聚合页形态：横向条形图 + 分页表 + 下钻 Tab，见 `ref/neko-master/apps/web/app/[locale]/dashboard/components/content/index.tsx:111-142`。

## Requirements

### R1. 概览页

- 顶部六格：controller meter、可归因观测、其他连接、未归因 gap、over-attributed、活跃连接。**前五格为上/下行成对读数**，第六格为活跃连接计数 + 覆盖状态 + 健康状态与中文下一步。
- 套用 neko 统计卡外观：`rounded-xl` 卡、图标底色为对应色 15% 透明度、标签 `uppercase tracking` 小字、数值 `tabular-nums`。
- `null` 显示「未知」，不显示 0。`prefers-reduced-motion` 下关闭数字过渡。
- 中部：分钟级流量趋势面积图，档位 30 分钟 / 1 小时 / 24 小时，bucket 按区间自适应。
- 下部：Top 主机 / Top 链路 / Top 进程 三列，每列可切换按流量 / 按连接数排序，带「查看全部」跳到对应聚合页。
- 保留「重点分类」数据表作为口径文本兜底，`categoryRows` 的 null 语义不变。

### R2. 四个聚合页

- 主机 / 规则 / 链路 / 进程四页共用一套骨架：顶部横向条形图（Top N）+ 中部分页表 + 底部下钻区。四页由 `DimensionKind` 参数化，不复制四份。
- 排名与合计一律取 `run_report` 的 `rankings` / `totals`；占比分母取 `totals`，不取可见行之和。
- Top N 档位提供 10 / 20 / 50 / 100。
- 排名中 identity 为 `__unknown__` 的行按「未知」渲染，不提供下钻入口。
- 表格保留 `<table>` 语义与 `aria-sort`。

### R3. 下钻与能力降级

- 下钻通过 `ReportQuery.filters` 二次查询实现，不新增命令。下钻时用被点行的 `identity` 填对应 `filters` 字段。
- `cross_dimension: false` 时隐藏下钻入口并显示 `note_zh`；`exact_top_n: false` 时排行区显示能力说明而不是空表。
- 不缓存 `drilldownCapability`：每次响应都用最新值渲染。

### R4. 请求竞态

- 请求序号递增，过期响应丢弃；`timeRange` 归整到分钟边界后入参，避免每秒重查；失败时保留上一次结果并单独暴露 `errorZh`，不清空界面。

### R5. 双语与检查

- 新增字符串同时进 `zh.ts` 与 `en.ts`。`src/i18n/index.test.ts:7-8` 已有键集合一致性断言，沿用即可，不新增重复测试。
- 图表旁必须有同口径的数据表或 Top 列表。

## Out of scope

- **不改任何 Rust**。分钟粒度、派生聚合键、filters 注入、五维物化、能力报告修正全部归 `08-21-c3-dimension-capability`。
- 不改 `src/dto.ts`（`granularity` 联合类型的扩展归 C3 子任务）。
- 不做 Regions 世界地图、不引入 MMDB。
- 不引入 `@xyflow/react` 链路 DAG。
- 不动实时连接页、家宽页、报告页、告警页、设置页。
- 不实现 neko 的 today 档、自定义日期区间与稀疏数据自动回退粒度（本产品用 coverage 缺口表达数据稀疏，不偷偷换粒度）。

## Acceptance Criteria

- [ ] AC1 (R1)：概览六格与 `LiveOverview` 字段一一对应；把 `meterUpload`、`gapUpload`、`overUpload` 置为 `null` 时界面显示「未知」，不显示 0；`categoryRows` 的缺失键仍为 null。
- [ ] AC2 (R1)：趋势图在 30 分钟 / 1 小时 / 24 小时三档下向后端请求分钟档粒度值，且请求的 bucket 与区间的映射关系有单测。
- [ ] AC3 (R1)：Top 三列的排序切换与「查看全部」跳转可用；三列数据分别来自 grouping `Host` / `Chain` / `Process` 的 `rankings`。
- [ ] AC4 (R2)：四个聚合页由同一骨架按 `DimensionKind` 参数化产出（源码级确认，无四份复制）。
- [ ] AC5 (R2)：四页的排名与合计均来自 `run_report`；前端源码中不存在对连接数组分组求和产出排名的代码路径；占比分母取 `totals`。
- [ ] AC6 (R2)：`identity == "__unknown__"` 的排名行按「未知」渲染且无下钻入口（有测试）。
- [ ] AC7 (R3)：`cross_dimension: false` 时下钻入口消失并显示 `note_zh`；`exact_top_n: false` 时排行区显示能力说明而非空表（有测试，用构造的 `ReportResult` 驱动）。
- [ ] AC8 (R3)：下钻用被点行的 `identity` 填 `filters`；下钻后的排名与趋势为子集而非全局（依赖 C3 子任务，实测确认）。
- [ ] AC9 (R4)：过期响应被丢弃、失败保留上次结果、`timeRange` 归整到分钟边界，三条各有测试。
- [ ] AC10 (R5)：`zh.ts` / `en.ts` 键集合一致（沿用 `src/i18n/index.test.ts` 的既有断言）；每个图表旁有同口径数据表或 Top 列表。
- [ ] AC11：`npm --prefix residential-monitor run typecheck && lint && test && build` 通过；本子任务无 Rust 改动，`cargo test --workspace` 仍通过。
- [ ] AC12 (R1/R2)：四款主题 × 中英文 × 1200×800 / 窄窗口实拍无溢出；`aria-sort` 与键盘可达；`prefers-reduced-motion` 下数字过渡与图表动画停止。
