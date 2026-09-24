# 实施验收状态

本文件记录本次已授权源码与隔离验证的结果；不把源码测试、短smoke或进程I/O混作安装态性能验收。完整命令见 `final-checks.md`、`benchmark-harness.md` 和 `run-isolated-bench.ps1`。

| AC | 已有证据 | 尚需证据 / 状态 |
| --- | --- | --- |
| AC1 | 实际facade提交、同步archive tick和reader查询工具；各迭代源码/二进制哈希；SQLite成功xWrite和native进程指标；aux-series主场景及11对矩阵已测 | WebView、所有应用文件写入、实际collector/background worker与真正冷缓存未覆盖；后续源码需新身份实测 |
| AC2 | 完整Rust检查含3600虚拟tick、到期/积压/失败退避、时钟回退、时区边界、worker公平/暂停恢复回归 | 安装态长期行为未验证 |
| AC3 | metadata实际DML、canonical merge、policy/generation、exact retry/FIFO、零流量ID重用与10000 churn回归通过 | 大负载性能另列 |
| AC4 | 内部无spool、operation取消隔离/清除、snapshot租约/TTL/配额、隐藏恢复/单在途/导出持有回归通过；前端300项通过 | 安装态窗口/WebView的隐藏与恢复没有操作验证 |
| AC5 | 完整日/跨表破坏/取消回滚回归通过；A50/A250/A1000首日raw分别72000/360000/1440000条，经完整日验证后退出；三档恢复均守恒、可回收辅助行归零、quick_check通过 | 原失败/主动中断与恢复计时分别保留；恢复耗时不冒充从空staging开始的整日耗时 |
| AC6 | NULL-ended退役证明、活跃/pending保护、receipt并集/边界/空epoch/重放回归；45天A4 raw/session全清且守恒；修复完成信号后同hash副本推进至end+397天，精确维度及1024个孤立字典全退出、daily core保留 | 45天输入老化不冒充396天持续输入容量；完整期限模型和安装态另列 |
| AC7 | aux-series三对5分钟实时AB完成：OS CPU合计15.40625→1.984375秒（87.1197%）；SQLite xWrite下降53.4974%；三对ingest p95均改善，native private p95均≤基线110% | 第3对candidate的OS CPU增量为0，不解释为零计算；SQLite子集不能代替全部应用文件写入门；11对矩阵多项ingest/query超过110%，尚未通过 |
| AC8 | aux-series `just ci`在命令级隔离TEMP/TMP下通过：frontend300、Rust534+process3、root139；30天三档生成/行数及首日保留恢复通过；45天fixture的手动VACUUM释放390746112B且守恒/完整性通过 | A250/A1000 30天报告超时，未通过完整查询容量；paired 10k/1Hz/30min、native+WebView、24h安装态soak等仍待完成 |

`AUTO_DELETE_ENABLED=false`。现有安装程序、真实库、路由与凭据未修改；未提交、发布或归档任务。需要安装态验证时，应基于可审阅构建单独获得安装/重启授权。

## 源码与构建身份

`performance-20260919/build-identity.json` 固定源码archive、候选源码manifest和可执行文件SHA256。候选是未提交工作树；不能仅以base HEAD表示它。Rust release `monitor-bench` / `monitor-db` 构建成功（37.19秒），release library测试程序已预编译；开始计时前所有Cargo进程已结束。

## 初始smoke

`performance-20260919/smoke-{baseline,candidate}.json`：A8、3个虚拟tick、metadata不变、档案齐全，fixture hash一致，均3次提交且字节守恒。native CPU样本过短且量化到0，不用于优化比例。SQLite xWrite为131840/140080B；此小样本新增正向coverage写入也包含在候选中，不证明主要场景写入下降。

## 首轮性能发现与下一迭代

首对A250实时结果：baseline/candidate CPU为7.015625/0.1875秒，下降97.33%；SQLite xWrite为23531752/14901296B，下降36.68%，未达到50%目标。ingest p95为14.4176/11.3861ms，两者流量守恒、无1秒overrun。另两对仍使用已复制的相同exe继续；源码后续修改不会冒充这份二进制的结果。

