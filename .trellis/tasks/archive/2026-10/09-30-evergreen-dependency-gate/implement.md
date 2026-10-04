# T01 已批准执行清单

用户于 2026-09-30 批准项目内实施；子任务按执行顺序启动。

## Order

1. 保存完整审计、依赖链、锁文件 hash，重新核对公告状态。
2. 只更新已确认受影响的 lock 子树；需要 major 或其它根依赖变更时停止并提交范围变更。
3. 添加独立审计 recipe 和 CI step，扩展既有 CI 合同负向测试。
4. 运行下列检查；强模型审查范围、退出传播和说明，再回写验收。

## Required Checks

- `npm --prefix residential-monitor ci`
- `npm --prefix residential-monitor audit --include=dev --audit-level=high`
- `npm --prefix residential-monitor audit --omit=dev --audit-level=high`
- `npm --prefix docs ci`
- `npm --prefix docs audit --include=dev --audit-level=high`
- `cargo audit --file residential-monitor/src-tauri/Cargo.lock`
- `node --test tests/sync-monitor-version.test.js`
- `actionlint .github/workflows/ci.yml`
- `just dependency-audit（本项新增后）`
- `just ci`
- `just docs-build`
- `git diff --check`

新脚本、recipe 与测试命令是获批后的交付，不表示当前已存在或已通过。命令的预期失败、环境阻断和正式验收必须分别记录。仅最后一条 native 命令成功不能覆盖前面的失败。

## Review

强模型逐项核对 PRD、diff、检查证据和持久回写。共享文件按照父任务顺序串行处理；先读取前序改动，不得覆盖其他任务工作。

## Closure

全部 AC 有证据后才能声明本任务完成；缺少正式或动态证据保持未完成。提交、归档另需用户授权。
