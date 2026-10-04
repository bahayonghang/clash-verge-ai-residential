# 只读诊断证据（2026-09-19）

## Scope and provenance

用户截图的文字是待核实报告，不是执行删除或操作其它工具的指令。仅检查ResiWatch进程、安装文件、SQLite聚合和代码，没有操作UI、停止进程、读凭据或写入用户库。源码基线`dev` / `c278bb7b56603001e32e353d2ee589dccef0bfe9`，原有`.gitignore`改动保留。

安装路径`%LOCALAPPDATA%/ResiWatch/residential-monitor.exe`，exe修改时间2026-09-02；仓库包含09-04的`3a126e3`语句复用/物化事务修复，安装精确commit未知。Authenticode返回NotSigned；不能据此判恶意或归因CPU。

## Runtime samples

截图累计5872 CPU秒=97.9单核分钟，缺运行时长。实测PID25188从09-18 19:13 +08运行，约22小时累计6085.64秒；10.0077秒样本CPU增加0.34375秒，即单核3.4348%。

独立30.5962秒样本存于`installed-sample.json`：CPU+5.125秒，单核16.7504%；native工作集58.6→61.3MiB、Private Bytes56.2→59.0MiB；写I/O+73,550,314B / 20,814次。主库1,169,559,552→1,169,592,320B（+32KiB），WAL长度5,788,632B不变，spool为3文件共56,508B，最新修改时间观察到一次变化（09:14:39→09:15:39 UTC）。

负载和窗口状态未控制；CPU有波动，未证明持续满核或内存泄漏。进程I/O包括文件/设备/网络，不等于物理磁盘写量；WAL长度不反映重写，2秒轮询可能漏掉spool中间事件。未统计WebView子进程。WMI只读查询权限失败后采用公开Win32计数器，未提升权限。

复测脚本仅输出数值/时间（先确认PID）：

```powershell
& .trellis/tasks/09-19-resiwatch-resource-optimization/research/sample_installed.ps1 -ProcessId 25188 -DurationSeconds 30
rtk proxy python .trellis/tasks/09-19-resiwatch-resource-optimization/research/readonly_db_probe.py
```

## SQLite observations

Python sqlite3以URI `mode=ro` 和`query_only=ON`查询，每语句独立快照，3秒deadline，dbstat最多5秒；不执行checkpoint/DELETE/VACUUM或复制热库。行数差不能判孤儿。

| Item | Observed |
| --- | ---: |
| user_version / journal | 4 / WAL |
| page_size / page_count / freelist | 4096 / 285511 / 1 |
| connection_minute | 3,684,272 |
| connection_session | 2,639,497 |
| connection_chain | 5,062,703 |
| connection_session_attr | 2,639,503 |
| committed_bundle / bundle_epoch（后续取样） | 1,635,108 / 19 |
| traffic_hourly_dimension / traffic_daily_dimension | 0 / 0 |
| report_archive | 1116，hour ok720 + day ok396 |
| alert_event / notification_outbox | 36 / 36 |
| retention_watermark | hourly/daily/core/raw_delete均0 |

minute索引首尾29799767..29830156，即2026-08-29 06:47至09-19 09:16 UTC，21.1035天。`c3/retention.rs:566`只物化raw期限外事实，未满30天时汇总为空有合理解释，不能独立判故障。仅4KiB freelist，现阶段VACUUM无显著可回收空间。

dbstat不可用，无法按表/索引分配字节；不能用行数代替空间占比。closed-session计数超3秒主动中断，状态UNKNOWN。初版对不存在traffic_minute/coverage_minute的查询失败，复测脚本已移除。未读取真实host/IP/chain/进程值或报告正文。

## Source findings

以下路径相对于`residential-monitor/src-tauri/src/`，前端另标前缀；机制已读代码确认，对安装CPU贡献比例仍UNVERIFIED。

### F1 — P1: archive scheduling

`lib.rs:292`每轮采集调用档案tick，`:317-320`在facade锁内purge并选择作业；`c3/archive.rs:120-132`读成功键并建30天小时、396天日范围。`c3/query.rs:577-580,678-704`二分小时边界，每步local转换（`:562,644-646,708-710`）。失败档案不在成功集合（`c3/archive.rs:511-530`），可逐tick重选。