三对现已全部完成：合计CPU下降96.1074%，每对SQLite xWrite均下降36.6758%；native private p95未超过各自baseline的110%。完整汇总为 `performance-20260919/comparison-summary.json`。这是第一版的已测结果，新的layout2尚待gate和独立测量，表中整体AC7/8不能据此改为通过。

静态追踪显示candidate普通tick仍写11个WAL frame，包括明细、7个健康规则实例、receipt、watermark及正向coverage。健康状态的评估时间与证据保持原语义。下一迭代仅改可重复的物理记账成本：coverage索引不含每tick变化的结束时间；receipt以data_version为整数主键并保留bundle唯一约束；当前version从持久receipt与legacy floor读取，移除每commit对独立singleton的重复写入。预计每tick可少写约3页，但只有实际新构建回归/AB结果才能证明。实施与新验证未完成，旧构建结果仍单独保留。

## 最终layout3验证身份

在layout2完整容量生成中发现重复minute索引；最终layout3仅通过未发布v5删除该索引，并保留已发布迁移。真实v4迁移/查询计划回归和完整 `just ci` 通过（79.66秒）；详情见 `layout3-checks.md`。fixture生成器cache改动仅影响批量构造，不代表生产内存优化。

最终测量目录为 `performance-20260919-layout3/`，源码manifest SHA256为 `5EE8BA6B57D9E293F94C52927B51742746D9F2FA0B0A78E614C0A00727C3C629`；bench SHA256 `3B1DCE49B9B0A0F7DCB393D3AA3B0938D8B23B16CCACB76D2ED88AFE73D2DE34`。旧布局证据完整保留，不覆盖、不混称最终结果。

## Layout3完整容量与查询实测

三档30天生产schema生成完成且全部行数匹配。A50/A250/A1000主库分别为544940032/1697222656/6124961792B，生成耗时67.27/158.30/384.11秒；完成时WAL与freelist均0。相比layout2，A50减少41947136B、A250减少213262336B。生成器cache只影响构造性能，不作为产品内存证据。

查询由真实CLI调用完整报告路径，含CLI启动；没有驱逐OS cache。每个成功窗口有首读1次+重复20次；相同窗口失败2次即停止重复。结果在 `performance-20260919-layout3/capacity-summary.json`：

| A | 30天报告 | 单日报告重复p95 | 全窗口network计数 |
| --- | --- | --- | --- |
| 50 | 21/21成功；重复p95 8405.04ms | 265.60ms | 324000，等于期望 |
| 250 | 两次10秒deadline，exit7 | 1389.66ms | 超时，计数未知 |
| 1000 | 两次10秒deadline，exit7 | 5744.88ms | 超时，计数未知 |

不能将非零exit改为零计数或宣布全部容量查询通过。共享raw路径存在totals/attribution重复扫描；当前仅融合这两项精确聚合并验证取消检查频率，schema保持layout3，后续可复用这些fixture但必须记录新exe身份。30天大窗口超时是已知限制；retention/45天/VACUUM、最终AB及峰值仍待完成。

## Query1改进与首个保留容量故障

Query1重复p95：A50 30天5983.26ms（原8405.04ms），单日A50/A250/A1000为249.69/1115.18/4853.48ms。五个代表性CLI输出除生成时刻外逐字段一致；30天A250/A1000仍deadline。详见 `performance-20260919-layout3-query1/`。

A50完整库过期首日测试在163.30秒后失败：1892个chunk中，72000条raw已通过完整日聚合/复核后退出；辅助ledger cleanup在254.81ms处被250ms预算中断，剩余14400个可回收session和601000条应裁剪receipt。测试exit101、complete=false；不能宣布保留门通过。quick_check=ok，观察到staging最大82468864B（每32块采样，非瞬时峰值）。DB为571006976B，其中217563136B可复用，尚未VACUUM。

