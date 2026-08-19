# 技术设计：表头排序

合同以父任务 `08-19-live-table-sort-width/design.md`「排序」节为准。

## 本子任务边界

- `c2/query.rs`：扩展 `sort_key`；未知在后的比较与游标编码。
- 前端：`th` 内可点区域 `data-sort`；`liveQuery.sortField` / `descending`；delta 后用当前查询。
- 单测：各字段顺序、未知在两侧都最后、非法 `sort_field` 回落 identity、下载 0 视为有值。

## 不改

- 列宽 JSON 与显隐。
- `clauses` 数值 mode。
- Channel schema。

## 风险

- 简单 `reverse` 会把未知翻到最前。必须分段比较。
- 拖动手柄与排序热区分离。
