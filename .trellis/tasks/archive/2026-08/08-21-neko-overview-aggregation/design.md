# 设计：概览页与四个聚合页（前端）

## 1. 组件结构

```
components/features/overview/
  index.tsx              页面装配：CaliberGrid + TrendCard + TopColumns + CategoryTable
  caliber-grid.tsx       六格口径卡
  caliber-card.tsx       单格：上/下行成对读数或单值
  trend-card.tsx         趋势卡：档位切换 + 合计行 + TrendArea
  top-columns.tsx        三列 Top（主机 / 链路 / 进程）
  category-table.tsx     重点分类数据表（口径文本兜底）

components/features/dimension/
  dimension-page.tsx     四页共用骨架，由 DimensionKind 参数化
  rank-bar-card.tsx      顶部横向条形图 + Top N 档位
  rank-table.tsx         分页表 + aria-sort + 未知行渲染
  drilldown-panel.tsx    下钻区，由 drilldownCapability 驱动
  capability-note.tsx    能力不足时的中文说明块
```

`dimension-page.tsx` 只按 `DimensionKind` 参数化。四页差异仅在：标题、图标、`DimensionKind`、下钻目标维度列表。

## 2. 新增 UI 基元登记

本子任务在 `components/common/` 新建并登记（供家宽页与报告页复用）：

- `stat-card.tsx`：neko 统计卡外观。props `{ icon, label, value, subvalue?, color, loading?, unavailable? }`。`value` 接受 `string`（已格式化）以便传「未知」。`color` 为 hex（图标底色 `${color}15`）。
- `overview-card.tsx`：卡片容器。props `{ title, icon, action?, footer?, children }`。
- `top-list-item.tsx`：序号徽章 + 图标 + 标题 + 数值 + 占比进度条。props `{ rank, icon, title, subtitle?, value, total, color?, valueFormatter? }`。占比分母为传入的 `total`（调用方必须传 `totals`，不得传可见行之和）。

在 `components/charts/` 新建：

- `trend-area.tsx`：双序列面积图。输入 `{ data: { bucketUtc, upload, download }[]; loading?; emptyHint?; locale? }`，`locale` 默认 `zh`。颜色固定 `#3b82f6` / `#a855f7`，`isAnimationActive={false}`，高度 200px。报告页 `import` 此文件。
- `rank-bar.tsx`：横向条形图。输入 `{ data: { label, value }[]; loading?; emptyHint?; locale; valueFormatter? }` + 色序 `--chart-1..5`。

两个图表封装必须接受 `loading` 与 `emptyHint`，无数据时渲染虚线边框空态（对照 `ref/neko-master/.../trend-chart.tsx:235-243`），不渲染 0 高度的图。

`share-donut.tsx` 本子任务不用，归报告页子任务建立。

## 3. 数据 hook

新增 `hooks/use-report.ts`。按父任务 `design.md` 第 4 节的约束，`components/**` 不得直接 `invoke`。

报告页子任务复用同一 hook。`grouping` 取完整 `ReportQuery["grouping"]`（含 `category` / `network`），不限四个聚合页的 `DimensionKind`。

```ts
function useReport(input: {
  grouping: ReportQuery["grouping"];
  timeRange: TimeRange;
  granularity: ReportQuery["granularity"];
  topN: number;
  filters?: ReportFilters;
  enabled?: boolean;
  sort?: ReportQuery["sort"];
}): { result: ReportResult | null; loading: boolean; errorZh: string | null }
```

内部：请求序号递增、过期响应丢弃、`timeRange` 归整到分钟边界、失败保留上次结果、不缓存 `drilldownCapability`（每次成功响应用新 `ReportResult` 整份替换）。

概览页调三次（grouping 分别 `host` / `chain` / `process`；趋势图复用 host 查询的 `series` 与 `totals`，避免第四次查询）。四个聚合页各调一次，下钻再调一次（带 `filters`）。

登记导出（报告页 `import` 此文件）：

- `useReport`
- `buildReportQuery`
- `snapTimeRangeToMinute`
- `granularityForTrendPreset` / `granularityForTimeRange`
- `emptyReportFilters` / `filtersForDrilldown`
- `TREND_PRESETS` / `UNKNOWN_RANK_IDENTITY`
- `beginReportRequest` / `finishReportRequest`

## 4. 趋势图档位与 bucket

C3 已落地。JSON 字面量：`minute1` `minute2` `minute5` `minute10` `hour` `day` `month`。既有 `hour` / `day` / `month` kebab-case 不变。

| 档位 | 区间 | 请求 granularity |
|---|---|---|
| 30 分钟 | now-30min → now | `minute1` |
| 1 小时 | now-1h → now | `minute2` |
| 24 小时 | now-24h → now | `minute10` |

顶栏其余预设映射（不自动回退）：`5m` → `minute1`；`today` → `minute10`；`7d` → `hour`；`30d` → `day`。分钟档落到非 raw tier 时后端返回 `capability_unsupported`，前端显示能力说明，不升粒度。

不实现 neko 的 today 档与稀疏数据自动回退：数据稀疏时应显示 coverage 缺口，不偷偷换粒度。

## 5. 下钻

下钻用被点行的 `identity` 填 `ReportQuery.filters` 的对应字段，再以新 `grouping` 查一次。`ReportFilters`（`src/dto.ts:261-268`）已有六个字段，够用。

下钻目标维度：

| 当前页 | 下钻目标 |
|---|---|
| 主机 | 规则 / 链路 / 进程 |
| 规则 | 链路 / 主机 |
| 链路 | 规则 / 主机 |
| 进程 | 主机 / 链路 |

前提：`filters` 在 totals / series / rankings 三处都生效，且 `filters.chain` / `filters.rule` 的匹配语义与排名键一致——两者都由 `08-21-c3-dimension-capability` 保证。**本子任务不自行改 SQL**；若 C3 子任务尚未落地，下钻区渲染 `capability-note` 说明「该能力待后端就绪」，不展示可能是全局数据的排名或趋势。

`identity == "__unknown__"` 的行不提供下钻：`filters` 无法表达「维度值缺失」。哨兵字面量是 `"__unknown__"`（C3 已落地）；后端 `label` 为 `"未知"`；前端按当前语言的 `common.unknown` 渲染。

下钻 `filters.rule` 必须用排名行 `identity`（SQL 规则键），不要拼 `rule(payload)`。`filters.chain` 是最后一跳，不是完整 `a>b>c`。

## 6. 兼容与回滚

- 无 Rust 改动，无 DTO 改动。
- 概览页与四个聚合页替换 `<PagePending />`；回滚是把路由分发改回占位。
- 新增基元落在 `components/common/` 与 `components/charts/`，后续子任务复用不重造。

## 7. 开放项

- 三档趋势图 granularity 已按 C3 交接回填第 4 节：`minute1` / `minute2` / `minute10`。
- `__unknown__` 哨兵已确认：identity `"__unknown__"`，后端 label `"未知"`，不参与下钻。
- GUI 实拍（四主题 × 中英文 × 宽/窄、超 raw 期下钻消失、C3 下钻子集）留待有 Tauri 窗口时补。