独立生成公式旁证确认raw+已完整删日core合计upload30240060/download101519550B严格守恒，原始结果为 `retention-a50-independent-conservation.json`。EXPLAIN确认候选、raw引用和DELETE均使用索引；固定1000-session事务包含3000次未缓存DELETE prepare及多索引删除，硬超时后cursor一起回滚，无法前进。正在复用prepare并在现有硬预算之前持久化已处理前缀；取消仍必须回滚，预算不放宽。

## Cleanup1通过恢复容量测试

Cleanup1完整 `just ci` 通过（77.54秒）：frontend300、Rust527+process3、root139。独立review无未修复发现。源码manifest与bench/CLI/test/desktop SHA256在 `performance-20260919-cleanup1/build-identity.json`；desktop仅构建/复制，未运行或安装。

保留原失败A50库，以源/副本相同SHA256 `736ED958718A1FF79B9B57210EF42CF4613011BA5C43F271B2123F32FF6854D1` 创建恢复fixture。新release测试90.75秒通过，2106个chunk、最大226.39ms、complete=true。upload30240060/download101519550B前后严格相同，expired raw=0、可回收session=0、eligible receipt=0、quick_check=ok；417600个仍有raw引用的session保留，receipt由701000降至100000。一次完成全量inventory校验。

恢复后主库571006976B，其中active289222656B、freelist281784320B；可复用空间尚未作为OS已回收空间。原首日构建/核对/删除与本次续跑分别有证据，不把恢复耗时称完整首日处理耗时。A250/A1000首日完整运行及45天、VACUUM、最终AB/峰值仍待完成。

## A250完整首日与容量驱动的重试边界

A250首日运行639.24秒、7363个chunk后记录exit101/complete=false。360000raw已验证删除，upload151200035/download507599545B前后严格守恒，quick_check=ok；receipt已减至100000且eligible=0。session已持久回收13905个，另58095个可回收者仍在；最后ledger_cleanup在809.43ms记录中断，先前chunk约185/189ms。整个run最大chunk1198.92ms，staging观测最大243585024B。这是已推进cursor之后的预算中断，区别于最初A50固定事务永不前进。

产品将非用户取消的interrupted映射为DeadlineExceeded，将busy/locked映射为StorageBusy；worker在后续tick继续。容量工具原先遇到第一个这类可重试让出便退出。正在让驱动复用产品分类，仅在原max_chunks上限内记录并继续这两类错误；不放宽产品deadline，不忽略用户取消、空间、完整性或其它错误。驱动的加速重试不等于产品60秒退避的墙钟吞吐。原始失败结果保留，并离线复制相同hash的A250状态用于恢复证明。

## 保留驱动完成观察修正

retry版A250恢复驱动于13:07:30Z启动，748.74秒后主动停止，exit=-1。已校验所停PID92348的exe路径属于本任务隔离测试；原始日志、停止原因和现场保留在 `performance-20260919-retention-retry/`。没有成功结果JSON，不计作保留通过。

只读代码审查确认测试驱动缺陷：仅在 `chunk.more_pending=false` 时观察session/receipt扫描游标，但该标志也受独立coverage/dictionary页扫描影响。session到尾归零可能发生在标志为true时，下个tick又被推进，导致完成事件反复漏记。产品不使用此测试完成判定。修复方向为每个已提交的辅助清理tick分别锁存完成状态，保留至允许最终inventory；仍以实际可回收数/守恒/完整性判定通过，不缩小验收范围。

## 大范围报告的剩余查询成本

只读审查发现host报告仍分别执行summary、series、ranking和Top-N出口四个raw窗口。CLI家宽过滤要求attr关联，不能直接移除该join。可能的下一步是用每bucket索引范围替代全窗口series GROUP BY，或先取Top-N session再通过既有session/minute索引seek出口流量；两者都须先取得实际SQL阶段耗时/计划，再保持distinct、空bucket、范围裁剪、host权威身份、家宽归属与出口tie/mixed语义进行等价测试。它们目前只是源码候选，不是已实施优化或10秒目标保证。本轮已测查询改进仅query1的共享summary扫描，失败窗口不被隐藏。

