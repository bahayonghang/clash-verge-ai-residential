# 实施记录

## 2026-09-30 批准

用户明确回复「批准实施」。按 T01 → T03 → T02 → T04 → T05 → T06 执行。每项保留修复前证据，独立记录实施、检查及未完成的正式验证。

授权限于批准计划的项目内范围。不含提交、归档、push、PR、远端 workflow、全局安装/PATH/权限/信任调整、真实私有配置及凭据/数据库操作。

## T01 本地验收通过

仅更新三个锁节点，新增独立安全门与 CI 审计。独立审查复现 omit=dev 隐藏开发依赖，已加入 --include=dev 和反例。正式产品门、docs、审计、退出传播通过；完整结果见 T01 research/review.md。RustSec 七条 warning、六个 ignored 和 hosted/native 边界保留。

## T03 本地验收通过

共享合同、Codex V1/V2、TOML 授权和只读审查已回写；新增合同检查及 39 项定向测试。独立审查修正 Kimi 任务路径冲突处理。首次 just ci 在未修改的 storage_lifecycle 测试遇 SQLITE_INTERRUPT；定向及随后完整复跑通过，触发原因未查明部分保留，不宣称修复。最终 root 182/182，docs-build 通过。

## T02 本地与实际七目标验收通过

同步 21 个旧文件；21 个新备份与覆盖前 SHA256 一致，30 个既有额外文件保留。七份生成器均 26/13/13；check=0，二次 written=0。独立复算确认共 72 文件，ignored 副本未 tracked。just ci（root182/Vitest300/Rust542+3，6 ignored）及 docs 通过。

## T04 本地与隔离 bootstrap 验收通过

新增只读入口诊断及 19 项 fixture；独立审查修正 Windows Unicode 输出与路径规范化。固定 Trellis CLI/core 0.7.0-beta.3 在独立目录同入口 version/init=0，四覆盖 hash 保持，五平台资产存在，当前 manifest 不变。just ci（Node201/Vitest300/Rust542+3，6 ignored）及 docs 通过。默认 Codex 包装器损坏与 Trellis 版本不匹配仍为 EXPECTED BLOCKED；全局环境未改。见 T04 research/review.md。

## T05 恢复测量中

T05 已按批准顺序启动。c278bb7 的 residential-monitor 子树 tar SHA256 与历史 87684724D5E5B1B3A18C0212CD594CA05F9454EDD6C28582CD879270F9749EB7 完全一致。独立执行前审查完成；baseline 五文件接入、当前二进制身份和正式测量继续执行。生产首读需检查 JSON status、wall 和 oracle，阶段测试退出 0 不足以证明报告通过。all_application_file_write_bytes 仍缺证据，AC7 不得销项。原 10 秒、1.10 比值、30 天容量、21 次和实时轮次保持。

baseline/candidate/test 构建均通过，四个隔离程序身份经独立审查，T05 AC1 通过。首次生产探针使用错误测试全名，实际 0 tests/无 JSON，验证正确失败；原始收据保留。补测一分钟 14.0227 ms 与固定 oracle 通过，明确缓存已被阶段读取；30 天 production 在 10018.5676 ms 触发 DeadlineExceeded。新回归在原算法发现 network unknown 与 SQL/既有合同不一致。经历史及跨层强审查，先在已批准 raw_fold.rs 恢复仅 process 的特殊缺失过滤，再做投影缓存；不改变门槛或 SQL oracle。

投影缓存 v1 的独立审查发现未引用字典文本会继续驻留返回 pool。新增资源 fixture 在 v1 退出 101；按需字典缓存后 raw-fold 为 5 passed、0 failed、1 ignored。两对已读缓存局部比较中，all/residential 投影中位耗时分别下降约 15.4%/10.5%；该结果不关闭正式容量或性能门。分钟去重候选经强审查后按既定文件范围实施，必须保留 dense 位图尾部补齐位语义及独立 oracle。最终身份、正式 matrix/primary、容量与完整产品门仍待验收。

主会话已把 Network 等值过滤、字典/chain 不变量、实际选中测试计数和生产 JSON 判定、缓存与完整性能门边界回写到 storage/sqlite-contract.md，并标明适用五工具。T06 捕获器与统一只读 prompt 在 research 中准备，尚未冻结候选或启动客户端推理；五工具动态结果仍为 UNVERIFIED。

分钟去重语义回归为 7 passed/1 ignored；MinuteSet 布局从 96 B 增至 112 B。候选最终完整 just ci 退出 0，用时 73.4485 秒，8 个原 PowerShell 步骤分别退出 0：Vitest 73 文件/300 项，Rust 单元 546 项及集成 3 项通过、6 项 ignored，根 Node 203 项通过。release raw/service/bench 分别为 7/53/12 项通过，忽略数为 1/3/1。writer WAL/FULL 计量已改为原连接实际读取；相同仪表同步到隔离 baseline。两边新程序身份与前后源码清单等待独立终审，分钟去重局部配对和正式性能结果另行记录。

