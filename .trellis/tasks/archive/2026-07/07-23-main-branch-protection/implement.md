# Implementation Plan

1. 确认 `main` 最新 SHA 与 `Required checks` 成功，确认仓库仍只有预期管理员权限。
2. 清除当前进程的 `GH_TOKEN`/`GITHUB_TOKEN` 覆盖，使用 GitHub CLI keyring 凭据。
3. 将批准策略构造成结构化 JSON，通过 `gh api --method PUT .../branches/main/protection --input -` 应用。
4. 立即 GET 保护配置，核对 required check、strict、PR、审批数、管理员、对话、线性历史、强推和删除设置。
5. 记录保护摘要和回滚入口；不更改其他分支、仓库权限、merge 方法或 secrets。
6. 在保护生效后，通过普通 PR 提交 Trellis 归档与会话记录，证明维护流程可用且未绕过管理员约束。

## Validation Commands

- `gh api repos/bahayonghang/clash-verge-ai-residential/commits/main/check-runs`
- `gh api repos/bahayonghang/clash-verge-ai-residential/branches/main/protection`
- `gh pr checks <closeout-pr> --watch`

## Rollback Point

保留批准请求的完整字段摘要。若配置失败或部分生效，先回读实际状态，再以完整 PUT 修正；不自动删除保护。
