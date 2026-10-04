# T05 新身份重建阶段审查与执行合同

日期：2026-10-02，America/Chicago。状态：REVIEWED_STATIC_PLAN；等待主会话审查具体执行合同。没有构建、生成语料、打开数据库或执行性能负载。本轮仅新增本文件及同名 observations.json；共享指针仍为 T04，主会话明确允许显式读取并写入 T05 research。

## 授权与结论

原 `implement.md:8` 明示从源码重建 baseline/candidate；`:11` 明示严格复跑同窗口矩阵、AC7 实时三轮和完整容量。`design.md:38–40` 要求回退后重新审查新身份、固定输入、隔离路径及资源，并明确旧六次 primary 不能迁移到回退源码。本合同落实原批准范围，不要求用户重复许可。主会话审核合同并派发后，按阶段 preflight 执行；静态方案和工具可用性不能填为 build/generate/正式性能 PASS。

执行顺序为：T06 关闭、T04 与其它重负载结束 → 路径/工具/源码准备 → 独立构建与接入 smoke → A50/A250 完整语料与独立 oracle → matrix 22 次 → primary 6 次 → capacity 106 次 → 独立终审。所有测量串行。没有新算法、schema、SQL、deadline、种子或阈值改变。A1000 完整语料不生成，原容量门保留 NOT_RUN；matrix 中 A1000 的 35 秒实时场景仍按原参数执行。

`prd.md:22–23` 的 AC4/AC5 是新候选当前验收所需的 matrix/primary 原文依据。容量可分别记录自己的结果；不把 matrix 性能全 PASS、全部应用文件计量全 PASS 或物理冷缓存证明新增为 106 次启动门。历史失败和 UNVERIFIED 不因新批次启动而销项。

## 固定路径及输入身份

主会话审查后使用以下唯一新根；本次只确认路径尚不存在，没有创建。执行前重新 resolve、检查父路径及 reparse 属性，并确认子路径仍在指定根内。已有目录或文件时停止，不覆盖、不删除、不就地接续；另定唯一根并重新绑定。

| 名称 | 绝对位置或根内位置 |
| --- | --- |
| Repository | `D:/Documents/Code/Github/clash-verge-ai-residential` |
| AssetRoot | `<Repository>/bench-data/t05-rebuild-20261002-4f8a8c68` |
| EvidenceRoot | `<Repository>/.trellis/tasks/09-30-evergreen-performance-evidence/research/rebuild-20261002-4f8a8c68` |
| 源码 | `<AssetRoot>/sources/baseline`、`<AssetRoot>/sources/candidate` |
| 构建输出 | `<AssetRoot>/targets/baseline`、`<AssetRoot>/targets/candidate` |
| 复制程序 | `<AssetRoot>/executables` |
| 语料 | `<AssetRoot>/corpus-a50-30d`、`<AssetRoot>/corpus-a250-30d` |
| 回放数据 | `<AssetRoot>/data/smoke`、`data/matrix`、`data/primary`，每个 case 新空目录 |
| 新状态/身份/驱动 | `<EvidenceRoot>/state.json`、`inputs/`、`runners/`、`build/`、`generator/`、`oracle/`、`matrix/`、`primary/`、`capacity/` |

`git check-ignore -v --no-index bench-data/t05-rebuild-20261002-review-path` 已返回 `.gitignore:45:bench-data/`。资产根是稳定 ignored 路径，不使用 Temp 保存唯一资产。研究收据和资产各有 manifest，程序、库和 marker 在该根保留；不自动清理。仅复制具名公开源码及公开 dist，不复制安装库、local 配置、四覆盖、凭据、原生历史、target 或 node_modules。

本轮 HEAD 观测为 `d3a25b4164343f5cbeab8a51efe1a1d3974cdf4a`，仅作版本背景。当前 rollback 的 `raw_fold.rs` SHA256 为 `4D6E8FEB88B142E09F8E62D60BAF20BAC970F11CB8D8E9108B08CFFBF717E6AA`。旧 rollback 108 文件 manifest SHA256 为 `8EE21DEACDFE15166BC97A92D78E24445EF26AC90D4813FB30EB368C180996CA`；本轮核对 manifest 文件、raw_fold 和五个接入文件，没有重新哈希全部 108 文件。执行者复制前必须逐项核对实际源码，不能只写 HEAD。

## P0：构建与生成前置

