# Technical Design

## Protection Mechanism

使用 GitHub classic branch protection REST API 精确配置 `bahayonghang/clash-verge-ai-residential:main`。
请求通过 GitHub CLI keyring 凭据执行，避免当前细粒度环境 PAT 的权限限制。

## Approved Policy

- required status checks：`strict=true`，context 为 `Required checks`
- require pull request：启用，required approving review count 为 0
- enforce admins：启用
- required conversation resolution：启用
- required linear history：启用
- force pushes：禁用
- deletions：禁用
- restrictions、deployments、CODEOWNERS、bypass actors：不配置

## Preconditions

- CI 改动已通过 PR 合入 `main`。
- `main` 最新提交已产生成功的 `Required checks` check run。
- 保护请求目标、字段和凭据来源再次核对无误。

## Verification and Rollback

执行 PUT 后立即 GET branch protection，逐字段核对批准策略。
若返回字段不一致，停止后续操作并使用修正后的完整请求恢复；删除保护属于破坏性回滚，必须另行明确授权。
