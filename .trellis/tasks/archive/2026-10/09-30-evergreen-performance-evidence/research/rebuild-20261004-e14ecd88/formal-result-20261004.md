# T05 正式计量结果（rebuild-20261004-e14ecd88）

日期：2026-10-04，America/Chicago；执行 UTC 2026-10-04T05:45:58Z–07:56:11Z。身份为方向 1（`idx_session_attr_chain_rule` 复合索引、layout4、字典回收跳跃枚举）与方向 2（bench 堆计数）实施后的源码。P1 构建与 P2 语料、oracle、4 个 probe 均通过，见 `builder-driver.receipt.json`、`generator-driver.receipt.json`。五个阶段由 `runners/run-stages.sh` 串行执行，未并行构建或其它负载。

## 执行与退出

| 阶段             | native 次数   | native exit                         | 校验错误 | 竞争负载 | driver exit | 含义                   |
| ---------------- | ------------- | ----------------------------------- | -------- | -------- | ----------- | ---------------------- |
| matrix           | 22/22         | 全部 0                              | 0        | 无       | 2           | 有门 FAIL              |
| primary          | 6/6           | 全部 0                              | 0        | 无       | 3           | 无 FAIL，有 UNVERIFIED |
| capacity         | 106/106       | 全部 0                              | 0        | 无       | 3           | 无 FAIL，A1000 NOT_RUN |
| retention        | 2/2           | 101 / 0                             | 0        | 无       | 2           | A50 未完成，A250 完成  |
| heap-diagnostics | 12/12（重跑） | 8 次 0，4 次 0xC0000142；重跑全部 0 | —        | 未检查   | 2；重跑 0   | 诊断，非正式门         |

matrix、primary、capacity 的 driver stderr 均为 0 字节。

## matrix（F1–F12，限值 1.10）

| 门  | 场景 / 指标                          | baseline | candidate | 比值   | 10-03 比值 | 结果 |
| --- | ------------------------------------ | -------- | --------- | ------ | ---------- | ---- |
| F1  | a1000-metadata ingest p95 ms         | 34.6675  | 37.9611   | 1.0950 | 1.1109     | PASS |
| F2  | a250-backlog ingest p95 ms           | 21.1398  | 20.6627   | 0.9774 | 1.1081     | PASS |
| F3  | a50-unchanged ingest p95 ms          | 13.2942  | 12.7736   | 0.9608 | 0.9039     | PASS |
| F4  | a250-failed ingest p95 ms            | 19.5814  | 20.3866   | 1.0411 | 1.1020     | PASS |
| F5  | a250-metadata ingest p95 ms          | 20.5354  | 23.5408   | 1.1464 | 1.0172     | FAIL |
| F6  | a1000-metadata native private p95 B  | 11816960 | 13070336  | 1.1061 | 1.1513     | FAIL |
| F7  | a1000-unchanged native private p95 B | 10612736 | 12546048  | 1.1822 | 1.1677     | FAIL |
| F8  | a50-metadata SQLite xWrite B         | 1483200  | 1606800   | 1.0833 | 1.4235     | PASS |
| F9  | a250-failed SQLite xWrite B          | 3232096  | 3688912   | 1.1413 | 1.1018     | FAIL |
| F10 | a1000-metadata SQLite xWrite B       | 7880872  | 8893528   | 1.1285 | 1.1787     | FAIL |
| F11 | a250-backlog native CPU s            | 0.71875  | 0.671875  | 0.9348 | 0.2727     | PASS |

F12 空窗口首读 p95：11 个场景中 2 个 PASS（a50-unchanged 1.0100、a1000-counters 1.0817），9 个 FAIL（a50-counters 2.1750、a250-unchanged 1.4934、a250-counters 1.1961、a50-metadata 1.1750、a250-metadata 1.1786、a1000-unchanged 1.1728、a1000-metadata 1.1176、a250-backlog 1.1355、a250-failed 1.1528）。F12 非空生产 oracle 为 UNVERIFIED。

F8 本次窗口内两边都没有 checkpoint，比值只含 WAL。F1–F5、F12 的延迟比值在 10-03 与本次之间变化方向不一致（F2 −0.13、F5 +0.13），同一程序在两次运行间的波动大于 0.10 门的余量。

