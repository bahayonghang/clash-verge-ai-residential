# 实时表数值筛选条件

## Goal

用户用「添加条件」按下载、上传、速度或时长缩小实时表。数字带单位，比较的是原始字节或毫秒。

## Background

父任务：`08-19-live-table-sort-width`。本子任务交付 R3 / AC6。文本条件与「只看家宽」已存在。用户选择数字框 + 单位下拉。

对展示串做包含会误伤（`38.6 KiB` 包含 `38`）。

## Requirements

- 字段增加：`download` `upload` `rateDownload` `rateUpload` `duration`。
- 数值 mode：`gt` `gte` `lt` `lte` `eq`。文本字段仍为 `exact` / `contains`。
- 单位与换算见父 `design.md`。默认 KiB、KiB/s、分钟。
- 前端换成整数写入 `clauses.value`。空、负、非数字、溢出忽略该行。
- 未知速率/时长不命中。`download`/`upload` 的 0 可命中。
- 最多 8 条 AND，与「只看家宽」叠加。只留当前会话。
- 字段类型切换时重置 mode、单位、值。

## Out of Scope

- 表头筛选菜单、列宽、表头排序。
- 解析自由文本 `38.6 KiB`。

## Acceptance Criteria

- [ ] AC6 数值条件（父 AC6）。
- [ ] 文本精确/包含回归：`chatgpt.com` 精确不命中 `ws.chatgpt.com`。
- [ ] 8 条上限与空值忽略保持。
- [ ] typecheck / lint / test / build 通过；`c2::query` 数值条件测试通过。

## Notes

- 可与 header-sort 分先后。二者都改 `query.rs` 时不要并行无合并。
- 不依赖列宽子任务，但建议在 columns 之后做，以免工具条与列按钮抢布局。
