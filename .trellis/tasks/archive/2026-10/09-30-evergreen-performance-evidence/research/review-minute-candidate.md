# T05 连续重复分钟候选独立审查

日期：2026-09-30。审查对象：`minute-set-candidate.md` 与现有 `raw_fold.rs::MinuteSet`、`Acc::add_bytes`、`fold_window`。本代理只读审查并写本文件；未运行构建、测试、基准或数据库负载，未修改产品。适用 Claude Code、Codex、Grok Build、Kimi Code、OMP；算法与结果判读保持强模型。

## 判断

**可在 T05 已批准的 raw_fold.rs 范围内实施。** 增加每个 MinuteSet 自有的 `last: Option<i64>`，仅在 `minute == last` 时短路该集合的重复处理，可以保持分钟集合与 distinct count。范围内不增加 SQL、schema、扫描次数、session/group/exit 重构或阈值调整。

实施前须澄清计划的 dense 测试边界：当前 dense insert 使用分配后的 word 范围，未显式保存或检查 end_min。独立 oracle 必须保留该既有行为；不能借此次短路增加 logical end 检查。生产 SQL 的 `[start_min,end_min)` 过滤另有边界，继续保持。

阶段 scan 较重支持开展局部优化实验；尚未量化重复 MinuteSet insert 的耗时占比。因此本审查不承诺任何 CPU、延迟或容量门通过。

## 当前路径与语义

- `Acc::add_bytes` 先累计 upload/download，再调用 MinuteSet::insert。短路必须放在集合方法内，所有 byte 累加保持执行。session distinct 的 ScanMark 与出口聚合在 fold_window 中单独执行，不能跟着短路。
- totals、missing、每个 series bucket 与每个 ranking group 各自拥有 MinuteSet。不能共享一个全局 last，否则不同累加器第一次遇到同一分钟时会漏计。
- dense 由 `MinuteSet::new(start_min,end_min)` 构造，使用 `checked_sub` 计算长度，再按 64 位 word 向上取整。sparse 使用 HashSet<i64>。两条原路径都只为新成员增加 count。
- fold_window 对本窗口继续 prepare/query 一次 `RAW_MINUTE_SCAN`；current/previous 的既有调用次数保持。查询仍使用 `utc_minute >= ? and utc_minute < ?`。短路不依赖输入排序，不新增 ORDER BY、sort、第二遍扫描或预遍历。
- series bucket 标签仍用 Rust/SQLite 整数除法向零取整。零字节行仍进入集合，空桶省略不变；同一报告的 snapshot、取消与 deadline owner 不改。

## 必须保持的状态不变量

`last=Some(x)` 只能表示 x 已经被当前集合成功处理、属于当前集合。集合只增加成员，因此再次遇到 x 时跳过原去重操作，不改变集合和 count。

1. 两个构造器均初始化 `last=None`；不得以 0、i64::MIN 或 i64::MAX 作为空哨兵。
2. sparse 路径在 HashSet::insert 完成后更新 last；无论返回新增还是已存在，该 minute 都已处理。
3. dense 路径在 checked_sub、usize 转换、word 查找成功，并完成原有 bit/count 逻辑后更新 last。bit 已置位也属于成功处理。
4. dense 算术失败、负 index 转换失败、word 越界时返回并保持原 last；不得缓存未处理值。
5. 不同输入间隔后重复出现同一 minute，继续经原集合判重。乱序、负 minute 和多次返回旧值不要求排序。

## Dense 尾部补齐位

现有 `MinuteSet::insert` 仅通过 `words.get_mut(index / 64)` 判断存储边界，没有 `index < end_min-start_min` 检查。例如 `new(0,1)` 分配一个 word，直接 insert 1 至 63 仍会设置对应位；insert 64 才被 word 边界拒绝。

正式调用点的 SQL 不会给该对象传入逻辑窗口外的行，故此实现细节不表示报告实际读取了窗口外数据。此次优化必须保持 primitive 的现有可表示范围，避免将另一项边界修复混入性能短路。

dense 的独立参考集合应按以下既有条件决定是否接纳输入：`minute.checked_sub(origin)` 成功；差值可转为 usize；`index / 64 < words_count`。words_count 按构造窗口长度向上取整产生。参考代码直接计算这些条件并使用独立 HashSet，不调用候选 insert 或读取候选 last/count 生成 expected。

