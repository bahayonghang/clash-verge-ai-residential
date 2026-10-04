# Design

Files:
- `tests/install-agent-skills.test.js`：扩展至七目标及真实payload临时安装；保留源脚本调用合同，不新增“直接执行副本脚本”要求。
- `docs/agents/residential-rule-tuning.md`：源库、五工具/共享目录、--create/--platforms/--check；明确 install-all 包含OS/全局副作用。
- `skills/residential-rule-tuning/SKILL.md`、`reference.md`：仅必要分发/适用工具说明，不改生成器业务映射。
- 忽略目录 `{.agents,.claude,.codex,.cursor,.omp,.grok,.kimi-code}/skills/residential-rule-tuning/{SKILL.md,reference.md,scripts/build-inputs.js}`：批准后由既有 installer 同步。
- `scripts/install-agent-skills.js` 默认不改；若新验收揭示具体缺陷，只作必要修正并记录，不新增配置表面。

CI 用临时目录验证installer，实际本机交付用 --check。不要把 ignored 目录缺失时的空检查接入 CI 冒充五工具就绪。Trellis适配归C3，此处仅业务skill。
