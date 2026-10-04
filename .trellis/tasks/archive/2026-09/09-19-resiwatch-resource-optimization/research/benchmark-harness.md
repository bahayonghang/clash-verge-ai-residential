# 实际链路与容量基准

## 已验证边界

- `monitor-bench replay-facade` 调用生产 `AppFacade::ingest_snapshot`（含 `commit_alert_bundle`）、`archive_tick_at`、可选 live 与两次独立 reader 的 `run_uncached`；没有 HTTP、Tauri 事件循环、WebView 或安装态操作。
- 2026-09-19 基准模块 8 个测试通过：真实核算守恒、报告、档案写失败、隔离目录拒绝、SQLite VFS 写参数/错误转发、WAL/FULL/读连接/checkpoint/integrity。会注册默认 VFS 的测试均在单测试独立子进程运行，父测试进程不注册。
- SQLite 计数来自只归属隔离目录文件的成功 `xWrite` 请求，分 DB/WAL/其它 SQLite 文件。全部回调转发原 VFS 或原文件；不改变 durability、锁或 checkpoint。此数不含 mmap/shm、spool、日志及设备写放大，不能称作全部应用或物理磁盘写入。
- Windows 当前进程 CPU、RSS、Private Bytes、句柄及 I/O 是真实 Win32 读数；其它平台未支持的字段保持 null。没有枚举其它进程。首次/重复 reader 都不驱逐 OS page cache，不能称物理冷读。ingest 延迟含 accounting/告警/提交，非独立 COMMIT 计时。
- `benchmark-baseline-smoke.json`：A8、3个虚拟 tick、档案齐全、计数变化、无展示查询。流量守恒、3次提交，SQLite 成功写入131840B。仅验证工具可用，不能替代 AC7 的实时 AB。

## 冻结基线

源码提交：`c278bb7b56603001e32e353d2ee589dccef0bfe9`；源码 tar SHA256 由主会话记录：`87684724D5E5B1B3A18C0212CD594CA05F9454EDD6C28582CD879270F9749EB7`。

基线程序：

```text
C:/Users/lyh/AppData/Local/Temp/resiwatch-perf-baseline-68253a0cde334e058766b87f8a797380/monitor-bench-baseline-v2.exe
SHA256 11C2542BC2C9F98A4B7517E67215E76884D8C01D25D64AE7FAF478571BF7633E
```

构建只向原始源码副本加入相同 bench.rs、monitor-bench.rs 与 bench/{facade,process,write_vfs}.rs。原有 dist 副本仅满足 Tauri release 嵌入，不声明 UI 构建一致。基线可执行文件已独立复制，不会被后续 candidate build 覆盖。

v2补齐xOpen失败路径的内外pMethods初始化，后续AB必须用v2。旧`monitor-bench-baseline.exe`及其SHA256 `9D5BF09001FCA3330A0BF49122D937F33646A64255271E5A21D1CBDBBBA59E2D`保留，用于对应已有smoke记录。原始源码副本不加入候选专属corpus CLI或`run_chunk`实现。

## AC7 实时 AB

`counters` 表示所有流量计数变化、metadata 完全不变；`unchanged` 连计数也冻结，是另一场景。默认seed=20260919、start_utc=1800001800、hz=1。每轮使用新的空目录，JSON输出必须位于该目录之外。

```powershell
& <baseline-exe> replay-facade --active 250 --hz 1 --duration-secs 300 --warmup-secs 30 --workload counters --archive complete --source-revision c278bb7b56603001e32e353d2ee589dccef0bfe9 --dir <isolated-baseline-round-1> > <baseline-round-1.json>
& <candidate-exe> replay-facade --active 250 --hz 1 --duration-secs 300 --warmup-secs 30 --workload counters --archive complete --source-revision <candidate-source-label> --dir <isolated-candidate-round-1> > <candidate-round-1.json>
```

顺序重复第2/3轮；不得加`--virtual-time`。每轮包含30秒实时预热及300秒测量，档案初始化时间额外单列。不要与 Cargo、前端测试或容量生成同时运行。比较时核对fixture_hash、seed/时间/参数、平台、时区、源码与exe哈希；CPU至少下降30%，SQLite子集写入至少下降50%不自动证明全部应用文件门。

场景覆盖使用A50/250/1000；`--workload metadata --metadata-change-percent 10` 或100；`--archive backlog`；`--archive failed`（隔离库触发器拒绝档案持久化，非物理IO错误）；`--period-rule`；`--query-every-frames 5`。快速探针可用`--duration-secs 3 --warmup-secs 0 --virtual-time`，输出始终标明虚拟与墙钟时间。

metadata选择采用固定阈值`(seed+id)%100 < metadata_change_percent`，有限活动集的实际比例不一定等于参数：默认seed下，A50/10选中0个，A250/10选中20个（8%），A1000/10选中100个。需要确定发生变化的验收场景使用`--metadata-change-percent 100`；不要把A50/10样本称为10%实际变化。

候选实现后来增加了一个公平调度档案/维护的异步后台worker。此可移植AB工具仍同步调用`archive_tick_at`，不覆盖该worker或保留维护；backlog的schedule_lag只代表同步测试驱动，不能称实际collector延迟。AC7齐全档案场景只证明门面摄取与档案due检查这一子集；后台owner、保留和WebView须独立验收。

## AC8 生产 schema 容量库