F9/F10 的写入拆分（baseline / candidate）：

| 场景           | WAL B             | 主库 B（checkpoint） | xWrite 调用 | writer_rows_changed |
| -------------- | ----------------- | -------------------- | ----------- | ------------------- |
| a250-failed    | 2867552 / 3238352 | 364544 / 450560      | 1482 / 1683 | 52800 / 52830       |
| a1000-metadata | 7049384 / 7914584 | 831488 / 978944      | 3627 / 4083 | 210300 / 210330     |

两场景窗口内两边都有 checkpoint 主库写入。各 metadata 类场景 `writer_rows_changed` 仍比 baseline 多 30（每次提交 1 行，来自 `coverage_interval` 的 `ended_utc` 更新）。复合索引仍包含完整 `chain_key`，每帧切换 `chain_key` 时仍需更新该索引；本设计没有让每次提交帧数回到 baseline。用 10-03 的 `walattr.py`（只改程序路径并允许传入 `--archive failed --period-rule`）在本身份上重测，结果在 `../f6-f10-attribution-20261004/`，只用于归属：

| 对象（每次提交页数）          | a250-failed b / c | a1000-metadata b / c | a1000-metadata 10-03 c |
| ----------------------------- | ----------------- | -------------------- | ---------------------- |
| 每次提交总帧数                | 23–25 / 26–28     | 57 / 64              | 67                     |
| `idx_session_attr_chain_rule` | 0 / 3             | 0 / 7                | —                      |
| 原两个 chain 表达式索引合计   | —                 | —                    | 10                     |
| `coverage_interval`           | 0 / 1             | 0 / 1                | 1                      |
| `data_version`                | 1 / 0             | 1 / 0                | 0                      |

a1000-metadata 的每次提交增量全部来自复合索引（7 页，原方案 10 页）；`coverage_interval` 与 `data_version` 的 ±1 相抵。a250-failed 另有 `connection_chain` −0.33、`sqlite_master` +0.17 页的小差。F9/F10 的剩余超限主要来自复合索引页与窗口内 checkpoint。

## primary（AC7，三轮合计）

| 门                      | baseline | candidate | 比值   | 限值 | 结果           |
| ----------------------- | -------- | --------- | ------ | ---- | -------------- |
| CPU 秒                  | 18.75    | 0.453125  | 0.0242 | 0.70 | PASS           |
| SQLite xWrite B（子集） | 70595256 | 32730336  | 0.4636 | 0.50 | PASS（仅子集） |
| 全部应用文件写入        | null     | null      | —      | 0.50 | UNVERIFIED     |

逐轮 candidate CPU 为 0.03125 / 0.296875 / 0.125 秒，baseline 为 7.234375 / 4.484375 / 7.03125 秒。candidate CPU 计量偏低的现象与 10-03 相同，原因未查明。

## capacity（10 000 ms 门）

| 场景             | 通过 / 要求     | 最大 wall ms | 10-03 最大 wall ms | 结果 |
| ---------------- | --------------- | ------------ | ------------------ | ---- |
| a50-host-30d     | 21/21           | 1521.5       | 1788.8             | PASS |
| a50-host-1d      | 21/21           | 87.1         | 89.8               | PASS |
| a50-network-30d  | 1/1（诊断单次） | 1106.9       | 1212.0             | PASS |
| a250-host-30d    | 21/21           | 6917.9       | 7132.2             | PASS |
| a250-host-1d     | 21/21           | 251.7        | 247.0              | PASS |
| a250-network-30d | 21/21           | 5871.7       | 6170.5             | PASS |

所有 native 查询都在 preflight 读库之后执行，没有物理冷页证明。A1000 仍为 NOT_RUN。layout4 语料比 layout3 大 5.4 MB（A50）与 27.1 MB（A250），原因是复合索引存完整 `chain_key`。

## retention（隔离回收容量，first-day）

执行 UTC 06:41:08Z–07:39:47Z。口径沿用 09-19 benchmark-harness：语料副本，维护 now = start_utc + 31 天，CHUNKS=100000，cfg(test) 隔离删除门。生产 AUTO_DELETE_ENABLED 仍为 false。源语料 hash 前后一致。

