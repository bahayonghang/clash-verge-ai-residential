# T02 恢复业务 skill 单一源与当前七个实际投递目录的一致性，并明确后续源修改的交付检查。

## Goal

恢复业务 skill 单一源与当前七个实际投递目录的一致性，并明确后续源修改的交付检查。

## Evidence

F2；21 项差异，源 26/13/13，旧副本 26/12/14。 详细证据见 ../09-30-evergreen-five-harness-audit/research/audit.md。

## Requirements

- R1 解决本任务证据对应的问题，不扩大到相邻清理。
- R2 按下面可观察结果验收，保留失败与未验证边界。
- R3 实施后将批准结果回写并注明适用工具。

## Acceptance Criteria

- [x] AC1：确认差异只有已审查旧副本；同步保留 .bak-UTC 和额外用户文件，不更改私有 TOML/JS。
- [x] AC2：现有实际七目标均与单一源相同，--check 退出 0；二次安装 written=0。
- [x] AC3：真实 payload fixture 验证 26 个 routing 键、13 supported、13 unsupported，extra_anyrouter 对应 anyrouter.top，extra 保持 unsupported。
- [x] AC4：文档区分 fixture 交付、实际副本一致、客户端发现三种证据；零目标 --check=0 不再被误读为安装成功。

## Dependencies

T03 完成涉及业务 SKILL.md 的合同变更后执行最终同步；先读本轮已保存的差异证据。

## Authorization

用户于 2026-09-30 明确批准项目内实施。具体文件及步骤见 design.md / implement.md。全局配置、真实凭据/数据库、安装、push、PR、远端 workflow、提交与归档不在默认授权内。
