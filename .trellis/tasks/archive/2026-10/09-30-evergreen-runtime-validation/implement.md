# T06 已批准执行清单

## 2026-10-02 继续实施

按 research/operational-continuation-plan-20261002.md 修复参数与协议准备，先独立审查，再停止保护文件写入并冻结新快照，逐工具执行一次 parent 基本读取。Kimi 必须先收到原生 plan 模式确认，再发送 prompt；权限请求一律取消或拒绝。实际退出、API/认证/敏感停止与超时分别记账，不自动重试或切模型。本轮不派 child；原完整验收继续保持独立状态。

用户于 2026-09-30 批准项目内实施；子任务按执行顺序启动。

## Order

1. 检查前置工具、项目覆盖和 skill hash，冻结候选 source SHA。
2. 在各客户端的新会话中读合同并输出阶段/权限收据，模型选择保留强审查角色。
3. 按已授权流程运行一个受限研究子代理，回读实际 JSONL/spec、委派前缀和越界拒绝；不派自动修复角色。
4. 确认共享目录 diff 只有获准 research 产物，更新五行独立状态。
5. 仅在外部动作另行授权后取得候选同 SHA 的 hosted checks；缺失则保持待验。

## Required Checks

- `node scripts/check-agent-contract.js（T03 完成后）`
- `node scripts/check-harness-environment.js（T04 完成后）`
- `node scripts/install-agent-skills.js --check`
- `五客户端 fresh-session + 授权范围内的受限 child，分别保存原始收据`
- `gh run list --commit <候选SHA> --json databaseId,headSha,status,conclusion,workflowName`
- `gh run view <对应run-id> --json headSha,jobs,conclusion`
- `just ci（候选源码变化后）`
- `just docs-build`
- `git diff --check`

新脚本、recipe 与测试命令是获批后的交付，不表示当前已存在或已通过。命令的预期失败、环境阻断和正式验收必须分别记录。仅最后一条 native 命令成功不能覆盖前面的失败。

## Review

强模型逐项核对 PRD、diff、检查证据和持久回写。共享文件按照父任务顺序串行处理；先读取前序改动，不得覆盖其他任务工作。

## Closure

全部 AC 有证据后才能声明本任务完成；缺少正式或动态证据保持未完成。提交、归档另需用户授权。