| 指标                         | A50                      | A250                 |
| ---------------------------- | ------------------------ | -------------------- |
| test exit                    | 101（complete 断言失败） | 0                    |
| complete                     | false                    | true                 |
| chunks（上限 100000）        | 100000                   | 7455                 |
| 日删除块 / 辅助清理块        | 1892 / 98108             | 7347 / 108           |
| 日删除阶段 wall s            | 86.0                     | 616.4                |
| 总 wall s                    | 2750.9                   | 748.3                |
| 单块 wall ms p50 / p95 / max | 24.3 / 35.3 / 191.7      | 75.8 / 159.5 / 433.9 |
| 块错误 / 重试                | 0 / 0                    | 0 / 0                |
| 删除 raw 行                  | 72000                    | 360000               |
| sessions 前 / 后             | 432000 / 417600          | 2160000 / 2088000    |
| receipts 前 / 后             | 2592000 / 100000         | 2592000 / 100000     |
| 结束时可回收 session / 收据  | 0 / 0                    | 0 / 0                |
| 守恒 / 剩余过期 raw          | true / 0                 | true / 0             |
| quick_check                  | ok                       | ok                   |
| staging 采样最大 B           | 82468864                 | 243585024            |
| WAL 采样最大 B               | 10526632                 | 8911592              |

A50 的实际回收工作在前 1892 块（86.0 s）内完成：过期 raw 为 0，可回收 session 与合格收据均为 0。之后 98108 块全部是辅助清理块，每块 `more_pending=true`，`completion_inventory_checks=0`，直到用尽 100000 块。

未完成的原因已确认，属于 b8a64a1（2026-09-24）引入的已有逻辑，本次改动未涉及：

1. `c3/retention_day.rs` 的 `cleanup_expired` 每块各推进字典游标与覆盖游标一页（128 行），页不满时游标置 0，下一块从头开始。两份语料字典均为 1085 行、覆盖区间均为 720 行，因此字典周期 9 块，覆盖周期 6 块。
2. `storage_lifecycle.rs:272-282` 在 session 清理删除了行时，只把字典游标置为 −1，覆盖游标不变。session 删除持续若干块，字典与覆盖的相位差因此改变；删除结束后相位差固定。
3. 完成判定要求同一块之后两个游标都为 0（`bench/corpus.rs:347-351` 与 `bench/corpus.rs:443`）。设两游标的块内位置为 i（mod 9）与 j（mod 6），每块 i、j 各加 1，(i − j) mod 3 不变。只有该值为 0 时两者才会同时为 0。
4. A50 运行中 12 次采样的 (i − j) mod 3 全部为 1，所以两个游标永远不会同时为 0。A250 的相位差为 0，在辅助清理第 108 块完成。两个语料的结果不同，只因 session 删除结束时的相位不同。

同一条件也用于生产调度：`c2/facade.rs:2241-2254` 在 AUTO_DELETE_ENABLED=true 时读取辅助游标，非 0 时把下次维护设为 1 秒后。若开启删除且相位差不为 0，维护会每秒调度且不会停止。当前 AUTO_DELETE_ENABLED=false，该路径不执行。修复需要修改回收或完成判定逻辑，超出本轮批准范围，未修改。

09-19 的约 240 s 参考早于 b8a64a1，不与本次对比。

### 修复后复测（retention-fix）

用户批准修复后，`c3/retention_day.rs` 的 `cleanup_expired` 改为两个辅助扫描同一轮结束（先结束的一方停在 0），见 design.md「2026-10-04 回收不结束修复」。执行 UTC 08:19:51Z–08:34:17Z，`runners/run-retention-fix.py` 只替换测试程序与输出目录，其它参数不变。测试程序为修复后工作树的 `cargo +1.98.0 test --release --lib --no-run` 产物 `candidate-fix-library-tests.exe`（SHA256 `0FEA3618…DED280C`），不是 P1 构建身份。竞争负载在开始与结束时均为空。

