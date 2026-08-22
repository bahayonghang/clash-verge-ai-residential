# 进程归因：查找进程与进程页

## Goal

用户打开进程页时，能按进程 identity 阅读流量，并能把一行下钻到主机或链路。控制器未提供进程字段时，页面说明缺字段与下一步，未知行仍可检查组成。家宽核算口径是该页的可选过滤，不是默认集合。

## Background

Grill 已定（2026-08-22）：

- 扩展脚本写 Mihomo 顶层 `find-process-mode: always`，不注入进程路由。
- 用户可在 Clash Verge Merge / 附加配置把查找进程写到顶层并重载，用于立刻核对。
- 进程页默认全量连接；可切换到核算口径（`primary_category_id` 非空）。
- 进程未知行允许下钻到主机 / 链路。
- 字段归因不可用时给出空态与当前帧 `processPresent` / `processPathOnly` / `processAbsent`。
- 监控不做操作系统进程反查。

本机证据见 `research/runtime-process-coverage.md`。Clash Verge 把 always 写在 `profile:` 下，运行中内核为 `strict`，当前 126 条连接进程字段全空。

子任务：

| 目录 | 交付 |
|---|---|
| `08-22-process-lookup-observation` | 扩展脚本顶层查找进程 |
| `08-22-process-page-capability` | 进程页过滤、未知下钻、空态 |

## Requirements

- R1. 查找进程与进程路由分开。脚本把顶层 `find-process-mode` 设为 `always`；`routing.ai_process_fallback` 仍默认 false，不注入 `PROCESS-NAME` / `PROCESS-PATH`。
- R2. 文档写明：内核只读顶层键；写在 `profile:` 下等于未启用。
- R3. 进程页默认 grouping=`process`、无核算过滤。开关打开后只保留核算口径会话。
- R4. raw 期内，进程维 `__unknown__` 可下钻到主机与链路；过滤只含空进程 identity 的会话。
- R5. 进程维字段归因 `unavailable` 时，不把单根 100% 条形图当作已归因排名。展示缺字段说明、当前帧覆盖计数，以及把查找进程写到顶层的下一步。排名表仍列出控制器未报告进程，供下钻。
- R6. 历史空进程 identity 不回填。不按源端口反查操作系统。

## Out of scope

- 开启 `routing.ai_process_fallback` 或注入进程路由规则。
- 回填已写入的空 `process_id`。
- 为 gVisor TUN 查找失败改 TUN 栈（always 生效后若仍全空，另开任务）。
- 进程行的会话列表（`includeSessions`）。
- 规则 / 链路维未知行下钻。
- 主机 / 规则 / 链路页的核算口径开关。
- 把监控库从 `%TEMP%` 迁走。

## Acceptance Criteria

- [x] AC1：默认 AI-only 脚本输出顶层 `find-process-mode: always`，规则中无 `PROCESS-NAME` / `PROCESS-PATH`。输入为 `off` 或 `profile.find-process-mode: always` 时，输出顶层仍为 `always`。
- [x] AC2：进程页未知行在 `crossDimension` 下可下钻；下钻结果只含空进程会话。
- [x] AC3：进程页核算口径开关打开后，排名与趋势只含 `primary_category_id` 非空的会话；关闭后回到全量。
- [x] AC4：进程维 `attributionQuality.status=unavailable` 时，条形图区为缺字段说明而非一根 100% 蓝条；说明含当前帧进程覆盖计数。
- [x] AC5：`just ci` 与 `just monitor-check` 通过。公开模板凭证仍为占位。

## Key Decisions

- 查找进程默认 always；进程路由默认关闭。见 `docs/adr/0001-process-lookup-vs-process-routing.md`。
- 进程未知行可下钻。见 `docs/adr/0002-unknown-process-drilldown.md`。
- 进程 identity 只来自控制器。见 `docs/adr/0003-controller-only-process-identity.md`。