分钟去重两对 scan 中位数仅改善约 1.35%/1.36%，单次 30 天生产报告改善约 0.18%；证据不足以支持稳定收益。主会话与独立强审查决定回退该实验的 last 字段及短路，保留独立集合/极值/重复累加 oracle、按需字典与 chain 缓存、Network 合同修复及实际 writer 配置计量。residential 投影反向波动不作因果归因。candidate-final-20260930 保留为已退实验身份，其 just ci 结果不能直接填作保留版本验收；新候选使用新目录重建复核。正式性能负载尚未开始。

A1000 边界再次核对：09-24 原 PRD R3 与本 T05 measurement-prerequisites.md 要求先有阶段方案支持 10 秒预算，再生成约 6 GB 语料。当前前提未满足，因此先执行正式矩阵、主场景和现有 A50/A250 容量，A1000 为 NOT_RUN，AC3 保持未完成。完整 30 天、21 次和 10 秒要求均保留；A250 行数外推不构成 A1000 实测失败或不可能性的证明。

保留候选 candidate-retained-20260930 经独立身份审查通过：103/108 项源码清单、前后 hash、4 个程序和 manifest 绑定一致。回退后完整 just ci 再次退出 0（80.44 秒），相关 release 检查通过。当前 raw_fold 生产段与已审查 lazy 快照一致；MinuteSet 实验已退出且独立 oracle 保留。已读缓存非空生产样本为 1 分钟 4.6001 ms、30 天 6735.2968 ms，固定 totals、连接数与 720 行 series 通过。新身份的正式 22 次 matrix 和 6 次 primary 开始执行；结果尚未判定。

正式 matrix 于 15:46:01 UTC 完成：22/22 次 native exit 0，结构、配置及样本校验无错误；整体性能门未通过。F1/F3/F4/F5/F11 PASS；F2/F6/F7/F8/F9/F10 FAIL，其中 F2 ingest p95 比 1.9233、F6/F7 native private 比 1.1368/1.1066、F8/F9/F10 SQLite xWrite 比 1.4235/1.1018/1.1787。F12 原 11 个空窗口比值全部超过 1.10，独立非空比值门仍 UNVERIFIED。原始失败保留，不能将 c278 之后的全部产品差异直接归因到本轮缓存，也不以空窗口比值单独改 SQL。primary 与现有容量继续独立采证。

matrix 退出码边界更正：实际外层 PowerShell session exit 为 1；22 个 native exit 均为 0；原 Python driver 的独立退出码未采集，标为 UNRECORDED。源代码中的 `sys.exit(2)` 仅表示预期逻辑，不是本次实测。独立审查已重算全部 F 门并核对 22 份原始 gzip 日志、参数、身份、实际 writer WAL/FULL 和样本，`review-formal-matrix-validation.json` 的 errors 为空。primary 使用新 wrapper 独立保存 driver 的实际退出码；该记录不追补 matrix 缺失的原始退出证据。

## 2026-10-03 继续实施

verge-owned-field-writes：`npm run ci` 227/227 与模板安全检查通过，`npm --prefix docs run build` 通过；PRD 前 8 项 AC 已勾选，真实 Clash Verge Rev v2.5.5+ 粘贴确认待用户执行。

T05：ferrots 相关进程仍存在（python.exe 3312/32472/32704/61752/65044，pwsh.exe 56192 等）。runner 启动断言会拒绝运行，因此未启动；matrix 0/22、primary 0/6、capacity 0/106 保持。未终止用户进程。T06 各路线的单次基本调用已用完，Claude API400、Kimi READ_PATH_NOT_ALLOWED、OMP credentialId 停止保持原状态，不自动重试。

T05 正式计量（ferrots 结束后，UTC 2026-10-04T01:50–02:51）：matrix 22/22、primary 6/6、capacity 106/106 native 全部 exit 0，无竞争负载；driver exit 分别为 2/3/3。matrix 有 8 个 F 门及 5 个 F12 空窗口场景 FAIL；AC7 CPU 0.0091 与 SQLite 子集 0.4635 PASS，全部应用文件 UNVERIFIED；A50/A250 容量全部 PASS（A250 30 天最大 7132.2 ms），A1000 NOT_RUN。T05 AC2–AC6 未完成。详见 T05 research/rebuild-20261002-4f8a8c68/formal-result-20261003.md。

T05 F6–F10 归属排查：F8–F10 每次提交的额外 WAL 页精确归属到 v5 两个 `chain_key` 表达式索引（a1000 各 +5 页）、`coverage_interval` 更新 +1 页、`data_version` −1 页；F8 另含窗口内一次 checkpoint（380928 B）。两个索引只服务 retention_day.rs 字典回收检查。F6/F7 无单调增长，峰值分配来源未查明。改进需要修改 schema/retention_day.rs 或 bench 分配计数，超出已批准文件清单，待用户决定。见 T05 research/f6-f10-attribution-20261003/findings.md。

## 2026-10-04 T05 方向 1 与方向 2 实施

