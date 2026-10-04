# Research: T05 性能测量前置条件

- Query: 恢复 c278bb7 baseline、确定相同 instrumentation、核对 F1–F12 与 AC7/AC8 的输入和证据边界。
- Scope: internal；源码、历史收据、合成 fixture 的 JSON/文件元数据；未打开数据库。
- Date: 2026-09-30
- Status: 测量前置研究完成。未编译、未运行测试或压测、未生成容量库、未启动系统 trace。
- Applicable tools: Claude Code、Codex、Grok Build、Kimi Code、OMP。强模型负责归因、SQL/核算设计、门的解释和最终审查；固定收据整理可交给低成本模型。

## Findings

### 1. 结论

1. 主会话已导出 c278bb7 源码。当前三份测量模块与历史 v2 使用的 candidate manifest 完全一致；下述最小适配可恢复 baseline 源码形态，编译兼容性尚未验证。
2. 原 baseline exe 已不存在。现存 release 文件对应 09-24 历史结果，不能代表本轮全部改动。正式结果必须附新构建身份。
3. A50/A250 完整 30 天合成 fixture 与 manifest 仍存在。仅核对 JSON、大小和文件名；未重验数据库内容、完整性或读缓存状态。未找到现存 A1000 fixture。
4. benchmark 将 all_application_file_write_bytes 和 spool_write_bytes 明确写为 null。WPR/FileIO 可用不能关闭文件归属缺口。AC7 全部应用文件写入门仍未通过。
5. F8–F11 的 SQLite xWrite/CPU 比值仍需验收。T05 PRD AC4 只显式列 p95/private，遗漏这两类原门；实施时应保留并回写。
6. 正式运行必须取得独占构建/负载时段。本研究不操作安装态、控制器、私人库、权限或全局设置。

### 2. Files Found 与相关规范

下文 R19 指 .trellis/tasks/archive/2026-09/09-19-resiwatch-resource-optimization；R24 指 .trellis/tasks/archive/2026-09/09-24-resiwatch-failed-gate-followup。Rust 路径前缀为 residential-monitor/src-tauri/。

| 文件 | 用途 |
| --- | --- |
| 本任务 prd.md、design.md、implement.md | 六项 AC、准许文件与执行边界 |
| 本任务 research/source-archive.json | 主会话导出的 revision、tar hash、解包位置 |
| R19/prd.md:41–48 | 原始 AC1、AC7、AC8 正式门 |
| R19/research/benchmark-harness.md:11–41 | baseline v2、instrumentation、主场景与矩阵限制 |
| R19/research/benchmark-harness.md:43–81 | 完整容量生成及 retention 运行边界 |
| R19/research/implementation-evidence.md:7–15 | 历史归档范围、dist 用途、Rust 1.98.0 |
| R19/research/run-isolated-bench.ps1:16–91 | smoke/primary/matrix/peak/capacity 原参数 |
| R19/research/run-capacity-queries.ps1:10–38 | host 21 次及 network 历史驱动 |
| R19/research/performance-20260920-aux-series/ | 11 对矩阵、三对主场景、构建身份与源码 manifest |
| R19/research/performance-20260920-sparse-peak/peak-comparison.json | 历史 10k/1Hz/30 分钟对照，仅适用当时身份 |
| R19/research/performance-20260924-current-binary/ | 11 份 candidate 矩阵、三轮主场景，缺同窗口 baseline |
| R19/research/performance-20260924-raw-fold/ | A50/A250 合成语料与容量/阶段结果 |
| R24/prd.md:24–61；research/note.md:5–36 | F1–F12、21 次要求、短窗口修复与长窗限制 |
| src/bench/{facade,process,write_vfs}.rs | facade 测量、Win32 进程采样、SQLite VFS 计数 |
| src/bench.rs；src/bin/monitor-bench.rs | 模块和 CLI 接入 |
| src/c3/{raw_fold,sql,service}.rs | 投影/扫描及整份报告 deadline/取消 |
| .trellis/spec/residential-monitor/storage/{index,sqlite-contract}.md | 性能证据、核算、删除、历史层能力 |
| .trellis/spec/residential-monitor/backend/index.md | backend 门与运行证据边界 |
| CONTEXT.md；.trellis/workflow.md | 领域术语、角色和任务流程 |

### 3. Baseline 身份与最小接入

