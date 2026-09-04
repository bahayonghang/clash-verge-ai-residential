# 设计：实时分页与 Channel 缓存

## 边界

前端 `ipc/reducer.ts`、`use-monitor-stream.ts`、`use-live-page.ts`、`features/live/`。不改 Rust `ConnectionPage` 契约。

## 数据流

- 表格：`query_live_connections` + `cursor` state（当前页栈或单 next/prev）。
- Channel：只更新 overview / seq / close 所需 `Set<identity>`。
- summary 始终来自当前查询响应，不因翻页改热点。第一页与后续页都带后端 summary；UI 热点卡片在翻页时保持第一次查询的 summary（hook 记住 `summaryPinned`）。

## 权衡

「加载更多」追加行更简单，但 matchedCount 大时内存回升。采用显式下一页，只保留当前页 rows。
