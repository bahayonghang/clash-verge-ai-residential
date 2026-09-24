# Implementation evidence

## Authorization and baseline

2026-09-19 用户明确回复「审阅完毕，开始实施」；task.py start 已将任务从planning改为in_progress。产品代码和隔离验证已授权，现有安装态、真实数据库写入、提交与发布不在本轮范围。

原始源码：`c278bb7b56603001e32e353d2ee589dccef0bfe9`。修改前通过git archive保存`residential-monitor/`到：

`C:/Users/lyh/AppData/Local/Temp/resiwatch-perf-baseline-68253a0cde334e058766b87f8a797380`

`source.tar` SHA256：`87684724D5E5B1B3A18C0212CD594CA05F9454EDD6C28582CD879270F9749EB7`。

仅为Tauri release编译复制已有dist到baseline目录；基准不运行WebView，不将该dist当源码匹配UI证据。新基准模块可同时加入baseline和candidate；baseline生产实现不得修改。基准旧`generate`/`verify-design-db`是C0候选schema与简化提交，不能替代C3完整容量证明。

工具链：cargo/rustc1.98.0，Node26.7.0，npm12.0.2，just1.58.0。初始已有`.gitignore`单行用户修改，保持不动。

## Implementation ownership

| Owner | Exclusive scope |
| --- | --- |
| ledger_impl | storage/accounting/c2 facade/c0 contract/schema与存储辅助模块 |
| retention_impl | c3 retention/query与其辅助模块 |
| runtime_impl | lib/archive/service/period与原生可见性边界 |
| frontend_impl | residential-monitor/src/ |
| benchmark_impl | bench、monitor-bench CLI及基准辅助模块 |
| operation_impl | c2 shell OperationRegistry与c3 snapshot租约 |

主会话负责集成、规格/文档、证据与最终检查。实现agent不递归派实现或检查agent，避免共享文件重叠。

## Integration decisions within approved scope

- 指定operation ID决定取消范围；未指定ID的内部查询不能借用任意同kind操作的取消标志。完成操作显式移除，SQL持有的Arc保持有效。
- 每次成功snapshot insert获得一个内存租约，get/export不新增；同fingerprint仍复用token，释放最后一个租约才删spool。TTL/LRU强制淘汰不受租约阻挡。前端同token刷新也须平衡旧租约，避免影响其它持有者。未新增DTO、配置或schema。
- 查询读取实际已验证汇总边界；维护积压保留的raw不能被误读为汇总零值。日coverage保留实际秒数，不能把少量gap扩成整日gap。

## Gate status

代码已进入集成验证，最终状态以本节之后的日期记录及 `final-checks.md` 为准。性能AB、容量、峰值和24小时安装态证据仍待实测。自动DELETE保持关闭，直到规定守恒与容量门通过；不以短测试替代。

## Integration checkpoint — 2026-09-19

- operation实现已增加完成移除、取消隔离及snapshot租约/替换配额测试；scoped rustfmt/diff检查通过。独立review发现settings未透传operationId，以及writer持有facade锁阻塞取消命令；分别回交frontend与registry/runtime修复，尚未确认关闭。
- retention第一版20项focused测试通过（含8项真实DELETE/fault/396d回归）；随后识别整日1秒事务存在大日块无进度问题，正改为持久游标分块，因此这些结果不能视为最终retention版本通过。
- benchmark新增隔离VFS xWrite计数；review发现默认VFS在并行测试中的生命周期风险，正改为子进程隔离测试与原VFS回调转发。尚未完成最终AB，不以进程I/O或文件增长代替写字节。
- Basic Memory检查点首次写入被自动审批拒绝；用户随后明确同意保存，本次checkpoint已创建。仅记忆写入获准，安装态和真实库维护边界未改变。

## Verified intermediate outputs

- 完整前端 `npm --prefix residential-monitor run check` 通过：icons、TypeScript、ESLint、73个测试文件/300项测试、production build；包括隐藏恢复、慢HTML过期响应、settings取消ID、导出期间token租约。安装态WebView行为仍未验证。
- 8项benchmark focused测试通过；VFS相关测试在独立子进程执行，避免改变并行测试进程默认VFS。它只计数隔离SQLite文件的成功xWrite，不等于物理磁盘写入或所有应用文件写入。
- 修改前源码release基准已编译并复制至baseline目录的`monitor-bench-baseline.exe`，SHA256 `9D5BF09001FCA3330A0BF49122D937F33646A64255271E5A21D1CBDBBBA59E2D`。该副本不会被candidate链接覆盖。
- 后续仅同步VFS打开失败初始化修正并重建，最终AB使用`monitor-bench-baseline-v2.exe`，SHA256 `11C2542BC2C9F98A4B7517E67215E76884D8C01D25D64AE7FAF478571BF7633E`。旧基准副本保留；其smoke结果不作为v2性能数据。
- 最新task.py结构校验与scoped git diff --check通过；它们不证明产品验收。