原始 revision：c278bb7b56603001e32e353d2ee589dccef0bfe9。

历史 source.tar SHA256：87684724D5E5B1B3A18C0212CD594CA05F9454EDD6C28582CD879270F9749EB7。历史记录明确只归档 residential-monitor/（R19/research/implementation-evidence.md:7）。

2026-09-30 导出 tar SHA256：4067688E237714566CA1164DB5D03FC7891746B0A52DBD8B494B6F34C8FD6D75。与历史容器不同，不把不同容器 hash 当作源码相同证明，不覆盖旧收据。revision 已由主会话核对；本研究读取以下解包源码：

~~~text
C:/Users/lyh/AppData/Local/Temp/resiwatch-approved-baseline-20260930-50086853fbcc43bea6015946489e3006/source
~~~

历史 baseline v2 exe SHA256：11C2542BC2C9F98A4B7517E67215E76884D8C01D25D64AE7FAF478571BF7633E。旧 v1 的 9D5B… 不可代替。v2 修正 xOpen 失败时内外 pMethods 初始化；当前 src/bench/write_vfs.rs:191–198 保留修正。

以下当前 hash 与 R19 的 performance-20260919、performance-20260920-aux-series、performance-20260924-raw-fold 三份 candidate-source-manifest.json 全部相同：

| src-tauri 下文件 | SHA256 |
| --- | --- |
| src/bench.rs | B65BCD0B854BAE926AFADCE5CF7DE097517CBC7D2EDF1BE025C625FF9A4FF204 |
| src/bin/monitor-bench.rs | 1589AFE4164162CBB08C5EAC15358FF7472662138600E28F18FF809262C76EA2 |
| src/bench/facade.rs | 278C54DE3D5DA7A5E7AC16B462CA4171D53B0D337606412216E8249E14038250 |
| src/bench/process.rs | 83734ABF3F95C1BC7313EF9ACEB7BC20E6C5799D416C48CA3723BF9AA9E906EA |
| src/bench/write_vfs.rs | C3B5D8849E3EA206C036515D86E9E7D77E75399D037EFEBBEC15685D6E48DA7A |

归一换行后，baseline 的 Cargo.toml、Cargo.lock、build.rs 与当前相同。baseline 原字节为 CRLF，当前为 LF，因此原始字节 hash 不同。tauri.conf.json 字节相同。baseline 未发现 rust-toolchain 或 .cargo/config.toml。保持原锁文件及生产源码，不更新依赖。

最小适配只发生在隔离 baseline 副本：

1. 复制当前三份 src/bench/{facade,process,write_vfs}.rs。
2. 原 src/bench.rs 增加 pub mod facade、mod process、mod write_vfs 和当前 cfg(test) run_isolated_test helper。其它代码归一换行后相同。不要加入 pub mod corpus。
3. 原 src/bin/monitor-bench.rs 增加当前 facade import、ReplayFacade enum 和 match arm，对应当前 :3–5、:52–83、:204–244。保留默认参数。不要加入 corpus import、GenerateCorpus、RetainCorpus。
4. baseline src/lib.rs:301 已有 archive_tick_at(&Mutex<AppFacade>, i64)。src/c2/facade.rs:387、779、1007、1162、1457 有 boot、ingest_snapshot、query、save_targets、upsert_alert_rule；storage/hub/snapshots/bundle_seq 字段可见。src/c3/service.rs:146 的 run_uncached 参数匹配。src/c3/archive.rs:116、156 有 next_job/persist_outcome。源码检查未发现需改生产实现的接入点。
5. 已有 dist 可复制到隔离副本 residential-monitor/dist 满足 Tauri release 嵌入，记录 manifest。该 dist 不构成 UI 同源证据。不得运行或替换安装程序。

以上尚无新 baseline build/smoke 证明。重建 exe 不必复现旧二进制 hash；应记录本次工具链、锁文件、patch 和新 exe hash。

以下可复制 Python 程序落实上述五文件接入。已验证语法、五个输入 hash 和内存变换；未执行写入循环。它只写已导出的隔离 baseline。调用前先保存 baseline 原源码 manifest；不修改 baseline 生产算法或 Cargo 文件。

~~~python
from pathlib import Path
import hashlib