## 最终本地构建及A250恢复

最终helper完整gate通过，81.27秒：frontend300、Rust529+process3、root139；2项默认忽略仍显式单列。详情见 `retention-driver-checks.md`。Release产品/bench/CLI构建43.93秒，release test编译18.17秒。冻结源码manifest SHA256 `3322AF300A0DEFEBD4E3F21D35C91EFDEF783E69DFC31DE50CBA2223CCC90773`，二进制身份见 `performance-20260919-final-local/build-identity.json`；桌面exe仅构建，不代表已安装或运行。

从原cleanup1失败状态再次创建相同hash的A250副本，修正驱动99.36秒通过：2162个chunk、0重试、一次完成inventory。upload151200035/download507599545B前后严格守恒、expired raw=0、可回收session=0、eligible receipt=0、quick_check=ok；2088000个有raw引用的session保留，receipt为100000。最大chunk361.85ms，仍不能将250ms VM预算声称为严格墙钟上限。恢复后DB1697603584B，active1407135744B、freelist290467840B；未VACUUM，不声明物理文件已释放。

## A1000保留热路径检查

最终本地构建的A1000首日测试于13:29:12Z启动，661.25秒后由主会话验证唯一exe路径并主动停止；exit=-1、无结果JSON，不称容量通过。中止时仍在build阶段，最后定时采样为620928/1440000条首日raw已加入staging；原库、WAL与进度日志保留。

只读审查确认新保留路径对rule/chain每raw每遍共4次未缓存identity lookup，完整A1000首日双遍约1152万次prepare；publish/audit也对有限的三张聚合表逐行prepare。`apply_raw`的member/aggregate语句本来已缓存，真实精确成员写入仍需执行。下一版仅复用prepared statement，保留全部bindings、读取/校验、trigger、事务与取消，不缓存查询结果，也不更改删除条件。

工具的dbstat每32块采样成本仍计入总时间，未计入各chunk.wall_ms。原A250首次运行的driver wall616.10秒、chunk合计516.02秒（差额100.08秒含采样和最终报告/检查）；恢复run对应87.80/78.26秒。差额不能全部归因dbstat，也不能把总elapsed视作生产60秒调度下的维护吞吐。

## Retention-cache构建与A1000恢复通过

最新源码manifest SHA256 `EF0619E162AD78AA63F2AB164ADA8817BEE393E0D1BCEDBC461E22E95A33A1E4`；bench/CLI/test/desktop身份在 `performance-20260919-retention-cache/build-identity.json`。完整 `just ci` 使用命令级隔离TEMP/TMP通过（51.49秒）；默认TEMP两轮root rename EPERM及一次不适合的ESM包内temp诊断分别记录在 `retention-cache-checks.md`，根因未证实，没有修改用户全局设置或无关root实现。

A1000恢复测试exit0，外部elapsed3885.82秒；30352个chunk、1次可重试ledger deadline（307.61ms且writer_autocommit=true）、2次完成inventory。整个run最大chunk1372.95ms；driver内部wall3847.81秒，chunk合计2900.11秒，差额包含采样及最终检查，不作为生产调度吞吐。新缓存相对旧版的加速比例没有同状态A/B证据。

upload604800000/download2030399685B前后严格守恒，expired raw=0；8352000个有raw引用的session保留，可回收session和eligible receipt均0，receipt保留100000，quick_check=ok。最后一个仍有raw的UTC日默认报告成功（3433.06ms），这是默认全流量报告，不替代CLI家宽30天容量。staging观测最大681545728B；结束主库6559043584B，有效页5698715648B、可复用860327936B，未VACUUM。

45天A4完整fixture生成22.76秒：minute259200、session51840、chain155520、receipt3888000，全部计数匹配；DB423993344B、WAL/freelist均0。其全raw过期与后续老化结果见下。

## 45天全量过期、维度老化与隔离VACUUM