## Review-driven integration requirements

- 已核对并修复operation独立取消、settings ID传递、archive过期表单及导出租约；真实SQL取消回滚仍须跑最终回归。
- ledger review要求维护/失败restore不能丢弃pending/deferred；FIFO回放不能使用后到网络状态解释先到snapshot；policy更新后lifecycle不得确认旧分类metadata。这些由ledger owner修复并添加回归。
- legacy `epoch`/`closed` 仅记录生命周期，不是正向采样证据。新采样覆盖从连续有效sample时间段同事务写入，按UTC日合并；不倒填旧覆盖。`gap`、未观测、未来区间保持区分，period查询不超出now。
- 任何raw查询（含share）必须先检查实际删除/封存边界。配置raw期限变长不能让已经删除的明细变成「精确0」。

## Integration freeze — 2026-09-19 18:42 +08:00

- 所有实现owner已冻结，最终完整 `rtk proxy just ci` 由独立 `trellis-check` owner串行执行。主会话仅更新spec、任务和package说明。保留删除、ledger、取消和runtime的独立源码审查发现均已落实；测试状态仍独立记录。
- accounting去掉随历史churn增长的retired-ID集合，仅保留active；首次重新出现的inactive ID通过 `(epoch_id, connection_id)` 索引查持久记录，查询/预留失败保留原输入。新增10000次churn与零流量ID重用回归。
- 最终retention focused结果：29 passed，1容量测试ignored；query24、backup3、vacuum2 passed。随后新增coverage写入期间破坏core的跨表故障回归，交给完整gate。
- 最新service focused结果47 passed；frontend完整门73文件/300项测试 passed。不能把这些中间结果合称完整 `just ci` 通过。
- 完整gate首次因sandbox中的npm spawn EPERM中止；按同一授权范围升级执行后，前端/格式/clippy通过。Rust发现4项失败，其中两项取消测试暴露小事务在取消后仍提交；由checker修复，尚未宣布通过。
- 新增 `run-isolated-bench.ps1` 串行执行smoke/三轮实时AB/变化矩阵/30分钟峰值/完整30天容量生成；PowerShell语法解析通过，尚未运行性能组。结果与DB目录分离，失败保留现场。
- checker定位取消失败根因：持续取消的SQLite progress handler会再次打断RAII rollback，导致共享writer仍停留在未提交事务。辅助维护、staged retention和既有chain修复的owner边界改为先清handler，再显式回滚尚未关闭的自有事务；新增取消期间raw删除与autocommit断言。修复后Rust library **513 passed / 2 ignored**，最终完整 `just ci` 正在重跑。另修复档案时钟回退后的未来retry时间，以及零流量ID重用测试的空SUM/基线预期。

## First build gate and measured iteration — 2026-09-19 19:31 +08:00

- 第一构建完整 `just ci` 通过：前端73文件/300项、Rust library513项+process integration3项、root139项，含fmt/clippy/build/secret scan。最后一项corpus工具优化另通过fmt/clippy及10项benchmark回归；完整证据与环境性EPERM重试见 `final-checks.md`。
- 修复容量工具在每个辅助清理chunk之后重复扫描整个session库的问题：持久扫描游标各自到达末端后才执行完整完成证明，保留最后全量inventory校验。不是删0行就宣称完成。
- 实时A250/1Hz/metadata不变/档案齐全，3对各预热30秒+测量300秒。六个run的fixture hash均为 `e4f93c4c583bec372f8e675e66e5004823987f6832bf1387c0a4f54e965004f1`。baseline合计CPU23.28125秒，candidate0.90625秒，下降**96.1074%**。每对SQLite xWrite均为23531752/14901296B，下降**36.6758%**，未满足50%写入目标。native private p95均在对应baseline的110%以内；不包含WebView。
- 全部run字节守恒、300次提交、无1秒tick overrun。candidate ingest p95为11.3861/9.1403/11.7149ms，最大12.796/12.7513/16.4384ms。主场景未请求报告查询，不能据此声称query p95通过。
- 原始JSON、源码manifest、exe哈希和汇总在 `performance-20260919/`。候选是首版 `D0600E41...`，不代表后续layout2源码；原始结果完整保留。
- 写入短板触发第二迭代：事务性重建receipt物理布局，统一receipt+legacy-floor版本读取，改用不随每tick变化的coverage-tail索引。健康实例评估时间及其索引保持原语义。仅六个Rust文件变化，独立源码审阅无阻塞发现；源码已格式化，新的完整gate正在运行。v5 checksum改为 `ledger-lifecycle-v5-layout2`，拒绝旧实验v5而不改已发布v1–v4 migration。

