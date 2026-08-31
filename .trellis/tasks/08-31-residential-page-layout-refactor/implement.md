# 执行计划 — 家宽页面排版与表格样式重构

按序执行；每步完成后跑该步的验证命令。所有命令 cwd 为 `residential-monitor/`（除非另注）。

## 检查单

1. **C2 共享表格模块先行**
   - 新增 `src/components/common/data-table.tsx`（class 常量 + Th/Td 封装），配 `data-table.test.tsx`
   - 验证：`npm run typecheck && npx vitest run src/components/common/data-table.test.tsx`
2. **C1 StatCard 令牌化**
   - `stat-card.tsx` 增 `colorToken` prop（`color-mix` 背景），保留 hex 兼容
   - 家宽页 6 处调用（monitor-section 3 处、share-readout 3 处）迁移到 `colorToken`
   - 验证：`npx vitest run src/components/common` + 既有 stat-card 相关测试全绿
3. **C3 层级与卡片分组**
   - index.tsx 页头档差；monitor-section 标题档 + note 区归位 + 占用紧凑读数
   - share-readout 降 h3 + 占比紧凑定义行；aggregate-section 结构对齐
   - 同步更新 `index.test.tsx`、`aggregate-section.test.tsx` 中受影响的 DOM 断言（守住 design.md 的测试钩子保护线）
   - 验证：`npx vitest run src/components/features/residential`
4. **C4 排名区块**
   - aggregate-section.tsx RankBlock：切换器分组；表格接 C2 规格 + 行首色点
   - rank-bar.tsx：导出 `CHART_COLORS`；标签字号/截断复核（`src/format/rank.ts` 只读，不改逻辑，仅调 rank-bar 侧展示参数）
   - 验证：`npx vitest run src/components/features/residential/aggregate-section.test.tsx`
5. **C5 趋势区块一体化**
   - TrendBlock 内图表与 TrendTable 的间距/边框节奏；不动 max-h 内滚动行为
   - 验证：`npx vitest run src/components/features/residential/trend-table.test.tsx`
6. **报告区卡片化**
   - report-section.tsx：操作行 + 状态区 + 三 Panel 装入卡片容器；状态文字固定位置与 `role="status"` 保持
   - 验证：`npx vitest run src/components/features/residential/report-section.test.ts src/components/features/residential/index.test.tsx`
7. **可选：概览 category-table 接入 C2**
   - 仅当 1~6 完成后且 diff 干净时做；视觉若有微调用截图向用户确认
8. **全量质量门**
   - `npm run check`（含 icons/typecheck/lint/test/build）
   - impeccable detect（一次）：`node C:\Users\lyh\.skillsmanage\skills\impeccable\scripts\detect.mjs --json <改动文件>`，在仓库根目录跑
   - 双主题截图核对（Mocha / Latte，1200×800），对照 PRD 验收标准 AC1~AC9

## 回滚点

- 每步一个逻辑 commit 单元；步骤 1、2 为纯增量（新文件/新 prop），可独立 revert
- 步骤 3 是最大 DOM 变更，如测试大面积失控，回滚到步骤 2 完成态重审设计

## 完成定义

PRD 的 AC1~AC9 全部勾选；`npm run check` 绿；detect 无新增违规；双主题截图确认。
