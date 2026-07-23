# 完善 main 分支保护与测试流程

## Goal

为公开仓库 `bahayonghang/clash-verge-ai-residential` 建立可重复、可审计的测试门禁，
并让 GitHub `main` 分支只接受满足该门禁的变更，降低路由边界、配置渲染和凭据安全回归风险。

## Background

- GitHub 默认分支为 `main`，当前没有 classic branch protection 或 repository ruleset。
- 现有 GitHub Actions `CI` 在 Ubuntu 上运行 Node.js 18、20、22，最近的 `main` 和 PR 运行均成功。
- 本地 `npm run ci` 已通过：29 个路由回归测试、6 个本地配置同步测试、语法检查与模板安全检查。
- 现有检查名直接来自 Node.js 矩阵，没有一个与矩阵实现解耦的稳定必需门禁。
- Node.js 测试不能替代 Clash Verge Rev、Mihomo 和真实订阅 Profile 的人工集成验证；该边界已在 README 和 PR 模板中声明。

## Requirements

- R1：通过功能分支和 PR 完善自动化测试与 CI 门禁，合入后再配置依赖该门禁的 `main` 分支保护。
- R2：保留 Node.js >=18、零生产依赖、CommonJS 和直接执行脚本的兼容性。
- R3：自动化门禁覆盖路由回归、本地 TOML 渲染、模板凭据安全和受支持运行环境。
- R4：分支保护使用稳定的必需检查名，不与可变矩阵项直接耦合。
- R5：保护规则必须阻止强制推送和分支删除，并明确管理员是否受规则约束、PR 审核强度及历史策略。
- R6：不读取、上传或提交被忽略的本地 TOML、生成脚本或真实代理凭据。
- R7：远程变更必须在目标规则、分支、检查名和副作用已明确后执行，并在执行后通过 GitHub API 回读验证。

## Child Tasks

- `07-23-automated-test-ci-gate`：完善测试运行方式、CI 稳定门禁和相关文档。
- `07-23-main-branch-protection`：在 CI 门禁可用后配置并验证 `main` 分支保护。

## Acceptance Criteria

- [x] AC1：CI 子任务的本地完整门禁通过，并在 GitHub Actions 产生稳定、成功的必需检查。
- [x] AC2：`main` 分支保护要求 AC1 的稳定检查通过，且阻止强制推送和删除。
- [ ] AC3：不满足门禁的变更不能合入 `main`，满足门禁的授权维护流程仍可完成。
- [x] AC4：仓库文档准确说明本地自动化检查、GitHub CI 和人工集成验证的职责边界。
- [x] AC5：GitHub API 回读结果与批准的保护策略一致，最终工作区不包含本地凭据或生成文件变更。

## Out of Scope

- 在 CI 中启动 Clash Verge Rev、Mihomo 或连接真实住宅代理。
- 上传真实订阅、代理端点、用户名、密码或未脱敏连接日志。
- 引入生产运行时依赖、发布流程、部署环境或与测试无关的代码重构。
- 在没有独立收益证据的情况下引入大型测试框架、格式化器或静态分析平台。

## Key Decisions

- 使用独立维护者严格模式：必须走 PR，但 required approving review count 为 0。
- 管理员同样受保护；要求 CI 成功、分支最新、对话已解决和线性历史。
- 禁止 force push 与删除 `main`。
- 测试改动通过 squash merge 合入；最终保护启用后的 Trellis 收尾也通过 PR 合入，不绕过保护。
