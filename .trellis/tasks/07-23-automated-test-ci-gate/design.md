# Technical Design

## Test Runner

将两个现有测试文件迁移到 Node.js 内置 `node:test`，保留断言、fixture 和测试语义。
`npm test` 显式列出测试文件，保证 Windows `cmd.exe` 与 Unix shell 不依赖 glob 展开。

## Template Safety Scanner

把模板安全扫描拆成可调用函数与 CLI 入口，测试使用临时目录构造公开模板和可提交文本 fixture。
扫描相关可提交扩展名，包括 `.toml`；继续忽略明确命名的本地 TOML/生成脚本。

## GitHub Actions

- `test` 使用显式 include 矩阵：Ubuntu Node.js 18/20/22，以及 Windows Node.js 22。
- 每个矩阵任务保留可诊断名称，并设置超时。
- `Required checks` 聚合任务使用 `needs: test` 和 `if: always()`；仅当矩阵结果为 success 时退出 0。
- workflow 维持 `contents: read`，checkout 不持久化凭据，并按 PR 或分支取消过期运行。

## Documentation

更新 README 测试章节，说明本地完整门禁、CI 环境矩阵、稳定必需检查和人工集成验证边界。
PR 模板继续以 `npm run ci` 和脱敏真实 Profile 验证为验收入口。

## Compatibility and Risks

- `node:test` 在 Node.js 18 可用，不改变最低版本。
- 顶层测试默认保持非并发语义，避免共享常量临时修改和 console 替换产生竞态。
- Windows runner 增加少量 CI 时间，但能覆盖本地渲染脚本最常用的平台路径。
