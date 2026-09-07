# Implementation

1. 先保持planning，等待本项精确远端目标的单独授权。任何本地C1–C4批准都不代替它。
2. 只读GET当前保护、rulesets和仓库合并方式；核对C1同SHA hosted gate状态以及C2已完成，避免同时改quality spec。
3. 生成仅linear-history变化的完整请求及差异，提交审查；已有足够精确授权时按该授权继续，不重复询问。
4. 执行批准的API写入并立即GET，逐字段对比；确认strict/app绑定、PR、管理员、对话、force/delete保持。
5. 写validation与项目规范状态，注明五工具共享治理合同。下次正常PR的验证未发生则保留UNVERIFIED，不为验证创建破坏性测试PR。

规划、执行和复核均用强模型（Codex/Claude Code优先具有当前API访问的环境）。低价模型只能整理脱敏快照/文档，不直接写远端保护。

## C5 实施记录（2026-09-07）

- 新鲜 GET：`required_linear_history.enabled=false`。PUT 仅将该项设为 true。独立 GET 与 after 快照一致，其它保护字段未变。
- squash/rebase 仍可用。rulesets 为空。无强推、无改写历史、无 `git push`。
- 下次普通 PR 的 exact-head `Required checks` 仍为 UNVERIFIED。
