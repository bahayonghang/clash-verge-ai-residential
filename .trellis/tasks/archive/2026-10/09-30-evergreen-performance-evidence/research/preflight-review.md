# T05 执行前独立审查与最小测量顺序

日期：2026-09-30。派发目标：本 T05 任务。主会话明确确认研究目标，允许在共享活动任务仍为 T04 时显式读取 T05；本审查未改任务指针。只写本文件，未修改产品、基线副本、其它研究文件或私有数据。未构建、运行测试/压测、打开或 hash corpus 数据库。

## 2026-09-30 勘误：生产探针名称与首次读取边界

本预审遗漏了对可执行测试全名的核对。首次执行器使用 `c3::service::tests::isolated_nonempty_report_stage_proof`；该名称不存在。真实名称是 `c3::service::raw_stage_probe_tests::isolated_nonempty_report_stage_proof`。2026-09-30 13:49:49 UTC 的首次命令退出 0，但实际运行 0 tests，且没有生成 JSON。`stages-20260930-initial/production-first-validation.json` 正确记录 `pass=false`；原命令、日志及 FAIL 收据必须保留。

之后的 `a250-minute-all` 等阶段探针已经读取 corpus。本轮未经主动预热的生产 reader 机会已经丢失。补测必须标注“此前阶段已读取数据库”；进程内首次 `run_uncached` 不能证明本轮首次数据库读取或物理冷缓存。不得清除缓存、覆盖旧结果或用补测替换首轮失败。

后续执行器先用同一个已核验哈希的测试程序运行 `--list`，精确核对名称；执行后同时要求退出 0、运行且通过恰好 1 test、JSON 存在，以及下文的字段和 oracle 校验。任何一项失败都保留原始收据并标 FAIL。以下是更正后的命令示例，仅供执行器采用；本审查未运行：

```powershell
$TestName = 'c3::service::raw_stage_probe_tests::isolated_nonempty_report_stage_proof'
$TestList = & $TestExe --list
$ListExit = $LASTEXITCODE
if ($ListExit -ne 0 -or $TestList -notcontains ($TestName + ': test')) {
    throw '测试列表中没有精确名称；不得读取 corpus'
}
if (Test-Path -LiteralPath $ProbeJson) { throw '输出路径已存在；必须使用新文件' }
$env:RESIWATCH_NONEMPTY_STAGE_DB = $ApprovedSyntheticDb
$env:RESIWATCH_NONEMPTY_STAGE_OUT = $ProbeJson
$env:RESIWATCH_NONEMPTY_STAGE_START = '1787184000'
$env:RESIWATCH_NONEMPTY_STAGE_END = '1787184060'
$NativeOutput = & $TestExe --exact $TestName --ignored --nocapture --test-threads=1 2>&1
$NativeExit = $LASTEXITCODE
# 执行器另存原始 stdout/stderr、退出码、参数、环境和时间；失败也保留。
$NativeText = $NativeOutput -join [Environment]::NewLine
if ($NativeExit -ne 0 -or
    $NativeText -notmatch '(?m)^running 1 test\r?$' -or
    $NativeText -notmatch '(?m)^test result: ok\. 1 passed; 0 failed; 0 ignored;' -or
    -not (Test-Path -LiteralPath $ProbeJson -PathType Leaf)) {
    throw '探针没有完成恰好一个测试或 JSON 缺失；保留 FAIL'
}
# 继续执行下文的一分钟 JSON 语义检查，不能在此宣布生产门通过。
```

下文未注明更新的预审结论保留执行前时点；新的构建与阶段结论另见 `review.md`。

## 结论

可以在主会话取得独占负载时段后，继续已批准的隔离基线接入、构建、smoke 和候选阶段测量。静态审查未发现五文件接入需要修改 baseline 生产算法的依据；可编译性仍须由实际 build 和 smoke 验证。当前证据不足以指定长窗口算法修复，也不足以关闭 AC7/AC8。

