# T04/T06 收口与 post-init 合同矩阵独立复审

日期：2026-10-02，America/Chicago。状态：PASS_STATIC_EVIDENCE_WRITEBACK；T04 为 INIT_PASS / CONTRACT_FAIL，T06 完整 runtime 验收未通过。当前派发为 T05，已按主会话明确范围读取 T04/T06/T03。T05 P1 正在准备或编译，本轮没有运行 native init、客户端、测试、just ci 或 docs-build。

本轮只新增本报告与同名 JSON。没有修改产品、任务、meta、docs/spec、四本机覆盖、Git 索引或已有证据。T04/T06 owner 已明确 STOP_WRITING。文档的新正式构建等待 P1 结束，不挪用旧 docs PASS。

## 结论与验收边界

| 范围 | 当前结论 | 独立核对 |
| --- | --- | --- |
| T03 AC5 | 2026-10-01 本地合同验收保留 | PRD 区分 09-30 历史与 10-01 的55/55 fixture/fullci/docs。meta 明确 PASS_LOCAL_CONTRACT，native_role_loaded/hosted/fresh_session 均 UNVERIFIED；task 仍 in_progress |
| T04 AC3 | PARTIAL；未勾选 | 同一0.7.0-beta.3 CLI 的 version/init均0，post-init checker1与7项错误独立保留；task仍in_progress |
| T04 旧验收 | 仅09-30历史 | meta.historical_verification_scope限定旧implementation_status/verification_summary/review；新optional_overrides_validation另列实际失败 |
| T06 | CLOSED_NO_FURTHER_CLIENTS；INCOMPLETE | 5 parent + 2 child实际尝试；3条child路线BLOCKED/NOT_RUN；五AC未勾，task仍in_progress |
| docs/spec新回写 | 静态与限定空白通过 | post-init checker结果独立；本轮docs-build/完整gate NOT_RUN，等待P1结束 |

T04 PRD新增结果段、research/implementation.md追加段、task.json新optional_overrides_validation与docs/agents/harnesses.md的bootstrap段相符。旧成功只属于原候选、原合同和原时间，不迁移为“缺失后默认生成”的新AC3通过。

## Findings (fixed)

无产品或合同修复。授权范围为独立复审和两份新研究产物。

## Findings (not fixed)

未发现本轮回写中的实质不一致。以下未完成项保持原状态，不能用文档修订关闭：

- 固定Trellis默认覆盖未满足当前checker的7项标记；没有修改默认模板、覆盖字节或checker。
- T06实际backend/有效权限/hook执行/完整child生命周期缺失，Codex中文损坏、各首失败及输入/pyc偏差保留。
- 新文档正式构建未运行；P1重负载结束后由主会话安排。
- T05新构建身份、30d生成、matrix/primary/106及其它原门不由本报告代替。

## T04 首证据与生成结果

本轮逐项复算 bootstrap-missing-overrides-20261002/evidence-manifest.json 的52项size/SHA256，全部一致。manifest本身SHA256为 `642A31D907B68DC7C163C6F5B2B640FF056673019497D5959B986E7F68D19EE1`。其中31份JSON用保留空属性名的AsHashtable模式解析，均通过；npm lock的空根package key不构成JSON损坏。

此前独立核对8条native步骤与16个stdout/stderr原字节hash全部一致。同CLI/core0.7.0-beta.3，lock59节点与原 `fd71957d4e2f20706bf8499f8cbe91e188acabb86fd94ff78febd06e94c4cfa7` 字节相同。init前checker0，init0，postchecker1；Python driver、PowerShell观察及实际exec outer均1，observer绑定session29005/chunk108e7c。环境诊断0不覆盖checker1。

7项首错误为：Kimi check的3 AUTH与2 DISPATCH标记，Codex config的2 DEPTH说明。四文件init前均missing、未选用、未复制，init后由CLI生成；saved默认内容/hash保留。没有用本机覆盖修补该批次。

67个复制文件前后相同，218个新增文件数已独立重算。7个必要平台资产存在，11个具名资产/覆盖命中候选忽略规则。保护9项及index/HEAD/branch前后相同。全局配置/trust/私有文件的无修改结论来自执行命令边界，未新增内容读取或作全盘逐字节证明。最小67文件候选不能称完整fresh checkout，也不证明原生角色已加载。

## docs 与 quality spec

`docs/agents/harnesses.md`的bootstrap补充明确：
匹配入口及init0只证明初始化；缺失可选覆盖本身不阻断init；该固定版本会生成默认覆盖；生成后必须另跑候选checker，首失败保持可见，不自动复制本机覆盖、不放宽checker。真实例子分别列INIT_PASS/CONTRACT_FAIL、67文件、5+2错误与fresh-checkout限制。

`.trellis/spec/frontend/quality-guidelines.md`的Optional Local Harness Overrides矩阵明确：
缺失覆盖允许当前合同检查；已有Codex config/Kimi check违反受检标记则失败；Kimi implement/research仅校验读取，不声称正文合同全部通过。init0加无效默认标记必须记INIT_PASS/CONTRACT_FAIL。原字节保存、复制保留、版本不匹配BLOCKED、角色加载与字节/结构检查分别说明。没有因固定版本默认生成失败降低marker要求。

该矩阵与checker现有条件和本次首失败相符。本报告不运行checker或native来重复确认已有结果。

## T06 结果与索引

result-20261002.md、evidence-index-20261002.json与task.json新verification_runs相符。已核对7个attempt的native exit、stop marker、event count与原receipt一致，各native/driver/actualouter引用存在。5 parent、2 child、3未运行child分开，outer0不覆盖TIMEOUT/API/参数/敏感停止。Codex旧session17160实际toolouter0有owner原查询收据，先前跨代理Unknown查询的边界仍记录。

Grok只有一次原生spawn请求，未获得tool_result/child ID/完成；同主session且parent_tool_use_id=null的读取不能归child。Codex只有空receiver wait，没有原生spawn证明。模型选择元数据、实际child模型/权限、自然语言拒绝、OS强制与hook分别记账。纯输出协议不覆盖角色持久化。

旧source snapshot `9a31d79c85465e3213e6f364f670ebac4a45911897e385c74e5909a740778cf9` 已retired。meta旧source_sha是历史HEAD，不能代表未提交候选；本报告和结果/meta在release后写入，后续T04/T05/source与docs变化不得沿用旧快照宣告PASS。保护snapshot一致只覆盖指定范围；pyc生成偏差及单文件授权清理、Job best-effort和无绝对no-orphan边界均保留。

## Verification

- JSON：相关task/meta/index及T04清单内31份JSON解析通过。
- Evidence SHA/size：52/52一致；限定task结果/清单/首失败引用存在；T06七attempt字段/引用核对无错误。
- Whitespace：8个相关回写文件无尾空白；限定git diff --check退出0。Git既有LF→CRLF提示保留，不变更证据换行。
- Lint/TypeCheck/Tests/native init/client/just ci/docs-build：本轮NOT_RUN。T05 P1期间未启动重负载。
- 写入：仅本报告及同名JSON；现已STOP_WRITING。

