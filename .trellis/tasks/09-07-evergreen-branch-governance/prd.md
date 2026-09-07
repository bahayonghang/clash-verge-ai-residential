# 核对并恢复分支治理契约

## Goal
解决项目规范要求线性历史、GitHub main 实际关闭的冲突；远端修改单独批准。

## Evidence
父任务 research/ci-audit.md 的实时 protection GET：required_linear_history.enabled=false；`.trellis/spec/frontend/quality-guidelines.md:98-105` 和已归档07-23规划要求true。当前 strict/app-bound Required checks、PR/管理员/对话解决及禁止force/delete仍符合已记录合同。设置变更时间和操作者UNVERIFIED。

## Requirements
- R1 保留当前设置快照和冲突说明，不凭历史文档推断live state。
- R2 推荐恢复 linear history=true，仅修改此项，所有其他保护字段保持批准值。
- R3 只有单独明确批准“修改 bahayonghang/clash-verge-ai-residential 的 main 保护”后才执行外部写入；C1–C4批准不包含本项远端写入。
- R4 结果回写项目规范/证据；若用户选择保留现状，先确认该决策，再改规范，不把规范偷偷改成现状。

## Acceptance Criteria
- [x] AC1 / R1：精确repo/branch、前后完整快照与差异可审查，检查GitHub是否还有影响合并的rulesets。
- [x] AC2 / R2–R3：获单独批准后 linear history=true，其他保护字段无非授权变化；失败不得报告成功。
- [x] AC3 / R2：确认现有squash/rebase路径可用；不改写历史或强推。
- [x] AC4 / R4：仓库规范与最终批准状态一致；下一次获授权正常PR的exact-head检查是后续运作证据，未发生则UNVERIFIED。

## Out of scope
新增CODEOWNERS/审批层级、Actions SHA policy、Dependabot、历史重写、调整其他分支。

## Dependencies
推荐在 C1 对应提交托管通过且 C2 完成后执行，quality spec 的治理回写必须串行到 C2 之后。当前仅planning；默认本地实施批次不含本项。
