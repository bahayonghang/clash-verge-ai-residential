# Implementation Plan

1. 创建本地功能分支 `chore/test-ci-gate`。
2. 将 `tests/regression.test.js` 和 `tests/sync-local-config.test.js` 接入 `node:test`，保持现有 35 个测试语义。
3. 重构 `scripts/check-template-safety.js` 为可测试模块，增加隔离的安全扫描测试并覆盖 `.toml`。
4. 更新 `package.json` 的 test/check/ci 命令，继续保持零依赖。
5. 更新 `.github/workflows/ci.yml`：显式跨平台矩阵、超时、并发取消、checkout 凭据收敛和 `Required checks` 聚合 job。
6. 更新 README 测试流程说明和 `.trellis/spec/frontend/quality-guidelines.md`，核对 PR 模板无需产生冲突。
7. 先运行新增测试，再运行 `npm run ci`、`git diff --check` 和最终 diff 审查。
8. 形成一个原子本地提交，在展示 SHA 和远程副作用并获得确认后推送、创建 PR、等待 CI、squash merge。

## Validation Commands

- `node --test tests/check-template-safety.test.js`
- `npm test`
- `npm run ci`
- `git diff --check`
- `gh pr checks <pr-number> --watch`

## Risky Files and Rollback

- `.github/workflows/ci.yml` 决定未来必需检查名，合入前必须从 Actions 实际结果确认 `Required checks`。
- `scripts/check-template-safety.js` 涉及凭据防护，测试必须同时证明应拦截和应忽略的路径。
- 测试迁移必须保持原测试数量与断言；若 Node 18 行为不兼容，回退到现有同步入口并仅保留 CI 聚合改造。
