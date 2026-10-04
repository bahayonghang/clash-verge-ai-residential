# T05 正式 matrix 独立终审与失败归属

日期：2026-09-30。审查代理：`/root/t02_review`。本轮只读源码与已保存的小型 JSON/gzip 日志，只写 `research/review.md`与`research/review-*`；没有构建、测试、基准执行，也没有打开或 hash corpus。

## 1. 结论

**执行证据 PASS，性能门 FAIL，AC4 未通过。** 22 份 native 调用全部退出 0，逐份校验与独立复算 `errors=[]`。F1–F11 中 5 项 PASS、6 项 FAIL。F12 原 11 个空窗口比值全部 FAIL；独立非空门仍为 UNVERIFIED。保留候选准入与功能检查通过不能覆盖本次正式性能失败。

证据：[逐份核验](review-formal-matrix-validation.json)、[归属证据提取](review-formal-matrix-attribution.json)、[原始 summary](formal-matrix-20260930/summary.json)、[原始 results](formal-matrix-20260930/results.json)。原始证据未改写。

## 2. 执行、身份与计量边界

- 实测时段：2026-09-30 15:30:07.032823 至 15:46:01.574235 UTC。11 对场景按 baseline → candidate 顺序执行，22 条调用时间不重叠。该记录不证明整机无其它工作负载。
- 固定 seed=20260919、start_utc=1800001800、1 Hz、5 秒预热+30 秒测量、每5帧查询。没有 virtual-time。每份30帧/30次提交/30份进程采样，first/repeated reader 各6次。
- 实际报告、执行合同及已审冻结身份一致：baseline `06BAC94FD88E37DC...`，candidate `9604FBE6ED24C6DC...`。本次只核对已保存身份，没有重新 hash exe。两侧原 writer 初末均 WAL/FULL；baseline schema4、candidate schema5。
- 原始gzip字节SHA256、可读日志、native stdout与保存JSON一致；固定argv、fixture hash重算、platform和配对时区均通过。实际两侧偏移均为 -18000 秒。每对流量守恒与期望增量一致。
- memory p95/max从30份原始进程样本重算，CPU由native_after-before重算，SQLite写入由DB/WAL/其它成功xWrite加总重算。延迟JSON没有逐次原始延迟，故只能核对count、分位数顺序、已保存p95及比值，不能独立重建延迟分位数。
- 退出边界：native为22×0；实施owner报告的原始PowerShell session 30889外层退出为1。Python driver原始exit没有单独实采，标为UNRECORDED。源码的 `sys.exit(2)` 是预期路径，不能补造为实测退出2。
- baseline A50 unchanged 有1个frame budget overrun，candidate为0；其余21份均0。不得套用旧历史“全部无overrun”的描述。
- `all_application_file_write_bytes` 与 `spool_write_bytes` 全部null。SQLite子集不能替代全部应用文件写入、设备写入、安装态、WebView或真实后台worker证据。

## 3. F1–F11 原门逐项复算

全部比值为candidate/baseline，原门均保留≤1.10。

| 门 | 场景 | 指标 | baseline | candidate | 比值 | 状态 |
| --- | --- | --- | ---: | ---: | ---: | --- |
| F1 | A1000 metadata | ingest p95 ms | 30.1777 | 31.8543 | 1.055558 | PASS |
| F2 | A250 backlog | ingest p95 ms | 8.3431 | 16.0465 | 1.923326 | FAIL |
| F3 | A50 unchanged | ingest p95 ms | 366.4164 | 315.4738 | 0.860971 | PASS |
| F4 | A250 failed | ingest p95 ms | 13.2234 | 11.8697 | 0.897628 | PASS |
| F5 | A250 metadata | ingest p95 ms | 14.7366 | 14.9702 | 1.015852 | PASS |
| F6 | A1000 metadata | native Private Bytes p95 B | 11497472 | 13070336 | 1.136801 | FAIL |
| F7 | A1000 unchanged | native Private Bytes p95 B | 11030528 | 12206080 | 1.106573 | FAIL |
| F8 | A50 metadata | SQLite xWrite B | 1483200 | 2111360 | 1.423517 | FAIL |
| F9 | A250 failed | SQLite xWrite B | 3232096 | 3561216 | 1.101829 | FAIL |
| F10 | A1000 metadata | SQLite xWrite B | 7880872 | 9288904 | 1.178664 | FAIL |
| F11 | A250 backlog | process CPU s | 0.6250 | 0.3125 | 0.500000 | PASS |

## 4. F12：空窗口与非空验收分开

源码确认：`bench/facade.rs:185–212`保留原秒级端点；6次结束时间为start+10/15/20/25/30/35秒。`c3/service.rs:417–420`把当前窗口两端div_euclid(60)，当前raw分钟范围为空。该矩阵保留reader固定成本与原回归门，不能代表完整非空报告聚合。fixture hash仅证明输入参数一致。