| 指标                         | A50                  | A250                 |
| ---------------------------- | -------------------- | -------------------- |
| test exit / complete         | 0 / true             | 0 / true             |
| chunks                       | 2511                 | 7433                 |
| 日删除块 / 辅助清理块        | 1892 / 619           | 7347 / 86            |
| 日删除阶段 wall s            | 124.9                | 563.2                |
| 总 wall s                    | 166.8                | 682.6                |
| 单块 wall ms p50 / p95 / max | 55.8 / 107.8 / 377.9 | 70.0 / 151.7 / 419.6 |
| 块错误 / 重试                | 0 / 0                | 0 / 0                |
| 结束时可回收 session / 收据  | 0 / 0                | 0 / 0                |
| 守恒 / 剩余过期 raw          | true / 0             | true / 0             |
| quick_check                  | ok                   | ok                   |

A50 的日删除块数（1892）与修复前相同，辅助清理在 619 块后结束。A50 日删除阶段 wall 由修复前 86.0 s 变为 124.9 s；该阶段代码未改动，差异原因未查明，不作为修复效果。

## heap-diagnostics（F6/F7 分阶段归属，非正式）

执行 UTC 07:39:47Z–07:56:11Z。使用 matrix 相同程序与 a1000 unchanged/metadata 参数，只额外设置 `RESIWATCH_BENCH_HEAP=1`，baseline/candidate 交替各 3 轮。计数器加在两边，只用于归属。

首次运行 `heap-diagnostics/` 的前 8 次 exit 0。后台 stages shell 在 07:45 达到 2 小时时限被停止，之后 4 次（a1000-metadata r2/r3）在 0.02 s 内以 0xC0000142（STATUS_DLL_INIT_FAILED）启动失败，属于进程启动失败。`runners/run-heap-diagnostics-rerun.py` 只改输出目录，参数不变，重跑 12/12 exit 0，见 `heap-diagnostics-rerun/`。下表来自重跑；首次 8 次有效运行的分阶段数值与重跑一致（差 ≤16 B）。

| 指标（3 轮）                   | unchanged b           | unchanged c           | metadata b            | metadata c            |
| ------------------------------ | --------------------- | --------------------- | --------------------- | --------------------- |
| native private p95 MB          | 11.20 / 10.70 / 10.35 | 12.62 / 12.61 / 12.37 | 11.46 / 11.58 / 11.57 | 12.96 / 12.63 / 13.35 |
| native private 最小值 MB       | 9.47 / 9.58 / 8.84    | 9.32 / 9.36 / 9.08    | 10.10 / 10.05 / 9.81  | 10.11 / 10.02 / 10.01 |
| Rust 堆窗口峰值 B              | 1972637               | 1939574               | 3000550               | 3012071               |
| SQLite 窗口峰值 B              | 194424                | 238464                | 198784                | 251568                |
| Rust ingest 阶段峰值 B         | 1360386               | 1327323               | 2291723               | 2303244               |
| Rust archive 阶段峰值 B        | 235955                | 165                   | 235955                | 164                   |
| SQLite ingest / 首读阶段峰值 B | 49560 / 194424        | 69824 / 238464        | 49560 / 198784        | 84936 / 251568        |

窗口峰值以测量窗口开始时的 live 字节为 0 点，三轮完全相同。live_query、snapshot、sample 阶段两边相同。

结论：

1. private p95 中位数差为 +1.91 MB（unchanged）与 +1.39 MB（metadata），最小值两边接近。
2. 同一窗口内 Rust 堆峰值差为 −33063 B 与 +11521 B，SQLite 峰值差为 +44040 B 与 +52784 B。两者合计不超过 0.06 MB，不能解释 F6/F7 的差额。
3. candidate 的 archive 阶段在测量窗口内 `next_job` 未返回任务（`archive_tick_at` 同步执行），所以该阶段分配接近 0；工作没有移到阶段窗口之外。
4. private 字节的差额来自这两个计数器之外的内存。计数器不覆盖 Windows 堆提交与碎片、不经过 Rust 全局分配器或 `sqlite3_malloc` 的分配、线程栈与映像页。具体来源原因未查明。继续定位需要新增堆提交统计（例如每阶段采样 private 字节或 Windows 堆摘要），需要重新批准范围。

## AC 状态

