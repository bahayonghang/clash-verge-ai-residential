# Planning review

2026-09-19 独立只读规划审查发现两项P1，主会话已在规划中修正；这是设计修订，不是产品修复证明。

| Finding | Correction | Acceptance |
| --- | --- | --- |
| 逐小时删除可能丢失daily distinct所需session集合，hourly标量不能重建精确日统计 | 整个UTC日raw保留到daily/core/count/duration/coverage全部最终核对；小时可分块物化，删除按已确认日边界 | AC5增加跨小时session和最后小时中断 |
| 当前大量ended_utc=NULL旧会话可能永远不进入仅针对未来close事件的回收 | 要求durable retired-controller-epoch及无active/pending/raw引用证明；不伪造end time，保护和单列证据不足对象 | AC6增加旧schema升级、活跃零流量与不确定会话fixture |

原始审查结论在这两项解决前不可进入实施。最终修订仍须用户审阅；代码实现、migration、守恒、性能、容量与安装态证据均未运行/未验证。