`performance-20260919-retention-cache/retention-a4-45d.json`：外部395.33秒、10845块、0重试，quick_check=ok。259200条raw与51840个无引用session全部退出，保留100000条受保护receipt；upload3628890/download12182020B严格守恒。平均chunk35.74ms、最大422.56ms；主库428007424B，有效页39985152B、可复用388022272B，staging采样最大8404992B。保留45天coverage_daily和90条分类daily core。附带小时报告在36.62ms明确返回capability_unsupported：多个身份汇总不足以恢复逐小时去重活跃分钟；不把该报告算成功，也不输出伪零。

离线相同hash复制后推进至fixture end+397天（不是396天持续写入），`retention-a4-aged-dimensions.json` 在71.74秒、2565块、0重试完成；raw/hourly/daily精确维度和coverage_interval均0，90条daily core与45条coverage_daily保留，字节守恒且quick_check=ok。过期高基数报告明确返回capability_unsupported（0.80ms），没有返回伪零。有效页10526720B，可复用417480704B。

此老化run的complete=true暴露了完成证据缺口：字典仍有host800/process120/chain60/rule_group40/network4共1024个无引用值，另1个category由daily core合法保护。源码审查确认prune_finish丢弃辅助扫描pending信号，跨日prune后旧扫描游标也不能代表最终引用删除后的完整扫描；driver未检查孤立字典。必须修复并重跑，不能只凭退出码把AC6改为通过。旧结果保留为原身份证据。

另从45天清理后的原库创建相同hash离线副本 `corpus-a4-vacuum`，真实CLI手动VACUUM在1.07秒exit0，内置integrity_check通过。物理主库428007424→37261312B，实际释放390746112B；独立生成公式在VACUUM前后均确认upload3628890/download12182020B完全相同。证据为 `vacuum-a4-measurement.json`、`vacuum-a4.json`、`vacuum-conservation-{before,after}.json`；只操作隔离副本，用户库未变。

## 辅助完成信号与series最终构建

修复后采用同一个45天清理库（DB SHA256 `0BB7CBB963C99C2141A016F8A71D066680DC9FFD7A686D29658B250B10873B1F`）再次复制并老化。新结果 `performance-20260920-aux-series/retention-a4-aged-summary.json`：65.35秒、2574块、最大142.12ms、0重试；byte守恒、quick_check=ok。minute/session/attr/chain/hourly/daily维度/coverage_interval均0；daily core90、coverage_daily45、receipt100000保留。字典只剩daily core引用的1个category，两个辅助cursor均0。此旁证补全旧run未检出的1024个孤立字典；不覆盖旧结果。

同一版包含逐桶raw series改进：A1000保留29天单对旧/新35.60→26.30秒，696行摘要一致；源SQL阶段与边界详见 `performance-20260919-query-stages/series-stage-findings.md`。totals30.57秒和rank53.54秒仍分别超10秒，不能把局部改善说成完整报告通过。

独立完整gate77.61秒通过，见 `auxiliary-series-checks.md`。Rust release构建36.88秒、test构建1.45秒，source manifest SHA256 `38C9D3B07969494FDC5EEB8EA24BEB2FB16DC0CB90CB4B40E024EDE258DBB106`（本版含Rust与frontend源码）。二进制冻结身份见 `performance-20260920-aux-series/build-identity.json`。该身份的三对实时AB结果见下。

## 最新三对实时主场景

`performance-20260920-aux-series/comparison-summary.json`：A250、1Hz、计数全变化/metadata不变、档案齐全，每份30秒预热+300秒实测；六份fixture_hash完全相同，均300次提交且字节守恒。这里不包含展示查询、WebView或真实collector/后台worker。

| 对 | baseline/candidate CPU秒 | baseline/candidate SQLite写B | baseline/candidate ingest p95 ms | baseline/candidate native private p95 B |
| --- | --- | --- | --- | --- |
| 1 | 6.734375 / 1.78125 | 23531752 / 10942880 | 14.2259 / 11.4576 | 9846784 / 9265152 |
| 2 | 5.125 / 0.203125 | 23531752 / 10942880 | 14.0508 / 11.2950 | 9310208 / 9551872 |
| 3 | 3.546875 / 0.0 | 23531752 / 10942880 | 13.7228 / 11.0800 | 9625600 / 9551872 |