另保留现有 SQL 窗口 oracle，验证实际 report 没有接纳 end_min 或窗口外的数据。若需要改变 primitive 的补齐位接纳行为，应单独提出依据，不在本候选中改变。

## 必须通过的回归

| 场景 | 应验证的结果 |
| --- | --- |
| 首个值为 0、负数、i64::MIN、i64::MAX | sparse 正确增加一次；不会误命中未初始化 last |
| 连续重复、非连续重复、乱序 | 每一步 count 都与独立参考集合 cardinality 相同 |
| Dense 连续位、63/64 word 边界与补齐位 | 保持原有可表示范围与去重，不新增 end 检查 |
| Dense origin 之前、分配 word 之外、checked_sub 溢出 | 不接纳该值；后续有效值仍正确处理 |
| Dense 零长度、逆序窗口、长度溢出 | 保持原空对象或原错误，不为测试构造巨大分配 |
| 接近 i64 两端的小 dense 窗口 | 验证算术边界；只分配少量 word |
| 多个独立 MinuteSet 同分钟 | 各自计入一次，禁止共享 last |
| 零字节事实、同 session 跨窗口/桶 | active duration、session distinct 与 grouped SQL 一致 |
| 原 grouped oracle、筛选、出口与取消/deadline | 原有核算、排序与整报告预算不变 |

## 内存与性能判定

记录同一 target 下 `size_of::<Option<i64>>()`、MinuteSet 修改前后大小，必要时同时记录 Acc 大小。Rust 布局可能含对齐，不能将字段常见尺寸直接当作结构增量或实际 native private。

固定状态的新增对象规模是 `2 + series.len() + groups.len()` 个 MinuteSet，分别对应 totals、missing、series、group；不按 session_pk 最大值或分钟事实行数分配第二份集合。总内存仍需实测，不能从 size_of 推断整个进程的 p95。

每个实施阶段先冻结源码/程序身份，保留 lazy dictionary 与 minute 短路各自的前后边界。禁止在构建期间并发压测，禁止重新利用缓存-v1 的旧程序计时声称新组合通过。按相同已读缓存定义做有限配对，projection 与 scan 分列；未改善或出现回归时保留收据并按任务回滚约定处理。

正式 F1–F12、主场景三轮、各 21 次容量、全部应用文件归属及安装态门仍独立。局部序列 oracle 或少量 stage PASS 不关闭任一正式门。

## 审查记录

本判断已发主会话与实施 owner。产品实施及负载由 `/root/t02_skill_sync` 独占，本代理不覆盖其修改。首次失败、候选回退和未测边界继续保留。
# MinuteSet 实验结果与取舍（2026-09-30）

最终实验的 `last: Option<i64>` 实现只跳过同一分钟的集合操作。构造器均初始化None；dense成功处理后及sparse插入后更新last，dense失败不更新。独立i128/HashSet oracle保留了原word padding接纳边界，覆盖连续/非连续重复、乱序、负分钟、MIN/MAX附近小窗口、无效窗口。字节与session累计保持在MinuteSet短路之外。静态与语义审查通过。

实施收据实际记录MinuteSet从96B变为112B，Option<i64>为16B。没有把该布局变化转换为native private实测结论；Acc整体布局未单独实测。

两对已预读缓存对比：all scan ratio 0.9865458，residential scan ratio 0.9863865；单个30天all生产样本6696.6586 → 6684.6417ms，约下降0.18%。residential projection增加7.36%，该路径没有被MinuteSet修改，原因未查明，不能直接归因。

强模型取舍：建议回退MinuteSet的新增状态与短路。有限样本中的名义改善不足以支持稳定收益，不为该名义变化扩大局部样本。保留lazy字典修复、Network合同修复、独立集合/累计oracle及全部实验收据。回退后的候选须使用新manifest/exe并重验；实验身份保留在 `candidate-final-20260930`。正式门仍未通过。

收据核验：`review-minute-experiment-validation.json`，25条命令日志/测试与4个复制exe身份一致，errors为空。实验正式matrix/primary尚未运行。
