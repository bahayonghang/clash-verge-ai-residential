# Design — ResiWatch 资源优化

状态：2026-09-19 用户已审阅并批准实施。需求/验收以prd为准，源码/安装态边界见research/diagnosis.md。

## Ownership and flow

保留collector → AccountingEngine canonical metadata/delta → StorageCoordinator事务 → live/report链路。现有runtime owner调度档案和有界维护；C2只协调不写SQL，C3负责物化/守恒/保留，storage负责会话与receipt生命周期，C4复用查询，React只管理展示。

不增加通用scheduler、配置层或长期查询缓存。扩展现有monitor-bench。顺序为同源基线、无损热路径优化、安全回收、集成验收。共享storage文件的写任务串行。

## D1: real baseline and archive scheduler — R1/R2, AC1/2/7

基准驱动真实AppFacade摄取/commit_alert_bundle及档案tick入口，固定seed、时间、活动数、变化率、累计counter和DB特征。旧C1 CSV replay保留原范围，不作为本任务性能结论。

档案owner保存下次到期和有界积压位置；启动从DB恢复一次成功键/缺口，稳态tick先做常数时间due判断。仅到期/积压/有界失败重试时查下一候选，purge低频执行。失败允许其它作业推进，成功档案不改写。小时/日边界沿用既有语义，timezone变化、时钟回拨、重启重新计算；不能用固定秒数相加替代DST边界。

每轮至多一个chunk/档案作业，不在facade锁内做长扫描。HTTP/采集频率不变，不跳过AccountingEngine事件。实施review发现同步档案查询会阻塞collector下一帧，持续失败积压也会饿死维护；runtime使用一个有界后台工作owner公平安排档案/维护，与HTTP采集分开，仍共享唯一StorageCoordinator writer，shutdown时取消并收束，不增加第二collector。

## D2: metadata delta persistence — R2, AC3

AccountingEngine保留canonical metadata唯一所有权。仅新会话、规范化值或policy变化时写元数据；chain按整组有序值比较，仅变化时替换。复用当前活跃会话状态表达尚未durable的变更，不再建长期历史缓存。

dirty状态仅在commit确认后清除；unknown commit保留原bundle和payload/hash，rollback不推进已持久化版本。generation重置、restore/reopen重建基线。facts/coverage/alerts/outbox/receipt同事务保持。canonical生命周期给出会话结束状态供清理使用，不能把空metadata/零流量误当关闭。

延迟输入按接收顺序解释generation，不能使用后到网络状态重解释先到snapshot。生命周期事件不拿旧显示投影确认新policy的dirty metadata。维护/restore等owner切换须先解决或拒绝未完成账本输入，失败操作不能丢弃待重试bundle和队列。

## D3: internal report and visible views — R3, AC4

period/archive复用run_uncached的query规划、取消和deadline，不进入ReportSnapshotStore；档案仍写既有持久表。公共run_report继续返回有界token，fingerprint复用、TTL/LRU、导出保持，不新增通用结果缓存或token不可变语义。为落实「只释放自己持有的token」，每次成功insert有一个内存租约，get/export不增加租约；同token刷新也释放旧租约，最后租约退出才主动删除。TTL/LRU仍可强制淘汰，不新增公共ID或schema。

每视图一个在途查询和一个最新意图；完成后只发仍需的最新意图，旧响应不得覆盖新选择。通过现有Tauri窗口生命周期提供可信可见状态，隐藏暂停live/report/rolling range展示查询，显示后时间对齐并合并刷新一次。托盘后台采集/告警继续，用户手动暂停语义保持。取消本视图长查询，释放自己持有的token，不能影响其它消费者。

## D4: conservation-gated retention — R4/R5, AC5/6/8

沿用raw30天/max90天，精确维度与raw coverage396天，daily total/category/coverage长期保留。用户同意汇总守恒后的过期回收；当前约21天仍在raw期内。

