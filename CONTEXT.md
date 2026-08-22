# 家宽流量监控与 AI 家宽路由

本仓库同时包含 Clash Verge 全局扩展脚本与本机「家宽流量监控」桌面应用。下列词汇约束脚本、监控与讨论使用同一套名字。

## Language

**重点目标**:
用户配置的家宽节点名。核算口径只对链路中的精确节点名匹配。
_Avoid_: target（当作产品目标）、预期 target、家宽节点（口语）

**核算口径**:
链路节点精确等于某个重点目标时，写入 `primary_category_id` 的分类。
_Avoid_: 家宽筛选、实时家宽（那是筛选口径）

**筛选口径**:
实时连接过滤：精确命中重点目标，或节点名含「家宽」。
_Avoid_: 核算口径

**查找进程**:
Mihomo 顶层 `find-process-mode` 是否把源进程写入连接元数据。`always` 每条连接都查；`strict` 仅在存在进程规则时查；`off` 不查。
_Avoid_: 进程匹配、进程模式、profile.find-process-mode（内核不读该嵌套键）

**进程路由**:
`PROCESS-NAME` / `PROCESS-PATH` 规则把整进程流量送进某个出站。由 `routing.ai_process_fallback` 控制。
_Avoid_: 查找进程、进程兜底（单独使用时与查找进程混淆）

**进程 identity**:
进程维的稳定名字：控制器 `metadata.process`，否则 `processPath` 的文件名。完整路径不进入排名、字典、质量 DTO 或日志。
_Avoid_: PID、processPath、完整路径

**控制器未报告进程**:
进程 identity 缺失时的排名行，`identity` 为 `__unknown__`。
_Avoid_: 未知进程（听起来像真实进程名）、未归因进程（主机维用语）

**跨维下钻**:
把当前排名行的 identity 当作过滤器，按另一个维度重新查询排名。不是该行的会话列表。
_Avoid_: 详情、下钻会话、打开进程

**字段归因**:
当前 grouping 下已知与缺失的字节、连接计数；状态为完整、部分或不可用。时间覆盖用 coverage，不用这个词。
_Avoid_: coverage、归因质量（口语）

**观测下界**:
控制器 meter 与可归因观测分开读。缺口、未知和能力不支持不得写成零。
_Avoid_: 账单、用量精度