主会话新增的 `source-subtree-archive.json` 解决了历史归档身份问题：独立读取并计算该 `residential-monitor/` tar 的 SHA256，结果为 `87684724D5E5B1B3A18C0212CD594CA05F9454EDD6C28582CD879270F9749EB7`，与原 benchmark-harness 相同。完整仓库 tar 的另一哈希仍保留。源码身份已确认，尚无本轮 baseline 可执行文件身份。

本次读取了 T05 PRD/design/implement/check.jsonl、measurement-prerequisites、原 09-19 benchmark-harness/运行驱动/PRD、09-24 PRD、backend 及 SQLite 规范、指定 bench/raw-fold/service/sql 源码。沿用已经读取的共享 AGENTS/workflow/审计与领域说明。

## 必须保持的正式参数

| 门 | 固定输入与判定 |
| --- | --- |
| AC7 主场景 | baseline/candidate 各 3 轮；A250、1Hz、counters、complete、query_every_frames=0；每轮实时 warmup 30 秒 + measurement 300 秒。使用原驱动明确传入 metadata_change_percent=100，seed=20260919、start_utc=1800001800；counters 的 metadata 实际不变。CPU 秒下降至少 30%，归属全部应用文件写入下降至少 50%。禁止 virtual-time。 |
| 11 对矩阵 | A50/250/1000 × unchanged/counters/metadata，加 A250 metadata backlog/failed。1Hz、5 秒预热 + 30 秒实测、每 5 帧查询、metadata_change_percent=100、相同 seed/start。backlog/failed 开 period-rule；complete 不开。 |
| F1–F5 | 对应场景 ingest p95 candidate/baseline ≤1.10。 |
| F6–F7 | A1000 metadata/unchanged 的 native private p95 比值 ≤1.10。 |
| F8–F10 | A50 metadata、A250 failed、A1000 metadata 的 SQLite xWrite 比值 ≤1.10。 |
| F11 | A250 backlog CPU 比值按原 AC4 的同一规则保留 ≤1.10。CPU 基线为 0 时比值无法定义，保留计数分辨率与未通过状态，不用零或无限精度代替。 |
| F12 | 原 11 对 first-reader p95 比值 ≤1.10；first/repeated reader 分开，空/非空窗口分开。空窗口失败保留，不据空窗口比值单独修改 SQL。 |
| 完整容量 | A50/A250/A1000 保留完整 30 天、seed=20260919、窗口 `[1787184000,1789776000)`；A250 host/top20、network/top100 各 21 次，A1000 host/top20 21 次，逐次 exit 0 且 wall≤10000ms。A50 与原 1d/30d 组合分别保留。CLI rank 默认 residential=true、显式 tz=UTC。 |
| AC8 独立门 | 10k/1Hz/counters/complete，30 秒预热 + 1800 秒实测；24h 安装态、native+WebView private p95≤基线110%、真实后台 worker/collector 队列分别验收。保留 durable commit p95<1.5s、正常 max<3s、队列不持续>2帧及页面2秒/报告10秒。 |

参数依据：原 `run-isolated-bench.ps1:16–91`、09-19 PRD AC7/AC8、09-24 PRD R2/AC1–AC6。矩阵每侧每场景产生 6 次 first reader 和 6 次 repeated reader；不改变样本数或分位计算来降低失败比值。主场景与矩阵最低实时回放共 2750 秒（45 分 50 秒）；峰值两侧另需至少 3660 秒（61 分钟）。这些时长不含构建、初始化、容量或归属测量。

当前 T05 PRD AC4 已补入 F8–F11 的 xWrite/CPU 条款。`measurement-prerequisites.md` 中“AC4 遗漏”的记录属于较早状态，本审查不把该项继续列为当前遗漏。

## 五文件基线接入审查