1. 主会话确认 T06 `CLOSED_NO_FURTHER_CLIENTS`，T04 初始化/检查及其它本任务重负载结束。只记录竞争负载，不停止用户程序。构建、生成、oracle、性能回放不得互相重叠；正式测量时不隐式运行 Cargo。
2. 本轮 UTC `2026-10-02T14:08:12.4370301Z` 实测 `rustc +1.98.0 -vV`、`cargo +1.98.0 -vV`、`rustup target list --installed --toolchain 1.98.0` 均退出 0。rustc 为 `88d9e12ae`，cargo 为 `797e8a9bc`，host/已安装 target 为 `x86_64-pc-windows-msvc`，LLVM 22.1.8。命令环境仅本进程 `RUSTUP_AUTO_INSTALL=0`。
3. `vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath` 返回 `C:/Program Files/Microsoft Visual Studio/18/Enterprise`；SDK `C:/Program Files (x86)/Windows Kits/10/Lib/10.0.26100.0` 存在。这里只证明元数据可用。执行前记录实际选用 linker/SDK、构建环境及 Cargo config 影响，不宣称链接成功。缺少工具链、target、linker/SDK 或 locked 依赖时保留 BLOCKED/原错误，不安装全局工具、不改锁文件。
4. 本轮 D free 为 `1209097580544 B`，C free 为 `537873358848 B`。构建和每次生成前重录目标盘 freebytes。历史 A50/A250 主库合计 `2242162688 B` 仅是规模参考，还需容纳两套 source/target、dist、exe、WAL/SHM/索引/临时文件和日志。峰值及耗时未知；不据历史主库大小保证峰值，不清磁盘、不缩短语料。记录每次生成实际 native/progress 资源与失败时已提交小时。
5. 新 state 固定所有绝对路径、baseline revision、source/dist manifest、instrumentation hash、工具链/target、环境覆盖、tz/offset、各 runner hash、原参数和 case 数。builder preflight 通过才构建；generator preflight 通过才生成。preflight 状态与运行结果分别保存。

## P1：独立源码构造与 build

Baseline revision 固定 `c278bb7b56603001e32e353d2ee589dccef0bfe9`；本轮 `git cat-file -t` 已确认对象为 commit。在新资产根执行具名 `git archive` 导出该 revision 的 `residential-monitor/`，保存 tar hash 与解包前后源清单，不覆盖旧导出或工作树。先保存 original-source manifest，再只在隔离 baseline 接入以下五文件：

| baseline 接入文件 | 构造及期望 SHA256 |
| --- | --- |
| `src/bench.rs` | 按既有转换保留 facade/process/write_vfs 及 cfg(test) helper，去掉 candidate corpus 模块；`499F3B905DBA9E48C1D9840F4FC57803B03B3CDB6E3B3C2ECD40006684023EA8` |
| `src/bin/monitor-bench.rs` | 按既有转换保留 ReplayFacade import/enum/arm，不加入 GenerateCorpus/RetainCorpus；`7BA08896AC884DD764B1AD954DC3B579CDB38ADADF272A77D8CC4422FE7A3BFC` |
| `src/bench/facade.rs` | 复制当前已审双边 writer PRAGMA instrumentation；`ED069D6B822732E3E8B91E1B3ED4A0C9941C676D5415DE35A220ED443009AB4A` |
| `src/bench/process.rs` | 两侧原字节相同；`83734ABF3F95C1BC7313EF9ACEB7BC20E6C5799D416C48CA3723BF9AA9E906EA` |
| `src/bench/write_vfs.rs` | 两侧原字节相同，保留 v2 xOpen 失败初始化；`C3B5D8849E3EA206C036515D86E9E7D77E75399D037EFEBBEC15685D6E48DA7A` |

路径前缀均为 `residential-monitor/src-tauri/`。转换机制在 `measurement-prerequisites.md` 的五文件接入；实际 instrumentation 以 `candidate-retained-20260930/baseline-source-before.json` 和当前源为准。该 retained manifest SHA256 本轮为 `1120B9D2B5C457F856BCD5F51ED9C88E0612E978AB4C7B76B4816640E1C6678F`，103 项。较早 baseline-build 中 facade 的 `278C54...` 已被后续双边 PRAGMA 接入更新，本轮不得误用较早 facade 配对当前 `ED069D...`。保存新 patch 和五项实测 hash；baseline 生产实现、Cargo/lock、schema 不改。历史 baseline schema4、candidate schema5 各自保持，不将两侧 schema 人为统一。

