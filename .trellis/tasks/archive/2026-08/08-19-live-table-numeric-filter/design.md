# 技术设计：数值条件

合同以父任务 `08-19-live-table-sort-width/design.md`「数值条件」节为准。

## 本子任务边界

- `matches_clause`：数值字段走 `u64` 比较；未知 `None` 不命中；交叉 mode 忽略该行。
- 前端：条件行按字段类型渲染数字+单位或文本+精确/包含。
- 纯函数换算（建议 `src/format/live-filter-units.ts`）：单位因子、`Math.round`、非法输入返回空（调用方忽略）。
- 单位留在前端条件对象上供重绘。Rust `FilterClause` 不增加字段。

## 不改

- 列宽 JSON。
- `sort_key`（若 header-sort 已改，保留其比较函数）。
- Channel schema。

## 风险

- 旧逻辑把未知 `mode` 当成 `contains`。数值字段必须先分支，避免 `gt` 被当成子串包含。
- 条件行控件比文本行多一个 `<select>`。沿用 `.filter-row` 横向排列，不要竖堆成通栏。
