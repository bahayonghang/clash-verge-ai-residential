# 实时表头排序

## Goal

用户点击实时表数据列表头，按该列对当前查询页排序。未知值排在有值之后。刷新后保持所选排序。

## Background

父任务：`08-19-live-table-sort-width`。本子任务交付 R2 / AC4–AC5。依赖 `08-19-live-table-columns` 已提供 `data-col` 表头与固定列宽。

`sortField` / `descending` 已在查询合同里。前端固定传 `identity`。`sort_key` 缺速率、链路、时长、来源、目标、组合类型。

## Requirements

- 十二数据列可点。操作列不排。循环：降序 → 升序 → `identity` 升序。
- 走 `query_live_connections`。改排序清 cursor，查第一页。delta 后保持 `liveQuery` 排序。
- `sort_field` 取值与未知策略见父 `design.md`。
- 表头 `aria-sort` + 可见方向。文案跟 `uiLocale`。
- 拖宽手柄不触发排序。
- 排序只留当前会话。

## Out of Scope

- 列宽/显隐（由 columns 交付）。
- 数值条件（由 numeric-filter 交付）。
- 前端对 200 行本地重排、翻页 UI。

## Acceptance Criteria

- [ ] AC4 表头排序（父 AC4）。
- [ ] AC5 未知排在有值之后（父 AC5）。
- [ ] 默认进入页面仍为 identity。只看家宽与文本条件仍生效。
- [ ] typecheck / lint / test / build 通过；`c2::query` 排序测试通过。

## Notes

- 须先完成 `08-19-live-table-columns`。
- 与 `08-19-live-table-numeric-filter` 都改 `query.rs` 时串行合并。
