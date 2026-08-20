# 实时连接方向热点摘要

## Goal

在实时连接表上方提供两张与当前筛选同步的方向热点卡：最高下载累计连接、最高上传累计连接。摘要必须来自后端同一筛选快照，不能从截断的 UI rows 伪造全量 Top 1。

## Dependencies and confirmed facts

- 父任务：`08-20-live-connection-ux-optimization`；依赖 `08-20-live-filter-workspace` 的 applied query 状态，依赖父任务的固定重绘边界；不改变 collector/Channel/关闭单条连接。
- 当前 `ConnectionPage` 只有 `rows`/`nextCursor`（`src/ipc/live-session.ts:31-34`；Rust `src-tauri/src/c2/query.rs:59-64`），查询层已在 `query_connections_with_targets` 中构造完整 matched 集合（`query.rs:347-396`）。
- 现有规范禁止前端自行实现 Top N/守恒、未知不得填零、图表必须有对应数据表；摘要可用同口径文本卡而不必引入图表库。

## Requirements

- 扩展实时 query response，返回 `matchedCount`、`sampleUtc`、`topDownload`、`topUpload`；每个热点含 identity、可脱敏 label/host/process/destination 和方向 value。命中集合为空时为 null，不返回假 0。
- Rust 从完整 filter matched 集合计算方向最大值，数值相同用 identity 稳定 tie-break；rows 仍按用户 sort/cursor 分页，summary 不受 `limit=200` 截断影响。
- rows 与 summary 必须来自同一 hub snapshot；前端 decoder 对字段/类型/未知能力 fail closed，并将 sample/status 与 monitor snapshot/collectorRunning 对齐。
- 两张卡显示方向标签、可识别连接标签、单位化数值、采样时间和状态；无匹配/暂停/缺口/未连接/能力未知不显示过期值或零。
- 卡片图标若使用，必须有文本等价和同口径说明；中文/英文、键盘 focus、对比度和窄窗口通过检查。

## Acceptance Criteria

- [ ] AC1：Rust query tests 证明筛选后全量集合的最高下载/上传与稳定 tie-break，`limit=1/200` 不改变 summary。
- [ ] AC2：TS decoder/state/render tests 覆盖合法、缺字段、null、无匹配、暂停/缺口和旧响应；未知不写 0。
- [ ] AC3：界面两张卡与当前 applied query/样本时间同步，显示主机/进程/目标 fallback 且不泄漏 secret/raw payload。
- [ ] AC4：前端/Rust 相关 typecheck、lint、unit tests、fmt/目标 cargo tests 通过，手动实拍另行标记证据状态。

## Out of scope

- 不实现历史 Top N、趋势图、分类守恒、报告导出、详情抽屉或任何前端全量聚合。