1. **关闭chunk与增量聚合。** 小时物化可分chunk推进，但删除截止保守取完全关闭且过期的UTC日边界，最多比名义raw期限多留不足1天。任何一天的raw小时都必须等该日完整raw上的daily/core精确聚合最终确认后才能删除，不能依靠已丢失session集合的hourly标量恢复daily distinct。从实际最早待处理数据和已验证水位开始，不每次自epoch0扫描。首次物化修复v2水位等于cutoff的空区间，覆盖仍有raw旁证的host/process/rule_group/chain/network与分类；绝不从残缺raw重建已冻结小时或从裁剪后hourly重建完整日。
2. **真实守恒。** 同可靠快照/事务边界按分类和每个维度分别比upload/download，不能把五维相加。count/duration遵从正式定义，daily distinct从完整保留raw计算，不能简单累加hourly标量；日finalization失败则保留该日全部raw，已完成小时可复用但不能授权删除。coverage先拆分跨日区间、对重叠求并集并保留gap，open区间不得当closed删除。实际校验通过才写证据和连续水位；历史文字checksum不能作为删除凭证，须重验。单chunk失败/取消/空间不足回滚结果、水位与删除。
3. **相关对象生命周期。** 仅删除已核对raw；session/chain/attr须已关闭且没有保留raw、active/pending retry或其它引用。零流量长连接受保护。旧schema的ended_utc=NULL不能单独判活跃或关闭：由现有恢复/collector owner证明controller epoch已持久退役、不会重新激活，且没有活跃或待重试引用、所有raw已完整汇总并过期，才允许该epoch的无引用历史会话退出；不伪造结束时间，不以「epoch编号较小」或「当前帧缺席」单独判断。升级fixture同时含可证明退役的NULL-ended旧会话、仍活跃零流量会话和证据不全会话；前者回收，后两者保护。证据不足的存量单列数量和字节（不可测则unknown），不得宣称这些存量已回收。raw coverage先汇总再按期限退出；dictionary只清理raw、汇总和分类都无引用的项，不重用仍引用ID。daily core/冻结档案按原策略保留，epoch唯一性/摘要不得误清。
4. **Receipt裁剪。** 保留最近24h与最新100000条的并集，先持久化每epoch不可重放边界/连续水位和必要摘要，再分块删。现schema缺时间，需要时追加migration而不改已发布DDL；不得伪造旧commit时间。旧receipt缺证据时保持，依据可证明的epoch生命周期和安全观察窗口逐步退出；不以mtime猜安全。存在sequence间隙、unknown结果或pending引用不能越界。全部receipt已删的epoch仍拒绝过期bundle，恢复后边界有效。
5. **维护调度。** 接入现有runtime owner，启动恢复一次，到期低频触发，有积压逐chunk推进。自动/手动维护共享串行入口，避免另开未协调writer竞争。chunk可取消/中断，采集交互优先；增量物化空间不足时保留原数据并真实降级，不能未核对先删救空间。

先补实现与故障证明，再跑完整守恒/容量门，最后才启用AUTO_DELETE_ENABLED及更新规范。仅翻常量不构成实现。若门未过，保持关闭并标任务未完成。

实施细化：整日扫描放在1秒writer deadline内会使大日块反复超时且没有进度。使用仅一个待处理日的持久游标与精确成员集合，逐块构建/复核；chunk与游标同事务，完整raw保留到日级确认后，再有界删除。session/minute distinct不能以小时计数相加代替。相关staging只服务该维护日，不成为通用查询缓存；取消/重启从已确认chunk恢复。完成容量实测前不声称此机制满足吞吐或空间门。

实施口径：旧`epoch`/`closed`事件不是正向采样旁证。新增连续有效sample时间段与facts同事务持久化，按UTC日合并，baseline/gap/pause/generation切换打断连续性；旧覆盖不倒填。任何raw/share/对比查询均检查实际删除状态，不能通过增大配置期限恢复已删除能力。后删除区间只提供能由当前日级精确统计支持的count/duration；无法从标量恢复的任意跨小时/跨日distinct查询明确unsupported。既有周期告警只需要字节和覆盖，复用C3内部用量投影，不伪造公开ReportResult中的精确计数。

## Capacity, migration and rollback

固定输入分布下raw、精确维度、receipt受窗口约束；daily core允许预期低基数增长，总DB不是固定GB承诺。分别显示有效页、freelist、DB/WAL/spool实际文件。DELETE产生可复用页；不自动VACUUM。现有手动VACUUM只在用户选择维护、空间充足且writer安全停用时回收文件。

旧窗口保留聚合；不能恢复的session/任意跨维细查明确unsupported/unknown。先在隔离fixture/经在线backup得到的副本验证迁移和恢复，不复制热库丢WAL。新schema只向前migration，旧程序遇未来schemafail closed；回退使用验证过的backup，不降原库schema。无损代码可撤回，已删除raw只有backup可恢复。安装部署和真实库维护不包含在本轮规划授权内。
