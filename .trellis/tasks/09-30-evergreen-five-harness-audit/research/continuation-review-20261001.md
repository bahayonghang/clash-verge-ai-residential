# 父任务继续实施的静态审查

日期：2026-10-01（America/Chicago）。审查代理：`/root/scope_review`。本轮范围为父任务和 T01–T06 的批准合同、既有审查、当前差异及 T05/T06 下一步。主会话已明确确认父任务目标；初始 hook 注入的 T05 上下文只用于 T05，不作为父任务加载证明。已读取 hook 保存文件、父任务 JSONL/PRD/design/implement、子任务合同和相关证据，并调用 `read_thread` 读取指定参考会话。

**父任务未完成。T01/T03/T02/T04 有既有本地验收；当前四覆盖交付合同发生变化。T05 正式 matrix FAIL；primary 恢复可以按已审合同继续；T06 动态验收仍未完成。** 本轮仅写本报告与小型静态核对 JSON，没有修复产品、配置、任务或 spec，没有运行测试、构建、客户端、负载或数据库扫描，没有提交、归档或远端操作。

## 批准与证据边界

用户于 2026-09-30 回复「批准实施」，依据为父 `prd.md`、`design.md`、`implement.md` 和 `research/implementation-log.md`。批准限于 T01–T06 已列项目文件和行为。全局配置、PATH、信任、真实私有配置/数据库、安装、push/PR/workflow、提交与归档仍不在本轮范围。计划中已有的性能门和回退条件保持；本报告没有增加新的正式门。

四份既有独立审查分别为：T01 `research/review.md`、T03 `research/review.md`、T02 `research/review.md`、T04 `research/review.md`。本轮复用其结论，没有重复执行其正式门或广泛重算哈希。

| 项目 | 既有证据 | 本轮判断 |
| --- | --- | --- |
| T01 | 三个 lock 节点的范围内更新；完整 dev audit；独立 CI/recipe 非零传播；本地产品、docs、审计通过 | 历史本地 AC1–AC4 PASS；不代表当前全部工作树或 hosted CI 通过 |
| T03 | 共享授权/入口/V1–V2；Kimi 路径冲突修复；负例；最终产品/docs 通过 | 历史 AC1–AC5 PASS；当前四覆盖与 checker 缺失策略须单列漂移 |
| T02 | 21 个源文件同步；21 个备份；30 个额外文件保留；实际七目标、26/13/13、二次写入 0 | 历史 AC1–AC4 PASS；不证明客户端发现或执行；本轮未重验实际七目标 |
| T04 | Unicode/路径修复；固定 0.7.0-beta.3 同入口隔离 init；四覆盖 hash 保持；产品/docs 通过 | 历史 AC1–AC5 PASS；默认 PATH 故障仍独立 BLOCKED；当前取消跟踪改变四覆盖的交付前提 |
| T05 | AC1 身份重建；局部功能/资源 oracle；保留身份产品门；22 次 matrix | AC4 FAIL；其余正式门按下述独立状态处理 |
| T06 | 合成捕获、argv、拒绝 GO 与去敏准备 | 准备不填动态 PASS；本轮未 freeze 或启动五客户端 |

T03 首次 `just ci` 的 `OperationInterrupted` 原失败仍保留；定向和完整复跑成功没有证明该次预算耗尽的根因。RustSec 六个 unmaintained、一个 unsound 警告和各 ignored/native/hosted 边界不能因后续普通测试通过而关闭。

## Findings (fixed)

### R1：恢复方案中的历史转录错误已在新合同纠正

