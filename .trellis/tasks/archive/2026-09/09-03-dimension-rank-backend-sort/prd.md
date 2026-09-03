# 维度排名走后端排序

## Goal

维度页改排序字段或方向时，重新 `run_report` 且 `query.sort` 为白名单字段。禁止把 download Top N 在浏览器里重排成 upload/name 冠军。

## Background

证据：`rank-table.tsx:110-129` 对 `result.rankings` 本地 sort；`use-report.ts:101` 请求 `sort.field = download`。规格：C3 排名必须在 `LIMIT top_n` 前应用 `ReportQuery.sort`。家宽聚合段已走后端 sort。

## Requirements

- R1 表头 sort 变化触发新的 `run_report`，`sort.field` 为 `upload` / `download` / `name` / `identity`，方向与 UI 一致。
- R2 删除对 `rankings` 的客户端重排（分页切片可以保留）。
- R3 报告页若存在同类「本页重排 Top N」，一并改为后端或去掉该排序，不得留下第二条分叉。

## Acceptance Criteria

- [ ] AC1 同一窗口 `top_n=1`：download desc 与 upload desc 可返回不同 identity（用现有 fixture 或 vitest + stub invoke）。
- [ ] AC2 前端源码维度表不再 `rows.sort` 聚合字节列。
- [ ] AC3 既有 download 默认序回归仍绿。

## Out of scope

- 改 SQL 模板或 `render_rank_sql`。
- 跨维下钻语义。
