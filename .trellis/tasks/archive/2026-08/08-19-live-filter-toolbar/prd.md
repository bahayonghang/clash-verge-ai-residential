# 实时筛选工具条

## Goal

用户在「实时连接」用一行工具条控制「只看家宽」和条件，表格占据剩余高度。筛选结果与现在一致。

## Background

父任务：`08-19-ui-catppuccin-layout`。依赖主题提供 `.btn-secondary` 与 `label.inline`。

筛选语义已由 `08-19-live-table-filter` 交付。当前缺陷是全局堆叠 `label` 与通栏主按钮，见父任务截图分析。

## Requirements

- 工具条：健康 + 采样一行；「只看家宽」横向开关；「添加条件」次要按钮。
- 条件行：字段、匹配方式、文本、删除（次要）。最多 8 条。空值忽略。
- 表格容器吃满主区剩余高度，可横向滚动。
- `liveQuery` 与 `query_live_connections` 形状、默认只看家宽、AND、会话级、单条关闭、五类空态均不改。
- 不新增搜索、关闭全部、列排序。

## Out of Scope

- 主题口味与概览口径。
- 改 Rust 查询。

## Acceptance Criteria

- [ ] 「只看家宽」文字与勾选同一行，勾选不落在卡片中线。
- [ ] 「添加条件」不是通栏主色条。
- [ ] 有条件时默认窗口表格仍在首屏。
- [ ] 现有筛选测试与空态测试仍通过。typecheck / lint / test / build 通过。

## Key Decisions

- 只重构 UI。查询合同冻结。