repo = Path('D:/Documents/Code/Github/clash-verge-ai-residential')
base = Path('C:/Users/lyh/AppData/Local/Temp/resiwatch-approved-baseline-20260930-50086853fbcc43bea6015946489e3006/source')
assert repo.resolve() != base.resolve()
prefix = Path('residential-monitor/src-tauri/src')
expected = {
    'bench.rs': 'B65BCD0B854BAE926AFADCE5CF7DE097517CBC7D2EDF1BE025C625FF9A4FF204',
    'bin/monitor-bench.rs': '1589AFE4164162CBB08C5EAC15358FF7472662138600E28F18FF809262C76EA2',
    'bench/facade.rs': '278C54DE3D5DA7A5E7AC16B462CA4171D53B0D337606412216E8249E14038250',
    'bench/process.rs': '83734ABF3F95C1BC7313EF9ACEB7BC20E6C5799D416C48CA3723BF9AA9E906EA',
    'bench/write_vfs.rs': 'C3B5D8849E3EA206C036515D86E9E7D77E75399D037EFEBBEC15685D6E48DA7A',
}
inputs = {name: (repo / prefix / name).read_bytes() for name in expected}
for name, data in inputs.items():
    assert hashlib.sha256(data).hexdigest().upper() == expected[name], name
module = inputs['bench.rs'].decode('utf-8')
assert module.count('pub mod corpus;') == 1
module = module.replace('pub mod corpus;\n', '', 1)
cli = inputs['bin/monitor-bench.rs'].decode('utf-8')
corpus_import = 'use residential_monitor_lib::bench::corpus::{generate_corpus, retain_corpus};\n'
assert cli.count(corpus_import) == 1
cli = cli.replace(corpus_import, '', 1)
first = cli.index('    /// 批量生成生产 schema')
last = cli.index('    /// 真实门面 + 档案 tick', first)
cli = cli[:first] + cli[last:]
first = cli.index('        Commands::GenerateCorpus {')
last = cli.index('        Commands::ReplayFacade {', first)
cli = cli[:first] + cli[last:]
assert 'GenerateCorpus' not in cli and 'RetainCorpus' not in cli
assert 'bench::corpus' not in cli and 'pub mod corpus;' not in module
outputs = dict(inputs)
outputs['bench.rs'] = module.encode('utf-8')
outputs['bin/monitor-bench.rs'] = cli.encode('utf-8')
for name, data in outputs.items():
    target = base / prefix / name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
    print(name, hashlib.sha256(data).hexdigest().upper())
~~~

内存变换得到的预期输出：bench.rs 为 16564 B、SHA256 499F3B905DBA9E48C1D9840F4FC57803B03B3CDB6E3B3C2ECD40006684023EA8；bin/monitor-bench.rs 为 11870 B、SHA256 7BA08896AC884DD764B1AD954DC3B579CDB38ADADF272A77D8CC4422FE7A3BFC；其余三份 hash 保持上表值。该验证没有运行 Rust 或写 baseline 文件。

### 4. 可用合成 fixture 与现存二进制

根目录：C:/Users/lyh/AppData/Local/Temp/resiwatch-resource-measure-20260924-raw-fold。

| 观测或 manifest 字段 | corpus-a50-30d | corpus-a250-30d |
| --- | ---: | ---: |
| monitor.sqlite3 当前大小 B | 544940032 | 1697222656 |
| 当前 WAL / SHM 大小 B | 0 / 32768 | 0 / 32768 |
| minute / session / chain / receipt 行 | 2160000 / 432000 / 1296000 / 2592000 | 10800000 / 2160000 / 6480000 / 2592000 |
| start_utc / end_utc | 1787184000 / 1789776000 | 1787184000 / 1789776000 |
| days / seed / SQLite schema | 30 / 20260919 / 5 | 30 / 20260919 / 5 |
| spec_hash | e8a32307e3507924e5a85c621f5e647305a8169d66df56a6a108dfe850a96c8a | 3c163f819d02827bf4b1f4bc2036eb44b499bd8ff64b155fb9965789ab096c47 |
| production-corpus.json SHA256 | AA4A7ABD598F07C7D9BD0C29F7043C5C862DE7BABDC6DC8A59D552B1945C101B | 667A21E9FBE3FCF100974411D8E714E1589907E6458902AFD50683CEE68A3D42 |

