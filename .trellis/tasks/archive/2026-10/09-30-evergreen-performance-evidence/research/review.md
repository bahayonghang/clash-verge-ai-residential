# T05 独立阶段审查

日期：2026-09-30。审查代理：`/root/t02_review`，强模型 `trellis-check`。任务保持未完成。

本轮仅写 `research/review.md`、`research/review-*`，并按主会话派发给 `preflight-review.md` 添加日期勘误。没有运行 build/test/benchmark，没有打开或 hash corpus，没有修改产品、instrumentation、任务指针或其它代理文件。以下测试和测量结论来自实施代理已落盘的原始收据；本代理核对了日志、JSON、身份和源码。

## 当前状态与最终输入（2026-09-30，正式准入）

**保留候选准入 PASS，正式matrix性能门 FAIL，T05 未完成。** 当前生产代码保留 Network 合同修复与 lazy 字典缓存，已回退 MinuteSet 新增状态并保留独立 oracle。保留候选的完整检查、身份、实际 writer WAL/FULL smoke 与两个非空单样本已审查。22次正式matrix原始收据及指标重算通过核验；F2/F6/F7/F8/F9/F10及F12全部11个空窗口比值失败，F12非空门UNVERIFIED。primary正在按原计划独立采证；容量执行器已静态准入，排在primary之后独占。

| 身份 | 当前有效索引 | SHA256 前缀 |
| --- | --- | --- |
| baseline bench | [保留输入身份](candidate-retained-20260930/executable-identity.json) | `06BAC94FD88E37DC` |
| candidate bench | 同一身份文件 | `9604FBE6ED24C6DC` |
| candidate db / tests | 同一身份文件 | `8EE782EDE8F6F619` / `74ECF63AC02659AF` |
| candidate源码manifest | [108项before](candidate-retained-20260930/candidate-source-before.json)，与after及当前源码相同 | `860165F3183AC8BF` |
| 独立准入核验 | [review-retained-candidate-validation.json](review-retained-candidate-validation.json) | `errors=[]` |

`candidate-final-20260930` 是已退回的 MinuteSet 实验身份，不能作为当前正式输入。完整64位hash及原始路径见身份文件。

| AC | 当前状态 | 未完成边界 |
| --- | --- | --- |
| AC1 | PASS | 当前重建身份已核对，不宣称复现历史exe字节 |
| AC2 | PARTIAL | 语义oracle与非空单样本通过；完整生成语料的结果集合和边界验收待终审 |
| AC3 | NOT_COMPLETE | A50/A250容量待正式结果；A1000阶段前提未满足，NOT_RUN，原30天/21次/10秒要求保持 |
| AC4 | FAIL | matrix执行证据有效，但6个F1–F11门及11个F12空窗口比值失败；非空门UNVERIFIED |
| AC5 | IN_PROGRESS | primary独立采证中；全部应用文件归属为独立UNVERIFIED，SQLite不能替代 |
| AC6 | PARTIAL | 当前代码与smoke保持WAL/FULL、schema及删除门；10k/安装态/WebView/worker等独立证据未完成 |

当前终审：[正式matrix与失败归属](review-formal-matrix.md)、[22次独立复算](review-formal-matrix-validation.json)、[容量执行器准入](review-capacity-runner.md)。matrix退出边界为native 22×0、owner报告的外层PowerShell为1、Python driver原exit未单独实采；不能把代码预期2写成实测。

