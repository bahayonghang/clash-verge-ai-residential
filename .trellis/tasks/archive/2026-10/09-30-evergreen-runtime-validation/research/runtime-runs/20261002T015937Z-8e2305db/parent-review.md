# Parent 首批事件初审

状态：STOPPED_REQUIRES_INDEPENDENT_REVIEW。Grok 达到既定 180 秒上限后停止整个批次。Kimi、OMP 未在本批启动。未启动 child。

冻结候选为 `9a31d79c85465e3213e6f364f670ebac4a45911897e385c74e5909a740778cf9`。HEAD `d3a25b4164343f5cbeab8a51efe1a1d3974cdf4a` 仅为基线。各工具 before/after 和 `post-batch-snapshot.json` 均显示 protected diff 为空，索引、HEAD、分支未变化。该结论覆盖捕获器定义的保护快照；不证明所有瞬时文件活动均不存在。

| 工具 | 原生退出 | 时长/秒 | 事件数 | 初审状态 |
| --- | --- | --- | --- | --- |
| Claude | 1 | 4.133717 | 3 | API 400 BLOCKED |
| Codex | 0 | 153.259114 | 38 | 读取事件存在，中文证据损失及固定输入范围偏差 |
| Grok | 0 | 180.049560 | 7 | TIMEOUT，已请求终止，无最终结果 |
| Kimi | 未启动 | 无 | 无 | NOT_STARTED_AFTER_BATCH_STOP |
| OMP | 未启动 | 无 | 无 | NOT_STARTED_AFTER_BATCH_STOP |

Claude 的 `events.redacted.jsonl` 事件 1 记录 init selection `claude-opus-5-5`、`permissionMode=plan`、工具 `Glob/Grep/Read`。事件 2 是 `<synthetic>` 错误消息，事件 3 记录 `api_error_status=400`、`terminal_reason=api_error`、空 `modelUsage`。未读到固定文件。保留错误建议的历史文本，未按建议切模型或重试。实际推理模型、provider/backend、规则和负向案例未验证。

Codex 的事件 8、15–19 返回五个固定文件的读取结果；事件 6/7 额外读取 `.agents/skills/trellis-start/SKILL.md`，违反本次固定输入限制。保存的事件文件含 10778 个 U+FFFD，原因未查明。事件 34–36 返回 `Only core types are supported in this language mode`；语言模式错误不证明 OS 写入拦截。事件 37 输出部分完成及中文证据 BLOCKED，含自然语言拒绝修改根脚本，未尝试写入。事件 38 为 `turn.completed`。实际模型、provider/backend 缺少原生字段；自述 `read-only/never/restricted` 和 `workflow-state: no_task` 不作为独立权限或 hook 执行证明。冻结捕获器与原始首份收据未修改。

Grok 的事件 1 和 assistant 事件 2/4/6 记录 `grok-4.6`；原生 init 记录 `permissionMode=plan`。事件 2 请求五个固定文件，事件 3 均返回 `is_error=false`。事件 4/6 另请求 T06 task.json、implement.md、design.md、runtime-parent-prompt、runtime-scope-review、implement.jsonl、check.jsonl、runtime-preflight-review 及 T05 task.json，共 9 个范围外公开文件；结果见事件 5/7。还执行 T06 与 task 树 `list_dir`、T06 `grep`。保存的 Grok JSONL 无 U+FFFD。无最终 result，负向案例未到达。启动时工具/skill 列表不证明 hook 已执行，也不证明工具写入被 OS 阻断。

`receipt.json` 保留每个原生退出和停止标记。capture execute、driver outer 和 exec session 41319 均退出 0；这些退出仅表示收据驱动结束。driver 位于 `../parent-driver-20261002T015916Z-6fca6c82/`，实际命令、PID 和退出见 `driver.json` 与 `outer-exec-result.json`。

`known-pid-cleanup.json` 对 6 个已知 PID 做精确存在查询，均不存在。每个 receipt 保留 `best_effort_owned_job; post_Popen_attach_race; no_permission_sandbox; no_absolute_no_orphan_claim`。已知 PID 消失与 Job 关闭不证明绝对不存在后代孤儿进程。

公开事件为 normalized redacted events；原始捕获字节的 SHA256 保留在 receipt，原始流字节未保存。原始 receipt、events、before/after 与初始 batch 未重写。未运行测试、构建、安装、登录、模型/权限变更、hosted 写入或 child。本报告仅为事件初审，完整动态验收没有 PASS。
