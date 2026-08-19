# 技术设计：列宽与显隐

合同以父任务 `08-19-live-table-sort-width/design.md`「列布局」节为准。

## 本子任务边界

- 新增 Rust 模块（建议 `live_table_layout.rs`，与 `theme.rs` 同级）：解析、夹宽、隐藏消毒、默认表。
- `AppFacade`：内存持有 `LiveTableLayout`；`bootstrap` 带出；`save_live_table_layout` 写 `machine_setting`。
- 前端：`colgroup`、拖动手柄、列面板、`paint` 滚动、拖动锁。
- `th` 写 `data-col="<id>"`。本任务表头仍是文字，不放排序按钮。

## 不改

- `c2/query.rs` 过滤与 `sort_key`。
- Channel schema。
- 筛选 `clauses`。

## 风险

- `main.ts` `paint()` 全页替换：必须在重写前读 wrap 滚动，写后恢复。
- 拖动中若仍 `paint()`，指针会丢：设 `liveTableDragging` 跳过。
