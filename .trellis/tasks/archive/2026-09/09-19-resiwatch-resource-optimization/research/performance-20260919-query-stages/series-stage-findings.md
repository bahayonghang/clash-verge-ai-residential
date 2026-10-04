# 原始时间序列分桶优化：阶段证据

本轮完成一个局部查询优化，**没有通过 A1000 完整报告的 10 秒容量门**。总量与排名各自已超预算，不能用时间序列改善代替整份报告验收。

## 隔离库与测量范围

- 只读生成库：`C:/Users/lyh/AppData/Local/Temp/resiwatch-resource-measure-20260919-retention-cache/corpus-a1000-resume/monitor.sqlite3`，未操作安装库。
- `production-corpus.json` 的 A=1000、seed=20260919、原始30天范围为 `[1787184000, 1789776000)`。本次仅取首日之后的29个完整保留日：`[1787270400, 1789776000)`，小时粒度696桶、家宽过滤、Host Top20。
- 该克隆已完成首日清理；已有 `../performance-20260919-retention-cache/retention-a1000-resume.json` 记录 complete=true、quick_check=ok、剩余过期raw=0、8,352,000个仍被raw引用的session、100,000个receipt。这里引用既有状态记录，未为探针重新全库盘点。
- 使用 Rust release 测试可执行文件内与产品相同的 SQLite 3.53.2、`open_interruptible_reader` 与查询函数注册；不是 Python SQLite 的替代耗时。连接以 `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_FULL_MUTEX` 打开，所有SELECT在一个显式只读快照内。`PRAGMA query_only=0` 不取消打开标志所保证的只读限制。
- 每阶段单独60秒诊断上限；生产页/报告2秒/10秒预算未改。输出只有SQL/源文件哈希、计划、耗时、行数及结果摘要，没有host/chain等明细。没有清除OS缓存，没有并发重负载，也没有统计重复20轮。

## 原始阶段测量

详见 [a1000-29d-before.json](a1000-29d-before.json)。所有阶段成功，未取消或达到诊断上限。

| 阶段 | 单次耗时 ms | 输出行数 |
| --- | ---: | ---: |
| 总量与归因融合 | 30,567.243 | 1 |
| 原始时间序列 GROUP BY | 37,025.613 | 696 |
| Host 排名 | 53,544.078 | 20 |
| 入选 Host 的出口补充 | 11,587.435 | 60 |

四阶段都以分钟主索引范围读取，再逐行关联session/attr。时间序列额外出现 `USE TEMP B-TREE FOR GROUP BY`，以及两个精确distinct的临时树；排名另有 GROUP BY 与 ORDER BY 临时树。总量和排名已独立超过10秒，证据不支持「只优化出口」或「只优化时间序列」便能过整份报告门。本轮未改动总量融合或出口查询。

## 最小产品改动与等价性

`c3/service.rs::load_raw_series` 在原查询快照中，复用同一prepared statement，按升序分钟桶裁剪半开区间并读取；`c3/sql.rs::SERIES_RAW` 保留原JOIN、过滤、SUM、distinct session、distinct minute ×60，只移除全区间 GROUP BY/ORDER BY。`HAVING count(*) > 0` 省略无匹配桶，同时保留零字节观察。原SQL仅作为 `#[cfg(test)] SERIES_RAW_GROUPED` 对照。

桶标签仍为 SQLite 的 `(minute / width) * width`，整数除法向零截断：负标签b覆盖 `[b-width+1,b+1)`，零标签覆盖 `[1-width,width)`，正标签b覆盖 `[b,b+width)`。从请求起点依次推进到当前桶的排他上界，最后裁到请求终点；因此每条匹配原始行恰好进入原来的一个桶。宽度来自现有Granularity枚举。没有增加schema、依赖、配置或历史回填，没有将日/小时去重标量相加。

生产仍使用外层注册的共同progress handler及开始时间，分桶不重置deadline。执行中的取消和已过期handler均有回归。

## 同快照旧新时间序列对照

详见 [a1000-29d-series-pair.json](a1000-29d-series-pair.json)。单次先旧后新，缓存状态不受控，因此是局部诊断证据，不是AC7三轮AB结果。

| 实现 | 单次耗时 ms | 行数 | 摘要一致 |
| --- | ---: | ---: | --- |
| 原 GROUP BY | 35,604.236 | 696 | 是 |
| 索引逐桶 | 26,297.359 | 696 | 是 |

本次减少9,306.878ms，即26.14%。结果摘要均为 `b839e385b0f7826291d941d2393ae9733c8d8ae473784f3d74069baaecbd8eac`。新计划仍搜索 `sqlite_autoindex_connection_minute_1 (utc_minute>? AND utc_minute<?)`，保留distinct临时树，已没有 GROUP BY/ORDER BY 临时排序。局部结果仍超过10秒，不能宣称容量门通过。

## 验证与复测

`cargo test --release --manifest-path residential-monitor/src-tauri/Cargo.toml --lib raw_bucket_series -- --nocapture`：3 passed，1.97秒。覆盖七种粒度 × 六种含负数、跨零、裁边、空区间的窗口 × 十一种过滤；独立零字节桶断言；分钟索引计划；实际执行中取消、共同已过期handler及连接恢复。第一次新fixture列名错误已修正，最终三项全绿。

只读阶段探针：1 passed，61.91秒；两个阶段分别受60秒上限保护。owned Rust文件rustfmt及diff whitespace检查通过；全产品门由主任务独立执行，不在这里冒充完成。

复测脚本（输出路径必须尚不存在）：

```powershell
& .trellis/tasks/09-19-resiwatch-resource-optimization/research/probe-raw-query-stages.ps1 `
  -Fixture 'C:/Users/lyh/AppData/Local/Temp/resiwatch-resource-measure-20260919-retention-cache/corpus-a1000-resume' `
  -Output '<新的研究JSON路径>' -SeriesOnly
```

源码SHA256已随每份JSON保存。后续完整30天报告、A250/A1000 deadline及性能验收仍需主任务处理；本轮不扩大为索引/缓存/查询框架重构。