两份 manifest 均为 full_30_day_input=true、counts_match=true、generation_cache_kib=65536；共同 workload 为 5 分钟会话、3 跳 chain、800 host、120 process、40 rule、60 chain、4 network、1Hz、nonzero_minute_ratio=1。peak_active=10000/peak_minutes=30 是定义字段，不能证明峰值运行。

本轮没有打开、hash 或复制数据库。actual/counts_match 仅为生成时收据。正式运行仍需只读核对当前内容与完整性。数据库 hash、复制、quick_check、阶段探针均会读取文件；其后首 reader 不可标为未经页缓存预热。

2026-09-30 只读磁盘快照：C 剩余 460871311360 B，D 剩余 1155180908544 B。R19/research/acceptance-status.md:42 的历史 A1000 30 天主库为 6124961792 B；该值不能保证本次 target、WAL、临时文件与 trace 峰值。当前可用空间足以安排单份历史规模语料，但执行前仍应重查空间与路径，不做清理。上述快照不证明磁盘吞吐、负载或缓存可比。

当前 target/release/monitor-bench.exe SHA256 为 4C17FA3F7A8A37FD461387B7068F428031D7B2939718D0AEC8B5A45682878EC3；monitor-db.exe 为 D0DFEF0572F1AE72934F109D0EE9E52ADC6779BC59FF2D2119DFD059B133793E。对应 09-24 历史收据。它们与 performance-20260924-raw-fold/build-identity.json 更早一轮构建不同；各历史身份应保留。

### 5. F1–F12 原参数、阈值与收据

原门来源：R24/prd.md:24–61。参数来源：R19/research/run-isolated-bench.ps1:25–31、64–72，并与 aux-series 的 11 份 baseline JSON options 核对。

全部矩阵固定 hz=1、seed=20260919、start_utc=1800001800、warmup=5 秒、duration=30 秒、query_every_frames=5、metadata_change_percent=100、virtual_time=false。first/repeated reader 各 6 个样本。complete 的 period_rule=false；backlog/failed 为 metadata、period_rule=true。

| 项 | 场景 | 指标及保留阈值 | R24/prd.md 来源 |
| --- | --- | --- | --- |
| F1 | A1000 metadata / complete | ingest p95 candidate/baseline ≤1.10 | :30、59 |
| F2 | A250 metadata / backlog / period-rule | ingest p95 ≤1.10 | :31、59 |
| F3 | A50 unchanged / complete | ingest p95 ≤1.10 | :32、59 |
| F4 | A250 metadata / failed / period-rule | ingest p95 ≤1.10 | :33、59 |
| F5 | A250 metadata / complete | ingest p95 ≤1.10 | :34、59 |
| F6 | A1000 metadata / complete | native private p95 ≤1.10 | :35、60 |
| F7 | A1000 unchanged / complete | native private p95 ≤1.10 | :36、60 |
| F8 | A50 metadata / complete | SQLite xWrite ≤1.10；旧比值 1.4235 | :26、37、60 |
| F9 | A250 metadata / failed / period-rule | SQLite xWrite ≤1.10；旧比值 1.2979 | :26、38、60 |
| F10 | A1000 metadata / complete | SQLite xWrite ≤1.10；旧比值 1.1787 | :26、39、60 |
| F11 | A250 metadata / backlog / period-rule | CPU 按 AC4 同一规则保留 ≤1.10；旧比值 1.1724 | :40、60 |
| F12 | A50/250/1000 × unchanged/counters/metadata，加 backlog、failed 共 11 对 | first reader p95 ≤1.10；空、非空分别记录 | :41、43、60 |

阈值解释：原 R2 第 26 行明确 SQLite xWrite 大于 1.10；CPU 没有单独阈值句，但 F11 列为失败，AC4 第 60 行要求 F6–F11“用同一规则收口”。本研究保守继承 ≤1.10，不把 backlog CPU 门改为主场景 CPU 下降 30% 的门。当前 T05 AC4 应明确这层来源，避免只检查 p95/private 后关闭 F8–F11。

同轮 baseline/candidate 为 R19/research/performance-20260920-aux-series/matrix-<scenario>-{baseline,candidate}.json。09-24 candidate 为 performance-20260924-current-binary/matrix-<scenario>-candidate.json；comparison-against-archived-baseline.json 仅为跨日对照。旧 baseline 的 local_utc_offset_seconds=28800；新一轮两侧记录并匹配实际偏移，不修改全局时区复刻旧数字。

