# T04 设计

## 2026-10-02 基本运行修复范围

用户要求继续实施其他内容并以正常运行为目标，不等待竞争负载。主会话已审查 research/operational-bootstrap-design-20261003T021935Z.md，并批准 research/operational-bootstrap-plan-20261002.md 的独立 bootstrap 入口、四份公开模板、新 fixture、package.json 与 justfile 接入。具体路径以该执行方案与 task.json 的 relatedFiles 为准。原诊断继续只读；当前四份本机覆盖及旧证据不修改。公开模板仅在明确指定的新隔离目标中补缺失文件，已有文件逐字保留。原 checker 不放宽。

## Mechanism

增加小型只读环境探针，使用可注入的进程执行/路径 fixture 测试失败分类。版本从可执行程序和 .trellis/.version 读取，不复制固定版本到代码。机器专用 Codex hash 路径只放证据。可使用显式已安装工具路径或隔离的固定版本执行器；获取/安装工具须遵守用户授权，不替用户修全局环境。

实施选择（2026-09-30）：用户已批准父任务的项目内实施与隔离 bootstrap。当前没有匹配的已安装入口，registry 已确认固定包存在，因此采用原设计的隔离固定版本执行器：仅在本任务新建临时工具目录及独立 npm cache 中取得 `@mindfoldhq/trellis@0.7.0-beta.3`，保留依赖解析与入口证据，再在单独候选目录 init。此步骤不升级全局包、不修改 PATH、不安装 ResiWatch、不调整 trust；诊断脚本自身仍保持只读。

2026-10-01 用户批准保留本机覆盖不跟踪策略。四个具名可选覆盖为 `.codex/config.toml`、`.kimi-code/skills/trellis-implement/SKILL.md`、`.kimi-code/skills/trellis-check/SKILL.md`、`.kimi-code/skills/trellis-research/SKILL.md`。新隔离候选目录按需要和既有授权复制所选覆盖，逐项记录未选用或缺失；缺失报告沿用环境诊断的 project.overrides 字段，缺失本身不阻断 bootstrap。对所选且已复制的文件记录 init 前后 SHA256，并核对任务路径和授权限制。没有覆盖或静态资产存在均不证明对应原生角色已加载。2026-09-30 的 tracked 验收及四文件 hash 保持证据保留，适用时间不转移。本轮不修改本机覆盖内容。

## Owned Files

- scripts/check-harness-environment.js（新增）
- tests/check-harness-environment.test.js（新增）
- package.json（仅将新增脚本与 fixture 测试接入既有显式检查列表）
- justfile
- docs/agents/harnesses.md
- docs/agents/residential-rule-tuning.md（bootstrap 衔接）

## Model And Harness

强模型判断环境与权限边界；较低成本模型可写路径/版本 fixture 和文档。不得下放全局修复。 执行工具按父任务五工具矩阵选择；工具原生能力不扩展授权。

## Writeback

harnesses.md 的 bootstrap、版本、fallback 适用五工具；Codex 和 Trellis 环境故障独立标注。

## Rollback

仅回退仓库脚本/docs；临时目录保留证据后按明确范围清理。不修改/删除全局 npm 包或应用文件。
