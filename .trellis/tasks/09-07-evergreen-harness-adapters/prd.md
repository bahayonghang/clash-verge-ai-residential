# 对齐五套 Harness 的启动与委派边界

## Goal
让五套 harness 的实际项目接入方式与共享工作流一致，且必要修正在新 checkout 可恢复。

## Evidence
- `.trellis/workflow.md:104-106` 把 UserPromptSubmit hook 说成所有平台统一入口；实际 Claude/Codex 是 hook 配置，OMP 是 extension，当前 Grok/Kimi Trellis 走显式 pull。
- `.trellis/workflow.md:223,356-364,490-502,530-544` 的 Kimi built-in 与通用 trellis-* 委派说法不一致。
- `.kimi-code/skills/trellis-implement/SKILL.md:49`（check/research 同类）把“本项目没有自定义代理”误写为“Kimi 不支持”；当前第一方资料已提供项目代理。
- `.codex/config.toml:12-19` 的 hooks 特性开关文案过时；hook信任和实际执行仍需各客户端证据。
- `.gitignore:30-36` 忽略所有平台目录；仅修本地文件不能让fresh clone继承修正。已核实项目/CLI Trellis均0.7.0-beta.3。

## Requirements
- R1 记录 native capability、project configuration、actual runtime evidence 三层，逐工具给出入口、委派、权限和fallback。
- R2 修复共同 workflow 的平台条件分支；保留 Kimi built-in coder+skill pull，不新增项目代理迁移。
- R3 以窄 allowlist 保存必要本地覆盖文件；其他生成资产通过版本明确的 init --skip-existing 恢复，不复制全套harness。
- R4 不改全局设置、不自动信任hook、不启动付费模型实验；无法获得的运行证据保持UNVERIFIED。
- R5 规划/根因/最终审查用强模型；低价执行者必须有独占文件范围、明确AC与升级条件。

## Acceptance Criteria
- [x] AC1 / R1–R2：五工具的启动/委派表与本地真实文件及第一方文档一致；不再声称“所有平台都会自动注入”。
- [x] AC2 / R2：Kimi所有相关说明均采用同一路由；保留Active task首行、上下文pull和子代理禁止递归。
- [x] AC3 / R3：隔离候选checkout缺少ignored资产时，已安装的0.7.0-beta.3 init --skip-existing补齐五平台，并保留四个tracked override文件原字节；之后shared workflow/override仍一致。
- [x] AC4 / R1,R4：逐工具记录版本、已读AGENTS/spec、planning状态、委派前缀、hooks/pull、权限与实际结果；未运行或不可用项标UNVERIFIED，不冒充全工具runtime PASS。
- [x] AC5 / R3–R5：gitignore只放行设计指定四文件，无secret/local settings/cache误纳入；docs build、task validate、diff检查通过；review model不被全局降档。
- [x] AC6 / R4：静态适配可单独验收；“五工具动态对齐完成”必须五行真实证据齐全。缺证据时任务报告明确静态完成/动态未完成，父任务不得宣称全运行验收。静态完成；五行动态 smoke 为 UNVERIFIED。

## Out of scope
Kimi自定义代理迁移、Grok新增hook、Codex路径wrapper重构、模型价格基准、全局Trellis升级/配置、hook自动批准。

## Dependencies
先完成C2共享合同；C4业务skill同步可并行，最终smoke在C4后。