- 重新在内存中应用既有变换：`bench.rs` 只新增 facade/process/write_vfs 模块和 cfg(test) 独立测试 helper；`bin/monitor-bench.rs` 只新增 facade import、ReplayFacade enum/arm。没有写入隔离副本。
- 两份变换输出分别为 16564 B / `499F3B905DBA9E48C1D9840F4FC57803B03B3CDB6E3B3C2ECD40006684023EA8`，11870 B / `7BA08896AC884DD764B1AD954DC3B579CDB38ADADF272A77D8CC4422FE7A3BFC`，与前置研究一致。另外三份模块哈希也一致。
- baseline 的 `AppFacade::boot/ingest_snapshot/query/save_targets/upsert_alert_rule`、public storage/hub/snapshots/bundle_seq、`archive_tick_at`、`run_uncached`、`ReportArchiveService::next_job/persist_outcome` 均有当前 instrumentation 所用的接入点。Cargo.toml/lock、build.rs、tauri.conf.json 经换行归一化后相同。
- v2 `write_vfs.rs:193–198` 保留内外 pMethods 初始化。不要回用旧 v1 instrumentation。原始 baseline 不加入 corpus module、GenerateCorpus/RetainCorpus 或候选 retention 实现。
- 静态比较未覆盖 Rust 类型检查、Windows linker/SDK、Tauri dist 嵌入或 Win32/VFS 运行结果。构建失败先保留原始错误；仅修正隔离接入的可证明问题。若需要改 baseline 生产方法、依赖、迁移或锁文件，停止并提交范围/比较有效性审查。
- candidate 不能只用 HEAD 作为身份：记录未提交源码 manifest、构建参数、工具链和独立 exe/test-exe 哈希。同一 instrumentation 若增加测量字段，必须同步进入两侧并重建。复制 dist 只满足嵌入前提，不构成 UI 同源证据。

## 三个需要显式判读的证据边界

1. `service.rs:1803–1818` 捕获生产首读错误后继续诊断；`:1853–1890` 只要求后续分段得到非空连接并输出 JSON。因此 `isolated_nonempty_report_stage_proof` 可以在生产首读失败时仍退出 0。执行器必须同时检查 JSON 的 `production_first_read.status`、首读 wall、精确 totals 和非空条件。
2. `raw_fold.rs:1206–1224` 的 all 使用 default-auto query：local 时区、previous_equal_window=true；residential 使用 UTC、comparison=None。因此两个探针不只相差过滤器。分别记录 projection_start/end、previous_scan_ms、查询定义，不能把总耗时差直接归因于 residential 过滤。raw-fold 诊断不启用生产 10 秒 deadline；无预算拆分成功不等于报告门通过。
3. `facade.rs:193–205` 只检查报告字节非负，fixture_hash 在 `:263–268` 只覆盖输入参数。原矩阵的相同 hash 不证明报告非空或等价。F12 原 11 对继续保留；额外非空对照要固定同一窗口并输出 totals/连接数/series 摘要。可先审查现有 `--start-utc` 参数构造跨分钟输入的方案，保持原规模与时长；实际非空与等价以结果确认，不能预先宣称。

### 非空一分钟生产首读的短检查示例

以下仅读取已经生成的 JSON；本轮未运行。适用于已核对身份的 A250 30 天合成库、原 default-auto 一分钟窗口。不同窗口或过滤器不得套用该固定 oracle。native test 的原始退出码仍须由调用方单独保存。

