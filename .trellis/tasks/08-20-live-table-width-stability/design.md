# 设计：固定列宽交互

- `<colgroup>` 和 table inline pixel width 是唯一尺寸源，`.live-table-wrap` 只负责滚动；禁止由内容测量或 `width:100%` 覆盖自定义总宽度。
- resize handle 记录 `pointerId/startX/startW/col`，在 handle 上 `setPointerCapture`。pointermove 只更新目标 col 与 table width，结束阶段统一清理并持久化一次。
- `pointercancel`、`lostpointercapture`、window blur 走同一取消/结束函数；`liveTableDragging` 只用于暂停整页 paint，结束后恢复 scrollTop/scrollLeft。
- handle 用可聚焦 separator 或相邻键盘增减控件，公开 min/max/current 值；不把 table 改为 ARIA grid，排序仍使用 header button + `aria-sort`。
