# T06 设计

## 2026-10-02 基本运行恢复

用户要求继续实施其他内容并以正常运行为目标，不等待竞争负载。主会话批准 research/operational-continuation-plan-20261002.md 中的版本化捕获器、最小公开读取 prompt、必要合成回归及同版本 Kimi ACP plan driver。原捕获器、候选与首失败保留。实际运行必须在独立准备审查后绑定新快照，且不改变账户、模型、trust 或权限；竞争负载仅记录。最小读取结果不替代原完整 runtime、hook 或子代理验收。

## Mechanism

五工具使用同一个 planning-only 合成审查任务和只读输入；各自选择原生入口及当前可用权限。Grok inspect/CLI --version/doctor 仅是前置检查。共享 smoke 协议记录事实，不为取得 PASS 自动启用 hook、信任项目、换模型或修改权限。收费会话和远端写入须在对应范围获批后执行。

## Owned Files

- docs/agents/harnesses.md
- 本任务 research/ 五工具 smoke 收据
- 本任务 research/ hosted exact-SHA 收据
- 相关已有 spec 的证据状态（仅确有新证据时）

## Model And Harness

强模型规划协议并审查五份原始收据和实际 diff；低成本模型可整理固定格式，不得把缺证据改为 PASS。 执行工具按父任务五工具矩阵选择；工具原生能力不扩展授权。

## Writeback

docs/agents/harnesses.md 的每工具版本/边界/结果；共用质量 spec 只记录有直接证据的状态。

## Rollback

失败保留收据，移回对应 owning task 修复。不通过关闭权限、篡改 fixture、删除失败结果来通过 smoke。