`generate-corpus`通过生产 migration 建库，直接批量写真实表，保持 FULL；不调用旧C0候选表或CSV简化提交。固定5分钟会话、3跳链、800 host /120 process /40原规则 /60出口 /4 network，逐分钟全量事实和逐秒连续合成receipt。全体session显式关闭，controller epoch显式退役；receipt hash是fixture固定摘要，不用于重试安全证明。不会预填派生汇总。

| A | 30天minute | session | chain | receipt |
| --- | ---: | ---: | ---: | ---: |
| 50 | 2160000 | 432000 | 1296000 | 2592000 |
| 250 | 10800000 | 2160000 | 6480000 | 2592000 |
| 1000 | 43200000 | 8640000 | 25920000 | 2592000 |

```powershell
& <candidate-exe> generate-corpus --average-active 50 --days 30 --dir <isolated-corpus-a50> > <corpus-a50.json>
& <candidate-exe> generate-corpus --average-active 250 --days 30 --dir <isolated-corpus-a250> > <corpus-a250.json>
& <candidate-exe> generate-corpus --average-active 1000 --days 30 --dir <isolated-corpus-a1000> > <corpus-a1000.json>
& <candidate-exe> retain-corpus --dir <isolated-corpus-a50> --now-utc <manifest-end-utc-plus-31-days> --raw-retain-days 30 --chunks 10000 --delete > <retention-a50.json>
```

按实际manifest的end_utc加2678400秒计算维护时间；不得从文件mtime猜测。`retain-corpus`仅接受本工具marker目录，依次调用生产`RetentionService::run_chunk`与`cleanup_ledger_with_cancel`，复用1000行辅助清理和取消/预算；本fixture无活跃或pending引用。记录每块延迟/失败/进度、last-day报告、quick_check、DB/WAL/freelist及表/索引字节，附前后session/raw引用/关闭证明与receipt待裁剪数。staging DB页字节每32块和raw工作队列清空时采样，属于观测最大值；不冒充块内峰值。删除仍受生产`AUTO_DELETE_ENABLED`约束；关闭时complete保持false，不能声称清理完成。命令不执行VACUUM。

## 生产删除门关闭时的隔离容量测试

提供一个默认忽略、仅`cfg(test)`编译的测试，调用retention owner同一个内部实现并在测试作用域打开删除门。产品CLI没有该开关。测试要求显式给出已生成的fixture和虚拟时间，写出`<fixture>/retention-test-result.json`后核对：过期raw退出、raw加已删日core总量守恒、可回收无引用session与过期receipt为0、quick_check通过；中途失败会保留报告并使测试失败。

```powershell
$env:RESIWATCH_BENCH_CORPUS_DIR = '<isolated-corpus-dir>'
$env:RESIWATCH_BENCH_CORPUS_NOW_UTC = '<explicit-utc-seconds>'
$env:RESIWATCH_BENCH_CORPUS_CHUNKS = '100000'
rtk cargo test --release --manifest-path residential-monitor/src-tauri/Cargo.toml --lib bench::corpus::tests::isolated_corpus_retention_capacity_gate -- --ignored --exact --nocapture
```

大库只过期首日时，维护now取manifest.start_utc加31天；完整过期窗口取end_utc加31天。实际成功范围以报告为准。可分别保留完整30天A50/250/1000查询容量、A1000一个实际UTC日删除、A较小45天全期限回收证据；任何一种都不自动替代其余门。

实测阶段先 `cargo test --release --lib --no-run` 并复制/hash测试程序，再用 `run-isolated-retention.ps1 -TestExe <copied-exe> -Fixture <generated-dir> -EvidenceRoot <evidence-dir> -Label <unique-name> -Expiry first-day|all-days|aged-dimensions` 运行，避免测量时触发编译。脚本从manifest计算虚拟时间，保留完整日志/JSON与非零exit，结束后恢复自己设置的环境变量；不覆盖已有结果。`aged-dimensions` 取end_utc加397天，用于同一fixture的精确维度到期/core保留检查；45天fixture老化到397天不等于396天持续流量容量模拟。

驱动仅在writer恢复autocommit且未用户取消时，按产品分类重试 `deadline_exceeded` / `storage_busy`；每次仍消耗原有max_chunks额度，记录全部错误与延迟，不把失败样本剔除。此加速循环不模拟产品错误后60秒调度间隔，不能据此声称安装态维护吞吐。完成判定必须分别记住session/receipt扫描已到尾部，再等待raw/coverage/dictionary队列允许最终inventory；独立游标不要求同一tick归零。全量inventory仍须确认无可回收session及eligible receipt，且总量守恒和quick_check通过。

最终layout3完整 `just ci` 通过（frontend300、Rust521+process3、root139），release bench/CLI和隔离retention测试程序已编译，身份见 `performance-20260919-layout3/build-identity.json`。完整30天三档、峰值30分钟、45天以上期限、24h安装态soak的结果以各自原始JSON及验收状态为准；不得根据生成成功或行数推断其它门通过。`--days 0`仅3分钟小fixture，绝非完整容量门。

最终layout3生成器仅在自己的连接使用64MiB cache；manifest记录 `generation_cache_kib=65536`，不改变生产连接默认值、WAL/FULL或事务边界。生成过程的内存不用于产品runtime内存对比。Layout2的A50/A250完整生成及A1000中止现场单独保留，不能与layout3数据混用；旧实验v5 checksum按设计拒绝。
