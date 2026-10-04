# T01 清除本轮发现的两项 monitor 开发依赖 high，并让后续安全审计失败可被质量门识别。

## Goal

清除本轮发现的两项 monitor 开发依赖 high，并让后续安全审计失败可被质量门识别。

## Evidence

F1；monitor audit exit 1，omit=dev exit 0。 详细证据见 ../09-30-evergreen-five-harness-audit/research/audit.md。

## Requirements

- R1 解决本任务证据对应的问题，不扩大到相邻清理。
- R2 按下面可观察结果验收，保留失败与未验证边界。
- R3 实施后将批准结果回写并注明适用工具。

## Acceptance Criteria

- [x] AC1：monitor 完整 npm audit 在 high 阈值上退出 0；不能仅以 omit=dev 通过销项。记录修复前后依赖链。
- [x] AC2：锁文件变更限于已审查的漏洞依赖和必需父链；不升级 ESLint/TypeScript major，不关闭规则，不修改产品代码。
- [x] AC3：CI 中审计使用独立 step，原有 test/monitor/docs 聚合与 pwsh 失败传播保持。
- [x] AC4：just ci、docs-build、审计及失败传播测试通过；说明写明审计工具前提、网络失败分类与适用五工具。

## Dependencies

无实施依赖。先执行；共享 justfile/ci.yml 后续任务必须读取本项结果。

## Authorization

用户于 2026-09-30 明确批准项目内实施。具体文件及步骤见 design.md / implement.md。全局配置、真实凭据/数据库、安装、push、PR、远端 workflow、提交与归档不在默认授权内。