合计OS报告CPU下降87.1197%，SQLite成功xWrite下降53.4974%。第3份candidate的GetProcessTimes读数没有推进，原始0保留，但不据此宣称没有计算或精确100%降低；前两对也分别下降73.55%和96.04%。candidate最大ingest285.77ms；baseline第3份最大1813.55ms并有1次1秒frame overrun，candidate三份均0次；这些尾部样本不删除。没有查询样本的指标在汇总中为null，不把原始空统计的0当查询延迟。

以上通过主场景的CPU与SQLite子集写入目标，仍不是AC7完整应用文件门，也不替代AC8安装态/峰值门。随后完成的场景矩阵如下。

## Aux-series场景矩阵与metadata回归

同一冻结二进制完成11对场景，每份5秒预热+30秒实测，含6次首reader报告与6次重复报告。每对输入hash一致，22份均字节守恒，均无1秒frame overrun。下面是candidate/baseline的比值；大于1.10表示未满足相应门，不能因短窗口或绝对耗时小就免除失败。原始JSON及汇总均在 `performance-20260920-aux-series/`。

| 场景 | ingest p95比 | 首reader报告p95比 | native private p95比 |
| --- | --- | --- | --- |
| A50 unchanged | 0.90112 | 1.24271 | 1.00636 |
| A50 counters | 0.87258 | 0.98177 | 1.11950 |
| A50 metadata | 1.16685 | 0.99515 | 1.00141 |
| A250 unchanged | 0.75213 | 1.21161 | 0.93972 |
| A250 counters | 0.80458 | 1.09189 | 0.91846 |
| A250 metadata | 1.07185 | 1.20477 | 1.03715 |
| A1000 unchanged | 0.64305 | 0.91867 | 1.09439 |
| A1000 counters | 0.68126 | 1.18608 | 1.09907 |
| A1000 metadata | 1.90660 | 1.17420 | 1.07822 |
| A250 backlog | 1.69123 | 1.12476 | 1.22189 |
| A250 failed | 1.07246 | 0.91310 | 1.20164 |

A1000 metadata的ingest p95为27.8172→53.0364ms，SQLite写7880872→11268472B，CPU1.125→1.265625秒。A50 metadata的p50也由4.2503→5.0945ms，说明155ms尾部单样本不能独自解释p95回归。其WAL恰由每frame12页升为18页；新增DB写372736B与checkpoint形状相符，但尚无逐页归属证明。

源码检查发现fixture只改变host与chain，当前attr UPSERT却在SET列出全部7字段；实际捆绑SQLite按SET列集合决定索引维护，会重写未变化字段对应索引。正在以隔离EXPLAIN/WAL实验验证复用已有canonical SELECT并仅更新实际变化字段的最小修复。保留chain相关必要索引，不增加新表/配置/长寿命影子状态；修复收益须新构建实测。10k峰值对暂缓至该回归处理后，现有失败不覆盖。

### 矩阵中其它回归的当前边界

只读源码追踪显示 backlog/failed 的 native private 增长与 candidate `ArchiveScheduler` 保留完整 `PendingArchive` 队列相符：调度刷新为最多约1116个小时/日任务分配 `ReportQuery`、fingerprint 与字符串；baseline按需遍历并只返回下一项。现有JSON只能证明两场景candidate p95增加约22.19%/20.16%，不能证明泄漏或归因全部来自队列。下一步应在隔离基准中记录队列长度、capacity和拥有字符串字节，再决定是否压缩待处理描述；不得凭p95直接改调度器。

首reader报告的矩阵窗口从同一分钟开始，raw range实际为空，因而A50/A250/A1000及backlog的约1.12–1.24比值主要反映固定开销与小样本，不能代表非空报告聚合。应补一组非空分钟范围，分别记录plan snapshot、finalized days、durable version、raw query与reader close阶段；重复报告与失败场景已保留，不降低既有110%门槛。

