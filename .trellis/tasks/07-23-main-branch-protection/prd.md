# 配置 main 分支保护

## Goal

在稳定 CI 门禁已部署并成功运行后，为 GitHub `main` 配置可执行、可回读验证的分支保护策略。

## Background

- 仓库为公开仓库，当前账号对仓库具有 admin 权限。
- GitHub API 已确认 `main` 当前未受 classic branch protection 保护，仓库也没有 repository ruleset。
- 当前环境变量中的细粒度 PAT 无法读取保护规则；GitHub CLI keyring 凭据具有 `repo`/`workflow` scope，能够读取保护状态。
- 分支保护应在 CI 工作流产生稳定聚合检查后配置，避免绑定短期矩阵检查名。

## Requirements

- R1：保护目标必须精确为 `bahayonghang/clash-verge-ai-residential` 的 `main`。
- R2：要求稳定 CI 聚合检查成功，并要求分支在合入前与 `main` 保持最新。
- R3：阻止 force push 和分支删除，要求 PR 对话已解决。
- R4：采用独立维护者严格模式：管理员受规则约束、必须走 PR、审批数为 0，并要求线性历史。
- R5：不配置部署门禁、CODEOWNERS 审批或其他仓库当前不存在的外部系统。
- R6：执行前保存预期请求配置，执行后通过 API 回读并核对每个关键字段。

## Acceptance Criteria

- [ ] AC1：GitHub API 返回 `main` 已受保护。
- [ ] AC2：required status checks 包含且仅依赖批准的稳定 CI 聚合检查，并启用 strict/up-to-date。
- [ ] AC3：force push 与 branch deletion 均被禁止，conversation resolution 已启用。
- [ ] AC4：管理员、审批数量和线性历史设置与用户批准的策略一致。
- [ ] AC5：记录执行后的保护配置摘要和可逆的回滚方式。

## Out of Scope

- 其他分支、tag 保护、环境部署审批、CODEOWNERS 或组织级 ruleset。
- 更改仓库可见性、协作者权限、merge 方法或 GitHub Actions secrets。

## Key Decisions

- 当前只有仓库所有者 `bahayonghang` 一位协作者，因此不要求他人审批，避免无法自助合并。
- 保护对管理员生效，并要求 PR、稳定 CI 门禁、分支最新、对话已解决和线性历史。
- 保持现有仓库 merge 方法配置不变；受保护 `main` 实际使用 squash merge 或 rebase merge。
- 禁止 force push 和 branch deletion，不配置 bypass actor。