历史阶段索引：[缓存前失败](#初轮结论缓存实施前)、[cache-v1](#缓存实现复审2026-09-30-1449-utc)、[lazy资源修复](#lazy-资源修复复审2026-09-30)、[MinuteSet实验](#minuteset-实验与执行器修正复审2026-09-30)、[执行器修正](review-formal-runner.md)。旧章节中的状态仅属于该时点，原始失败和局部结果均保留。

## 初轮结论（缓存实施前）

- **AC1 PASS**：本轮固定历史源码的隔离构建身份已恢复；三个构建退出 0，四个复制程序的身份核对一致。该结论不要求新程序与历史 exe 逐字节相同。
- **一分钟样本 PASS，有边界**：固定窗口 `[1787184000,1787184060)` 的生产查询为 `ok/Raw`，14.0227 ms，250 个连接，upload 3497、download 11725，1 行 series。该结果属于此前已读取页缓存的单个样本，不证明物理冷读，也不关闭 AC2 或 F12。
- **30 天生产样本 FAIL**：固定窗口 `[1787184000,1789776000)` 返回 `DeadlineExceeded("report query")`，10018.5676 ms。native test 退出 0 不能改变该判定。
- **首项投影缓存方案可在批准范围内实施**：限定 `raw_fold.rs`，必须保持下文的空值、fallback、原始 chain、过滤、snapshot 与 deadline 合同。测量支持投影优化实验，尚未证明具体字符串操作占全部投影耗时。第二项 scan 预计算仍未通过本轮实施审查。
- **正式门仍未通过**：同窗口 F1–F12、实时主场景三轮、各 21 次容量、全部应用文件归属、完整 AC8 与安装态证据不能由本轮阶段结果替代。

## Findings (fixed)

### R1：预审遗漏精确测试全名

- 文件：`preflight-review.md`；原执行证据：`stages-20260930-initial/a250-minute-production-first.receipt.json`。
- 问题：预审没有检出执行方案中的错误名称 `c3::service::tests::isolated_nonempty_report_stage_proof`。2026-09-30 13:49:49 UTC 的调用退出 0，实际为 0 passed / 0 failed / 0 ignored，未生成 JSON。
- 修正：预审追加 2026-09-30 勘误，写出 `c3::service::raw_stage_probe_tests::isolated_nonempty_report_stage_proof`，要求执行前用同一 exe 的 `--list` 精确核对；执行后同时核对退出码、恰好 1 个测试通过、JSON 存在与语义。两个 PowerShell 代码块仅做语法解析，0 个解析错误。
- 保留：原 `production-first-validation.json` 的 `pass=false`、错误 argv、原始 gzip 日志和“0 tests / 无 JSON”均未改写。随后 `a250-minute-all` 已读取语料，因此未经本轮主动预热的生产 reader 机会已经丢失。补测使用新目录，明确标记此前阶段已读取数据库。

适用工具：Claude Code、Codex、Grok Build、Kimi Code、OMP。测试名称匹配与输出语义检查属于共享执行合同，任一 harness 的 exit 0 都不扩展该证据。

## 构建与输入身份

`review-build-identity.json` 记录前轮独立核对：baseline 原始 100 项、接入后 103 项、candidate 108 项、dist 36 项；五文件 baseline patch 与预审哈希一致，构建前后 source manifest 一致，实际 Rust 源码均被覆盖。三个构建的原始压缩日志、解压日志及可读副本一致。

| 构建 | 退出码 | wall 秒 | 边界 |
| --- | ---: | ---: | --- |
| baseline monitor-bench | 0 | 119.9713918 | 历史 revision 加批准的五文件 instrumentation |
| candidate monitor-bench / monitor-db | 0 | 74.7457162 | 冻结未提交源码 manifest |
| candidate library tests | 0 | 19.5409788 | `--no-run` 仅编译；运行证据另列 |

历史 revision 为 `c278bb7b56603001e32e353d2ee589dccef0bfe9`。子树 tar SHA256 为 `87684724D5E5B1B3A18C0212CD594CA05F9454EDD6C28582CD879270F9749EB7`，与历史输入一致。实际使用 Rust/Cargo 1.98.0、`--locked --release --target x86_64-pc-windows-msvc`、`RUSTUP_AUTO_INSTALL=0` 和分离 target 目录。两侧 `.rustc_info.json` 确认 rustc commit `88d9e12ae178fab0fb5cc050a94da85685d449ea`、LLVM 22.1.8。

`review-manifest-boundary.json` 补齐四个 excluded `gen/schemas/*.json` 的判断：锁定依赖 `tauri-build 2.6.3` 的 `acl.rs:116–137,400–433` 从 capabilities 与插件 manifest 生成 ACL/capabilities 输出；`tauri-utils 2.9.3` 的 `acl/schema.rs:309–343` 生成 Windows/desktop schema。构建 dep-info 包含 capabilities，未包含这四个生成文件。源 capabilities/default.json、Tauri 配置、build.rs 和 lockfile 已在输入 manifest 中。

构建收据记录 VS 安装路径、SDK 10.0.26100.0；保存的 build output 另显示 Resource Compiler 10.0.10011.16384。尚未记录实际 linker/cl.exe 的完整二进制版本。dist 复制仅满足 Tauri 嵌入前提，不证明 UI 同源或运行通过。baseline smoke 的实际 DB schema 为 4，candidate 为 5；这是固定历史 revision 与当前 revision 的差别，本轮不修改任一侧生产 schema。

## 独立收据核对

新收据 `review-stage-evidence.json` 核对 15 条命令的 gzip 原始日志 SHA256、可读副本、原始退出码、时间与路径，以及四个程序当前哈希。结果 `errors=[]`。记录的本任务命令串行，均晚于三条构建；这些收据没有证明整机不存在其它用户负载。

`--list` 包含正确名称，缺少错误名称。补测四条 probe 均实际运行且通过恰好 1 个 Rust test，并生成 JSON。生产判定另读 `production_first_read.status`，因此长窗口仍为 FAIL。

| 运行证据 | 通过 | 失败 | 忽略 | 判定范围 |
| --- | ---: | ---: | ---: | --- |
| raw-fold 聚焦测试 | 3 | 0 | 1 | 当前冻结 test exe 的原始日志 |
| report service 聚焦测试 | 53 | 0 | 3 | 当前冻结 test exe 的原始日志 |
| bench instrumentation 聚焦测试 | 12 | 0 | 1 | 当前冻结 test exe 的原始日志 |
| 双侧 A8/3 tick smoke | 各退出 0 | — | — | 各 3 次提交，字节守恒，相同 fixture hash；virtual-time |

smoke 的 FULL 从程序 JSON 核对；WAL 从实施驱动对 smoke 专用库读取后保存的 `smoke-validation.json` 核对。该 smoke 不包含正式时长、规模、WebView 或真实 collector 队列。

| 已读取缓存的样本 | 生产 wall ms | 分段 projection ms | 分段 current scan ms | 判定 |
| --- | ---: | ---: | ---: | --- |
| 一分钟 default-auto | 14.0227 | — | — | 固定一分钟摘要 PASS |
| 30 天 default-auto | 10018.5676 | — | — | 生产 deadline FAIL |
| 30 天 all raw-fold | 无生产预算 | 3894.5494 | 5421.8366 | 诊断完成 |
| 30 天 residential raw-fold | 无生产预算 | 2754.6016 | 4403.7208 | 诊断完成 |

30 天生产 probe 的 native 进程总 wall 为 29393.4375 ms，包含生产失败后的无预算拆分；JSON 的 `raw_query_ms=19321.9103` 属于该后续诊断。不得将进程总时间当成单次生产查询时间，也不得用后续 raw-fold 分段耗时和覆盖实际生产失败。

all 使用 local 时区、previous_equal_window=true；residential 使用 UTC、comparison=None。补测 all 的 previous scan 为 8.4517 ms。两个查询定义不同，初轮/补测的页缓存状态也不同，不能将总差值仅归因于筛选口径。production probe 使用 `now=end`，与 CLI 真实 now 保持分开。

all 的 2160000 连接、upload 151200035、download 507599545、720 行 series，与其初轮诊断相同；residential 的 1620000 连接、113399990、380699625 也两轮一致。上述一致性未提供独立完整 corpus SQL oracle、完整 rank/coverage 或物理冷缓存证明。

## 首项候选的源码审查

审查对象：`measurement-findings-and-candidate.md` 的第一项。审查时源码为 `build-20260930/candidate-source-before.json` 所记录的 raw_fold.rs；本节是实施前判断。

现有 `raw_fold.rs:154–174` 每个 session 再次 intern 字典文本，并复制 chain 文本后派生两个值。`:747–769` 明确区分空字典值与缺失 ID，Rule 的 fallback 独立于 chain identity。投影阶段 2.75–3.89 秒支持对该路径做最小实验；没有分配 profiler 证据证明优化一定达到容量门。

| 输入或机制 | 必须保持的输出或反例 | 必要验证 |
| --- | --- | --- |
| process/network/category 的 ID 为 NULL 或悬空 | unknown identity，`missing=true` | grouped totals/归因；process/network 的 unknown 过滤；category 保持现有字典查找语义 |
| 上述字典行存在且值为 `""` | unknown identity，`missing=false`；空白字符串不得自动 trim | 同时包含空串行和悬空行；检查同名 unknown 排名与不同 missing 归因 |
| Rule 行存在且值为 `""` | 空 Rule identity 保留；只有缺失 Rule 才回退 DIRECT | 空字符串 Rule 过滤与分组 oracle |
| 不同 kind 使用同一数值 dimension_id | process/network/category/rule 分别查找 | fixture 使用相同数字、不同文本，不能改成无 kind 的统一表 |
| 同一 `DIRECT` 原始 chain 配不同 Rule | 无 hop override；每个 session 分别应用其 Rule fallback | 同一链同时配 REJECT、空 Rule、NULL/悬空 Rule |
| `node>` 或 `node>  ` | Chain identity=node；`last_chain_hop=None`，Rule 走各自 fallback | Chain 与 Rule 同时按 grouped SQL 比较 |
| 原始 chain 仅为 `>` | chain identity=None，但非空原始 exit key 仍保留 | primary_exit/exit_mixed 显式断言 |
| 不同完整 chain 得到同一派生 identity | 允许共享派生 pool 值；原始 chain_key 不合并、不 trim | 等 download 时仍按完整 chain_key 升序决定出口 |
| chain 缓存内容 | 仅缓存 `(chain_identity, last_chain_hop override)`；不得按 chain 缓存最终 Rule | 相同 key、不同 fallback Rule 的反例必须通过 |
| 字典预先 intern | 可以改变内部 pool ID；名称排序、sentinel 与输出不随 ID 排序 | 全维排序/过滤/exit tie；记录新增 pool/cache 内存，性能以实测判断 |
| cache 生命周期 | 仅同一次 SessionIndex 构造；不得跨连接或 snapshot | 同库更新 metadata 后新 reader 看到新值；现有 snapshot 回归 |

`raw_fold.rs:950–1075` 的 `assert_sql_match` 比较 totals、归因、series 与排名五列，未比较 `primary_exit` 和 `exit_mixed`。原 fixture 已包含部分悬空 process/network、空白链、单跳、legacy residential、跨窗口及负分钟；没有完整覆盖本次复用缓存的反例。`service.rs:3835–3989` 已有 host/process/rule 出口、空链和 tie 测试，应保留并重跑；新缓存 fixture 仍需独立出口断言。

第一项实现的回归要求：

1. 在 `raw_fold.rs` 新增固定合成 fixture，覆盖上表；保留独立 grouped SQL oracle，不用新缓存实现生成 expected 值。
2. 重跑现有 all/group/filter、排序/缺失归因、跨窗口、稀疏 ID、负分钟、零字节和空桶检查；保留 legacy residential EXISTS，不能把 predicate join 到外层。
3. 保留 `WINDOW_SCOPED_PROJECTION_MAX_MINUTES=3120`、长窗口全表投影、原始分钟索引扫描、所有 SessionFact 语义、整份报告的 snapshot/cancel/deadline owner。使用现有 service 取消与跨桶 deadline 测试；不得为缓存初始化重新计时。
4. fmt、clippy、聚焦测试及任务规定的最终 `just ci`；冻结改动后源码 manifest 和新的 bin/test exe 身份。
5. 用相同已读缓存定义重测一分钟、30 天生产及 all/residential 阶段，保留首次失败；若核算或正式门回归，按任务回滚边界处理。

实施适用五套 harness；规划、核算语义和结果判读保持强模型。较低成本模型只能补已固定 expected 的 fixture、整理日志或核对清单，不自行改变算法、阈值、数据规模、缓存条件或授权范围。

## Findings (not fixed)

- **R2 / P1：30 天生产查询触达 deadline。** 上述 10018.5676 ms 是实际失败样本。首项候选尚未实施/验收，本代理没有产品 ownership。
- **R3 / P1：完整正式性能与等价证据不足。** AC2 的全部 group/filter/rank/coverage oracle、AC3 的 A50/A250/A1000 正式容量及各 21 次、AC4 的同窗口 F1–F12、AC5 的实时三轮均未由当前收据完成。
- **R4 / P1：AC7 全部应用文件写入归属缺失。** smoke JSON 仍有 `all_application_file_write_bytes=null` 与 `spool_write_bytes=null`。SQLite xWrite 或进程 I/O 不能替代该门；额外系统事件采集需要既定范围审查。
- **R5 / P2：物理冷缓存、24h 安装态、10k/1Hz/30分钟、WebView 与后台 worker 证据未齐。** 不执行安装态或缓存驱逐，不把未测状态写成通过。
- **R6 / P2：后续 runner 还应固化最终测试计数。** corrected 驱动检查 `running 1 test` 与 JSON，独立审查确认当前日志最终为 1 passed / 0 failed / 0 ignored；后续驱动应直接校验最终摘要与 `kind`，继续保存语义 FAIL。驱动由实施代理拥有，本代理仅回报。

## AC 状态

| AC | 当前判定 | 未完成部分 |
| --- | --- | --- |
| AC1 | PASS | 历史 exe 字节复现与 UI 同源不在本 PASS 声明中 |
| AC2 | PARTIAL / UNVERIFIED | 固定一分钟摘要通过；全部等价、coverage 与新候选取消/deadline 尚未验收 |
| AC3 | UNVERIFIED | 尚无既定 CLI 查询组合和 21 次正式收据；default-auto 30 天失败另行保留 |
| AC4 | UNVERIFIED | F1–F12 同窗口完整对照未跑 |
| AC5 | UNVERIFIED，归属缺口存在 | 实时三轮与全部应用文件写入门未通过 |
| AC6 | PARTIAL / UNVERIFIED | 本阶段未改生产 gate/schema；完整 AC8、安装态与 worker 证据未齐 |

## Verification

- Lint：本审查未重跑；尚无本次候选修改后的 lint 证据。文档/收据另做局部格式和 JSON 检查。
- TypeCheck：三条 release build 收据退出 0；本审查未重跑构建，未将该结果扩展为完整 frontend TypeCheck 或 `just ci`。
- Tests：已有聚焦 raw-fold 3/3、service 53/53、bench 12/12 通过，忽略项分别为 1/3/1；本审查未运行测试。新候选测试尚未执行。
- Evidence：15 条现有命令日志、程序身份与补测 JSON 独立核对 `errors=[]`；一分钟样本 PASS，30 天生产样本 FAIL。
- Document checks：两个 PowerShell 示例仅语法解析，0 错误；不访问 corpus。收据 JSON、行尾空白及 scoped diff 检查见 `review-document-validation.json`。

此报告是本轮阶段审查结果。没有提交、归档、push、PR、远端 workflow 或安装动作，也没有把 T05 标记为完成。

## 缓存实现复审（2026-09-30 14:49 UTC）

本次仍由实施代理独占产品与测试文件。审查代理仅只读 diff 和已落盘日志，写本报告及 `review-cache-implementation.json`；没有运行构建、测试或 corpus 负载。审查源码 SHA256：`9d5c68063d80d855e3c91c71c550c0430f1a9b92a32ab7b91a093c99ce940609`。

### 语义恢复单独验收

`network-unknown-review.md` 追踪了原公开合同与 Git 历史。实施代理先把 resolve_id 的 unknown 特例限定到 Process，保存 `raw_fold-contract-restored.rs`。该快照 SHA256 为 `e975d19f9ff52993efcc537dd36b5e1f8d7cd763770291817cf649533ae63edf`；独立比较确认其生产代码相对 HEAD 仅有此 guard 变化。

| 收据 | 原始退出码 | 实际测试结果 | 判定 |
| --- | ---: | --- | --- |
| `original-semantics` | 101 | 0 passed / 1 failed / 0 ignored | 原算法 Network unknown 与 SQL 不一致，FAIL 保留 |
| `network-contract-restored` | 0 | 1 passed / 0 failed / 0 ignored | 同一 fixture 在合同恢复后 PASS |
| `cache-raw-fold-tests` | 0 | 4 passed / 0 failed / 1 ignored | 缓存改造后的 raw-fold 聚焦回归 PASS |

三条日志 gzip 内容哈希均与原始 receipt 一致。缓存版本的完整 `#[cfg(test)]` 模块与合同恢复快照逐字相同；新 fixture 及 SQL oracle 没有为缓存结果改写。合同恢复的 PASS 发生在缓存实施之前，性能优化不能替代该 bug 的根因与修复记录。

### 缓存与新 fixture

- Dict 仍分别保存 process/network/category/rule，值改为 pool ID。Process/Network/Category 空串映射 unknown 但保留存在性；NULL/悬空 ID 返回 missing=true；Rule 空串独立 intern，缺失 Rule 才用 DIRECT。
- chain cache 的 key 为完整原始文本的 pool ID，value 只有 chain identity 和可选 hop override。最终 Rule 按各 session 的 rule_id 独立 fallback，因此同链不同 Rule、尾空 hop、仅分隔符链的差异保留。
- SessionFact 的原始 chain_key 没有被派生值替换；排名与出口仍通过 pool 中的原始字符串排序，内部 ID 的变化没有进入排序键。窗口阈值、SQL、分钟扫描、service snapshot/cancel/deadline owner 均未改变。
- 14-session fixture 覆盖相同数字跨 dictionary kind、空串/NULL/悬空 ID、同链不同 Rule、单跳、尾空 hop、仅 `>`、原始空白差异、稀疏 session_pk 与负分钟。六种 grouping 和固定 filters 逐项对照原 grouped SQL。
- 新 host 出口 oracle 直接在 session/minute/attr 上按完整 chain_key 聚合 download，排除 NULL/trim 空白链，再按 download 降序、原始 key 升序排序。该 oracle 未调用新缓存。每行分别比较 primary_exit 与 exit_mixed；等流量的 `" hop > exit "` 与 `"hop>exit"` 明确保留前者并标 mixed。现有 service 的 Process/Rule 出口回归仍须运行。

### 未引用字典值的影响

新 load_dict 会把受支持 kind 中未被当前 session 投影使用的字典文本加入 SessionIndex.pool。原实现的 Dict 在 load_sessions 返回时释放，pool 只保存实际使用过的文本。此处是已确认的对象生命周期差异。新缓存可能增加短窗口或空窗口仍存活的 pool 分配；当前没有该差异的 runtime 内存证据，不将其记为已发生的阈值回归，也不声明内存改善。

未引用文本会使部分 `intern_existing` 查找从“缺少 ID”变为“找到 pool ID”。随后 Host/Rule/Chain 过滤仍与实际 SessionFact identity 比较，未引用文本不会自行生成 session 或 ranking，因此静态审查未发现假命中路径。Process/Network/Category 的 exact 过滤仍经字典 ID 查找，未改用 pool 存在性判断。

当前 fixture 没有单独构造“只存在于字典、未被任何 session 引用”的文本。可在原 fixture 补该输入与同名 Host/Rule/Chain 过滤，验证空结果仍与 SQL 一致；已将补证建议交实施 owner。不得在正在构建或测量时并发改源码。正式 native private、短/空窗口和长窗口证据仍按既定门核验，不因静态语义通过而降低要求。

### 性能与完整检查状态

14:49 UTC 时，新缓存的 fmt/clippy、service/bench 和新的二进制身份正在由实施代理处理。本代理未重复执行。尚无此缓存源码对应的生产 minute/30d、all/residential 阶段或 native private 实测结果。初轮 14.0227 ms 与 10018.5676 ms 属于缓存改造前的程序身份，不能标为新缓存的性能。

本次结论为“合同恢复已通过同一 fixture；缓存静态检查与 raw-fold 聚焦回归通过；性能和完整门待证据”。T05 继续保持未完成。

## Lazy 资源修复复审（2026-09-30）

审查快照：`projection-cache-lazy-20260930/raw_fold-lazy.rs`，SHA256 `F6495AC9C01D99FB5F4D4B950AE3F255A59FF4FDC46ACA230FFD9A59E994C936`。该值与 lazy source manifest 相同。复制测试程序 SHA256 为 `EFBD43FFB61FC8D0CB290F116AC6F0FB6A5888C000332FC4C4E093429D3E37DC`，已重新核对文件；未运行程序。

原 `resource-before-lazy` 为 exit 101、0 passed / 1 failed，具体失败是 `unused-process` 被保留在返回的 pool。lazy 版本把 String 与 `Option<u32>` 留在局部 Dict，首次投影引用时才 intern；Dict 和 chain 缓存在 `load_sessions` 返回时释放。未使用字典文本不进入返回 pool。固定 unknown/DIRECT 常量不代表未引用字典文本驻留。

各 kind 仍为独立 map。Process/Network/Category 仅在已有字典项的空值上使用 unknown，missing 标志保持 false；NULL/悬空项仍为 true。Rule 空串保留，chain override 缺失时才按 session 的 Rule fallback。一个 DictValue 在同一 Pool 内复用 ID；Pool 仅追加且不重新编号，因此后续 intern 不改变已有 SessionFact/chain 缓存的 ID。最终过滤仍检查 SessionFact，未发现同名 Host/Rule/Chain 假命中路径。

`unreferenced_dictionary_values_do_not_enter_returned_pool` 显式加入四种未引用文本，逐项断言 pool 中不存在，并分别按同名 Host/Rule/Chain 过滤，对照原 grouped SQL 的空结果。原 14-session 语义回归继续保留。lazy raw-fold 收据为 exit 0、5 passed / 0 failed / 1 ignored；这是资源合同与局部语义的通过，尚非 native private 性能门。

### 收据与局部性能

`review-cache-lazy-validation.json` 核对 36 条收据、3 个复制 exe、源码快照及比较摘要，`errors=[]`。已核对 gzip 原始日志哈希、真实测试计数、固定窗口标量与串行时间。串行收据不能证明整机无其他用户负载。

- cache-v1 fmt/clippy、bin/test build、scoped diff 收据均 exit 0；raw-fold 4 passed/1 ignored，service 53 passed/3 ignored，bench 12 passed/1 ignored。这些结果不自动适用于后续 lazy/MinuteSet 最终身份。
- cache-v1 已预读缓存的一分钟生产样本：before 4.8246 ms，cache 4.1297 ms；二者均 ok/Raw，250 connections、3497 upload、11725 download、1 行 series。
- cache-v1 已预读缓存的 30 天生产样本：before 7178.2956 ms，cache 6730.6508 ms；二者均 ok/Raw，2160000 connections、151200035 upload、507599545 download、720 行 series。它们不能覆盖初轮 10018.5676 ms 的 deadline 失败。
- lazy 两对阶段样本：all projection 中位数 3229.7031 → 2732.31565 ms，ratio 0.8459959；scan 3816.21445 → 3844.9296 ms，ratio 1.0075245。residential projection 2615.66925 → 2341.11745 ms，ratio 0.8950357；scan 3419.09855 → 3387.0564 ms，ratio 0.9906285。

上述阶段探针禁用生产 deadline，数据库已被读取，未清除 OS page cache。all 使用 local timezone 与 previous 等长窗口；residential 使用 UTC 且无 previous。before 身份早于独立 Network 修复，不能据此把所有差异归因于缓存。各组标量与固定生成语料一致，但没有从摘要重建完整排名/coverage，因此不把局部比较升级为完整 AC2/AC3/F12。

### Spec 与正式门

已完整只读核对主会话新增 storage spec 的 `Raw projection identity and probe evidence` 七节。Network 字面过滤、字典存在性、链 fallback、未引用文本寿命、选中测试与生产结果的双层判定均与当前源码一致；文档没有把阶段证据声明为正式性能 PASS。

正式执行器预审见 `review-formal-runner.md`。原 22 次 matrix/6 次 primary 参数和三轮合计比值正确；实际 WAL/FULL、数值和计数、F12 非空、最终 manifest 绑定仍待修正后确认。T05 保持未完成。

## MinuteSet 实验与执行器修正复审（2026-09-30）

`review-minute-experiment-validation.json` 已核对25条实验阶段收据、四个复制程序及两组before/after源码manifest，`errors=[]`。实验candidate在 `candidate-final-20260930`；该目录名称不表示批准为最终交付。

MinuteSet代码语义审查通过。独立HashSet oracle保留dense尾部padding语义，重复分钟不跳过字节/session累计；空窗口、极值和乱序保持原行为。测得MinuteSet 96 → 112B，Option<i64> 16B；未单独测得Acc布局或原生进程内存影响。

两对all/residential scan分别只有约1.35%/1.36%的名义改善；单个30天生产样本6696.6586 → 6684.6417ms，约0.18%。一分钟为4.6843 → 4.8528ms，两端固定总量与series一致。该有限证据不足以支持稳定收益；residential projection的7.36%增加发生在非修改路径，原因未查明。强审查建议回退新增last状态，保留lazy与Network修复、独立oracle和原始实验记录，不为名义改善继续扩大局部样本。

实验candidate检查：release raw-fold 7 passed/1 ignored、service 53 passed/3 ignored、bench 12 passed/1 ignored；fmt及release clippy退出0。完整 `just ci` 经保留原PowerShell命令的记录壳执行，八个步骤分别退出0；前端73文件/300测试、Rust546 passed/6 ignored及process3 passed。这些结果绑定实验身份，不能替代回退后新身份的检查。

执行器初审问题已由实施owner修正。原writer初末实际WAL/FULL在测量区间外读取；两侧instrumentation相同。finite/nonbool数值、身份、形状、样本数、manifest及F12独立UNVERIFIED机制均已核对；五项纯验证测试退出0。详细结论和脚本SHA256见 `review-formal-runner.md`。

主会话按审查意见收窄了storage spec的未引用文本措辞：仅禁止因为字典条目存在而额外保留pool文本，允许预置常量和实际identity共用文本。回读确认与lazy源码一致。

当前待办是按主会话调度回退MinuteSet、冻结保留候选的新身份并完成相关复测；然后才运行原正式28次replay。AC1之外的完整门继续未完成。

主会话已明确本轮先执行原matrix/primary及现有A50/A250容量。A1000生成的原阶段前提尚未满足，记为NOT_RUN；完整30天、既定21次与10秒要求保持，AC3仍未完成。不得用A250四倍外推推断A1000不可能通过，也不得把未运行写成FAIL。后续阶段设计保留为未完成计划，不扩展本轮算法或文件范围。

## 保留候选准入终审（2026-09-30）

`candidate-retained-20260930` 的 raw_fold 生产段与已审 lazy 快照逐字相同。只保留新增的独立集合与累计测试；last状态和内部状态断言均已移除，布局回到96B。源码manifest前后相等且与当前文件逐项匹配：baseline103项，candidate108项。四个复制exe及manifest绑定均一致。

新身份的八份检查收据及八个Just原PowerShell步骤全部finished/exit0；full just ci为80.4352秒，release raw-fold 7 passed/1 ignored、service 53 passed/3 ignored、bench 12 passed/1 ignored，fmt/clippy通过。审查代理核对原始日志和测试计数，没有重跑检查。

双边A8/3tick实时时间smoke流量与提交守恒，同fixture/平台/时区。原writer初末均为wal/2，顶层FULL。历史baseline schema为4，当前candidate为5，各自初末保持；这是既有身份差异，本轮未改schema。该smoke只验证计量接入。

最终测试exe上的两个非空probe均实际选择1 passed/0 failed/0 ignored，并返回生产ok/Raw。一分钟4.6001ms，250连接、3497 upload、11725 download、1行series；30天6735.2968ms，2160000连接、151200035 upload、507599545 download、720行series。原始日志、固定窗口、exe路径和JSON已核对。这些均为已预读缓存的单样本，不关闭21次容量、同窗口F12或物理冷读边界。

storage spec新增的实际writer PRAGMA字段与计时边界已只读核对，与facade实现一致。正式执行器保持已审SHA256 `6c88acca5994c02266f262fbc185ebf9d09de1ab9ec96baa302993a0cbf381e9`。已通知实施owner进入原matrix22次→primary6次的正式独占阶段。正式结果须分别更新AC2–AC6；一个总体exit0不能覆盖缺失指标或未运行控制。