- 文件：T05 `research/review-primary-recovery-plan.md`、`research/primary-recovery-plan-20261001.md`、`research/formal-primary-recovery-20261001.preflight-final.json`。
- 问题：旧审查 prose 把 baseline hash 的 `06BAC…` 转为 `0BAC…`，把 query 字段留空，把 metadata 100 写为 10，把 start `1800001800` 写为 `180000180`。旧文本还允许局部补跑，但没有正式跨批合并合同。
- 修正：T05 owner 已在恢复计划增加 canonical JSON/runner 的权威字段，强制完整 3 对六次，无跨批拼接。新 preflight 明确四 executable、两份源 manifest、runner/wrapper、六次 argv、fresh paths、旧 PID 缺失及独立退出收据。原审查与首份失败 preflight 保留。本审查没有修改原证据。
- 核对：新 preflight 的 103/108 源项 before/after manifest 相等且 mismatches 为空；baseline bench `06BAC94F…`、candidate bench `9604FBE6…`，runner `6C88ACCA…`、wrapper `88550BB4…` 与已审身份一致。A250/1Hz、counters/complete、metadata=100、query=0、seed=20260919、start=1800001800、30/300 秒、无 virtual-time、baseline→candidate 三对与 runner 一致。Counters 保持 metadata 不变。
- 判定：**启动合同静态 PASS，正式 primary 仍 UNVERIFIED，等待新六次与实际退出收据。** 首份 `preflight.json` 的 `Concurrent-load check not clear` 没有被覆盖。新分类只证明短样本中常驻 MCP CPU 增量为 0；用户浏览器仍活动，不能声明整机隔离。

## Findings (not fixed)

### U1：四覆盖取消跟踪改变 T03/T04 的可交付合同

- 位置：Git index 中 `.codex/config.toml`、`.kimi-code/skills/trellis-{implement,check,research}/SKILL.md` 为 staged deletion；当前 `.gitignore` 整体忽略 `.codex/` 与 `.kimi-code/`。
- 归属：取消跟踪及相应 checker/docs 策略属于共享工作树的并行改动，不是本审查所作。提交者与批准来源未由本报告确定；本审查没有擅自修复或回退这些变化。
- 事实：四个文件均仍在本机磁盘。Codex 配置仍含 `[agents] max_depth = 1`；Kimi check 仍含路径一致性核对。取消跟踪不等于本机文件被删除。
- 当前机制：`scripts/check-agent-contract.js:7–14,44–53` 把覆盖列为可选，并在 ENOENT 时跳过；`:132` 起仅在 config 存在时校验 max_depth。`tests/check-agent-contract.test.js:129` 明确接受四覆盖缺失。`docs/agents/harnesses.md:51,66,78` 已改为本机资产、缺失不阻断、按需复制。
- 合同冲突：T03 PRD AC5/design/implement 仍要求四 override 和缺 override 负例；T04 PRD AC3 仍写四个 tracked override。root quality spec 仍写 checker 拒绝 missing overrides。既有审查的四覆盖 tracked 结论不能迁移为当前干净 clone 的保证。
- 下一步：主会话先确认该外部变化的授权与归属，再统一交付合同和相应 fixture/spec，并按受影响既有门验证。未确认前保留历史 PASS 与当前漂移两个状态。本审查不自动恢复、重建或取消 staged deletion，也不将本机角色判为缺失。

### U2：正式 matrix 失败后，下一阶段方案须落实原回退条件

- 位置：T05 `design.md:34` 要求「任何守恒、deadline、取消或正式性能回归失败即回退候选算法」。`research/next-stage-plan.md:21–25` 保留失败并提出进一步诊断，但没有明确本轮 lazy 缓存的交付准入与回退判定。
- 已审正式结果：native 22/22 exit 0，结构校验通过；F2/F6/F7/F8/F9/F10 FAIL；F1/F3/F4/F5/F11 PASS；原 F12 十一个空窗 first-reader 比值全部 FAIL，非空门 UNVERIFIED。独立审查见 `review-formal-matrix.md`。matrix driver 的原始 exit 未实采；外层 exit 1 与 native exit 0 分开保留。
- 原 c278 baseline 为 schema4，retained 为既有 schema5，含多项早于本轮缓存的生产变化。总体失败不能直接证明 lazy 缓存的因果责任；局部 projection 改善也不能豁免失败或回退合同。
- frozen `candidate-retained-20260930` 可以继续作为已批准 primary/capacity 的固定**测量对象**及诊断参考，保留其 hash、失败与缓存条件。该身份不能取代原 c278 正式 baseline，不能因继续采证而取得交付准入，也不能跨身份填 PASS。
- 下一步：已批准的 primary 六次恢复和随后现有容量收据继续按固定身份完成。之后主会话必须落实范围内回退，或给出经审查的因果判定及保留依据；不能仅以原因未查明无期限保留并声明完成。现有正式门保持 FAIL，不增加替代门。

