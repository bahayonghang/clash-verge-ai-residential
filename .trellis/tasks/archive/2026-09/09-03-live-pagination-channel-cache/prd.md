# 实时分页与 Channel 缓存

## Goal

匹配数超过一页时用户能翻到后续连接。Channel 增量不再在渲染进程保存全量 `LiveConnectionView`（含 `processPath`）。

## Background

证据：`live/index.tsx:58-61` 固定 `cursor: null`；`LIST_PAGE_DEFAULT = 200`。`reducer.ts:91-103` 每条 `connectionDelta` 克隆全量 Map。表格数据来自 `query_live_connections`。`visibleRows` 仅测试使用。规格：limit/cursor 不得改变 summary。

## Requirements

- R1 Live 页把 `page.nextCursor` 接到 `useLivePage`；提供下一页/上一页或“加载更多”。summary / 热点仍用后端预分页结果。
- R2 `MonitorState.connections` 改为身份集合（或只保留 close-mark 所需 id），不再存整行。
- R3 消失 id 仍能把 `accepted` 晋升为 `closed`。
- R4 空表文案继续用 `matchedCount`，不得用当前页长度冒充总数。

## Acceptance Criteria

- [ ] AC1 `matchedCount > limit` 时界面能请求 `nextCursor` 并得到不同 identity 集合；summary 与第一页相同。
- [ ] AC2 reducer 单测：upsert 1000 行后 state 不持有 `processPath` 字段。
- [ ] AC3 关闭标记：accepted 行从查询页消失后变为 closed。
- [ ] AC4 前端 `npm --prefix residential-monitor test` 相关套件退出 0。

## Out of scope

- 虚拟化整表。
- 改后端 `LIST_PAGE_MAX`。
- DTO 全字段严格解码（子任务 8）。