Candidate 从当前已检查 rollback 的 108 项公开清单逐项复制到独立 source；覆盖全部 Rust、Cargo.toml/Cargo.lock、build.rs、tauri.conf、capabilities、icons/资源、tests 等原 build 输入。保存 repo copy-before、source copy-after、build-before/build-after manifest，核对 byte size/hash。raw_fold 必须匹配本合同 4D6E8...；Process/Network 修复及全部 oracle 保留。若源码漂移，停止使用本绑定并由主会话核对新范围，不通过覆盖源修复身份。

Tauri `frontendDist=../dist`。原 108/103 项 Rust/build manifest 不包含 dist；另保存现有公开 `residential-monitor/dist/` 的逐文件 size/hash 和复制前后 manifest，并向两套源码复制相同 dist。旧记录为 36 个 dist 文件，实际新数量与 hash 以当前副本为准。复制仅满足 release 嵌入前提，不声明 frontend 同源或运行通过；本合同不另行构建前端。

以下变量来自新 state 的已核对绝对路径。每条保存 argv/cwd/env/native exit/raw stdout/stderr/hash/UTC/墙钟，失败不能被后一步覆盖：

```powershell
cargo +1.98.0 build --locked --release --target x86_64-pc-windows-msvc --manifest-path "$BaselineSource/residential-monitor/src-tauri/Cargo.toml" --target-dir $BaselineTarget --bin monitor-bench
cargo +1.98.0 build --locked --release --target x86_64-pc-windows-msvc --manifest-path "$CandidateSource/residential-monitor/src-tauri/Cargo.toml" --target-dir $CandidateTarget --bin monitor-bench --bin monitor-db
cargo +1.98.0 test --locked --release --target x86_64-pc-windows-msvc --manifest-path "$CandidateSource/residential-monitor/src-tauri/Cargo.toml" --target-dir $CandidateTarget --lib --no-run --message-format=json
```

仅进程环境设置 `RUSTUP_AUTO_INSTALL=0`，两侧记录同一构建环境，不改变全局。最后一步只编译 release test executable；从实际 compiler-artifact JSON 取唯一 library test 路径，不能猜文件名。复制/hash baseline-monitor-bench.exe、candidate-monitor-bench.exe、candidate-monitor-db.exe、candidate-library-tests.exe，绑定新 source/dist/toolchain/argv；原二进制 hash 不作为新 build 的预期值。构建后核对源和 dist 未变。

两侧沿原 A8/3 tick、counters/complete、hz1、seed20260919、start1800001800、warmup0、query0、metadata100 的 virtual smoke 验证接入，核对每侧 exit0、3 commits/frames、traffic.conserved、相同 fixture_hash、实际 writer 初末 WAL/FULL。该 smoke 不替代任一实时正式门。已有同源码的 fulljustci 收据保持当前局部检查证据，不把 debug test exe 当新 release exe；无新产品改动或新失败时不重复完整 justci。

## P2：A50/A250 完整生成、身份与 oracle

generator preflight 先确认新 candidate-monitor-bench hash/源/CLI 路线、fresh empty corpus 目录、目标盘空间和没有竞争负载。生成命令必须显式写 start，不能使用 CLI 的 `1800000000` 缺省值：

```powershell
& $CandidateBench generate-corpus --average-active 50 --days 30 --seed 20260919 --start-utc 1787184000 --dir $CorpusA50
& $CandidateBench generate-corpus --average-active 250 --days 30 --seed 20260919 --start-utc 1787184000 --dir $CorpusA250
```

每个 fresh 目录独立生成并保存 native exit/raw/progress/墙钟、native_before/after、实际 marker 与 DB manifest。固定 end 为 `1789776000`；固定 1Hz、5 分钟 session、3 跳 chain、800 host/120 process/40 rule/60 chain/4 network、nonzero_minute_ratio=1、generation_cache_kib=65536。cache 数字不等于总内存上限。不得 days0、retain/delete/VACUUM，不更新 generator、schema 或分布。

| A | minutes | sessions | chains | receipts |
| --- | ---: | ---: | ---: | ---: |
| 50 | 2160000 | 432000 | 1296000 | 2592000 |
| 250 | 10800000 | 2160000 | 6480000 | 2592000 |

核对 `kind=production-corpus`、workload、start/end、spec_hash、full_30_day_input=true、counts_match=true、expected/actual、schema5/layout3 `ledger-lifecycle-v5-layout3`、生成连接 FULL、journal WAL、零残留 WAL和 quick_check。新 marker 包含运行时间/进程/资源字段，新 marker/DB hash 必须实测，不能填写旧 AA4A.../667A.../e6fd...。不要因物理 byte size不同拒绝语义相同的新库；必须满足固定完整输入和实际计数。