### U3：matrix 缓存执行边界须区分初始化、命中和间接影响

静态调用链为 `bench/facade.rs:185–212` → `run_uncached` → `service.rs:811` 的 `fill_raw` → `raw_fold.rs:136` 的 `load_sessions`。显式默认报告不设任何 filter，`query.rs:672` 定义 previous equal window。

seed/start 与采样固定时，current 原秒端点是 start 到 start+10/15/20/25/30/35 秒；start 对齐 UTC 分钟，current 分钟区间为空。previous 投影是 start 前一分钟；fixture 从 start 开始写 raw，前一分钟没有分钟事实。SQL `sql.rs:34` 通过分钟范围选择 session，因此投影没有 session。

`load_dict` 仍执行，新的 `DictValue` 与其 `interned=None`、unknown/DIRECT pool 常量以及空 chain map 仍初始化。逐 session 的 `DictValue::intern` 和 `chains.entry` 缓存命中路径没有执行。backlog/failed 的自动档案由 `archive.rs:723` 的已关闭 hour/day 构造，包含 previous 的 day span 为 2880 分钟，未超过 3120 阈值；窗口位于 fixture 起点之前，亦无 session。`lib.rs:593` 仍可能执行这些报告与持久化。

Network 合同修复只在 `filters.network=Some(__unknown__)` 时影响 `resolve_id`。上述默认 query filters 为空，`raw_fold.rs:733` 直接返回 `Any`，本轮特例没有触发。F2 ingest 的直接计时路径不调用缓存；F6/F7 是整个进程采样，F8–F10 是整个测量期 SQLite 写量。共享 allocator、字典 layout、先前 reader/档案及其它产品变化的间接影响仍没有被现有收据分离。静态无命中不能升级为整体性能免责。

最小范围内回退建议仅涉及 `raw_fold.rs` 生产片段：以 `research/projection-cache-20260930/raw_fold-contract-restored.rs` 为合同修复后的原实现参考，恢复 `Dict` 的四个 `HashMap<i64,String>`、删除 `DictValue`/`ChainIdentities` 与本地 chains cache、恢复 `load_sessions` 逐 session 推导和原 `identity_from_dict`/`rule_name`/`load_dict`。保留 `resolve_id` 的 `kind == "process"` 修复、3120 阈值、SQL、全部独立 oracle 与原失败证据。不整体覆盖文件或移除新增测试。未引用字典 fixture 本就保护原实现的 pool 生命周期，恢复原实现应继续满足该资源合同；执行后仍按已有 raw/service、fmt/clippy 和 `just ci` 验证，不提前声称通过。storage spec 中按需缓存的实现描述须由主会话按最终方案同步。

### U4：T06 最终捕获版本的准备收据覆盖不足

- 既有 `runtime-runs/preparation-final-7a1dcc33/verification.json` 绑定 capture SHA256 `d44c0d1e…`，证明语法、十组 argv、plan 不启动进程等合成检查。
- 同目录 `redaction-followup.json` 绑定 `6ab03f26…`，只有 ordinary_language 和两份 synthetic_auth_header 三例。该收据不覆盖原有完整准备、私有路径字符串正反例或新版本的进程捕获行为。
- 本轮静态读取中 capture 已改为另一版本；T06 owner 正在更新。暂不冻结该观察版本或把旧收据迁移为最终版本 PASS。当前路径病例文件仍标 PREPARED_NOT_RUN。
- 静态机制保留：plan 默认不启动；freeze 排他创建；execute 校验 GO、snapshot 和执行器；公开 prompt 单 argv；shell=False；去敏事件与原始流内存 hash 分开；secret output 停批；Windows Job 明确为尽力清理。snapshot 记录 missing ASSETS，没有将 missing 本身作为动态角色加载成功。因此四覆盖缺失策略即使 checker 返回成功，也不证明 T06 AC3。
- 下一步：T05 独占负载结束后，针对最终捕获版本复核已有准备合同中的语法、十组 plan/无启动、私有路径字符串、相关去敏与捕获停止分支；复用未改动且可追溯的既有合成结果，不增加新的正式门。owner 完成最终版本/收据前，状态保持 PREPARED/UNVERIFIED。真实五工具 fresh-session、实际模型/权限/hook/pull/native child、冻结候选差异及 hosted exact-SHA 都仍独立未验证。

