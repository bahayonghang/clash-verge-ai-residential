# 监控：进程页未知下钻与空态

父任务：`08-22-process-attribution`。

## Goal

进程页在有进程 identity 时按进程排名并可下钻；在没有时说明缺字段，并把未知行当作可检查的集合。默认看全量，可选核算口径。

## Requirements

- R1. `filters.process === "__unknown__"` 匹配空进程 identity（raw：`process_id` 为空或字典缺失；精确层：`dimension_id = 0`）。哨兵不得写入 `dimension_dict`。
- R2. `filtersForDrilldown("process", "__unknown__")` 设置该哨兵。进程页在 `crossDimension` 下为未知行显示下钻，目标仍是主机与链路。
- R3. 进程页增加核算口径开关，默认关闭。打开时查询只含 `primary_category_id` 非空的会话。筛选口径不用在此开关。
- R4. 进程维 `attributionQuality.status === "unavailable"` 时，条形图区域改为缺字段说明，不画单根 100% 条。说明包含：控制器未提供 `process` / `processPath`；Clash 须把 `find-process-mode: always` 写在顶层；当前帧 `processPresent`、`processPathOnly`、`processAbsent`、`connections`。排名表仍显示控制器未报告进程行。
- R5. 当前帧覆盖计数来自已计算的 `MetadataCoverage`，经 LiveOverview 送到前端。进程页只读，不在浏览器里重算 Top N。
- R6. 更新 `residential-monitor/docs/reporting.md`：进程未知行可下钻；核算口径开关语义。

## Out of scope

- 操作系统进程反查。
- 回填历史 `process_id`。
- 其它维度页的核算口径开关。
- 规则 / 链路未知行下钻。
- 会话明细表。

## Acceptance Criteria

- [x] AC1：Rust 过滤用例：`filters.process = "__unknown__"` 的 raw 排名 / 序列只含空进程会话，totals 与字段归因守恒。
- [x] AC2：前端进程维未知行 `data-drill="1"`；选中后 `useReport` 带 `filters.process = "__unknown__"`。
- [x] AC3：核算口径开关打开后，请求带核算过滤；夹具中非空 `primary_category_id` 的进程出现，空分类会话不出现。
- [x] AC4：进程维 unavailable 夹具不渲染 100% `RankBar` 数据；出现缺字段说明与当前帧覆盖数字。
- [x] AC5：`just monitor-check` 通过。
