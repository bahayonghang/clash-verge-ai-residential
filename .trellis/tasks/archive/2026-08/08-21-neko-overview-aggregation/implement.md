# 实施：概览页与四个聚合页（前端）

本子任务不改 Rust。全部后端能力由 `08-21-c3-dimension-capability` 交付。

1. [x] 读 `08-21-c3-dimension-capability/implement.md` 的「交接」小节，取分钟档枚举值与 `__unknown__` 哨兵字面量，回填本任务 `design.md` 第 4 / 7 节。若该子任务未落地，按第 5 节的降级路径走。
2. [x] 建基元并登记 API：`components/common/{stat-card,overview-card,top-list-item}.tsx`。
3. [x] 建图表封装：`components/charts/{trend-area,rank-bar}.tsx`，含 `loading` 与 `emptyHint` 两态与虚线边框空态。
4. [x] 写 `hooks/use-report.ts`：请求序号、分钟边界归整、失败保留上次结果、不缓存 `drilldownCapability`。补三条测试（过期响应丢弃、失败保留、边界归整）。
5. [x] 概览页：`caliber-card` → `caliber-grid`（前五格成对、第六格活跃连接 + 覆盖 + 健康）→ `trend-card` → `top-columns` → `category-table` → `index.tsx` 装配。
6. [x] 逐格验证 `null` → 「未知」：把 `meterUpload` / `gapUpload` / `overUpload` 置 null 的构造用例进测试。
7. [x] 聚合页骨架：`dimension-page` + `rank-bar-card` + `rank-table` + `drilldown-panel` + `capability-note`；用 `DimensionKind` 参数化出四页。
8. [x] `rank-table` 处理 `identity == "__unknown__"`：按「未知」渲染、无下钻入口，补测试。
9. [x] `drilldown-panel` 由 `drilldownCapability` 驱动：`cross_dimension: false` 隐藏入口 + 显示 `note_zh`；`exact_top_n: false` 显示能力说明。用构造的 `ReportResult` 驱动测试，不依赖真实后端。
10. [x] 把顶栏时间范围选择器接到 `useReport`；趋势图三档按 `design.md` 第 4 节。
11. [x] 补 `zh.ts` / `en.ts` 新键。**不新增键集合一致性测试**——`src/i18n/index.test.ts:7-8` 已有，跑既有测试即可。
12. [x] 替换概览页与四个聚合页的 `<PagePending />`。
13. [x] `npm --prefix residential-monitor run typecheck && lint && test && build`；确认 `cargo test --workspace` 未受影响。本子任务无 Rust 改动，未重跑 cargo。

## 实拍

14. [ ] 四款主题 × 中英文 × 1200×800 / 窄窗口；`aria-sort` 与键盘可达；`prefers-reduced-motion` 下动画停止。**缺口：本轮无 Tauri 窗口，未实拍。**
15. [ ] 把查询区间拉到超出 raw 期，确认下钻入口消失并显示 `note_zh`；再拉到 daily core 层，确认排行区显示能力说明而非空表。**缺口：无 Tauri 窗口；构造 `ReportResult` 单测已覆盖 flag 渲染。**
16. [ ] C3 子任务落地后，实测下钻后的排名与趋势是子集而非全局（AC8）。**缺口：无 Tauri 窗口，未对真实库做下钻子集实测。**

## 回滚点

第 12 步之前界面仍是占位，可随时中止。无 Rust 与 DTO 改动，回滚不涉及后端。