T06 owner 后续已停写供审查，本轮进一步读取其最终五个准备文件：`runtime-capture.py`、`runtime-capture-verification.py`、`runtime-bindings-20261001.json`、计划的 10-01 节和 `runtime-runs/preparation-20261001-static/readiness.md`。捕获器 SHA256 `392b2f4c211c03f0bb9479dcbb5e6804660a8b3a82f31c618ef814d28036cf27`；验证器 `1e33d36a0f017dfe71c412c4ddb7b08376975440307b201b6f77b089b8b25a2c`。本轮只核对小文件身份，没有执行验证器。

最终实现将单工具缺入口/依赖/身份变化写成该工具 BLOCKED 收据，继续其余独立工具；源码 snapshot 改变和原捕获停止条件仍停批。新 Codex binding 限于既有 app bin 范围；`hash_files=True` 时未核验入口为 `BLOCKED_BINDING_NOT_VERIFIED`，不能启动推理。旧 c6fe 入口缺失与新 de8a 入口的 UNKNOWN version/help/hash 分开，未继承旧 0.159.2/V1 结论。OMP pwsh/bun/cli 仅影响 OMP 依赖判定。新绑定与验证器位于输出排除目录之外，参与冻结保护。定向验证器使用模拟 capture/snapshot，只在 T06 runtime-runs 下创建合成目录，并覆盖四种逐工具阻断、50 个私有路径字符串、正常授权文本/认证头及十组 prompt；实际执行仍 NOT_RUN。

**最终五文件静态准备可继续；运行准入仍等待既有准备检查、新 Codex version/help/hash 与主会话 schema 审核、候选冻结及对应 GO。** 没有发现需要在本审查写权限内修改的 T06 准备代码问题。该判断不关闭 U4 的实际收据缺口，不将最终版本标为合成或动态 PASS。

## 范围外变化与下一步顺序

`.trellis/tasks/09-30-verge-owned-field-writes/prd.md` 单独定义扩展 tun/ipv6 只读告警、示例、Node fixture、双语 docs 和 host 验证。当前 root extension、示例、regression、CHANGELOG、配置/DNS/排错文档等差异与该独立范围有关，不能自动并入 evergreen 验收或回退。该 PRD 不列四覆盖取消跟踪和 checker 缺失策略；无法仅凭该任务把四覆盖漂移归属到该变更。保留所有共享工作树变化。

先完成 primary 新批六次，并按 native/validation → 指标 → driver/outer exit 审查。随后仅按已审 capacity runner 完成现有 A50/A250 容量 106 次，不生成缺前置依据的 A1000。A1000 仍 NOT_RUN；原完整 30 天、21 次、10 秒要求保留。AC7 全部应用文件字段为 null 时继续 UNVERIFIED；SQLite 子集不能替代文件归属。AC8 10k/30分钟、24h、WebView、真实 worker 和安装态均保持独立未验证。

正式负载结束后落实 U2 的候选回退/判定、处理 U1 的交付合同归属，再完成 T06 最终准备并由主会话按已授权路线执行。P1 group/exit 算法在 `next-stage-plan.md` 中仍依赖阶段量化和强审查，没有当前实施准入；A250 外推不支持 A1000 必败结论。系统 trace、新 owner 文件、schema、调度器、安装态和远端动作仍须各自范围批准。

## Verification

- 本轮静态审查：已完成上述合同、源码路径与小型 JSON 核对；没有实际执行 lint/typecheck/tests/build/benchmark。
- Lint：本轮 NOT_RUN；T01–T04 与 retained 身份有既有 PASS 收据，仅绑定各自受检版本。
- TypeCheck：本轮 NOT_RUN；同上，不宣称当前所有工作树变化通过。
- Tests：本轮 NOT_RUN；没有用已读报告替代新正式性能结果。
- primary recovery 启动合同：静态 PASS；六次正式结果未由本报告验收。
- 当前父任务 AC4/AC5、T05/T06 整体验收：未完成。没有新增强制验收门。
- 本报告和 validation JSON 的行尾空白静态检查 PASS；validation JSON 已通过 `ConvertFrom-Json` 对应的结构读取/写入。本轮仅校验这两个小型研究文件。