每份reader只有6次；当前估计器为floor((n−1)×percent/100)，p95与p99均取排序后第5个。样本数不构成豁免1.10门的理由。

| 场景 | first p95 baseline ms | first p95 candidate ms | 比值 | first门 | repeated p95 baseline/candidate ms |
| --- | ---: | ---: | ---: | --- | --- |
| a50-unchanged | 1.2136 | 2.0053 | 1.652357 | FAIL | 0.8674 / 1.8385 |
| a50-counters | 1.1129 | 1.4892 | 1.338126 | FAIL | 0.8191 / 1.1090 |
| a50-metadata | 1.1124 | 1.3676 | 1.229414 | FAIL | 0.8573 / 1.0369 |
| a250-unchanged | 1.2601 | 1.4222 | 1.128641 | FAIL | 0.9294 / 1.1052 |
| a250-counters | 1.1440 | 1.6115 | 1.408654 | FAIL | 0.9913 / 1.2295 |
| a250-metadata | 1.1314 | 1.4148 | 1.250486 | FAIL | 0.9434 / 1.0949 |
| a1000-unchanged | 1.1502 | 1.3380 | 1.163276 | FAIL | 0.8437 / 1.1685 |
| a1000-counters | 1.2214 | 1.4217 | 1.163992 | FAIL | 0.8511 / 1.2032 |
| a1000-metadata | 1.2677 | 1.4968 | 1.180721 | FAIL | 0.8321 / 1.3934 |
| a250-backlog | 0.9998 | 1.1644 | 1.164633 | FAIL | 0.7981 / 0.9911 |
| a250-failed | 0.9432 | 1.5208 | 1.612383 | FAIL | 1.0103 / 1.3069 |

已审最终身份的1分钟与30天单样本为非空生产结果，但缺本轮同窗口配对reader分位数及完整结果集合对照，仍不能关闭F12非空门。

## 5. 失败归属分级

### 5.1 F8/F9/F10：SQLite成功写入差异

**已证实事实。** 三门差异均来自DB/WAL成功xWrite；其它SQLite文件差值为0，失败回调为0。

| 门 | DB增量 B | WAL增量 B | 总增量 B | 成功调用增量 | writer_rows_changed baseline/candidate |
| --- | ---: | ---: | ---: | ---: | --- |
| F8 | 380928 | 247232 | 628160 | 214 | 10800 / 10830 |
| F9 | 81920 | 247200 | 329120 | 140 | 52800 / 52830 |
| F10 | 172032 | 1236000 | 1408032 | 642 | 210300 / 210330 |

F8与F10的当前candidate `sqlite_xwrite` 所有字段、总字节和writer_rows_changed与09-24旧candidate逐项相同；旧exe为`4C17FA3F7A8A37FD...`，本轮为`9604FBE6ED24C6DC...`。该证据确认旧写入形状在本轮重现，不能证明具体责任函数，也不能证明lazy缓存无任何间接影响。

源码与表页收据确认：candidate schema5新增正向coverage及若干索引，baseline为schema4。`c2/facade.rs:946–988`形成observed_interval；`storage.rs:885–917`逐tick合并覆盖区间。最终coverage_interval为baseline 0行/candidate 1行，各metadata场景writer_rows_changed差30，与30次覆盖更新的路径相符。该对应不能把全部字节增量归给coverage。

`c3/schema.rs:28–41`包含v5分钟/coverage/attr索引；host与chain变化会触及必要的chain表达式索引。当前`storage.rs:1034–1104`已按实际变化字段构造UPDATE，09-20已修复的“全部7字段SET”不能当成本轮尚未修复的根因。历史独立索引实验只适用于当时的旧/新SQL对比。

**候选解释。** 必要索引维护、coverage页修改、DB checkpoint发生时机及页布局共同影响DB/WAL写入。F8 baseline DB写为0而candidate为380928 B，符合测量窗口跨checkpoint的形状。

**原因未查明。** 现有VFS收据缺页号、SQL/phase归属和xSync等待时间；没有把每一新增页映射到具体表/索引或证明checkpoint触发来源。不能凭F8/F10旧比值接近或相等断因。

### 5.2 F2与F11：ingest尾延迟和全进程CPU

**已证实事实。** A250 backlog的ingest p50为7.1565→8.4290 ms，p95为8.3431→16.0465 ms（1.923326，FAIL）；整个测量期process CPU为0.625→0.3125 s（0.5，F11 PASS）。archive_tick p95为12.6220→4.8720 ms，whole_tick p95为21.6456→20.9619 ms；两侧frame overrun和schedule lag均0。

`bench/facade.rs:158–175`将ingest与archive分别计时；全进程CPU包含两者及查询、采样等测量期工作。因此两个门衡量不同对象，CPU下降不能覆盖ingest p95失败。`c3/archive.rs:155–218`的缓存队列/到期检查与baseline `c3/archive.rs:116–153`每tick发现下一任务不同，源码支持archive工作减少的候选解释。

