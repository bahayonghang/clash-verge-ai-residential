# Implementation Plan

1. 完成并启动 `07-23-automated-test-ci-gate` 子任务。
2. 在本地功能分支实施测试运行器、安全扫描回归测试、CI 聚合门禁和文档更新。
3. 运行子任务的局部检查与完整 `npm run ci`，审查 diff 和凭据边界。
4. 在远程写入检查点展示分支、提交 SHA、PR 标题/正文、squash merge 和副作用，获得确认后推送并创建 PR。
5. 等待并核对 PR 的全部 GitHub Actions 检查，以匹配 head SHA 的 squash merge 合入。
6. 验证 `main` 上 `Required checks` 成功后，完成 `07-23-main-branch-protection` 子任务。
7. 应用批准的保护策略，通过 API 回读检查 required check、strict、PR、管理员、线性历史、对话、强推和删除字段。
8. 完成全局验收，归档两个子任务和父任务，并通过受保护 PR 提交 Trellis 收尾记录。

## Validation Gates

- `npm run ci`
- `git diff --check`
- `git status --short --ignored`，确认本地凭据和生成文件未进入提交
- `gh pr checks <pr> --watch`
- `gh run view <run-id>` 与 `gh api .../commits/main/check-runs`
- `gh api repos/bahayonghang/clash-verge-ai-residential/branches/main/protection`

## Rollback Points

- PR 合入前：关闭 PR 并删除远程功能分支。
- PR 合入后、保护前：使用 revert PR 撤销仓库改动。
- 保护后：仅在明确授权下恢复或删除保护，并回读验证。