## 追加复审：可选本机覆盖合同获批并同步（2026-10-01）

本节更新 U1 的后续状态，保留上文的初次发现及历史证据。用户已明确回复「保留当前不跟踪策略，更新任务合同和规范」。主会话确认本次目标为父任务与 T03/T04 合同同步；活动指针仍为 T05，审查已显式读取确认目标，没有修改任务指针、四覆盖本机文件或 Git 索引。

主会话更新父 `design.md`、`implement.md`、`task.json` 与 root frontend quality spec。T06 owner 分别更新 T03/T04 的 `prd.md`、`design.md`、`implement.md`、`task.json` 和 `research/local-override-policy-20261001.md`，共十个任务文件。owner 停写后，本审查重新读取全部最终文件并核对当前 checker、fixture、环境诊断及 harness 文档。没有覆写既有 `review.md` 或 bootstrap 收据。

**U1 的授权与文档漂移已处理；新合同静态一致，受影响的实际检查仍未运行。** 四个路径仍为可选本机资产。缺失时 checker 跳过，既有环境诊断 `project.overrides` 逐项报告；不新增 checker 输出合同。存在时 Codex config 和 Kimi check 仅校验当前定义的标记；Kimi implement/research 仅纳入读取与引用清单，非 ENOENT 读取错误失败。角色任务路径和授权条件由实际使用时核对，不声称任意正文异常均由 checker 拦截。

隔离 bootstrap 仍要求同一成功且版本匹配的入口、既定失败传播和所需生成资产。四覆盖按需、经授权复制；只对已复制文件验证 init 前后字节，不要求每个 checkout 跟踪或持有这些文件。缺少覆盖、文件存在及静态检查通过均不证明对应原生角色、hook 或配置已加载。新 quality spec 的七段可执行合同与该边界一致，没有新增 bootstrap 或客户端运行 PASS。

T03 AC5、T04 AC3 已改为待验。两份 `task.json` 保留 2026-09-30 的实施字段，并通过 `historical_verification_scope` 明确其历史边界；10-01 `approval_updates.validation_status=prepared-not-run`，任务仍为 `in_progress`。T03 AC4 已明确仅在选用本机 Codex 覆盖时保留 V1 的 max_depth=1 作用；V2 忽略该键和提示词不构成硬沙箱的说明保持。父 metadata 记录同一批准原文与文件范围，没有扩大 T01/T02、覆盖文件、产品、全局或远端操作授权。

本次轻量验证见 `continuation-review-20261001.policy-validation.json`：十四个合同/spec/研究文件行尾空白检查 PASS，三份任务 JSON 解析 PASS；同步一致性静态 PASS。Lint、TypeCheck、Tests、bootstrap、客户端均 NOT_RUN。既定定向检查和正式门等待 T05 正式负载结束；不将先前 PASS 迁移为新合同验收。T05 原 FAIL、primary/capacity 恢复、T06 运行与父整体验收状态均未由该合同同步改变。


## 2026-10-01 可选本机覆盖合同的受影响 fixture 验证

主会话明确授权在 primary 六次结束、capacity 未启动的无正式负载间隙运行 `node --test tests/check-agent-contract.test.js tests/check-harness-environment.test.js`。运行完成：55/55 PASS，exit=0，stderr 为空，wall=1.640311 秒。真实 stdout/退出码/argv/PID/时间/hash 已保存至 `local-override-policy-checks-20261001.receipt.json` 及对应 stdout/stderr 日志。

fixture 覆盖无 .codex/.kimi-code 时合同 checker 通过，以及环境诊断对缺失覆盖只报告不阻断；保留现有破坏 marker 的失败案例。本次证据支持同步后的 T03 AC5/T04 本机覆盖 fixture 条款，不替代完整 just ci/docs-build、真实客户端、bootstrap/fresh-session 或全部任务验收。没有修改配置、本机忽略覆盖、Git 索引、产品代码或 task status/AC 勾选。

