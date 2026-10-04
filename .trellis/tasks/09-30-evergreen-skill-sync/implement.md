# T02 已批准执行清单

用户于 2026-09-30 批准项目内实施；子任务按执行顺序启动。

## Order

1. 审查七目录差异及额外文件；列出将替换的精确文件。
2. 补齐源更新后的检查流程与必要映射断言。
3. 在本项获批后使用现有 --force 备份覆盖已审查副本；不运行 install-all。
4. 验证七目标覆盖数量、内容、生成器结果、幂等和客户端尚未实测状态。

## Required Checks

- `node --test tests/install-agent-skills.test.js`
- `node scripts/install-agent-skills.js --check（修复前预计 exit 1，保留）`
- `node scripts/install-agent-skills.js --force（仅批准交付时）`
- `node scripts/install-agent-skills.js --check（修复后必须 exit 0，实际七目录）`
- `node scripts/install-agent-skills.js（二次 written=0）`
- `just ci`
- `just docs-build`
- `git diff --check`

新脚本、recipe 与测试命令是获批后的交付，不表示当前已存在或已通过。命令的预期失败、环境阻断和正式验收必须分别记录。仅最后一条 native 命令成功不能覆盖前面的失败。

## Review

强模型逐项核对 PRD、diff、检查证据和持久回写。共享文件按照父任务顺序串行处理；先读取前序改动，不得覆盖其他任务工作。

## Closure

全部 AC 有证据后才能声明本任务完成；缺少正式或动态证据保持未完成。提交、归档另需用户授权。