A1000 counters/metadata在无积压时也出现首reader p95回归（分别约+18.61%/+17.42%），而metadata native private仅+9.91%/+7.81%，说明报告固定开销与小样本波动不能只归因待处理队列。源码候选包括 `finalized_days`、durable version聚合及生命周期/deadline检查；在非空窗口阶段计时前不改动这些语义。

## Sparse metadata迭代实测 — 2026-09-20 01:28 +08:00

新源码通过独立完整gate（75.529秒；frontend300、Rust537+3 ignored、process3、root139），release `monitor-bench` 身份与source manifest写入 `performance-20260920-sparse-metadata/build-identity.json`。三对新candidate均使用与旧candidate相同的fixture hash、5秒预热+30秒实测、metadata全变化、archive complete与每5帧报告；两侧均守恒且无frame overrun。该证据比较两个candidate迭代，不把新candidate冒充对旧baseline的通过。

| 活跃数 | 旧candidate SQLite写B | 新candidate SQLite写B | 下降 | 旧→新 ingest p95 |
| ---: | ---: | ---: | ---: | ---: |
| 50 | 2,597,568 | 1,730,400 | 33.38% | 11.3908→9.3507ms |
| 250 | 3,936,112 | 3,441,712 | 12.56% | 13.9969→14.0672ms |
| 1000 | 11,268,472 | 9,280,712 | 17.64% | 53.0364→31.4212ms |

A50/A250/A1000新candidate相对旧candidate的native private p95分别为5,910,528/7,995,392/13,201,408B；A1000虽高于旧candidate的12,816,384B，仍需后续矩阵复测，不能把单次样本解释成泄漏。相对旧baseline的绝对SQLite写入仍受v5新增索引布局影响，因而新candidate的A50/A250/A1000分别为1,730,400/3,441,712/9,280,712B，不宣称AC7的baseline门已通过。完整逐项身份和守恒字段在 `metadata-sparse-iteration-comparison.json`。

## 10k 峰值配对 — 2026-09-20 02:28 +08:00

新candidate与未修改baseline各运行30秒预热+1800秒实测、A=10000、1Hz、counters全变化、archive complete；fixture hash一致，均守恒。完整证据为 `performance-20260920-sparse-peak/peak-comparison.json`。

| 指标 | baseline | candidate | 变化 |
| --- | ---: | ---: | ---: |
| native CPU seconds | 524.109375 | 332.765625 | 下降36.51% |
| SQLite xWrite bytes | 2,263,868,280 | 680,446,552 | 下降69.94% |
| native private p95 | 75,292,672 B | 71,614,464 B | 下降4.89% |
| ingest p95 | 279.076 ms | 196.703 ms | 下降29.52% |
| ingest max | 1,392.337 ms | 812.643 ms | candidate更低 |
| frame overrun count | 1 | 0 | candidate无overrun |

该峰值命中CPU/SQLite子集、native内存和commit延迟方向，但仍没有WebView、全部应用文件写入、真实后台队列或安装态24小时证据；`query_every_frames=0`，报告延迟为null。AC8的完整安装态门仍未通过。

## 续作 Rust 复核 — 2026-09-20

专项复核未发现应继续修改的 Rust 产品代码。`ArchiveScheduler` 已是紧凑描述符、有界 `VecDeque`、单在途、失败退避与队尾公平推进；档案回归22项通过。raw series 逐桶查询的3项回归通过。证据和命令见 `research/rust-continuation-20260920.md`。

该复核同时确认大范围报告的 series 局部优化不等于完整报告通过：29日 A=1000 series 为35,604.236→26,297.359ms且结果摘要一致，但 totals/attribution 为30,567.243ms、host ranking 为53,544.078ms，仍超过10秒目标。没有在缺少阶段计划和语义等价证明时重写这两个SQL阶段；AC7/AC8、WebView、安装态soak与完整应用文件写入证据继续保持未完成/未验证状态。