同日 T05 primary 独立静态复审完整执行证据通过，106 capacity 顺序前置已满足。详见 sibling T05 research/review-primary-recovery-20261001.md。CPU 首轮 300 个中间样本和前后累计值均为 9.765625、差值 0.0，原因未查明；allapplication 缺测和正式 matrix FAIL 保留。


## 2026-10-01 Capacity 资产阻断与恢复方案（DRAFT，未执行）

### 阻断归类

静态复审 T05 research/formal-capacity-retained-20261001.preflight.json：preflight=FAIL，阶段为 fixed_authorized_identity_and_fixture_paths，recorded_native_invocations=0/106；driver/outer均NOT_STARTED。runner hash匹配，观测时旧candidate source108项无偏差。冻结candidate-monitor-db.exe、A50/A250目录/marker/database缺失，actual hash=null；未打开数据库。该记录属于资产前置BLOCKED，不能解释为106次查询失败、性能回归、数据不守恒或容量PASS。缺失原因未查明，用户确认无备份。

复审额外只核对 executable-identity.json 已列出的固定 path/source 元数据：candidate-monitor-db.exe retained与原candidate-target两路径都不存在；candidate-monitor-bench.exe相同两路径也不存在。A50/A250两个已授权合成目录不存在。没有搜索其它目录、读取私有库或重建文件。旧primary完整执行与研究日志继续有效为历史收据；当前exe缺失不能反向改变历史exit，也不能使历史结果适用于回退后源码。

### 同 hash 找回与新身份重建的区别

若以后出现用户明确提供的原资产路径，只检查该路径并核对原exe hash、marker、DB身份、完整内容与WAL条件，才可讨论沿旧合同恢复。当前没有备份且固定两路径缺失，因此同hash找回路径不具备执行条件。不全盘搜索、导入安装库或伪造production-corpus marker。

重新编译相同源码也不承诺与旧exe逐字节相同。raw_fold缓存回退已经改变当前源码；即使结果语义相同，回退后candidate必须使用新source manifest、新exe hash及新证据目录。完整generator按相同参数具有确定性分布，但corpus.rs:181–201的marker含generation_wall_secs、native_before/after、progress等实测字段，marker hash必然不能按旧值承诺；SQLite物理bytes也必须实测，不因row counts一致推定旧DB hash。新marker不能只改hash来绕过旧assert。

### 可审查的具体重建顺序

1. 等当前raw_fold最小回退、owner必要检查/fulljustci与主会话storage spec同步完成，再独立审源码和收据。禁止与这些检查并行构建/生成。Network/Process缺失合同修复、全部oracle、SQL/schema/3120分钟条件与deadline/取消语义保持。
2. 拟资产根为 `<REPO>/bench-data/t05-rebuild-20261001-<unique>`，使用现有.gitignore的bench-data/忽略规则；所有文件只能为本轮公开源码构建或合成输入。先resolve该根及子路径，确认不覆盖原证据、无已有文件，不删除任何目录。实际新路径由主会话在执行方案里绑定；本DRAFT不创建路径。源码构建snapshot、target、executables、corpus-a50-30d、corpus-a250-30d分目录；可追溯JSON/raw日志仍保存在T05 research的新批次目录。资产与研究证据各有manifest，不能只依赖Temp清理后剩下的JSON。
3. 从检查后的dirty源码创建独立candidate source snapshot，包含Cargo.toml/Cargo.lock、实际Rust与build资源输入，并记录HEAD、dirty diff、before/after manifest、rustc/cargo/target和环境覆盖。沿原真实receipt的构建组合：cargo +1.98.0 build --locked --release --target x86_64-pc-windows-msvc，独立target-dir，bin monitor-bench和monitor-db，RUSTUP_AUTO_INSTALL=0；仅列为拟执行，未执行。toolchain不可用记BLOCKED，不自动安装。baseline如需重新正式对照，使用既有c278bb7b56603001e32e353d2ee589dccef0bfe9加已审双边仪表的重建合同；新baseline输出hash也须记录，不能从历史JSON推测可重建PASS。新容量查询只绑定新candidate-monitor-db身份，不把旧primaryCPU值转移。
4. 用新monitor-bench的generate-corpus，分别A50/A250，days=30、seed=20260919、start-utc=1787184000；每个fresh empty隔离目录独立执行并保存每条native exit/raw日志/墙钟/实际marker/DB身份。不得days=0替代、不做retain/delete/VACUUM、不修改generator或数据分布。固定end=1789776000，1Hz定义、5分钟session、3跳chain及原维度基数保持。生成结束后manifest expected/actual、full_30_day_input/counts_match、schema5/layout3、WAL/FULL、quick_check与独立oracle逐项核验。
5. 新runner作为T05 research的独立文件/版本，保留旧run-retained-capacity.py与9FA8C225...身份。新execution contract显式绑定新exe/source manifest、两新marker hash、两DB hash、generator spec/hash/argv与oracle证据；不能覆盖旧executable-identity.json或复用旧hardcoded output。源文件与exe在前后都核对，读前后DB hash和零WAL分别保留。新增绑定仅反映新资产，不降低schema/计数/精确输出验证。
6. 从原corpus-a250-verification.json保留独立SQL机制。新语料先生成新只读oracle收据；同固定输入的历史A25030d精确result仍可作额外语义对照，不能把旧database_sha256_after_probes强贴到新DB。新runner分别校验host30d/1d和network30d的完整排序结果、数值类型、unknown/zeroFlow、dataVersion、时间窗、generatedUtc实际调用时段与截断说明。预读hash/完整性/oracle已触页缓存，第一process不称物理冷页。
7. 固定容量仍为106次：A50host30d21+host1d21+network30d1；A250host30d21+host1d21+network30d21。每次exit0、process wall≤10000ms，保存每条receipt/rawhash/result、整批driver/outer实际退出、失败/部分结果；无早停减少样本后称PASS。A50network1次只为原诊断。A1000完整30d/host21次门保留NOT_RUN与原阶段前提，不把A50/A250通过当全AC3关闭。

