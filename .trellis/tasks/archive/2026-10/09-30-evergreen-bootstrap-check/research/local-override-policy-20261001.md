# T04 本机覆盖合同修订

日期：2026-10-01（America/Chicago）。状态：CONTRACT_UPDATED_CHECKS_NOT_RUN。

用户明确批准「保留当前不跟踪策略，更新任务合同和规范」。主会话派发本代理仅修改 T03/T04 的任务合同、元数据和本轮研究说明；规范由主会话同步。本次不扩大 T01/T02 授权，不修改产品、本机覆盖、Git 索引、任务指针或其它任务记录。

四个具名本机覆盖为：

- `.codex/config.toml`
- `.kimi-code/skills/trellis-implement/SKILL.md`
- `.kimi-code/skills/trellis-check/SKILL.md`
- `.kimi-code/skills/trellis-research/SKILL.md`

本轮只用文件存在性元数据核对，四文件 4/4 存在。当前本机文件存在不证明新 checkout 含这些文件，不证明原生角色已发现或加载，也不证明配置、hook 或权限已生效。

当前合同将四覆盖定义为可选本机资产，按需要和既有授权复制，不要求 Git 跟踪。bootstrap 逐项记录未选用或缺失；缺失报告沿用既有环境诊断的 project.overrides 字段，不新增 checker 输出门，缺少可选覆盖本身不阻断 bootstrap。仅对选用且获准复制的覆盖核对 init 前后 SHA256，同时保持任务路径一致性和授权限制。版本不匹配、实际进程失败、必需生成资产缺失等既定失败门不变。

2026-09-30 的独立 review/bootstrap 收据记录当日四覆盖 tracked、四文件 hash 保持及 AC1–AC5 本地通过。原 review、bootstrap-prerequisites 与所有旧收据保持原字节。修订后的 AC3 置为待验；旧 init 和测试结果不覆盖缺失可选覆盖的新场景，也不替代 T06 原生运行证据。本轮不重新获取工具或执行 init。

已改文件为 prd.md、design.md、implement.md、task.json 和本说明。任务状态保持 in_progress，meta 追加本轮 approval_updates，保留既有批准和实施结果。没有运行测试、构建、客户端或正式门。定向与正式检查等待 T05 正式负载结束。