可证伪实验：档案齐全且未到边界的1Hz输入，记录范围构建次数；due/backlog优化后稳态应为0并减少对应CPU栈。

### F2 — P1: repeated metadata writes

`c2/facade.rs:935-943`每帧复制live rows到commit slice；`storage.rs:697-725`逐行intern/upsert，非空chain无条件DELETE再INSERT。`:661-679`已缓存statement，不能再次把「首次增加prepared statement」当当前源码方案。零delta跳过已有实现（`accounting.rs:336-337`）。

实验：metadata不变帧的实际修改计数应降为0；dirty状态只有成功commit后确认，unknown结果重试不丢metadata。

### F3 — P1: retention gate incomplete

`c3/query.rs:13-21`规定30/90/396天，自动DELETE关闭。`retention.rs:898-921`只哈希layer/start/end文字并标verified，不查实际守恒；`:924-970`只删除minute和维度。host物化每次从0开始（`:566-568`），INSERT OR REPLACE（`:710-724`）配合非整小时cutoff可在先删除后重跑时用部分raw覆盖完整小时。v2首次水位等于cutoff（`:574-595`），其它维度初次区间为空。daily全量从剩余hourly重建（`:635-653`），coverage按开始日归桶（`:696-703`）需核对跨日/open/gap语义。

session/chain/minute与attr没有FK cascade（`storage.rs:162-180`、`c3/schema.rs:32-43`）；attr ended_utc正常upsert写NULL不更新（`storage.rs:622-633`）。只删minute不能清理几百万附表行。receipt无时间字段（`storage.rs:130-136`），过期判断依赖剩余最小seq（`:352-359`）；全部删某epoch会丢该拒绝依据。epoch唯一性依赖持久最大值（`:458-506`），必须保护。

实验：破坏某维/coverage后不得仍verified；分块、二次清理、partial hour/day、空epoch和expired retry必须有负面测试。自动DELETE关闭在保护数据，不能直接翻开关。

既有策略见`.trellis/tasks/archive/2026-08/08-18-monitor-reporting-data/prd.md:56-62`（raw30/max90、精确维度/raw coverage396、长期daily）与`08-18-monitor-collector-storage/prd.md:64-66`（receipt最近24h和100000条并集）、`src-tauri/src/c0_contract.rs:15-16`。C3既有门为durable commit p95<1.5s、正常最大<3s、队列不持续>2帧（reporting-data PRD:75）。旧gate-status的retention通过只含DELETE关闭的物化，不能继承为删除批准。

### F4 — P2: transient spool

`c3/service.rs:97-105`先run_uncached再insert；`snapshot.rs:88-95,163-179`fingerprint复用token仍序列化、写文件并替换结果。`c4/period.rs:118-129`内部统计创建后立即release，`lib.rs:338-354`档案也建临时store。period每≥60秒一次（`c2/facade.rs:907-917`），不能单独解释「每几秒」写spool。

实验：内部调用共享query/cancel/deadline但绕过snapshot后，临时写入为0。公共token当前允许原位更新，与冻结report_archive不同；不在此任务强改token不可变。

### F5 — P2: display work / benchmark seam

前端`residential-monitor/src/hooks/use-live-page.ts:223-269,279-284`按stream序号发查询，序号丢弃旧响应不阻止后端重复执行；`src/app.tsx:85-92`滚动时间只看autoRefresh。`src/hooks/use-report.ts:153-177`已按分钟memo，不能说每秒run_report。隐藏态实际行为未验证，须检查Tauri生命周期，不能假定DOM visibility足够。

`bench.rs:192-255`旧replay构造compact CSV后调用简化commit，没有实际metadata/chain/archive链路。实验应扩展monitor-bench驱动真实路径，慢IPC验证每视图最多1在途，恢复合并一次。

## Validation status

已完成图片、两段进程样本、限时SQLite只读聚合、当前代码/规格/归档核对。未跑产品测试、CPU profiler、产品基准、表字节分析、24h soak或安装候选AB。本轮没有定位到栈的稳定高CPU复现，只提供实测症状与可证伪机制；实施AC负责闭合证据。