仅以已绑定合成 DB 的 `mode=ro`、`pragma query_only=on`、同一 deferred snapshot 执行独立 SQL oracle。保留 `corpus-a250-verification.json` 中原 SQL 机制、排序及缺失语义，重新计算 A50/A250 的 host30d/top20、host1d/top20、network30d/top100 完整结果并保存原 SQL/params/rows；30d `[1787184000,1789776000)`，1d `[1789689600,1789776000)`。原 oracle 文件本轮 SHA256 为 `684240C5352CB3182D8B066F987ED5664787901FB9C842F5E4E065B1D1CC64EB`。旧 A250 同输入精确结果可作额外语义对照，不复用旧 DB hash。

保留 all/residential totals/distinct/minute coverage 的独立 SQL 结果，grouped 字段/排序/unknown、zeroFlow、dataVersion、整数类型逐项与 native rank 对照。查询前后 DB hash一致及零 WAL分别记录；完整性/hash/oracle 已触页缓存，后续 first process 不称物理冷页。

已有一分钟/30d非空生产 probe 仍属原未完成证据范围。使用新 test exe `--list` 核对 `c3::service::raw_stage_probe_tests::isolated_nonempty_report_stage_proof` 与 `c3::raw_fold::tests::isolated_raw_fold_stage_proof` 精确名称。运行时只设置新合成库/新输出及原 START/END/FILTER，要求真实运行恰好1 test和JSON存在，并分别检查 production.status=ok、Raw、原deadline、totals/非空series；test exit0不能覆盖 production error。诊断与正式 capacity 分别记账，不增加性能样本或改矩阵窗口；原 F12 非空同窗口门缺口不会因单次 probe 自动关闭。

## P3：新 runner 的绑定与静态准入

不编辑 `run-formal-replay.py`、`run-retained-capacity.py`、旧 `build-20260930/state.json`、旧 frozen identity、原 recovery wrapper 或任何旧 native 收据。原正式 runner SHA256 为 `6C88ACCA5994C02266F262FBC185EBF9D09DE1AB9EC96BAA302993A0CBF381E9`；原 capacity runner 为 `9FA8C225BEF9A9F7718676D7A5E2D00B1E174270784C99094ED5DCC2F8D9FAA4`。

复制机制到新 `<EvidenceRoot>/runners/`，新文件独立 hash/patch/审查。旧 replay 的 ROOT、BUILD、STATE[repo/baseline_source/run_root]、candidate-directory、OUT/DATA及 source label 都依赖旧树；旧 capacity 的 OUT=`retained-capacity-20260930`、CORPUS=旧Temp、FROZEN=`candidate-retained-20260930`、PREVIOUS旧DB hash、两旧 marker hash、build state 都必须改成新 state 的明确绑定。只改新副本的路径/身份/fixture绑定和错误收据，保留验证和计量语义。不能只传新 output-directory 继续使用旧 source/exe。

新 execution-contract 明确新exe/source/dist manifests、generator source/hash/argv、两新 marker/DB hash、oracle收据及driver hash。固定真实 now、UTC 与 process wall；不把 generatedUtc设为fixture end。每阶段 preflight 确认输出不存在、源/复制exe与构建绑定相同、对应两侧 fixture/options/offset一致、argv全字段及预期case数。新 validator 可做固定合成收据检查，真实 probe/计量仍 NOT_RUN。

所有 native 执行保留 stdout/stderr 原字节压缩、原字节 SHA256、标准化可读日志、JSON/result、argv/env/PID/UTC、wall、exit/validation；driver/预期 outer/实际 tool outer分别保存。原日志不重写，解析错误/中断保留partial count和未启动样本，不能混批补足正式样本。实际 outer未知时写UNKNOWN，不猜测。

## P4：matrix 22 次

按新 replay 副本 `--set matrix` 执行 11 对，case与variant顺序保持原循环：A50、A250、A1000分别 unchanged/counters/metadata，随后 A250 metadata backlog、failed；每case baseline→candidate。每侧 hz1、duration30、warmup5、query_every_frames5、metadata100、seed20260919、start1800001800、virtual_time=false；complete不开period-rule，backlog/failed开。每侧每case ingest/native memory/raw sample 30，first/repeated reader各6，不以小样本替代。

保持 F1–F5 ingest p95、F6–F7 native private p95、F8–F10 SQLite xWrite、F11 CPU的candidate/baseline比值≤1.10；F12每个原case first-reader p95≤1.10。源/exe/options/fixture/offset/平台、实际WAL/FULL、frames/commits/守恒与全部样本先核对，再解释性能。原窗口空读按原样保留，非空首读独立；不延长窗口或换参数消除失败。