用户批准 F6–F10 两个方向。方向 1：v5 DDL 用 `idx_session_attr_chain_rule(chain_key,rule_id)` 替换两个 chain 表达式索引，checksum 改为 `ledger-lifecycle-v5-layout4`，layout3 在 migrate 中原地升级；chain/rule_group 字典回收改为每个字典分块一次的跳跃枚举，与原三条 `exists` 逐条等价（新测试含 20000+20000 重复行与操作预算）。方向 2：新增 `bench/heap.rs` 计数分配器（仅 monitor-bench 注册，`RESIWATCH_BENCH_HEAP=1` 才计数）与 SQLite `sqlite3_status64` 统计，replay-facade 报告新增 `heap_phases`（未启用为 null）。storage/sqlite-contract.md 已同步。

独立静态审查无 blocker；按其 F2/F3/F4/F6c 改为语句每次枚举只准备一次、补充多 rule 的 hopless 键与大 hopless 键预算测试、修正 spec 措辞、SQLite 统计改为一次读取并重置。修正后完整 `just ci` 退出 0（Rust lib 548 passed / 6 ignored，Node 227/227，模板安全检查通过）。审查 F1 提示：复合索引仍含 `chain_key`，metadata 场景预计只减少约一半额外索引页，F8/F10 仍可能接近或超过 1.10。

`rebuild-20261004-7b8ce85b` 的 P1/P2 在修正前完成，已标为 SUPERSEDED。正式计量使用 `rebuild-20261004-e14ecd88`：P1 两边 release 构建与 smoke exit 0，变更文件集只含批准范围内 7 个文件；P2 A50/A250 语料、SQL oracle 与 4 个探针 exit 0，A250 30 天生产首读 7595.2 ms，totals 与 oracle 一致。layout4 语料比 layout3 大 5.4 MB（A50）/ 27.1 MB（A250），原因是复合索引存完整 `chain_key`。

T05 rebuild-20261004-e14ecd88 正式计量（UTC 2026-10-04T05:45:58Z–07:56:11Z，五阶段串行，无竞争负载）：matrix 22/22、primary 6/6、capacity 106/106 native 全部 exit 0，driver exit 2/3/3。F1–F4、F8、F11 PASS（F8 1.0833，窗口内无 checkpoint）；F5 1.1464、F6 1.1061、F7 1.1822、F9 1.1413、F10 1.1285 FAIL；F12 空窗口 2 PASS / 9 FAIL，非空 UNVERIFIED。AC7 CPU 0.0242、SQLite 子集 0.4636 PASS，全部应用文件 UNVERIFIED。A50/A250 容量全部 PASS，A250 30 天最大 6917.9 ms；A1000 NOT_RUN。

隔离回收容量（first-day，CHUNKS=100000）：A250 完成，7455 块，748.3 s，单块最大 433.9 ms，守恒与 quick_check 通过。A50 回收工作在前 1892 块（86.0 s）完成，之后 98108 块辅助清理不结束，test exit 101。原因：字典游标周期 9 块、覆盖游标周期 6 块，`storage_lifecycle.rs:272-282` 在删除 session 时只重置字典游标，相位差 mod 3 固定为 1，完成判定要求两游标同块为 0，因此永不满足。逻辑来自 b8a64a1，本次未改动；AUTO_DELETE_ENABLED=true 时生产维护会每秒调度不停止，当前为 false。修复超出批准范围，待用户决定。

heap 诊断：后台 stages shell 达到 2 小时时限，首次运行最后 4 次以 0xC0000142 启动失败；只改输出目录的副本重跑 12/12 exit 0。Rust 堆窗口峰值差 −33063 B / +11521 B，SQLite 峰值差 +44040 B / +52784 B，private p95 中位数差 +1.91 MB / +1.39 MB；F6/F7 差额不在这两个计数器中，来源原因未查明。WAL 重测：a1000-metadata 每次提交 57 对 64 帧，增量 7 页全部来自复合索引；a250-failed 复合索引 +3 页。详见 T05 research/rebuild-20261004-e14ecd88/formal-result-20261004.md 与 research/f6-f10-attribution-20261004/。T05 AC2–AC6 未勾选。未提交、未归档。

## 2026-10-04 回收不结束修复

用户批准修复 A50 回收不结束。`c3/retention_day.rs` 的 `cleanup_expired` 改为字典与覆盖两个扫描同一轮结束：先结束的一方停在 0，两者都为 0 才一起开始下一轮；`-1` 重扫、分页与删除条件不变，`storage_lifecycle.rs` 与 `bench/corpus.rs` 不改。新增回归测试（1085/720 行、错位游标）在修复前失败、修复后通过，retention 33/33 通过；spec storage/sqlite-contract.md 补充同轮规则。修复后 release lib 测试程序在语料新副本复测：A50 完成（2511 块，166.8 s，单块最大 377.9 ms），A250 完成（7433 块，682.6 s，单块最大 419.6 ms），守恒与 quick_check 通过。见 T05 research/rebuild-20261004-e14ecd88/formal-result-20261004.md。
修复后（含 cargo fmt）完整 `just ci` 退出 0：Rust lib 549 passed / 6 ignored，Vitest 73 文件 / 300 项，Node 227/227，模板安全检查通过。待提交文件另做宽模式密钥扫描（含解压 .gz），3995 个文件无命中。
