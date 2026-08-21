# 实施：报告 / 告警 / 设置页

## 报告页

1. [x] 建 `components/charts/share-donut.tsx`（若聚合页子任务未建）；`trend-area.tsx` 复用。
2. [x] 写 `inspect-context.tsx`：`{ pinned, hover, setPinned, setHover }`；`reportInspectModel` 入参改为直接传 key。
3. [x] 删 `src/format/report-svg.ts` 与 `report-svg.test.ts`；删 `report-inspect.ts` 的 `readReportScroll` / `writeReportScroll` / `inspectKeyExists` / `shouldSkipRoutinePaint` 及其用例，把删除的用例数与理由记入 `design.md`。
4. [x] 写 `query-form` / `archive-list` / `totals-row` / `share-donut-card` / `trend-card` / `ranking-table` / `coverage-panel` / `capability-panel` / `export-panel`，`index.tsx` 装配。
5. [x] 新 `ReportResult` 到达时清理失效的 pinned / hover key（用 `rankingInspectKey` / `trendInspectKey` 重算，不查 DOM）。
6. [x] preset / 归档变化时显式滚回顶部；`report-notes` 的展开状态改 React state。

## 告警页

6b. [x] 写 `hooks/use-alerts.ts`（八个告警/诊断命令，含竞态与失败保留）。
7. [x] 写 `rule-list` / `rule-editor`：三种 kind、四种 selectorKind、三种 direction、三种 period、阈值、恢复阈值、冷却、静默窗口、时区。
8. [x] 写 `center-list`（分页 + 五种 status，`not-evaluable` 单独态）与 `evidence-panel`（`observedValue` 为 null → 「未知」）。
9. [x] 写 `notify-panel`（四个能力字段 + 测试通知）与 `diagnostics-panel`（19 字段 + 导出 + outbox 扫描）。

## 设置页

9b. [x] 写 `hooks/use-settings.ts`（设置页全部命令 + `OperationProgress` 订阅与取消）。
10. [x] 写五分区 tab 壳与 `appearance-section`（含 `font-picker`，复用 `visibleFontChoices`）。
11. [x] 写 `connection-section`：地址、secret 回填与显示切换（走 `src/ipc/secret-field.ts`）、测试、断开、重连、targets 编辑。
12. [x] 写 `data-section`：保留预览与执行、备份、恢复、校验、数据目录、日志目录、vacuum。
13. [x] 写 `about-section`：`decodeAbout` 断言不放宽；补测试用 `signed: true` 的伪造响应断言抛错。
14. [x] 写 `danger-section`：删除预览、中文确认短语、逐项结果。只有一条摘要渲染路径且不含硬编码成功文案；补测试断言部分失败时不含「已全部删除」。
15. [x] 写 `operation-progress.tsx`：phase / current / total / unit / canCancel / status / redactedError + 取消。
16. [x] secret 三条禁令逐条源码确认：不进 DOM 属性、不进 console、不进 i18n 插值。

## Recovery 与不可用态

17. [x] 写 `features/recovery/index.tsx` 与 `unavailable.tsx`，替代 `renderRecovery` / `renderUnavailable`。

## 收口

18. [x] 补 `zh.ts` / `en.ts` 新键（既有键沿用不改名）；跑键集合一致性测试。
19. [x] 替换三条路由的 `<PagePending />`。
20. [x] 更新 `PRODUCT.md:33,37`（`:38` 不动）、`DESIGN.md`、`.trellis/spec/residential-monitor/frontend/index.md`；两条既有质量门在 spec 中保留。用 `git diff PRODUCT.md` 确认第 38 行未改。
21. [x] `CHANGELOG.md` 加 English 条目；`residential-monitor/docs/{first-run,reporting,alerts}.md` 中文同步。
22. [x] `npm --prefix residential-monitor run typecheck && lint && test && build`；`cargo test --workspace` 确认未受影响。

## 实拍

23. [ ] 报告页：五个 preset、归档选中、`reportSource` 四态、hover / pinned 高亮、导出预览与导出。GUI 缺口：本环境无应用窗口，未实拍。
24. [ ] 告警页：`not-evaluable` 与「无告警」分开；`observedValue` 为 null 显示「未知」；通知能力四态。GUI 缺口：本环境无应用窗口，未实拍。
25. [ ] 设置页：secret 回填与显示切换；长操作进度与取消；删除本地数据的部分失败路径。GUI 缺口：本环境无应用窗口，未实拍。
26. [ ] 四款主题 × 中英文 × 1200×800 / 窄窗口；`aria-sort` 与键盘可达。GUI 缺口：本环境无应用窗口，未实拍。

## 回滚点

第 19 步之前三页仍是占位，可随时中止。第 3 步删了文件与测试用例，回滚需一并还原。第 20/21 步的文档改动可独立回滚。