F12 的旧 35 秒窗口没有已落盘到所选窗口的 raw 报告行（R24/prd.md:16）。当前 facade.rs:193–205 只验证报告字节非负；:263–268 的 fixture_hash 只覆盖输入参数。hash 相同不能证明非空报告等价。保留原 11 对，另记明确非空的同窗口 reader 结果与摘要；不得通过延长原矩阵时长替换旧门。

### 6. Read / Build / Run 程序

以下仅为后续程序，本轮未执行。每个输出使用唯一 evidence 目录；每条 native command 立即保存退出码、stdout、stderr、起止时间，后续成功不能覆盖失败。

#### 6.1 冻结与构建

1. 主会话释放 Cargo/重负载时段。记录竞争负载，不停止用户程序、不改变权限或全局设置。
2. 保存 baseline 原源码 manifest，按第 3 节接入五文件，另存 patch/hash。candidate manifest 包含 Rust、Cargo、build/Tauri 配置；未提交树不能只标 HEAD。
3. 记录 rustc -vV、cargo -vV、linker/SDK、Cargo 环境与参数。本轮只读命令确认 rustc 1.98.0 (88d9e12ae)、cargo 1.98.0 (797e8a9bc)、x86_64-pc-windows-msvc、LLVM 22.1.8。R19/research/implementation-evidence.md:15 同样记录 Rust/Cargo 1.98.0。
4. 变量必须为核对后的绝对路径。两侧输出隔离，不覆盖旧 exe：

~~~powershell
cargo +1.98.0 build --locked --release --target x86_64-pc-windows-msvc --manifest-path "$BaselineSource/residential-monitor/src-tauri/Cargo.toml" --target-dir $BaselineTarget --bin monitor-bench
cargo +1.98.0 build --locked --release --target x86_64-pc-windows-msvc --manifest-path "$CandidateSource/residential-monitor/src-tauri/Cargo.toml" --target-dir $CandidateTarget --bin monitor-bench --bin monitor-db
cargo +1.98.0 test --locked --release --target x86_64-pc-windows-msvc --manifest-path "$CandidateSource/residential-monitor/src-tauri/Cargo.toml" --target-dir $CandidateTarget --lib --no-run --message-format=json
~~~

5. 复制并 hash 两侧 monitor-bench、candidate monitor-db、compiler-artifact 指定的 release library test exe。结束所有编译后，各跑 A8/3 tick 相同 smoke，核对 commits、traffic.conserved、fixture_hash、WAL/FULL。smoke 只证明接入。

#### 6.2 阶段与生产 deadline

运行已复制的 test exe，避免隐式编译。每次独立进程设置环境，结束恢复：

~~~powershell
$env:RESIWATCH_RAW_FOLD_STAGE_DB = "$CorpusRoot/corpus-a250-30d/monitor.sqlite3"
$env:RESIWATCH_RAW_FOLD_STAGE_OUT = "$EvidenceRoot/a250-minute-all-stage.json"
$env:RESIWATCH_RAW_FOLD_START = '1787184000'
$env:RESIWATCH_RAW_FOLD_END = '1787184060'
$env:RESIWATCH_RAW_FOLD_FILTER = 'all'
& $CandidateTestExe --exact c3::raw_fold::tests::isolated_raw_fold_stage_proof --ignored --nocapture --test-threads=1
~~~

再用唯一输出名运行 filter=residential 和完整窗口 [1787184000,1789776000)。该 probe 无生产 10 秒 deadline（raw_fold.rs:1293），仅用于 projection/scan 归因，且会预热页。

生产默认报告 probe 使用 RESIWATCH_NONEMPTY_STAGE_DB/OUT/START/END，执行 c3::service::raw_stage_probe_tests::isolated_nonempty_report_stage_proof。其第一项 run_uncached 不覆盖产品 deadline（service.rs:1777–1804）。独立报告首次结果与之后阶段，不能用拆分耗时和替代生产首读。

2026-09-30 实施勘误：本研究原先误写模块为 `c3::service::tests`。首次执行退出 0，但运行 0 tests 且未生成 JSON；`stages-20260930-initial/production-first-validation.json` 正确记录 FAIL。后续阶段已读取该语料，修正入口后的结果必须标为本轮已读取页缓存。后续执行先用同一 test exe 的 `--list` 核对精确名称，同时要求实际运行 1 个测试、输出 JSON 存在及语义验收通过。原始错误命令和失败收据保持原样。