### 可量化资源与未知项

原公开记录A50/A250主库为544940032 B与1697222656 B，合计2242162688 B（约2.09 GiB），属于历史规模参考，不能保证新物理大小或峰值。A50固定minute/session/chain/receipt为2160000/432000/1296000/2592000；A250为10800000/2160000/6480000/2592000。generator专用cache为65536KiB（64MiB），每小时事务共720小时；该cache限制不等于进程总内存上限。

还需容纳独立source/target/exe、SQLite WAL/SHM/索引与临时文件、构建缓存和收据。preflight旧观测D剩余1194720800768 B、C剩余526263181312 B仅为当时快照；正式生成前记录目标盘新freebytes，不清理磁盘。生成耗时、内存峰值、临时磁盘峰值当前未知；真实生成按原native/progress保存实际资源，若磁盘/资源失败保留已提交小时与失败收据，不就地重试覆盖，也不临时缩小数据规模。A1000历史主库6124961792 B仅参考，不在本次恢复默认生成范围。

### 仍未完成的正式门

旧matrix F2/F6–F10/F12失败、非空reader独立正式证明、全部应用文件归属、A1000容量、安装态/WebView/worker/24h/peak与物理冷缓存仍按原状态保留。旧primary首轮CPU异常原因未查明；新的算法或exe不能继承旧CPU/SQLite门的PASS。是否恢复新身份的matrix/primary成效测量及其固定顺序由主会话在阶段计划中明确；不额外将allapplication强制PASS增加成106容量的前置门。产品门通过仅证明代码检查，不补齐以上性能/运行时验收。

本段为静态方案，尚未实施source snapshot/build/generate/new runner/native查询。等待主会话审查具体身份、路径、资源与执行范围后再推进。当前无Lint/TypeCheck/Test/构建/DB运行。


## 2026-10-01 回退与最终本地审查收口

最小raw缓存回退独立审查通过，108源码身份/195历史native证据hash匹配。owner完整just ci真实计数前端300、Rust546+3（6ignored）、root203；本审查docs/agent/skill/diff/七task validate全部exit0。T03新AC5已按本轮55fixture+正式gate+docs勾选，T04新AC3仅部分证明且未勾选。完整结论与边界见final-review-20261001.md，实际收据在final-checks-20261001/。本审查结束后停止全部文件写入，T06全保护snapshot由主会话冻结。