**候选解释。** archive调度成本减少可能降低总CPU；ingest持久提交、I/O等待或调度等待可能同时增加尾延迟。

**原因未查明。** 没有逐frame ingest延迟序列、分阶段CPU、commit/xSync等待或锁等待收据，不能把1.923326归给某个函数、磁盘或lazy缓存；已有whole_tick改善也不取消F2门。

### 5.3 F6/F7：native Private Bytes

**已证实事实。** F6 p95增加1572864 B（13.6801%）；F7增加1175552 B（10.6573%）。30个原始样本与p95/max重算一致。F7测量前candidate已为12169216 B、baseline为9400320 B，测量末为11784192/11018240 B；差异并非只在本次30秒阶段形成。F6测量前candidate为10448896 B，低于baseline的11124736 B，说明两个场景的形成时点不同。

**候选解释。** schema/page缓存、持久metadata去重状态、ArchiveScheduler描述、reader临时分配和allocator保留均可能贡献进程私有提交内存。`accounting.rs:123`的已提交metadata状态及`c3/archive.rs:155–163`的队列是可检查owner；最终complete档案计数相等不能直接证明队列分配相等。

**原因未查明。** 收据没有heap分配归属、队列容量/字符串字节或阶段前后内存细分。不能据一个p95比值认定内存泄漏，也不能给lazy缓存分配责任。

### 5.4 F12：空reader固定路径

**已证实事实。** 当前Raw窗口为空的形成路径已定位，11项比值仍全部失败。当前`c3/service.rs:152–190,394–420,474–521`包含reader打开、快照能力与生命周期检查、durable version、deadline及关闭；baseline的build_result使用旧能力与singleton版本。当前`raw_fold.rs:145–161,790`还会加载字典。

**候选解释。** 固定reader/plan/version/字典加载成本和测量波动可能影响约1–2 ms的样本。

**原因未查明。** 收据没有分阶段延迟或非空配对结果。保留lazy只改变读投影的一部分，c278到当前同时存在多项生产差异。F12空窗口不能独自驱动SQL改造，也不能无证据要求回退或保留lazy。

## 6. 后续阶段探针与检查

当前primary按原计划独立采证；容量runner已静态准入，排在primary之后独占。以下为失败归属所需的后续诊断，不改写已完成的原门。

| 优先级 | 文件/检查位置 | 所需证据与必须保持的检查 |
| --- | --- | --- |
| P1 | `src/bench/facade.rs`，必要时受控`c2/facade.rs`/`storage.rs`计时点 | 保存逐frame ingest/archive/query延迟及CPU差值，分解prepare/persist/commit/health；保留原AB、5+30秒、WAL/FULL及全部F门。先审双侧同instrumentation与新身份，再独占运行。 |
| P1 | `src/bench/write_vfs.rs`；隔离诊断下的页号映射 | 为成功xWrite保留文件/offset/size/phase并记录原样转发的xSync耗时；关联checkpoint与表/索引页。必须通过现有VFS转发、错误、安全初始化及WAL/FULL测试；不得更改checkpoint、schema或持久语义求PASS。 |
| P1 | `src/c3/service.rs`现有stage probe、`src/c3/raw_fold.rs` | 分开reader open、snapshot plan、version、dictionary/projection、fold、close；使用明确非空同窗口双侧输入并核对totals/series/rank/coverage等结果。保留10秒、取消、原空窗F12与物理缓存边界。 |
| P2 | `src/accounting.rs`、`src/c3/archive.rs`、`src/bench/facade.rs` | 记录metadata状态条数/容量、archive pending/inflight数与拥有字符串字节、reader前后内存。对照30样本p95及生命周期，不凭单样本声明泄漏。任何新增owner文件先由主会话核对批准范围。 |

强模型负责定义因果假设、检查instrumentation对测量的影响与正式验收。受约束执行模型可按固定schema提取收据、生成表格及运行已审命令；不得自行修改门槛、数据集、退出判定或因果结论。以上证据合同适用于Claude Code、Codex、Grok Build、Kimi Code、OMP。

## 7. live driver退出留证

`run-live-recorded.py`静态审查PASS，AST parse PASS，SHA256 `88550bb4fca23d98a5e74abe7aeedab18d1ce7731cf5d890638a0e8f36219f5c`。wrapper通过`subprocess.Popen(argv)`继承stdout/stderr，wait返回child exit后先保存再sys.exit；不改变benchmark参数。primary新收据可证明该driver实际退出，不能追补matrix旧driver退出。进程尚未结束时status=running只表示未有完成收据；进程启动失败或wrapper中断仍可能没有finished状态，不能推定PASS。

## 8. 验证状态

- 本审查：22份JSON/gzip/收据、独立指标重算与文档检查完成；没有运行产品负载。
- lint/type-check/tests：沿用保留候选准入阶段已审收据，本阶段未重跑；正式负载独占期间不并行这些命令。
- primary、106次容量、全部应用文件归属、安装态与其它AC：按独立证据验收，本报告不作通过声明。
