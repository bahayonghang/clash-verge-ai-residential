# 设计：报告 / 告警 / 设置页

## 1. 组件结构

```
components/features/reports/
  index.tsx              页面装配
  query-form.tsx         preset + 区间 + 分组 + topN + 对比开关
  archive-list.tsx       归档列表 + kind/status 过滤 + 选中
  totals-row.tsx         totals + 环比 + 占比读数
  share-donut-card.tsx   ShareDonut + 同口径数据表
  trend-card.tsx         TrendArea + 同口径数据表
  ranking-table.tsx      rankings 表 + aria-sort
  coverage-panel.tsx     coverage.status / coveredSec / gapSec / slices
  capability-panel.tsx   drilldownCapability + policyMetadata + dataTier + namedSql
  export-panel.tsx       preview_export + export_report + RedactMode
  inspect-context.tsx    hover / pinned 状态与高亮联动

components/features/alerts/
  index.tsx
  rule-list.tsx / rule-editor.tsx
  center-list.tsx        分页 + 五种 status
  evidence-panel.tsx     AlertEvidence
  notify-panel.tsx       NotifyCapability + 测试通知
  diagnostics-panel.tsx  19 字段 + 导出 + outbox 扫描

components/features/settings/
  index.tsx              五分区 tab 壳
  appearance-section.tsx theme / font-picker / fontSize / density / locale / sidebarWidth
  connection-section.tsx address / secret / test / disconnect / reconnect / targets
  data-section.tsx       retention / backup / restore / validate / dirs / vacuum
  about-section.tsx
  danger-section.tsx     delete preview + 确认短语 + 逐项结果
  operation-progress.tsx OperationProgress + 取消
  font-picker.tsx

components/features/recovery/
  index.tsx              renderRecovery 的替代
  unavailable.tsx        renderUnavailable 的替代
```

## 1b. IPC 边界

按父任务 `design.md` 第 5 节，`components/**` 不得直接 `invoke`。本子任务建立两个 hook 并复用一个：

- `hooks/use-report.ts` —— 由 `08-21-neko-overview-aggregation` 建立，本子任务复用于报告页（`enabled: false` 以免与手动查询双发）。归档、导出与报告页显式 `run_report` 放在同目录 `use-report-archive.ts`（已登记：不改 `use-report` 核心查询 API）。
- `hooks/use-alerts.ts` —— `list_alert_rules` / `upsert_alert_rule` / `list_alert_center` / `alert_summary` / `test_notification` / `get_diagnostics` / `export_diagnostics` / `scan_outbox`。
- `hooks/use-settings.ts` —— 设置页的全部命令，含 `OperationProgress` 的订阅与取消。

三者都实现请求序号递增与过期响应丢弃、失败保留上次结果并单独暴露 `errorZh`。

## 2. inspect 交互的重建

现有 inspect 与手写 SVG 的 DOM 结构耦合，换 Recharts 后按 React 状态重建。

删除清单已由父任务 `design.md` 第 2 节「可删层」明确批准，本节只写替代方案：

- **保留**（纯函数，继续调用）：`rankingInspectKey`、`trendInspectKey`、`inspectGroup`、`inspectKeysMatch`、`reportInspectModel`。
- **删除**（为整页重绘模型而存在）：`readReportScroll`、`writeReportScroll`、`inspectKeyExists`、`shouldSkipRoutinePaint`，以及 `main.ts:695-726` 的 `activeInspectKey` / `inspectMarkClass` / `inspectTipText` 和模块级 `reportInspectPinned` / `reportInspectHover`。
- 替代：`inspect-context.tsx` 提供 `{ pinned, hover, setPinned, setHover }`，各图表与表格通过 context 读写。`reportInspectModel` 的入参从 DOM 查询改为直接传 `pinned ?? hover`。
- `shouldSkipRoutinePaint` 不再需要：React 重渲不会丢 DOM 状态，高亮存在 state 里。
- 失效清理（`main.ts:1461-1468`）改为：新 `ReportResult` 到达时，若 `pinned` / `hover` 的 key 不在新数据的 key 集合里则清空。判定用 `rankingInspectKey` / `trendInspectKey` 重算，不查 DOM。

