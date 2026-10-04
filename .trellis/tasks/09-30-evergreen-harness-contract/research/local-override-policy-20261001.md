# T03 本机覆盖合同修订

日期：2026-10-01（America/Chicago）。状态：CONTRACT_UPDATED_CHECKS_NOT_RUN。

用户明确批准「保留当前不跟踪策略，更新任务合同和规范」。主会话派发本代理仅修改 T03/T04 的任务合同、元数据和本轮研究说明；规范由主会话同步。本次不扩大 T01/T02 授权，不修改产品、本机覆盖、Git 索引、任务指针或其它任务记录。

四个具名本机覆盖为：

- `.codex/config.toml`
- `.kimi-code/skills/trellis-implement/SKILL.md`
- `.kimi-code/skills/trellis-check/SKILL.md`
- `.kimi-code/skills/trellis-research/SKILL.md`

本轮只用文件存在性元数据核对，四文件 4/4 存在。该事实只对应当前本机，不证明新 checkout 含这些文件，不证明客户端已发现角色，也不证明配置、hook 或权限已生效。

当前合同将四覆盖定义为可选本机资产，按需要和既有授权复制，不要求 Git 跟踪。缺失由既有环境诊断的 project.overrides 字段逐项报告，checker 跳过缺失覆盖，不新增输出门。已有覆盖按 checker 定义的结构约束校验，非 ENOENT 读取错误失败；需要使用覆盖时仍核对任务路径一致性和授权限制。缺失、文件存在和静态检查通过均不能作为原生角色加载证明。

当前 checker 对 Codex config 和 Kimi check 的指定标记有结构断言；Kimi implement/research 纳入已有文件读取和 harness 引用清单。没有要求四个覆盖的任意正文异常都必须失败，没有新增产品修改或验证门。

2026-09-30 的独立 review 记录四覆盖 tracked 和 AC1–AC5 本地通过。该历史记录保持原字节。当前修订把 AC5 置为待验；旧 checker、fixture 和正式门的 PASS 不覆盖缺失可选覆盖与逐项报告的新合同。AC1–AC4 的历史结果保留，不由本轮文档同步追加动态证明。

已改文件为 prd.md、design.md、implement.md、task.json 和本说明。任务状态保持 in_progress，meta 追加本轮 approval_updates，保留既有批准和实施结果。没有运行测试、构建、客户端、bootstrap 或正式门。定向与正式检查等待 T05 正式负载结束。
