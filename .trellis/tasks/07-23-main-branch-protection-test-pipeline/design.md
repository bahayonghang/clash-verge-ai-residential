# Technical Design

## Architecture and Boundaries

本任务分为两个交付层：仓库内的自动化测试/CI 门禁，以及 GitHub 远程 `main` 保护。
仓库内门禁先通过功能分支和 PR 交付；远程保护只绑定已在 `main` 上成功出现的稳定检查名。

## Delivery Flow

1. 在 `chore/test-ci-gate` 上实现并验证测试/CI 改动。
2. 在远程写入检查点确认分支、提交 SHA、PR 内容、squash merge 和副作用。
3. 推送功能分支、创建 PR，等待全部矩阵检查和聚合门禁成功。
4. 使用已确认的 head SHA 执行 squash merge，并验证 `main` 上的聚合门禁成功。
5. 通过 GitHub branch protection API 配置独立维护者严格模式，并回读核对。
6. 归档 Trellis 子任务/父任务，通过受保护流程提交收尾记录。

## Contracts

- 本地门禁：`npm run ci` 退出码 0 表示语法、标准测试套件和模板安全扫描全部成功。
- GitHub 门禁：job/check 名固定为 `Required checks`；只有完整矩阵成功时才成功。
- 分支保护：required status context 精确为 `Required checks`，`strict=true`。
- 凭据边界：忽略的 `*.local.toml`、`*.local.js` 和真实代理信息不得进入测试 fixture、日志或提交。

## Compatibility

- Node.js 最低版本仍为 18；使用 `node:test`，不引入 npm 依赖。
- Linux 验证 Node.js 18、20、22；Windows 验证当前最新支持版本。
- 保留 CommonJS、`just ci` 和 `npm run ci` 入口。

## Trade-offs

- 不设置审批数可让唯一维护者独立合并，但不能提供第二人审查；以强制 PR、CI、对话解决和管理员约束补偿。
- 稳定聚合门禁增加一个很小的 Actions job，但避免矩阵变化导致保护规则漂移。
- 不引入覆盖率门槛或第三方 lint/test 工具，避免给零依赖脚本仓库增加与当前风险不相称的维护成本。

## Rollback

- 测试/CI 改动通过 revert PR 回滚。
- 分支保护可用执行前同结构的 API 请求恢复，或删除保护；任何回滚都需再次明确授权并回读验证。
