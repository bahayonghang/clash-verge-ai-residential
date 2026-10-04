# T02 设计

## Mechanism

复用 scripts/install-agent-skills.js 的冲突拒绝、逐文件对比和备份机制；没有证据要求改写安装器。以真实 source payload 的临时七目标测试保证 clean checkout，再单独验证本机已存在目录。不得把 ignored 副本纳入 Git 或把依赖真实本机目录的检查作为 hosted PASS 条件。

## Owned Files

- skills/residential-rule-tuning/SKILL.md（交付说明）
- skills/residential-rule-tuning/reference.md（按需）
- docs/agents/residential-rule-tuning.md
- tests/install-agent-skills.test.js
- .agents/skills/residential-rule-tuning/
- .claude/skills/residential-rule-tuning/
- .codex/skills/residential-rule-tuning/
- .cursor/skills/residential-rule-tuning/
- .omp/skills/residential-rule-tuning/
- .grok/skills/residential-rule-tuning/
- .kimi-code/skills/residential-rule-tuning/

## Model And Harness

强模型裁决差异与映射含义；较低成本模型可处理备份同步、固定断言和文档。 执行工具按父任务五工具矩阵选择；工具原生能力不扩展授权。

## Writeback

业务 skill 源和 docs/agents/residential-rule-tuning.md；适用五工具及 .agents/.cursor 投递。

## Rollback

仅使用各自 .bak-UTC 恢复本次替换；保留用户额外文件和审计记录；不批量删除平台目录。
