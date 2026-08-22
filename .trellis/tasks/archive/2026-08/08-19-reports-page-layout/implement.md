# 实施计划：分析报告页档案后置与结果可视化

## 启动前门禁

- [ ] 用户已批准本任务最新规划摘要（Goal / In Scope / Out of Scope / AC / Key Decisions）。
- [ ] 已 `task.py start`（status → `in_progress`）。
- [ ] 读 `.trellis/spec/residential-monitor/frontend/index.md`、`view-state.md`、`dto-and-decoding.md`。

## 执行顺序

1. **展示模型 + 测试**  
   新增 `residential-monitor/src/format/report-view.ts` 与 `report-view.test.ts`：空名称、其余差额、画饼条件、份额分母、queryEcho → 表单、趋势空/单桶/多桶。

2. **SVG + 测试**  
   新增 `format/report-svg.ts` 与测试：双扇区、整圆、其余扇区、单桶柱、多桶折线。无 DOM。

3. **i18n**  
   `zh.ts` / `en.ts` 成对：其余、档案下行列、类型筛选、档案窗口、图 aria、份额、「—」。`index.test.ts` 会核对中英 key 集合。

4. **渲染与状态**  
   改 `renderReports`：R1 顺序、工具条 `selected`、指标格、`.report-visuals`、档案短表。会话内保存 `reportForm` 与 `archiveKindFilter`。档案 `kind` 筛选走现有 `list_report_archives`。点选、手动运行、导出委托保持。

5. **样式**  
   `styles.css`：工具条、三指标、两列图+表、表体滚动、档案约 8 行 `max-height`、`.bar` 按百分比铺满、`--chart-1..6`（四口味 + `prefers-contrast: more`）。

6. **门禁**  
   `npm --prefix residential-monitor run typecheck && npm --prefix residential-monitor run lint && npm --prefix residential-monitor test && npm --prefix residential-monitor run build`

## 验证

- 单元：其余 > 0 / = 0 / < 0；分母 0；`exactTopN false`；空 label；单桶趋势。
- 浏览器：默认 1200×800 Mocha 进分析报告页，确认首屏是结果、档案在页尾、点选日/小时、手动运行、三种导出、收起说明、类型筛选。窄于 64rem 时图+表改单列。切 Latte 看切片对比。
- 导出文件内无「其余」行；总量与屏上三格一致。

## 风险与回滚

- 风险：`main.ts` 的 `renderReports` 与 paint 回填。改完核对焦点 `id` 与档案 `data-archive-id` 委托。
- 回滚：恢复 `renderReports` 旧 markup，删除 `format/report-*.ts` 及新增 CSS / i18n 键。

## `task.py start` 前

- `prd.md` 无未决 Open Questions。
- `design.md` 与本文件已写。
- `implement.jsonl` / `check.jsonl` 已有真实 spec 条目。