#### 6.3 同窗口主场景与矩阵

复用 R19/research/run-isolated-bench.ps1。各 RunSet 使用新 DataRoot/EvidenceRoot；脚本按 baseline→candidate 执行并拒绝覆盖 JSON：

~~~powershell
& $ArchivedRunner -RunSet primary -BaselineExe $BaselineExe -CandidateExe $CandidateExe -CandidateSource $CandidateIdentity -DataRoot $PrimaryData -EvidenceRoot $PrimaryEvidence
& $ArchivedRunner -RunSet matrix -BaselineExe $BaselineExe -CandidateExe $CandidateExe -CandidateSource $CandidateIdentity -DataRoot $MatrixData -EvidenceRoot $MatrixEvidence
~~~

primary 为三对 A250/counters/complete/1Hz、30 秒预热+300 秒测量、query_every_frames=0，不得 virtual-time。脚本写 metadata_change_percent=100，counters 下 metadata 实际不变；不要混用 CLI 缺省 10 导致 fixture_hash 不同。

matrix 为 11 对、5+30 秒、每五帧查询。原 first/repeated 两列和全部 F 项分别判断。CPU 计数为 0 时保留读数与分辨率限制，不推出零计算。

最低实时回放：primary 6×330=1980 秒；matrix 22×35=770 秒，共 45 分 50 秒，不含初始化、统计、构建和容量查询。正式时段不得与构建或其它压测并行。

#### 6.4 完整容量与 oracle

核对 synthetic marker/manifest 与路径。完整内容校验后记录缓存已被读取。保留 A50/A250；阶段证据支持下一步后才在新目录生成 A1000 完整 30 天，相同 seed/start，不得 days=0：

~~~powershell
& $CandidateExe generate-corpus --average-active 1000 --days 30 --seed 20260919 --start-utc 1787184000 --dir $NewA1000Dir
& $CandidateDbExe --db "$CorpusRoot/corpus-a250-30d/monitor.sqlite3" --since 1787184000 --until 1789776000 --tz UTC rank --by host --top 20
& $CandidateDbExe --db "$CorpusRoot/corpus-a250-30d/monitor.sqlite3" --since 1787184000 --until 1789776000 --tz UTC rank --by network --top 100
~~~

rank 的 residential 默认 true（dbcli/mod.rs:121–128）；历史未传该开关仍为家宽筛选，不能标 all。A250 host/network 各 21 次、A1000 host 21 次，每次 exit=0 且 wall≤10000ms。保留 A50、1d 等原要求组合的独立结果。

旧 run-capacity-queries.ps1 的 host 为 0..20，但两次失败提前结束；network 只有一次（:27–31）。此驱动可保留失败诊断，不能原样宣称本次 network 21 次门通过。任何不足 21 次仍未通过。

monitor-db 使用真实 Utc::now（dbcli/mod.rs:215）。旧语料跨越当前 30 天 cutoff；当前历史汇总在未删除且精确汇总缺失时可退回尚存 raw（service.rs:474–515）。记录 generated_utc、层和 query echo。阶段 test 的 now=end 与 CLI 实际时钟不同，不能混称同一 capability 证据，不改系统时钟。

沿用 grouped SQL oracle 与短窗口跨窗回归，比较 totals、distinct session/minute、series、各 grouping/filter、排序/tie、缺失字段、零字节、负分钟、previous-window、coverage。service.rs:160–203 拥有整份报告取消和 deadline，不得按阶段重置。

### 7. AC7 文件归属与 AC8 可行性

facade.rs:149–150、238–240 定义测量区间；:294–296 分开 SQLite 字节和缺失的全应用字段；:309–315 排除初始化、warmup、结果输出、HTTP/WebView、真实 collector 队列。

process.rs:56–79 仅 GetCurrentProcess/GetProcessIoCounters，没有文件路径归属。write_vfs.rs:199–208 仅给隔离根下 SQLite 文件挂计数，:251–266 累计成功 xWrite 请求。不覆盖 mmap/SHM、spool、日志。初末目录大小（facade.rs:516–539）无法还原生成后删除的瞬时文件写量。

