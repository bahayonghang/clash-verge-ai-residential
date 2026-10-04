# T03 让五工具读取一致的授权和质量合同，并准确描述 Codex V1/V2 的递归边界。

## Goal

让五工具读取一致的授权和质量合同，并准确描述 Codex V1/V2 的递归边界。

## Evidence

F3/F4/F8；local.toml 规则冲突、self-fix 阶段条件、V1/V2 保证及过时导航。 详细证据见 ../09-30-evergreen-five-harness-audit/research/audit.md。

## Requirements

- R1 解决本任务证据对应的问题，不扩大到相邻清理。
- R2 按下面可观察结果验收，保留失败与未验证边界。
- R3 实施后将批准结果回写并注明适用工具。

## Acceptance Criteria

- [x] AC1：AGENTS 仍为共享权威，CLAUDE 只单向导入；frontend index 不把 Claude 入口当五工具前提。
- [x] AC2：默认禁止 agent 手改 local.toml 的边界保持；skill 明确形成建议由用户应用，例外必须来自用户明确授权。
- [x] AC3：只读审查不自动修代码；self-fix 限定为获批实施且不扩大文件范围。不得新增自动信任或绕过权限。
- [x] AC4：本机 Codex 覆盖存在且选用时，max_depth=1 保留 V1 作用；说明明确 V2 忽略该字段。未选用或缺失覆盖时明确报告，不推断该配置已生效，不声称提示词提供硬沙箱保证。
- [x] AC5：说明检查验证关键入口、门禁名称、已移动引用和约束结构。四个具名本机覆盖可选、按需复制，不要求 Git 跟踪；缺失由既有环境诊断的 project.overrides 字段逐项报告，合同 checker 跳过缺失覆盖，不新增 checker 输出门。已有覆盖按 checker 定义的结构约束校验，非 ENOENT 读取错误失败。需要使用覆盖时仍核对任务路径一致性与授权限制。破坏必需合同或已有覆盖中 checker 定义的结构约束的 fixture 必须失败；正常仓库及缺少可选覆盖的 fixture 均通过，且不据此声明原生角色已加载。

2026-09-30 的 AC1–AC5 本地通过属于当日合同与四覆盖 tracked 的历史状态，见 research/review.md。本机四个覆盖于 2026-10-01 仍存在。用户同日批准保留不跟踪策略并更新合同；修订后的 AC5 于 2026-10-01 完成本轮本地验收：55/55 定向 fixture、回退后完整 just ci、docs-build 和说明/skill 检查通过。旧收据仅保留历史范围，不转换为本轮 PASS；不据此声明原生角色已加载。收据与独立审查见 ../09-30-evergreen-five-harness-audit/research/final-review-20261001.md。

## Dependencies

依赖本轮审查证据；实施可在 T01 后进行。业务 skill 源变更先于 T02 最终同步。

## Authorization

用户于 2026-09-30 明确批准项目内实施。具体文件及步骤见 design.md / implement.md。全局配置、真实凭据/数据库、安装、push、PR、远端 workflow、提交与归档不在默认授权内。

用户于 2026-10-01 明确批准「保留当前不跟踪策略，更新任务合同和规范」。本次追加范围仅为本任务的本机覆盖策略合同同步及对应规范，由主会话同步规范；不扩大 T01/T02 或本机覆盖内容的修改授权。详见 research/local-override-policy-20261001.md。
