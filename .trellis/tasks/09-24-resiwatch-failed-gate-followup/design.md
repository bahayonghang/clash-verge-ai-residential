# 设计

## 边界

只改报告读取和已被同窗口证据归因的摄取成本。不改核算口径、删除门、schema 或安装态调度。

前序证据在 `.trellis/tasks/archive/2026-09/09-19-resiwatch-resource-optimization`。本任务的新测量使用新的隔离目录和新的 exe 身份，不覆盖旧 JSON。

## 数据流

默认自动报告调用 `run_uncached`，再进 `build_result` → `plan_snapshot_capability` → `fill_raw`。`fill_raw` 先 `load_sessions`，再 `fold_window`。2026-09-24 的 1 分钟探针显示 plan、finalized days、durable version、reader close 合计不到 2 ms，raw query 为 6756.665 ms。

因此第一刀只针对 raw query 内部：

1. 在同一只读快照上拆开未过滤 `load_sessions` 与 `fold_window`。
2. 若投影耗时占主导，把投影限制到查询分钟窗口里实际出现的 session，而不是先读完全表再丢弃。
3. 用现有分组 SQL oracle 证明连接数、字节、排名和前一等长窗口比较不变。
4. 家宽过滤下推保持现状；无分类过滤的默认报告是这次要修的路径。

不在投影归属完成前改 `ArchiveScheduler`、metadata UPDATE 或 backlog 队列。F1–F5 的当前 p95 来自跨日对照，A1000 metadata 从 53.036 ms 到 284.837 ms 足以要求复测，但不足以直接改调度器。

## 兼容

- 报告 DTO、deadline、取消和 tier 选择不变。
- 分钟窗口外的会话不得进入 distinct、排名或比较量。
- 前一等长窗口比较仍只在 raw 仍保留时计算；已清理则保持未知，不把缺口写成零。

## 取舍

先修已定位的报告投影，再决定是否生成 A1000。生成 6 GB 只为复现超时没有新信息。矩阵回归先同窗口复测；复测仍失败才做阶段归属，归属完成才改产品代码。

## 回滚

报告 SQL 改动必须有失败即回退的 oracle。测量 exe 复制到隔离目录，不覆盖用户安装目录里的 `monitor-bench.exe`。
