# T06 补齐当前候选版本在五工具中的实际规则加载、阶段控制和委派证据，区分本地与 hosted 验收。

## Goal

补齐当前候选版本在五工具中的实际规则加载、阶段控制和委派证据，区分本地与 hosted 验收。

## Evidence

F7；五工具新会话仍未测，当前 SHA 无 hosted 证据。 详细证据见 ../09-30-evergreen-five-harness-audit/research/audit.md。

## Requirements

- R1 解决本任务证据对应的问题，不扩大到相邻清理。
- R2 按下面可观察结果验收，保留失败与未验证边界。
- R3 实施后将批准结果回写并注明适用工具。

## Acceptance Criteria

- [ ] AC1：每工具分别记录版本、实际选择模型、规则文件、planning 状态、Active task 首行、实际工具权限与 hook/extension/pull 证据；不挪用其它工具结果。
- [ ] AC2：只读 smoke 不修改产品文件、不调用 start、不批准信任；写研究收据仅在任务 research/。正向加载和拒绝越界分别有记录。
- [ ] AC3：Codex 明确 V1/V2；V1 检查 max_depth，V2 不依赖该键。Kimi 采用既定 coder + role skill；Grok/OMP 使用对应原生 agent 调用。
- [ ] AC4：在获得明确推送/PR/workflow 授权后，hosted test/monitor/docs/Required checks 与候选 SHA 一致；否则保持 UNVERIFIED，不创建远端操作。
- [ ] AC5：未跑的 Windows WebView、托盘、真实控制器、凭据、长期 soak 保持各自未验证状态；五工具 smoke 不作为产品 native 验收。

## Dependencies

T03/T04 完成且 T02 最终副本同步；对被验证候选的其它代码改动稳定后运行。T05 未过门不能由本任务掩盖。

## Authorization

用户于 2026-09-30 明确批准项目内实施。具体文件及步骤见 design.md / implement.md。全局配置、真实凭据/数据库、安装、push、PR、远端 workflow、提交与归档不在默认授权内。
