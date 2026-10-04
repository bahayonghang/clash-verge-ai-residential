# T05 保留候选容量执行器静态审查

日期：2026-09-30。审查代理：`/root/t02_review`。正式matrix仍由实施owner独占。本审查只读脚本、CLI、原合同与既有JSON，写review文件；没有执行容量脚本、测试、构建，没有打开或hash corpus。

当前结论：**最终静态准入 PASS，容量尚未执行。** 原106次口径、CLI语义和公开排名字段的独立SQL对照通过静态审查；首轮三项及oracle留证顺序均已修正。准入不等于106次容量通过，不关闭AC3。

## 1. 固定场景与门

| 场景 | 次数 | 参数与边界 |
| --- | ---: | --- |
| A50 host 30d / 1d | 各21，共42 | top20，UTC，默认residential=true |
| A250 host 30d / 1d | 各21，共42 | 同上 |
| A250 network 30d | 21 | top100，保留09-24明确的21次门 |
| A50 network 30d | 1 | 原驱动单次诊断，不标21次门通过 |
| 合计 | 106 | 84 + 21 + 1 |

来源：原 `09-19-resiwatch-resource-optimization/research/run-capacity-queries.ps1` 的host 0..20及network单次；`09-24-resiwatch-failed-gate-followup/prd.md` 的A250 host/network各21次；本T05 `measurement-prerequisites.md:231–245`。脚本不保留原“两次失败提前中止”规则，正式21次逐次保留退出与结果。

完整窗口为 `[1787184000,1789776000)`，最后1天为 `[1789689600,1789776000)`。每次native exit必须为0，整个process wall不得超过10000ms。当前程序内部报告deadline仍由 `run_rank` 明确传入，脚本没有放宽。

A1000未满足原阶段前提，NOT_RUN；完整30天、21次及10秒要求保持。A50/A250结果不关闭整个AC3，也不能以线性外推把A1000判为FAIL。

## 2. CLI与oracle核对

`dbcli/mod.rs:99–128` 的默认格式为JSON，rank默认residential=true；脚本显式UTC、since/until/by/top。`execute` 使用实际 `Utc::now()`，不能把该查询与stage test的now=end混称同一capability条件。旧语料超出当前nominal raw cutoff时，`service.rs:474–515` 允许对未删除、尚存完整raw的已授权历史聚合回退Raw，脚本实际检查返回layer。

独立oracle来自已保存的 `corpus-a250-verification.json`。它直接按分钟范围连接session/attr/dictionary并GROUP BY identity，排序为download降序、identity升序，最后LIMIT；未调用raw_fold或候选缓存。住宅成员谓词包含category存在或历史链命中，与生产 `RESIDENTIAL_RAW_MEMBERSHIP_SQL` 对齐。A50和最后1天使用相同SQL在对应完整语料上计算；A250 30天另核对既有固定结果。

CLI公开rank行只有identity、unknown、upload、download、connectionCount、zeroFlow六个字段。脚本比较排序后的前四个值，对三个数值强制int，并检查两个布尔标志。oracle第五列active-duration没有CLI对应字段；本次容量结果不能声称CLI验证了duration。

`truncation.status=complete` 与当前CLI序列化一致，其含义是选定TopN视图完整，不代表所有身份或全窗口守恒。host的20行上限和network的100行上限须保持；不能通过放大TopN改变测试。全窗口totals、coverage和series不由这些rank行自动证明。

## 3. 身份、数据库与缓存

脚本绑定 `candidate-retained-20260930` 的db exe、source manifest及当前源文件hash。固定两份production-corpus marker、30天标志、seed、起止、规模，再检查完整实际行数、schema5、layout3 checksum、WAL及quick_check。A250还对照此前独立记录的数据库hash。

预读连接使用mode=ro、query_only和一致事务。所有hash、完整性检查与GROUP BY oracle均发生在native容量样本之前，因此r1只可称本序列第一个进程；没有物理冷页证明。结束后记录DB hash与WAL状态，不能仅根据native成功推断文件未改。

## 4. 首轮需修正的问题

1. **前/后置失败证据。** 首轮脚本在posthash断言之后才save。若hash不相同，实际after值不会落盘，旧preflight仍显示PASS。前置assert失败也没有最终summary。要求在失败前保存实际观测、FAIL状态、原因及未执行项，保持非零退出。
2. **generatedUtc。** 首轮只把字段抄进receipt，没有严格int或本次调用时段检查。要求核对类型和started/finished UTC秒范围，记录真实CLI时钟，拒绝null、字符串和过期值。
3. **预读连接释放。** Python SQLite连接上下文管理器处理事务，不负责关闭连接。要求finally显式close或contextlib.closing，使独立SQL连接在native循环前释放。

这三项只涉及研究执行器。已通知实施owner与主会话；没有修改产品、门槛、oracle、规模或过滤，禁止在当前matrix时段运行额外验证负载。

## 5. 修正版复审

首轮三项已静态确认。未捕获异常处理器保存execution-failure、已执行数量、部分结果和FAIL summary；schema、完整性与计数检查先保存实际/期望再断言。postflight先保存实际after hash、WAL bytes与状态，再执行断言。generatedUtc强制int并核对本次native开始与结束的UTC秒。contextlib.closing在native循环前释放只读SQL连接。

第二轮发现并要求补齐一项同类留证：SQL查询返回rows后，shape及A250 30d旧oracle相等断言仍位于save之前。匹配失败会丢失实际rows，只保留traceback。已要求实施owner在断言前保存SQL、params、实际rows、适用时的已有expected rows，并在failure_context中标出kind/window；随后记录校验状态。该要求不改变SQL、TopN、过滤或容量门。

复审期间的一次matrix状态快照为20份finished、1份running。随后22次均完成，独立终审另见[正式matrix审查](review-formal-matrix.md)。容量准入不授权与matrix或primary重叠运行。

## 6. 最终静态准入

最终脚本SHA256为`9fa8c225bef9a9f7718676d7a5e2d00b1e174270784c99094ed5dcc2f8d9faa4`，14142 B。只读AST parse通过；没有import或执行脚本，没有打开或hash语料。

第四项修正已确认：查询前保存SQL/params/适用expected_rows和running状态；查询后先保存实际rows、耗时、shape_ok、values_ok与PASS/FAIL，再执行断言。failure_context包含oracle_kind/window，结束预读后清除；查询异常仍由统一失败处理器保留上下文。

静态准入仅覆盖已审脚本、原106次场景及最终保留候选身份。必须在matrix、primary之后独占执行，逐条保留native退出、时钟、oracle、进程墙钟、前后DB身份与完整收据。A1000仍为NOT_RUN，AC3仍为NOT_COMPLETE。
