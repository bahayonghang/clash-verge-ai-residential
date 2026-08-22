# 技术设计：分析报告探查与排版

## Architecture and Boundaries

只改 `residential-monitor` 前端壳。Rust / SQLite / `ReportResult` / 导出格式 / 侧栏和其他四页信息架构不动。

| 层 | 职责 |
|---|---|
| `format/report-view.ts` | 已有展示比。可补探查 key（`identity` / `remainder` / `bucketUtc`）。不在此算弧度。 |
| `format/report-svg.ts` | SVG 命中区：`data-inspect`、`tabindex`、切片 class。仍无 DOM、无 i18n 句子。 |
| `format/report-ui-state.ts`（新建，可选） | 捕获/写回滚动、`details.open`、钉住 key。纯函数 + 最小 DOM 读写，便于单测。 |
| `main.ts` | `handleMonitorRaw` 跳过无关键 `paint`；事件委托悬停/钉住；`renderReports` 结构。 |
| `styles.css` | 报告页填满工作区、指标条、扇形图作色例、探查高亮与 fixed tooltip。 |
| `i18n/zh.ts` `en.ts` | 探查句、钉住 `aria`。 |

禁止 npm 图表包、Canvas 库、UI 框架、CDN、远程字体。

## Data Flow

```
Monitor Channel ──► reduceMonitor ──► 内存 state
                         │
                         ├─ route === "reports" 且非 bootstrap
                         │    且 errorZh 未变 ──► 不 paint、不 query_live
                         │
                         └─ 必要 paint
                              捕获 ReportUiState → innerHTML → 写回

ReportResult ──► reportShareModel / reportTrendModel
                 ──► reportPieSvg / reportTrendSvg（带 data-inspect）
                 ──► 表行 data-inspect 同一 key
                 ──► 悬停/焦点临时探查；点击钉住
```

探查文案只格式化展示模型已有字段。禁止按点击改 `grouping`、改 `filters`、或自动 `run_report`。

## Paint Policy

`handleMonitorRaw` 在 `route === "reports"` 时：

- `connectionDelta` / `healthChanged` / `summaryChanged` / `alertChanged`：更新内存 `state`，不 `refreshLivePage`，不 `paint`。
- 若 `state.errorZh` 与上次 paint 时不同：仍 `paint`，以便主区 `role="alert"` 出现或消失。
- `bootstrap`：允许 `paint`（订阅重建 / 冻结提示）。

必要 `paint` 的来源不变：运行报告、加载档案、改预设、档案筛选、切路由、切语言/主题/字号/密度。

每次实际 `paint`：

1. 捕获 `.workspace` `scrollTop`、`[data-report-scroll]` 各容器 `scrollTop`、`.report-notes` 的 `open`、当前钉住 key。
2. `innerHTML`。
3. 写回上述状态；按钉住 key 给图和表加 `data-inspect-current`。

给趋势表、Top N 表、档案表分别加 `data-report-scroll="trend|topn|archive"`，不要靠两个 `.report-table-wrap` 的查询顺序。

离开 `reports` 时丢弃滚动、details、钉住。加载另一份 `ReportResult`（档案或手动）时表滚动归零，details 与钉住按 PRD：details 保持；钉住 key 若在新结果中不存在则清除。

## Inspect Contract

Key：

| 表面 | key | 表行 |
|---|---|---|
| 扇形 / Top N | `rank:{identity}` | 同 identity 的排名行 |
| 扇形 / Top N | `remainder` | `kind=remainder` 行 |
| 趋势单桶柱 | `trend:{bucketUtc}:up` / `:down` | 该时间行（两柱共用一行，高亮整行） |
| 趋势多桶 | `trend:{bucketUtc}` | 该时间行。多桶折线命中用沿折线的不可见宽命中区或按 x 映射最近桶，不得只靠 2px stroke。 |

交互（Q1 已确认：点击钉住）：

- 未钉住：悬停或 `:focus-visible` 显示临时探查，高亮对应切片和表行。
- 点击切片、柱或对应表行：钉住。再点同一 key、点图表空白、或 Escape：取消。
- 已钉住时悬停其他 key：临时预览；`mouseleave` 回到钉住项。
- 键盘：切片与单桶柱 `tabindex="0"`；Enter / Space 钉住或取消。表行不改成按钮，点击行也可钉住。
- Tooltip：`#report-inspect-tip`，`position: fixed`，`role="status"`。内容：名称（空则「未知」）、下行或上下行、份额或时间。字节走现有 `formatBytes`。
- `prefers-reduced-motion`：高亮无过渡。

SVG 继续本地生成。`aria-label` 保留整图摘要；切片另用 `aria-label` 短句，避免只有颜色编码。

## Page Composition

默认主区约 980×800。Operate 精修，不换 Catppuccin 壳。

```
.reports 填满 #view（workspace:has(.reports) overflow:hidden，对齐 live-page）
  1. 查询面板：工具条、状态、覆盖、details
  2. 结果面板（flex:1，min-height:0）
       紧凑指标条（上行 / 下行 / 连接）
       .report-visuals 两列（flex:1）
         趋势：SVG + 可伸展表体
         Top N：左色例扇形 + 右表（窄容器改一列）
  3. 档案面板：筛选 + 约 8 行内滚
```

- 总量不再独占一张全宽空卡。
- Top N 表体用剩余高度，取消写死 `max-height: 14rem` 与结果区抢滚动；档案保持约 8 行封顶。
- 扇形图 `align-self: start`，表是数字权威面。
- `@container (max-width: 48rem)` 结果区改一列，DOM 顺序：指标 → 趋势 → Top N。

## Compatibility

- Command 与 DTO 不变。
- 进页仍先最新成功日、否则最新成功小时。
- 实时页 `.live-table-wrap` 恢复不得删掉。
- Recovery Shell 仍不渲染五页。
- 打印：现有 `@media print`；tooltip 不打印。

## Trade-offs

- 跳过 `paint` 比只恢复滚动更稳：悬停层不会被下一帧 `connectionDelta` 拆掉。恢复仍保留，覆盖运行报告等必要重绘。
- 不在报告页预取 `query_live_connections`。切回实时页时现有 `refreshLivePage` 会拉第一页。
- 多桶趋势不做精确点命中像素艺术；最近桶足够回答「哪一段时间最高」。

## Rollback

1. 恢复 `handleMonitorRaw` 无条件 `paint`。
2. 去掉 `data-inspect` 与 tooltip。
3. 恢复 `renderReports` 旧面板切分和 `max-height: 14rem`。
4. 删除 `report-ui-state` 辅助（若已抽出）。
