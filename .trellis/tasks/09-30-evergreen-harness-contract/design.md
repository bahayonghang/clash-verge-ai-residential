# T03 设计

## Mechanism

修改共享合同，明确本机覆盖可选、按需复制且不要求 Git 跟踪，避免复制五套产品事实。确定性 checker 使用 Node 标准库，校验可机械判定的路径/关键门/导入结构；缺失由既有环境诊断的 project.overrides 字段逐项报告，checker 跳过缺失覆盖，不新增输出门。已有覆盖按 checker 定义的结构约束校验，非 ENOENT 读取错误失败。需要使用覆盖时仍核对任务路径一致性与授权限制。语义冲突仍由强模型审查，checker 不声称证明客户端行为。测试只用临时 fixture，不读取真实用户配置或要求联网。保留 workflow phase tags、Kimi coder + role skill、Active task 首行和 pull fallback。

当前 checker 对 Codex config 和 Kimi check 的指定标记有结构断言；Kimi implement/research 纳入已有文件读取和 harness 引用清单，不承诺任意正文异常都会失败。

四个具名本机覆盖为 `.codex/config.toml`、`.kimi-code/skills/trellis-implement/SKILL.md`、`.kimi-code/skills/trellis-check/SKILL.md`、`.kimi-code/skills/trellis-research/SKILL.md`。新 checkout 可按需要和既有授权复制所选覆盖；覆盖缺失或静态检查通过均不证明对应原生角色、配置或 hook 已加载。2026-09-30 的 tracked 验收仅保留为历史证据；2026-10-01 用户批准保留当前不跟踪策略。本轮不修改本机覆盖内容。

## Owned Files

- AGENTS.md
- CLAUDE.md（只需导航时）
- .trellis/workflow.md
- .trellis/spec/frontend/index.md
- .trellis/spec/frontend/quality-guidelines.md（同步 T01 已批准安全门的 step 数与审计命令）
- docs/agents/harnesses.md
- skills/residential-rule-tuning/SKILL.md
- .codex/config.toml（注释）
- .kimi-code/skills/trellis-check/SKILL.md（若共享路由不足）
- scripts/check-agent-contract.js（新增）
- tests/check-agent-contract.test.js（新增）
- package.json

## Model And Harness

必须由强模型决定授权、schema 解释和最终审查。较低成本模型仅按冻结文字与 fixture 规格实施机械改动。 执行工具按父任务五工具矩阵选择；工具原生能力不扩展授权。

## Writeback

AGENTS/workflow/harnesses 适用五工具；Codex 配置注释仅 Codex；Kimi 覆盖仅 Kimi；业务源由 T02 投递。

实施衔接：T01 检查发现 root frontend quality-guidelines 仍记载六条 Windows native step。T03 将同一已批准安全门同步到该直接依赖说明；不增加产品行为或验证阈值。

## Rollback

逐文件回退本项说明/checker；不改全局 feature flags、模型、权限、hook trust；不广泛生成 harness 目录。
