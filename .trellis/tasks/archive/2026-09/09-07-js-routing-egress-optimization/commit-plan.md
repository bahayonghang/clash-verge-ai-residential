# 本地提交清单

状态：实施与自动化验收已完成，待用户确认本清单后执行本地提交。

建议将本次修改作为一个业务提交：运行时、配置渲染、审计映射、测试、文档和规范共同实现同一组路由合同，单次 revert 可完整回退。当前无已暂存内容，已识别的全部改动均属本任务；将仅按下方显式文件清单暂存，不使用全仓库自动收集。

## 提交消息

```text
feat: [AI] ✨ 补齐核心分流开关并约束家宽唯一出口

Why: 核心服务缺少独立控制，且已有家宽组额外字段可能引入其他出口
同步渲染、审计映射、中英文文档与回归测试，保留默认覆盖范围

Confidence: high
Scope-risk: moderate
Tested: just ci; docs build; V1-V13
Agent-Task: 09-07-js-routing-egress-optimization
Agent-Model: gpt-6
Generated-By: agent
```

使用本地 `git commit -F`。按仓库 CLAUDE.md 的明确规范保留 `[AI]` 和 emoji；消息含 Why 及 Agent-Task、Agent-Model、Generated-By 和质量记录。

## 文件清单（33 个）

- `clash-verge-ai-residential.js`
- `scripts/sync-local-config.js`
- `clash-verge-ai-residential.local.toml.example`
- `tests/regression.test.js`
- `tests/sync-local-config.test.js`
- `tests/install-agent-skills.test.js`
- `tests/fixtures/routing-default-v5.11.json`
- `skills/residential-rule-tuning/scripts/build-inputs.js`
- `skills/residential-rule-tuning/SKILL.md`
- `skills/residential-rule-tuning/reference.md`
- `docs/configuration.md`
- `docs/en/configuration.md`
- `docs/local-configuration.md`
- `docs/en/local-configuration.md`
- `docs/routing-scope.md`
- `docs/en/routing-scope.md`
- `docs/dns-and-leak-model.md`
- `docs/en/dns-and-leak-model.md`
- `.trellis/spec/frontend/index.md`
- `.trellis/spec/frontend/component-guidelines.md`
- `.trellis/spec/frontend/state-management.md`
- `.trellis/spec/frontend/quality-guidelines.md`
- `README.md`
- `CHANGELOG.md`
- `.trellis/tasks/09-07-js-routing-egress-optimization/check.jsonl`
- `.trellis/tasks/09-07-js-routing-egress-optimization/design.md`
- `.trellis/tasks/09-07-js-routing-egress-optimization/implement.jsonl`
- `.trellis/tasks/09-07-js-routing-egress-optimization/implement.md`
- `.trellis/tasks/09-07-js-routing-egress-optimization/prd.md`
- `.trellis/tasks/09-07-js-routing-egress-optimization/research/routing-analysis.md`
- `.trellis/tasks/09-07-js-routing-egress-optimization/research/validation.md`
- `.trellis/tasks/09-07-js-routing-egress-optimization/task.json`
- `.trellis/tasks/09-07-js-routing-egress-optimization/commit-plan.md`

## 验证和后续步骤

- 根 Node 125/125、监控前端 284/284、Rust 432 单元及 3 隔离进程测试通过；1 项需写 Credential Manager 的测试按原声明忽略。
- 实际 `just ci`、最终文档构建、独立复核、Trellis 上下文与差异检查通过；细节见 research/validation.md。
- 确认后先创建以上业务提交，再由 Trellis 执行任务归档和会话日志提交；不推送或发布。
- 真实网络部署、UDP/DNS 路径、固定 IP 和流量收益保持 UNVERIFIED。

确认来源：.trellis/workflow.md Phase 3.4 第5步要求 “Present the plan once, ask for one-shot confirmation”。本清单供该步骤使用，不视为已执行提交。