## Layout2 measurements and final layout — 2026-09-19 19:55 +08:00

- Layout2完整 `just ci` 首次即通过，81.12秒；frontend300、Rust520+process3、root139。详见 `layout2-checks.md`。
- A250/300虚拟tick探针：SQLite xWrite为23531752/11090864B，下降52.8685%，输入hash相同且流量守恒。虚拟探针不能替代3轮实时AC7；源码/exe和原始JSON在 `performance-20260919-layout2/`。
- 完整30天A50生成112.10秒、DB586887168B；A250生成329.78秒、DB1910484992B；minute/session/chain/receipt均与规定行数一致。dbstat发现 `idx_connection_minute_utc` 与primary-key index完全重复，A50单个重复索引占41943040B。尚未运行查询/保留容量门，不能把生成成功称为容量验收通过。
- 核对仅DDL引用该索引后，停止本任务拥有的A1000 generator PID17868（已验证exe路径）；其数据和失败现场保留，不作为成功容量数据。未触及安装程序。
- Layout3在未发布v5中删除重复索引，checksum更新为 `ledger-lifecycle-v5-layout3`，保留已发布v2文本。新增真实v4迁移和query-plan回归：迁移后重复索引消失、primary index仍有序range seek且无TEMP sort，2项focused测试通过。
- 仅fixture生成连接使用64MiB cache，保留WAL/FULL与原事务/行协议。manifest明确记录 `generation_cache_kib=65536` 并排除生成过程内存作为产品runtime证据。三个Rust文件变化，最终完整gate进行中；将重建独立layout3哈希和全套容量fixture。

## Full capacity and shared query follow-up — 2026-09-19 20:28 +08:00

- Layout3完整gate通过（79.66秒，frontend300、Rust521+process3、root139），三档30天生产schema fixture均已生成且行数完全匹配：A50 544940032B、A250 1697222656B、A1000 6124961792B。分别耗时67.27/158.30/384.11秒；原始manifest/dbstat见 `performance-20260919-layout3/`。
- CLI真实报告路径实测暴露30天大窗口限制：A50重复p95 8405.04ms；A250/A1000均两次命中10秒deadline（exit7）。单日三档均成功，p95为265.60/1389.66/5744.88ms。完整测试保留失败，未称全部查询容量通过。
- 独立源码审查发现raw totals与attribution扫描重复。仅在共享报告路径融合精确aggregate，保留独立旧算法为test oracle；相同snapshot中missing判定非NULL且session不变，known=total−missing保持distinct与字节精确。SQL progress检查64→1024，增加active-SQL取消/deadline回归。service50+SQL12通过，独立review未发现数据正确性问题。
- query1独立构建与source manifest在 `performance-20260919-layout3-query1/`；schema和generator不变，因此复用原30天库。A50 30天重复p95降至5983.26ms，单日三档p95为249.69/1115.18/4853.48ms。大窗口A250/A1000仍超时。五个代表性成功CLI输出除generatedUtc外完整相同，见 `query-equivalence.json`。
- review发现namedSql旧标签，已仅在query.rs更正为实际 `totals_raw_attribution` 并增加plan断言；该诊断修正晚于query1构建，待最终gate/rebuild，不将其冒充已测二进制。没有CLI专属跳过报告计算、汇总近似计数或新schema/cache。
- 12:26:35Z开始用已复制query1 release测试程序在A50完整30天fixture过期首日。生产自动删除常量仍false，隔离cfg(test)门用于验证完整日守恒和真实清理。大库/45天/手动VACUUM与最终性能组仍进行中。

## Capacity-driven cleanup progress fix — 2026-09-19 20:41 +08:00

