# FlClash 0.9.1 空规则集输入兼容修复

## 目标

解决用户最上方红色提示：`[AI-家宽] rule-providers 必须是对象，拒绝覆盖已有配置`。FlClash 0.9.1 传入顶层 `rule-providers: null` 时，脚本应正常完成配置变换，同时保留非法类型拒绝、规则集所有权及引用冲突校验。

## 背景与证据

- 用户确认版本 0.9.1，选择“脚本”及“家宽”后报错，栈含 `validateRegexDnsProviderOwnership`；补充截图显示侧栏“代理”消失，“配置”及订阅卡片仍在。
- `clash-verge-ai-residential.js:574-575,1645-1648,1939-1943`：null 不满足对象判断，main 在代理注入前执行该校验。`tests/regression.test.js:624-644` 当前明确要求拒绝顶层 null。
- 官方 0.9.1 缺省规则集经 Go JSON 可成为 null；生成失败后应用空配置，导航根据可显示分组决定“代理”入口。固定版本来源、行号和因果链见 [根因报告](research/root-cause-report.md)。用户实际输入和模式未现场采集。
- 初次调查基线：dev、HEAD 929b23d、产品工作区干净；16 场景、12 项旧测试及 Go 编码探针通过。这些证明旧行为，不代表本次修复通过。

## 授权与状态

2026-10-19（America/New_York）用户要求完善方案、先审阅、不要实施，并以红色失败为重点。本轮仅编辑本任务规划和元数据，保持 `planning`。不修改源码、测试、spec、CHANGELOG、本机 FlClash 或 local 配置；不提交、不推送。

## 需求

| ID | 要求 |
| --- | --- |
| R1 | 顶层 null 与字段缺省具有相同变换结果；覆盖 main(config) 单参数和现有双参数入口，不依赖宿主检测、新开关或关闭正则功能。 |
| R2 | 顶层数组、字符串（含空串）、数字（含 0）、布尔值继续拒绝；保留名称下的 null、非规范 provider、额外字段、未知 payload 仍拒绝。 |
| R3 | rules/sub-rules、复合 DNS policy、fake-ip-filter、sniffer 的专用 provider 冲突引用仍拒绝，含顶层 null 输入；规范托管 provider 允许重建，普通用户 provider 保留。 |
| R4 | 成功/失败都不修改原输入；保留原机场节点/组/未知规则，生成正常家宽节点/组。四组正则开关投影正确、重复运行稳定、全关不新建空映射；显式 {} 保留既有行为。 |
| R5 | 公开及临时虚构 TOML 生成脚本都覆盖兼容路径；零新依赖/路由域名，同步相关 spec 和中文未发布变更记录。 |

## 修复验收（结果见 research/validation.md）

| ID | 需求 | 可观察结果 |
| --- | --- | --- |
| F-AC1 | R1, R4 | 四组 Vertex/Cursor 开关下 null 输入正常返回，完整输出深等于同条件缺省输入，原组/节点/普通规则保留并产生家宽组/节点；含单参数 main。 |
| F-AC2 | R2 | 数组、空/非空字符串、零/非零数字、两种布尔值均拒绝；同名 provider 内部 null 等非法形态仍拒绝。全关不能绕过，错误不泄露私密值。 |
| F-AC3 | R3 | 保留已有引用冲突矩阵，补 null 顶层映射配合冲突引用；规范 provider rerun 通过，普通用户 provider 与规则不被覆盖。 |
| F-AC4 | R4 | 成功/失败输入深等于快照；null 输出重复执行深等于第一次；provider/policy 同步，全关不含 rule-providers，显式空对象仍保留。 |
| F-AC5 | R5 | 现有 renderer 集成测试用虚构 TOML 输出脚本，在子进程以 null、单参数 main 执行；已有五组开关/default 场景通过，模板/TOML 不变。 |
| F-AC6 | R1–R5 | 新 null 正例对独立旧模板确实失败，对修复模板通过；窄测试、npm run ci、项目要求的 just ci 分别记录。工具/网络阻塞、产品失败和未运行分开报告。 |
| F-AC7 | R1, R4 | FlClash 0.9.1 同一测试订阅应用新脚本不再出现目标类型错误，应用成功且原代理分组可见。无法实机执行时为 NOT_RUN，结论限于“代码验证完成、实机待验”。 |

## 范围与排除

计划五文件：`clash-verge-ai-residential.js`、`tests/regression.test.js`、`tests/sync-local-config.test.js`、`.trellis/spec/frontend/state-management.md`、`CHANGELOG.md`。机制见 [设计](design.md)，顺序与验证见 [执行计划](implement.md)。

不改 FlClash 导航/失败回退，不实现“任意错误保留旧配置”；不放宽通用对象判断、不清空合法 provider、不改路由/DNS 算法或上游选择；不处理 profileName 差异，不承诺修复全部 FlClash 错误。不读取/手改真实 local TOML/JS，不自动重新渲染、粘贴脚本或切换用户订阅。额外文件/行为返回规划确认。

## 已完成调查验收（保留原编号）

- [x] AC1：定位截图错误的精确条件，区分已确认与输入推断。
- [x] AC2：追踪官方 0.9.1 输入、失败与导航路径，保留证据限制。
- [x] AC3：完成脱敏最小复现，记录命令、结果和运行边界。
- [x] AC4：提供不删除订阅的临时恢复建议及校验保留边界。
- [x] AC5：持久化研究、官方链接、版本与未验证项，初次调查仅任务目录变化。

## 审阅状态

用户已批准方案，任务已激活并完成实现、独立复核及本机应用。完整 just ci 通过；原失败参数未现场捕获、Node 版本矩阵未运行，独立依赖告警未混入修复。当前保持未提交，详见 research/validation.md。

## 实施授权补充

用户已明确批准本方案实施和验证，要求先不要提交，并请求直接处理本地 FlClash 配置。允许在发现真实存储结构后，备份并仅修补已选家宽扩展脚本中的同一 null 兼容问题，必要时应用并观察结果；不扩展为订阅/凭据/路由变更，不手改仓库 local TOML/JS。真实脚本、备份和配置内容不进入 Git 或公开证据。原“待审阅/不实施”段落记录的是前一轮状态，本补充为最新授权；五文件产品范围不变。