`report-inspect.test.ts` 中覆盖被删函数的用例随之删除，覆盖保留函数的用例不动。

实施记录：删除 9 个用例。`shouldSkipRoutinePaint` 3 个（整页重绘抑制，React state 不再需要）；`report-svg.test.ts` 6 个（手写 SVG 被 Recharts 取代）。保留 `rankingInspectKey` / `trendInspectKey` / `inspectGroup` / `inspectKeysMatch` / `reportInspectModel` / `applyReportScrollReset` 的既有断言。

## 3. `report-svg.ts` 的处置

`reportPieSvg` / `reportTrendSvg` 及 `report-svg.test.ts` 删除。改用 `components/charts/{share-donut,trend-area}`。

`report-view.ts` 的 `reportShareModel` / `reportTrendModel` 是**数据模型**而非渲染，继续使用：`ShareDonut` 吃 `ShareModel.rows`，`TrendArea` 吃 `TrendModel.points`。`TrendSliceName` 这个回调类型随 `report-svg.ts` 一起删除；若 Recharts 的 tooltip 需要等价能力，在 `trend-card.tsx` 里以 props 传入格式化函数。

## 4. 滚动位置

现有 `readReportScroll` / `writeReportScroll` 是为整页重绘补偿。React 下报告页的滚动容器不被卸载，位置天然保持。`report-notes` 的 `<details open>` 状态改为 React state，不再靠重绘前抓取。

切换 preset / 归档时是否需要滚回顶部：现有 `reportScrollReset` 标志（`main.ts:1388-1389`）有这个语义，改为在 preset / 归档变化时显式 `scrollTo(0, 0)`，行为与改造前一致。

## 5. secret 处理

`src/ipc/secret-field.ts` 只被调用，不改。密码输入用受控 `<input type="password">`，值存 React state。三条禁令在实施与检查时逐条确认：

- 不写进任何 DOM 属性（不用 `data-*`、不用 `value` 之外的属性、不用 `title`）。
- 不进 `console`（含 `console.error` 的错误对象透传）。
- 不进 `zh.ts` / `en.ts` 的插值参数。

## 6. 危险区的部分失败

`DeleteReport.allDeclaredOk` 为 false 时，摘要文案必须来自 `summaryZh`，不得由前端拼「已全部删除」。实施时在 `danger-section.tsx` 里只有一条摘要渲染路径，且该路径不含任何硬编码的成功文案。测试用 `allDeclaredOk: false` + 部分 `ok: true` 的假响应断言渲染结果不含「已全部删除」。

## 7. 文档更新的最小 diff

只改与新栈冲突的句子，不重排文档结构：

- `PRODUCT.md:33`：「固定五页」→ 列出十段路由。
- `PRODUCT.md:37`：改栈描述，保留「禁止远程 URL 和 CDN」。
- `PRODUCT.md:38`：**不动**。
- `DESIGN.md`：front-matter 的 colors / typography / rounded / spacing / components 换成 neko 令牌值；正文的 Colors / Typography / Layout / Components 章节改写；`:167` 的三条 Don't 保留两条（远程字体、CDN），删「不引入 UI 框架」；「Don't 恢复顶栏横导航」保留（本次顶栏只放工具，不做主认页）。
- frontend spec `index.md`：Pre-Development Checklist 与 Quality Check 两节改栈，保留两条既有质量门。

## 8. 兼容与回滚

- 无 Rust 改动，无 DTO 改动。
- 删除的前端文件：`src/format/report-svg.ts` + 其测试；`report-inspect.ts` 的四个 DOM 依赖函数 + 其用例。回滚需一并还原。
- 回滚是把三条路由改回 `<PagePending />`；文档改动可独立回滚。