- A50首日retention测试在163.30秒后exit101，complete=false。72000条raw已通过完整日复核后删除，但辅助session cleanup在254.81ms被250ms上限中断，未完成14400个可回收session与601000条过期receipt；quick_check通过。旧结果和失败数据库保留。
- 独立生成公式与当前raw+已完整删日core对照：upload30240060/download101519550B严格守恒。此旁证补足旧ignored test在complete断言失败后未执行的末尾总量检查，不将失败改称通过。
- EXPLAIN确认候选、raw引用与三个DELETE均使用索引。问题是固定1000-session事务内3000次未缓存prepare及索引删除的fanout；超时把整个cursor一起回滚，反复无法前进。
- 仅storage_lifecycle.rs修复：复用prepare，在既有最多1000个candidate内按128行读取；session在100ms、receipt在200ms协作让出，并保留共享250ms硬deadline。仅已处理前缀推进cursor并与删除/digest同事务提交；user cancel仍回滚。13项lifecycle测试通过，含人为3ms/session时29条在107.06ms提交、reopen续跑及原取消/autocommit回归。
- cfg(test)容量测试现在在写JSON和complete断言之前记录combined_before/after、conserved及remaining_expired_raw，失败也保留字节旁证；不修改产品门或既有assert。该helper已格式化，待完整gate。
- 最终完整gate重新运行中；后续先复制失败fixture做resume证明，再跑尚未修改的A250/A1000及45天fixture，不覆盖原始失败证据。

## Capacity recovery and retention statement reuse — 2026-09-19 21:53 +08:00

- Cleanup1完整gate通过后，A50恢复90.75秒通过；A250原首日run守恒但辅助维护遇到可重试预算中断。驱动随后保留deadline/busy错误并按生产分类在原chunks额度内继续，且只在writer恢复autocommit、未用户取消时重试。
- A250恢复暴露测试观察缺陷：辅助session/receipt与coverage/dictionary游标异步结束，驱动仅在more_pending=false时观察会丢失EOF。改为每个已提交辅助tick锁存，最终仍由全量inventory/守恒/quick_check判定。新增异步wrap回归及独立审查通过；该版完整gate为frontend300、Rust529+process3、root139。
- 从原失败A250同hash副本恢复99.36秒通过，2162块、0重试，可回收session/eligible receipt均0，upload151200035/download507599545B严格守恒。最大chunk361.85ms，规格明确250ms是VM progress阈值，不是已证明的墙钟上限。
- A1000完整30天库过期首日的build阶段运行661.25秒后主动停止，原DB+WAL保留。已将二者一起复制并逐文件核对源复制前后及副本SHA256；随后只在副本恢复，不覆盖原中断证据。
- 保留热路径确有每raw每遍4次未缓存rule/chain identity lookup，以及每聚合行有限SQL形状的publish/audit prepare。仅7个调用点改为prepare_cached，保留每次绑定/执行/校验/trigger/事务/取消，未缓存结果。29项retention测试及格式检查通过，独立review无阻塞发现；完整gate记录另见 `retention-cache-checks.md`。速度改善尚未测得，原结果不挪用至新source identity。

## Auxiliary completion and bounded series — 2026-09-20 00:03 +08:00

- Retention-cache A1000恢复完成：首日raw退出、辅助可回收行归零、upload604800000/download2030399685B守恒，quick_check=ok；最大chunk1372.95ms保留为性能结果。45天A4 raw/session退出且守恒；隔离手动VACUUM将428007424B缩至37261312B。详情及限制统一见 `acceptance-status.md`。
- 45天输入推进至end+397天后，raw/hourly/daily精确维度均退出，但1024个无引用字典残留。修复现有cursor完成协议：引用删除与最终prune事务把相关cursor置-1，正数表示有界扫描中、0表示最后失效后的扫描完成；facade和容量驱动在后置ledger清理后重读pending。不开新表，不对热路径做全表孤立扫描，关闭删除门时不触发快速空转。
- 31项retention、13项lifecycle、4项corpus和59项facade专项测试通过，新增多日260键及后置201会话引用释放回归；独立只读审查无阻塞发现。原老化fixture和失败完成证据保留，在相同hash的新副本重测。
- bundled SQLite真实A1000保留29天阶段测量：totals30.57s、series37.03s、host rank53.54s、exits11.59s，各自独立60s诊断预算，不修改生产10s限制。仅将series全窗口GROUP BY改为同一快照/预算中复用语句逐桶索引读取；旧查询作为独立oracle，3项专项测试覆盖粒度/符号/裁边/空桶/零流量/filters及运行中取消。
- 同快照单对series35.60→26.30s（26.14%），696行SHA256完全相同。此局部诊断既不是完整30天，也不是3轮AB；总量/排名仍单项超过10s。详细证据 `performance-20260919-query-stages/series-stage-findings.md`。
- 最终独立 `just ci` 77.61秒通过：frontend300、Rust534+process3、root139；3项明确默认忽略。源码与release exe身份在 `performance-20260920-aux-series/build-identity.json`，source manifest SHA256 `38C9D3B07969494FDC5EEB8EA24BEB2FB16DC0CB90CB4B40E024EDE258DBB106`。桌面仅构建/复制；没有安装、用户库维护、提交或归档。

