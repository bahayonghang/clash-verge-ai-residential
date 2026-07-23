# 完善自动化测试与 CI 门禁

## Goal

在不引入生产依赖的前提下，把现有测试升级为标准、跨平台且适合作为分支保护依据的 CI 门禁。

## Background

- `npm run ci` 当前依次执行语法检查、35 个自定义同步测试和模板安全扫描，基线全部通过。
- 两个测试文件使用自建同步 `test()` 包装器，遇到首个失败即中止，未使用 Node.js 18 已内置的标准测试运行器。
- CI 仅覆盖 Ubuntu 上的 Node.js 18、20、22；本地渲染流程同时面向 Windows 与 Unix。
- 模板安全扫描是门禁的一部分，但扫描器自身没有正向/负向回归测试。
- GitHub 上当前检查名为 `Node.js 18`、`Node.js 20`、`Node.js 22`，矩阵变动会影响分支保护配置。

## Requirements

- R1：使用 Node.js 内置测试运行能力规范化测试发现、失败报告和退出状态，不新增第三方测试框架。
- R2：保留现有 35 个测试的语义与覆盖边界，并为模板安全扫描补充可隔离的回归测试。
- R3：模板安全扫描继续忽略明确的本地凭据文件，但覆盖所有相关的可提交文本格式。
- R4：CI 保留 Node.js 18、20、22 兼容性验证，并至少增加一个 Windows 最新 Node.js 的验证路径。
- R5：CI 增加最小权限、超时、并发取消和稳定的聚合门禁检查；分支保护只依赖该稳定检查。
- R6：文档区分本地快速/完整门禁、GitHub 自动化门禁和必须脱敏的人工 Clash Verge/Mihomo 验证。
- R7：同步更新 `.trellis/spec/frontend/quality-guidelines.md`，使后续开发遵循新的标准测试运行器和 CI 矩阵。

## Acceptance Criteria

- [x] AC1：现有 35 个测试在标准测试运行器下保持通过，新增安全扫描回归测试也通过。
- [x] AC2：`npm run ci` 在本地 Node.js 环境通过，且不读取被忽略的本地 TOML/生成脚本。
- [x] AC3：GitHub Actions 在 Ubuntu Node.js 18/20/22 和 Windows 最新 Node.js 上完成验证。
- [x] AC4：无论矩阵如何扩展，GitHub 都产生一个稳定命名、只有在全部验证成功时才成功的必需检查。
- [x] AC5：CI 工作流权限为只读，具备合理超时，并能取消同一 PR/分支的过期运行。
- [x] AC6：README 或贡献文档说明自动化与人工验证流程，PR 模板与该流程一致。
- [x] AC7：Trellis 质量规范准确反映最终测试命令、扫描范围和 GitHub CI 门禁。

## Out of Scope

- 真实代理连通性、Mihomo 内核或 Clash Verge Rev GUI 的 CI 集成测试。
- 覆盖率百分比门槛、性能基准或与当前风险无关的大型工具链。
- 新增生产依赖。
