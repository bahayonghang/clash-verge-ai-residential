# 技术设计：分析报告页档案后置与结果可视化

## Architecture and Boundaries

只改 `residential-monitor` 前端壳层。Rust / SQLite / `ReportResult` / 导出格式不动。

| 层 | 职责 |
|---|---|
| `format/report-view.ts` | 展示比：其余差额、份额、空名称、queryEcho → 表单、趋势点。纯函数，可单测。 |
| `format/report-svg.ts` | 由展示切片生成 SVG 字符串。无 DOM、无 i18n。 |
| `main.ts` `renderReports` | 阅读顺序、工具条 `selected`、`<details>`、指标格、图+表、档案短表。 |
| `styles.css` | 报告页网格、图+表分栏、档案 `max-height`、条宽、切片色 token。 |
| `i18n/zh.ts` `en.ts` | 其余、档案筛选、窗口来源、图 `aria-label`。 |

不在 `renderReports` 里现场算弧度。禁止 npm 图表包、CDN、远程字体。

## Data Flow

```
list_report_archives(kind?) → ReportArchivePage（最多 50）
get_report_archive / run_report → ReportResult
        ↓
reportShareModel(result)  → 排名行 + 可选其余行 + pieDraw: boolean
reportTrendModel(series)  → 点列 / 单桶标记 / 空
        ↓
reportPieSvg / reportTrendSvg → 内联 SVG
        ↓
图表 + 对应表 + 导出仍用 result.reportSnapshotToken
```

`其余` = `totals.download - sum(rankings.download)`。正差额只进入展示模型。不写回 `ReportResult.rankings`。

## Share and Pie Contract

输入：`totals.download`、`rankings[].download`、`drilldownCapability.exactTopN`。

| 条件 | 扇形图 | 表份额分母 |
|---|---|---|
| `exactTopN === false` | 不画 | 现有空能力文案 |
| `totals.download === 0` | 不画 | 「未知」 |
| `sum(rankings.download) > totals.download` | 不画 | 仍用 `totals.download`，百分比合计可超过 100% |
| 其余 > 0 | 画排名扇区 + 其余 | 同一分母 |
| 其余 = 0 且分母 > 0 | 只画排名扇区 | 同一分母 |

单扇区 100% 画整圆，不用 0 长度弧。其余行上行是「—」（i18n），不是 0。空 `label` 用现有 `unknownOr`。

切片色：每口味 6 个 `--chart-n`，循环。其余用 `--muted`。颜色不是唯一编码，表提供名称和百分比。

## Trend Contract

- 多桶：两条折线，Y 从 0 到 `max(upload, download, 1)`。0 是观测值，不是把缺口补成 0。
- 单桶：并排柱，不连假线。
- 空 `series`：无 path。
- 表列与现有一致，表体滚动，避免 24 桶把 Top N 顶出首屏。

## Page Composition

默认主区约 980×800（侧栏 13.75rem）。

1. `.report-toolbar`：预设、粒度、维度、主按钮「运行报告」、次要导出。
2. 状态一行 + 覆盖一句；`<details>` 收起 noteZh / 策略。
3. `.report-metrics`：三格上行 / 下行 / 连接。
4. `.report-visuals`：两列。左趋势图+表，右扇形图+Top N 表。`max-width: 64rem` 时改一列。
5. `.report-archives`：类型筛选 + 约 8 行高的滚动表。

视图状态（只留当前会话）：

- `reportForm`: `{ preset, granularity, grouping, windowSource: "preset" | "archive" }`
- `archiveKindFilter`: `"all" | "hour" | "day"`

`windowSource === "archive"` 且跨度无法映射到 3600 / 86400 / 7d / 30d 时，预设控件旁显示「档案窗口」，`buildQuery` 若用户未改预设则沿用当前 `queryEcho` 的 range；用户改成明确预设后按滚动窗口跑手动报告。

档案筛选 `change` 只再调 `list_report_archives`。当前已加载结果保持，直到用户点另一行或运行报告。

## Compatibility

- Command 形状不变：`run_report`、`get_report`、`list_report_archives`、`get_report_archive`、`export_report`。
- 进页仍先最新成功日、否则最新成功小时。
- Recovery Shell 仍不渲染五页。
- 打印：现有 `@media print` 隐藏侧栏和按钮；SVG 随主区打印。

## Trade-offs

- 其余放在表里，满足「图表必须有对应数据表」，导出仍无其余行。
- 排名之和大于总量时宁可不画饼，避免扇区重叠或把超额压进 100%。
- 档案不改 `ARCHIVE_LIST_MAX`。缩短靠 CSS 高度和 `kind` 筛选。

## Rollback

恢复 `renderReports` 旧四段顺序，删除 `format/report-*.ts` 与新增 CSS / i18n 键。