## Real primary pairs and failing matrix — 2026-09-20 01:04 +08:00

- 同hash老化副本65.35秒完成：字节守恒、quick_check通过，字典仅保留daily core引用的category1；1024个孤立值退出。45天输入老化不冒充396天连续容量。
- aux-series三对真实A250/1Hz/5分钟测量完成，CPU合计15.40625→1.984375秒，SQLite写23531752→10942880B/对；下降87.1197%/53.4974%。第3份candidate的OS CPU0保留为原始采样而不解释为无计算；全部尾部样本保留。
- 11对场景矩阵均守恒、无1秒frame overrun，但多项110%门失败。A1000 metadata ingest p9527.8172→53.0364ms，SQLite写增加42.9851%；A50 metadata p50/p95均回归。backlog/failed和A50 counters还有native private回归。完整比值见 `acceptance-status.md`，不宣称AC7/8通过。
- 首个有具体来源的修复候选是attr全字段UPSERT导致未变化索引重写。ledger owner进行有界SQLite证据与最小选择性UPDATE实现；其它源代码owner冻结，单heavy slot，峰值对在回归处理后再测。后续构建不能沿用本版性能身份。

## Sparse metadata write repair — 2026-09-20 01:28 +08:00

- `storage.rs` now reuses the canonical session lookup to read the persisted seven-field attribute row. New rows use one fixed INSERT. Existing rows build one UPDATE from the seven internal column names only for fields whose effective value changes; values remain bound parameters. Host/process/rule/network/chain keep null-preserving merge, while policy/category remain authoritative. Chain replacement, receipt/retry, transaction and retention invalidation semantics are unchanged. Presence uses `a.session_pk`, covering legacy rows whose nullable `policy_version` is NULL.
- Bundled SQLite 3.53.2 proof: host+chain over 250 rows ×10 FULL/WAL transactions fell from 374,952B/91 frames to 210,152B/51 frames; EXPLAIN no longer opens unchanged process/rule/network/category indexes. A seven-field update stays one statement and avoids the 50ms multi-update regression. Evidence: `performance-20260919-query-stages/metadata-index-write-proof.md`.
- Fresh release gate passed before measurement: 75.5292586s; frontend 300, Rust 537+3 ignored, process 3, root 139, fmt/clippy/build/syntax/secrets pass. New executable SHA256 `73E3082E6FB90D7A8BB2C4F4A38CF82F0BC9BC3804349E81FC37685F3BA08F60`; source manifest `5F0034CB1DC64A57015CEB812249DEF2FEA87ACE559B6FFC3C782150477B6CA6`.
- Three fresh same-fixture metadata pairs compare old candidate `68F8A54C...` with new candidate: SQLite xWrite falls 33.38%/12.56%/17.64% at A50/A250/A1000, and A1000 ingest p95 falls 53.0364→31.4212ms. These are iteration gains; v5's extra indexes mean absolute baseline comparison still retains a write-volume gap. Full identities and conservation are in `performance-20260920-sparse-metadata/metadata-sparse-iteration-comparison.json`.
- The paired 10k/1Hz/1800-second peak then completed on the same frozen candidate: CPU 524.109375→332.765625s (36.51% reduction), SQLite xWrite 2263868280→680446552B (69.94%), native private p95 75292672→71614464B (4.89% lower), ingest p95 279.076→196.703ms, max 1392.337→812.643ms, and frame overruns 1→0. Both fixtures conserved traffic. Query latency is null because no report queries were scheduled; WebView, all application writes, installed soak and real worker queue remain unverified. Evidence: `performance-20260920-sparse-peak/peak-comparison.json`.
