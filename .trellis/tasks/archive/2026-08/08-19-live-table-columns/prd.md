# 实时表列宽拖动与显隐

## Goal

实时连接表列宽固定可调：刷新时不再随字长晃动，用户可拖宽、隐藏数据列，重启后保持。

## Background

父任务：`08-19-live-table-sort-width`。本子任务交付 R1 / AC1–AC3。表头排序与数值条件由后续子任务交付。须给表头带上 `data-col`，供排序子任务接线。

根因：`.live-table { display: block }`、无固定列宽、单元格换行、1 Hz 整页重绘不恢复滚动。见父 `research/table-layout-and-sort.md`。

## Requirements

- 默认像素模板、最小宽、JSON 形状、设置键见父 `design.md`。
- `table-layout: fixed`。去掉 `display: block`。滚动只在 `.live-table-wrap`。单行省略。数字列右对齐。
- 数据列可拖右缘。操作列不拖、不藏。至少一列数据列可见。
- `save_live_table_layout` + `BootstrapDto.liveTableLayout`。非法回落默认。Recovery 无库只留内存。
- 重绘保持滚动。拖动期间不 `paint()`。
- 工具条「列」：十二数据列开关 + 恢复默认。文案跟 `uiLocale`。
- 不改 `query_live_connections` 语义。不排序、不加数值条件。

## Out of Scope

- 表头排序、数值条件、列重排、虚拟化。
- 改核算、关闭全部。

## Acceptance Criteria

- [ ] AC1 列宽稳定（父 AC1）。
- [ ] AC2 拖宽与显隐持久化（父 AC2）。
- [ ] AC3 滚动与拖动期间不中断（父 AC3）。
- [ ] 现有筛选、空态、关闭回归通过。typecheck / lint / test / build 通过。相关 Rust 设置测试通过。

## Notes

- 依赖：无。后继：`08-19-live-table-header-sort`。
- 父任务 `design.md` 为列宽与持久化合同。
