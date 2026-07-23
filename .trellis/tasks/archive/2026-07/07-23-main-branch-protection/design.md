# Technical Design

## Protection Mechanism

使用 GitHub classic branch protection REST API 精确配置 `bahayonghang/clash-verge-ai-residential:main`。
请求通过 GitHub CLI keyring 凭据执行，避免当前细粒度环境 PAT 的权限限制。

## Approved Policy

- required status checks：`strict=true`，app-bound check 为 `Required checks`
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

## Execution Evidence

- PR #2 head `b058b157340d8f17fbec2c94a460a1b4f5070366` 的五个检查全部成功，并以 `--match-head-commit` squash merge。
- `main` SHA `9b2ed57896e1694ca987a2eb2e41008800ddab5a` 的五个检查全部成功。
- `Required checks` 由 `github-actions` App ID `15368` 产生并绑定到保护规则。
- GitHub API 拒绝同时传 `contexts` 与 `checks` 的请求（HTTP 422）；改为仅传 app-bound `checks` 后成功，首次拒绝未产生部分配置。
- 独立 GET 已验证 strict、管理员、0 审批、对话、线性历史、强推和删除字段符合批准策略。