只读发现 C:/WINDOWS/System32/wpr.exe 与 Windows Performance Toolkit/wpa.exe。wpr -profiles 成功，版本 10.0.26100，包含 FileIO、CPU、DiskIO。未启动 capture、未验证采集权限、未变更权限。

AC7 文件归属还需经过审查的测量定义：

- 按 benchmark PID、测量起止时间、规范化文件路径归属应用文件；输出分类和总写字节。排除 trace 自身、结果 JSON、初始化，保留已删除 spool/log 的写事件。
- 两侧追踪配置相同，记录丢事件、失败写、映射写与未覆盖类别。逻辑文件写请求和物理设备写放大分别报告；未覆盖项不填零。
- 现有 JSON 仅 PID、相对 wall、虚拟 UTC。若采用外部事件流，最小 instrumentation 可在 facade.rs/process.rs 增加测量边界实际 UTC/单调时钟收据，并同步加入两侧。未验证前不填 all_application_file_write_bytes。
- 全系统采集不属于本研究动作。WPR 的存在不能证明精确归属、采集权限或无敏感信息。当前文件归属收据缺失，AC7 未通过。

AC8 峰值复用 RunSet peak：A10000、1Hz、counters、complete、30 秒预热+1800 秒实测，两侧共 61 分钟，仍无 WebView。历史 sparse-peak 结果不能替代本轮身份。

native+WebView 总 Private Bytes p95≤baseline 110%、真实后台 worker、collector 队列、隐藏恢复、24h 安装态 soak 均不由 replay-facade 覆盖。原 durable commit p95<1.5s、正常 max<3s、队列不持续>2 帧仍保留；facade_ingest_including_durable_commit 不能等同单独 COMMIT，schedule_lag 不能等同真实队列。安装态在默认批准范围外，相关项保持 UNVERIFIED。

### 8. 最小范围与性能假设

事实：load_sessions 对跨度≤3120 分钟窗口限定，更长区间全表投影（raw_fold.rs:22–24、116–131）。SQL 为避免第二次分钟索引扫描而保留长窗口行为（sql.rs:23–35）。fill_raw 顺序为投影、当前 fold、previous fold，共用读事务和预算（service.rs:811–856）。

事实：09-24 A250 未热页 host 首读 10121.934ms/exit7；无 deadline 阶段探针之后，热页 host/network 各 21 次通过。A1000 预测仅为 A250 行数四倍外推，未实测，不可用外推取消完整门。

假设 H1：长窗投影分配与分钟逐行折叠共同决定耗时。先在 raw_fold.rs/sql.rs/service.rs 范围记录 projection、scan、reader/coverage 与精确结果，再决定是否减少字典/SessionFact 分配或重复运算。不得直接把 3120 阈值扩大到 30 天；那会额外扫描分钟索引。

假设 H2：backlog/failed private 增长的一部分来自 ArchiveScheduler 待处理描述。R19/research/acceptance-status.md:166 只有源码形态与指标相符证据，缺分配归属。调度器不在 T05 文件范围，未获归因和范围批准前不改。

假设 H3：F1–F5 跨日差异可能混入机器负载、页/文件缓存、checkpoint 与产品开销。须同窗口 AB 和阶段证据区分，原因未查明。

最小 instrumentation 限于 bench/facade.rs、bench/process.rs、bench/write_vfs.rs 与现有忽略 probe；不变更 schema、核算、保留期、AUTO_DELETE_ENABLED=false、WAL/FULL、调度器、10 秒 deadline、1.10 比值门。新算法仍须完整 oracle 与 deadline/cancel 回归。

## Caveats / Not Found

- baseline API 已核对；本轮接入 patch、编译、smoke 未执行。
- 原 v2 exe、现存 A1000 fixture、全应用文件写入、真实 native+WebView 与安装态 soak 证据缺失。
- A50/A250 仅 metadata 检查；本轮数据库完整性未验证。
- 未执行 Cargo/Node 测试、构建、容量生成、retention/VACUUM、系统 capture 或 Git 操作；未改产品、PRD/spec、安装态、全局配置或团队记忆。
- research 角色未读 implement.jsonl/check.jsonl，使用派发任务产物与规范。原生 hook 注入 Active task=T01，明确派发为 T05；主会话确认后使用 T05 独占研究文件。T06 应记录该上下文边界。
- 外部参考：未依赖在线资料。版本来自只读本机命令，性能数字来自仓库原始收据；本轮未测量性能。
