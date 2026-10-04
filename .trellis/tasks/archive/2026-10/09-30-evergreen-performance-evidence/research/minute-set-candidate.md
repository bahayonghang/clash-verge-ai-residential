# 第二项候选：连续重复分钟短路

日期：2026-09-30。状态：待强模型审查，尚未实施。仅涉及 raw_fold.rs 的 MinuteSet；不改 SQL、schema、session/group/exit 布局或正式门。适用五套 harness。

阶段依据：cache-v1 三对已读缓存 A250 样本，all scan 中位 3906.8561 ms、residential 3757.0764 ms；扫描耗时仍高于各自投影耗时。MinuteSet::insert 在 totals、missing、series、rank 累加时对每条命中的分钟事实调用；series 当前使用 HashSet，相同 minute 的连续行仍重复哈希。上述证据证明扫描较重，尚未单独量化重复 insert 占比。

## 机制

给 MinuteSet 增加 `last: Option<i64>`。只有 `minute == last` 时立即返回；其它输入仍走原 dense bitset 或 sparse HashSet。仅在原路径成功处理了该 minute 后更新 last；dense 越界或算术失败继续按现有逻辑返回，不把未处理值记录成已处理。

连续重复 minute 已由上一条插入完成计数，跳过重复操作不改变集合。乱序、负 minute、间隔后再次出现相同 minute，仍经原集合去重；算法不依赖输入排序。0 字节行仍调用 insert 并计入活跃分钟。series 的 SQLite 向零取整桶标签和空桶省略保持。

## 资源与边界

一个 Option<i64> 的布局由 `size_of` 记录；x64 常见为 16 B，不先声明实测值。增加量按已建立的 MinuteSet 数量线性增长，不按 minute 行数或 session_pk 最大值分配；构造期无需额外遍历。仅缓存最近一次实际处理值，不保存第二份分钟集合。

保持一次 RAW_MINUTE_SCAN、同一 reader snapshot、既有过滤/排序/exit/distinct、3120 投影阈值、取消与整报告 deadline。不把本项扩大为 group/exit 结构重写。

## 必须通过的证据

1. 同一输入序列分别验证 dense/sparse：连续重复、非连续重复、乱序、负 minute、i64::MIN/MAX 边界、dense 窗外值。用独立 HashSet 计算有效分钟数量，不调用候选实现作为期望。
2. 保留全部 grouped SQL oracle、跨窗口、零字节、负分钟、取消/deadline 回归。
3. 记录 MinuteSet 前后 size_of；相同已读缓存 before/after 有限配对分别记录 projection 与 scan。若未改善或行为回归，回退本项并保留收据。
4. 最终正式 matrix/primary 与完整容量门独立；本项局部通过不关闭 A1000、21次、AC7全部文件写入或安装态门。