```powershell
$r = Get-Content -Raw -LiteralPath $ProbeJson | ConvertFrom-Json
if ($r.kind -ne 'nonempty-minute-report-stages' -or
    $r.query -ne 'default_auto_report_query' -or
    $r.range_start_utc -ne 1787184000 -or $r.range_end_utc -ne 1787184060) {
    throw '探针身份或窗口不匹配'
}
$first = $r.production_first_read
if ($first.status -ne 'ok' -or $first.tier -ne 'Raw') {
    throw '生产首读失败或查询层不匹配；保留 JSON 和原始退出码'
}
$ms = $r.production_first_read_ms
if ($null -eq $ms -or [double]::IsNaN([double]$ms) -or
    [double]::IsInfinity([double]$ms) -or $ms -lt 0 -or $ms -ge 10000) {
    throw '生产首读耗时无效或触达 10 秒 deadline'
}
foreach ($field in @{connection_count=250; upload=3497; download=11725}.GetEnumerator()) {
    if ($first.($field.Key) -ne $field.Value -or
        $r.('stage_' + $field.Key) -ne $field.Value) {
        throw ('一分钟 oracle 不一致：' + $field.Key)
    }
}
if ($first.series_rows -le 0 -or $r.stage_series_rows -le 0) {
    throw '未形成非空 series 证据'
}
if ($ms -ge 7690.685) { throw '未达到原非空分钟门的改善要求' }
```

该检查验证固定一分钟摘要，不替代 grouped SQL oracle、各维排序/缺失归因、previous-window、coverage、取消或整份 deadline 回归。检查失败须停止将本样本标为 PASS；后续阶段诊断可继续，但单独标为诊断。任何重新运行使用新输出文件，保留首个失败。

## 最小可执行顺序

1. 主会话结束 T04 构建和其它本任务重负载，启动 T05；不停止用户程序。核对独占时段、空间、绝对路径及工具链。只在新隔离输出目录重建 c278bb7 + 五文件 instrumentation；candidate 单独构建，使用 locked release 及已规定 x86_64-pc-windows-msvc target。先一次性生成并复制/hash candidate library-test executable，正式测量期间不再隐式 Cargo 编译。
2. 对 baseline/candidate 各运行原 A8/3 tick virtual smoke，核对守恒、提交数、fixture_hash、WAL/FULL与退出码。该 smoke 只决定 instrumentation 是否可继续；失败不修改正式规模或阈值。
3. 在读取 corpus 前核对 marker/manifest 和规范化绝对路径。现有 probe 依赖环境路径，未替执行器验证合成目录身份。数据库 hash、复制、完整性检查都会影响页缓存：若先执行这些检查，后续明确标“已读取页缓存”；若保留未经本轮主动预热的首次读取，先记录正式生产 reader，再做内容校验和拆分。两种程序都没有物理冷缓存证明，不使用缓存清理或重启来补证。
4. 使用复制的测试程序运行固定一分钟 production-first JSON 检查，再分别运行 A250 一分钟/30天的 all 与 residential 阶段，记录 projection、current scan、previous scan、reader/coverage 以及查询差异。保留真实 now 与 `now=end` 探针的区别。阶段已有页缓存影响必须写明。
5. 强模型依据新阶段和同窗口指标确定是否存在范围内的最小候选。先跑现有 grouped-oracle、跨窗口、取消/deadline 回归；同一 reader snapshot 内校验所有维度和过滤，不允许变更核算定义。新候选源变动后重新冻结身份。没有阶段依据时不先修改算法。
6. 串行运行原 primary、matrix。原 runner 可复用，但需保存每条 native command 的退出码/时间与 JSON，不能只依赖最后命令状态。F12 非空对照与空窗口分别判断。AC7 全应用写入缺口不妨碍收集 CPU/SQLite 子项；子项通过不能把 AC7 总项改成 PASS。
7. 阶段计划支持后再生成 A1000 完整 30 天，固定已有 seed/start；不以四倍外推作为实测。按原规模/窗口回归 A50/A250，完成 A250 host/network 及 A1000 host 各 21 次。旧 `run-capacity-queries.ps1:10–31` 的 host 两次失败就终止、network 仅一次；可以保留失败早停诊断，不能据旧驱动退出或少量样本宣称 21 次门通过。新的正式计数驱动只能改本任务 research 文件。
8. 完整回归与 `just ci` 结束后，将已验证合同写回既定 storage spec，记录 PASS/FAIL/UNVERIFIED。保留未运行的 AC8 门；不提交或归档任务。