无效/缺失/零分母指标保留UNVERIFIED；原始 before/after/delta照录，原因未查明时不归因计数分辨率。普通性能FAIL保留全固定批次与结果；计量输入、身份或执行完整性失败须先处理，不能对无效结果计算PASS。当前源码已完成缓存回退；本合同不因新FAIL自动实施第二种算法。

## P5：primary 6 次

matrix 22次完成并核对收据后，运行新 replay 副本 `--set primary`，顺序 r1-baseline、r1-candidate、r2-baseline、r2-candidate、r3-baseline、r3-candidate。各轮新空目录，A250/hz1/counters/complete、metadata100、query0、seed20260919、start1800001800、warmup30、measurement300、virtual_time=false。每次300个 ingest/raw/native memory样本、300frames/commits，真实预热/测量与初始化分别记录。

三轮各侧总 CPU 秒比值≤0.70；全部应用文件写入比值≤0.50，SQLite子集比值≤0.50单列。null不改0，不新增系统trace，不以SQLite子集替代全部应用文件门。CPU异常保存真实native_before/after及样本，原因未查明；不得迁移旧23.8125→2.828125或旧SQLite PASS。完整六次执行和技术验收核对后进入106容量；all_application缺测的UNVERIFIED不新增成容量禁止条件。

## P6：capacity 106 次

新 capacity 副本先核对新candidate-monitor-db/source与两库/marker/oracle锁定，重复记录DB前置 hash/quick_check/WAL及缓存边界，再按原case顺序串行：

| 顺序 | case | native 次数 |
| --- | --- | ---: |
| 1 | A50 host30d/top20 | 21 |
| 2 | A50 host1d/top20 | 21 |
| 3 | A50 network30d/top100 | 1，原诊断 |
| 4 | A250 host30d/top20 | 21 |
| 5 | A250 host1d/top20 | 21 |
| 6 | A250 network30d/top100 | 21 |

总计106。单次 argv 机制保持：`<CandidateDb> --db <bound synthetic DB> --since <case start> --until 1789776000 --tz UTC rank --by <host|network> --top <20|100>`；默认 residential=true按真实输出核对。所有21次case均须逐次exit0、process wall≤10000ms、完整精确排序/数值类型/unknown/zeroFlow/dataVersion/schemaVersion/window/capability/truncation匹配，generatedUtc落在本次实际进程UTC起止。记录全部result及raw hash、最大wall和样本数，普通失败不减少样本后声称PASS。输入身份/资源故障时保留partial/NOT_RUN；后续新批次不能与partial混合。

查询后DB hash与查询前相同、WAL为0，源/复制exe/hash前后一致；第一process只能记系列首次。A50network单次不称21次正式门。A1000完整30d/host21次≤10000ms仍NOT_RUN，AC3不因106通过而整体勾选。

## P7：判定、停止及保护

技术preflight成功是具体阶段执行资格。程序exit0、普通metricFAIL、缺测UNVERIFIED、输入身份FAIL分别保存；runner非零不能自动归为构建或native失败。matrix→primary→capacity的执行完整性由原计数和raw/identity验收确认，不把要求所有性能门全PASS新增为后阶段前置。主会话审查各阶段实际失败后按本合同继续独立证据收集；无权修产品或降低门槛来获得PASS。

源/程序/runner/fixture身份漂移、错误私有路径、语料不完整、锁文件变更、未结束重负载、disk/linker/dependency错误或取消均保存具体首失败并停止对应阶段，不写安装态或扩大权限。已授权新研究驱动的修复只处理新绑定/收据问题，不改原阈值、数据、SQL或既有历史驱动。

旧matrix FAIL、旧中断与恢复 primary、容量0/106阻断、195份历史 native hash保护收据、全部 UNVERIFIED均保留。新 exe/source/fixture身份只支持本批边界。完整106之外，非空同窗口F12、全部应用文件归属、A1000、installed/WebView/worker/24h/peak、物理冷缓存、hosted均按实测状态分别记录。没有commit/archive/push、全局安装、真实controller/凭据/生产DB操作。原T05验收不完整，任务保持未完成。

执行owner需回交 builder/generator预检、源/patch/dist/四程序manifest、新runner差异与hash、两完整generator/oracle、22+6+106逐条退出/raw/validation/summary、actual outer、source/exe/DB后检。强模型审核完成后回写当前结果；不得将本静态合同记为上述native PASS。