- AC2：capacity 的精确 oracle 比对通过；跨窗口 session、取消与 deadline 的完整条款没有在本轮单独验收。保持未勾选。
- AC3：A50/A250 所选门通过；A1000 NOT_RUN，冷页条件未分开报告。保持未勾选。
- AC4：F5、F6、F7、F9、F10 与 F12 的 9 个空窗口场景 FAIL，F12 非空 UNVERIFIED。保持未勾选。
- AC5：CPU 与 SQLite 子集通过，全部应用文件写入 UNVERIFIED。保持未勾选。
- AC6：AUTO_DELETE_ENABLED 仍为 false，WAL/FULL 未变；AC8 证据未在本轮验收。保持未勾选。

## 收据 SHA256

| 文件                                       | SHA256                                                             |
| ------------------------------------------ | ------------------------------------------------------------------ |
| matrix/summary.json                        | `ad42bd17e380d1e298e738d75cb0710053f01c24b66a05da8185a0b9716d285e` |
| primary/summary.json                       | `d32c586baa2e2f190ead7a14b6257cf22e51de111f7b667fa1f5e96123c740c3` |
| capacity/summary.json                      | `cc762ee25d3c1acccb5af6e23323c390fe7921bdc488376e2eb91328add32841` |
| matrix-driver.receipt.json                 | `5b3b83352b6484d23f62ac8f85839cb9adc288d7e41e01289ddb08d5f7ad9e8a` |
| primary-driver.receipt.json                | `4740c9986addd0c0a9b80d3d773196109b4ff69c90a68a3dc891e449ff8dfb65` |
| capacity-driver.receipt.json               | `82ff9f663a6cc7428511236a4dea6cea50aef164b4db66ce0145f36525b3bcd7` |
| retention/summary.json                     | `812fb21a347eeef79abfdbf7c50839febbb73d8c26dc1ee23eb8c7d2cb003bfc` |
| retention/a50-first-day.result.json        | `447c87c32d5e0f35d83feea2ac0cfa53a713ce8afec8d38b311028a113841975` |
| retention/a250-first-day.result.json       | `74a989118024a2b01d7b975328fc3ddb69f45855c2ee7538d043b36cdfafdf96` |
| retention-driver.receipt.json              | `0272287cfb3bb196c2c3b000d10a50a5e13f2cd894b85222c0c9651e6593612a` |
| heap-diagnostics/summary.json              | `7ec6d6d114a3b4fe407d8c004af46d5fe34a02f076e280551b58483f7210ae23` |
| heap-diagnostics-driver.receipt.json       | `3b6d52c9b553542aee7d37a14bab74bc71a6fa4aba9abd3a1abd70489e409290` |
| heap-diagnostics-rerun/summary.json        | `ab5925170d5e0666d95e3551e2f5379f9d9b3ec3aed6202777e1df0d00396e6a` |
| heap-diagnostics-rerun-driver.receipt.json | `6145228f002cf1680e9b33632fb6d5f77ea0967295b24129ec2704df370121d2` |
| runners/run-heap-diagnostics-rerun.py      | `893ab8bf033073adf14166e5ce9cfe1644608ef5ec2cdcf55906b0f375976f70` |
| retention-fix/summary.json                 | `8e8cab96bdede790e8646ef6f4c1f837665ca2ca191f70300f4e585b957373c8` |
| retention-fix/a50-first-day.result.json    | `3a03e7202fcb2fe7857604106f1847d3f600afba0bd4f1f13aa3dcd85459b02a` |
| retention-fix/a250-first-day.result.json   | `431ceda232e115deb21508618dfc5a4a80ccc002e62b2b42133ed08b9ef4890c` |
| retention-fix-driver.receipt.json          | `06f3501e2aa17373e7795805d4b04bb7e231c2ee52604b6e6390ad971a2d315c` |
| runners/run-retention-fix.py               | `88241c142c51f3ac94e438efbc138fcea4633b3646e76fcc76a8a2b39d6c5f09` |

正式阶段开始后未修改正式 runner、语料或门槛；新增 heap 重跑与 retention-fix 两个 runner 副本。matrix、primary、capacity 测得的源码不含回收修复；该修复只在删除开启时执行。