## 阶段证据后可审查的最小算法候选

| 观察到的主导阶段 | 可在现有文件内提出的候选 | 必须保持的验证 |
| --- | --- | --- |
| projection/分配占主导 | 在 `raw_fold.rs` 复用字典 identity、chain/rule 解析结果，减少相同值的重复字符串/哈希工作；必要时只调整 `sql.rs` 的等价投影。 | 全维字段缺失、legacy residential EXISTS、chain/rule/exit tie 与 filter oracle；内存/CPU前后证据；不能只按 host 场景省去其它必需字段。 |
| minute scan 占主导 | 在 `raw_fold.rs` 预计算 session-constant 的过滤/分组信息，减少每分钟重复运算；继续单次索引扫描、精确 distinct 和有界标记。 | 稀疏 session_pk、负分钟、零字节、跨窗口 session、空桶、省略/排序及全部过滤；取消和整份预算不重置。 |
| 空 previous window 仍重复大量工作 | 先证明可等价减少的分配或派生计算，再在 `service.rs/raw_fold.rs` 提出最小修改。 | previous unavailable 保持 absent，空 previous 保持正确零值；不以“无数据”推断删除或 coverage；当前/previous 共用 snapshot 与10秒预算。 |
| backlog/failed 调度或 writer 占主导 | 保存归因并提交范围变更。 | 调度器、facade生产摄取、storage写入不在本任务算法文件范围；当前不修改。 |

`raw_fold.rs:117–131` 的 3120 分钟阈值与 `sql.rs:23–35` 的长窗口全表投影是明确合同。没有 query-plan/阶段证据时，不直接扩大阈值到 30 天，不增加索引或物化层。`service.rs:160–203` 的整体 deadline 和取消由报告 owner 保持；`fill_raw:818–856` 的 previous/current/coverage 不得分别重新计时放宽。

## 确定阻断与重新批准边界

- **证据阻断**：尚无本轮 baseline/candidate build/smoke 身份、当前 corpus 内容验证、A1000 完整测量、同窗口 F1–F12/AC7、物理冷缓存证据。各项必须保持未通过；不把历史 JSON、发现的二进制、API 静态匹配或外推当作通过。
- **AC7 归属缺口**：`facade.rs:294–296` 的 all_application_file_write_bytes/spool_write_bytes 仍为 null；`process.rs:56–79` 只有进程 I/O，`write_vfs.rs:199–208,251–266` 只有指定目录内成功 SQLite xWrite。WPR 可执行文件存在不关闭该缺口。新增时间边界或局部计量字段可在批准的 bench 文件内提交审查，并同步两侧；全系统事件采集、权限提升或包含其它进程路径的数据采集须先说明范围并取得额外授权。
- **范围变更**：新增/修改 schema、迁移、索引、物化表、调度器、生产 storage/c2 facade/lib/query/dbcli 等未列文件，或改变报表/核算/保留期语义，均需停止候选修改并回交主会话取得新批准。隔离 baseline 的 bench.rs/CLI 接入仅按已批准五文件计划，不扩展为当前产品入口改造。
- **门变更**：10秒/2秒 deadline、1.10比值、AC7下降幅度、正式规模/轮数/seed/窗口/缓存条件、WAL/FULL、AUTO_DELETE_ENABLED=false 不得为通过而调整。需要任何变更时先停止并取得明确批准。
- **安装与外部动作**：真实控制器/凭据/用户数据库、安装/替换/重启、24h安装态、真实WebView/worker、缓存驱逐、停止用户程序、全局设置、提交/归档/push/PR/远端workflow不在本次默认实施授权内。

本预审只证明上述静态来源、参数和接口核对。没有声称新的 lint、typecheck、测试、性能门或客户端运行已经通过。
