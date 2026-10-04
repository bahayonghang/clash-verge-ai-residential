# Parent 补批事件初审

状态：STOP_WRITING。保持冻结 source `9a31d79c85465e3213e6f364f670ebac4a45911897e385c74e5909a740778cf9`。主会话单独批准 Kimi、OMP 各一个 parent 批次。未重跑 Claude、Codex、Grok，未增加超时，未启动 child。

Kimi 批次 `20261002T021201Z-e93cdf2e`：原生退出 1，时长 0.585635 秒，stderr 事件 1 为 `error: Cannot combine --prompt with --plan.`。没有推理或固定文件读取事件。保留参数不兼容首错误，未改参数或重试。实际模型、provider/backend、planning 和权限有效状态 UNKNOWN/UNVERIFIED。

OMP 批次 `20261002T021247Z-f0760b2b`：原生退出 0，时长 6.314756 秒；捕获器在事件 6 检测 `$.message.credentialId` 并脱敏为 `[REDACTED:secret_key]`，停止原因 `SENSITIVE_OUTPUT_REDACTED`，已请求终止。未观察到固定文件读取或最终结果。事件 6 原生 assistant message_start 记录 `api/provider=openrouter`、`model=stealth/space-bunny-alpha`；这些字段仅证明该响应的原生元数据。未修改模型。message_start 的零 usage 不作为最终费用或 token 数。

两个批次 execute 原生退出、driver outer 退出和 exec tool 退出均为 0。独立保存 `kimi.driver.json`、`omp.driver.json` 及对应 `outer-exec-result.json`；原生客户端退出和捕获停止标记保留在各批次 receipt。各工具保护 diff 为空。4 个精确已知 PID 均不存在，见 `known-pid-cleanup.json`。Job 收据仅为 best effort，保留启动后附加竞争和无绝对孤儿进程保证的限制。

最终 snapshot 通过 importlib 读取捕获器时，我漏设 `sys.dont_write_bytecode`，UTC 02:13:57 生成 `research/__pycache__/runtime-capture.cpython-314.pyc`。该缓存位于本阶段允许写入目录之外，构成执行范围偏差；保护 snapshot 排除忽略的 pyc，因此 snapshot 相同不能覆盖该偏差。已立即报告并停止；主会话单独批准仅清理该生成物及实际为空的目录。清理前核对绝对路径、非 reparse point、创建/修改时间、单文件 SHA256 和冻结捕获器 SHA256，收据为 `bytecode-scope-deviation-cleanup.json`。组合清理命令被自动策略拒绝（仅返回 `blocked by policy`）；分步只读核对和固定绝对路径非递归删除成功。没有删除其他文件。

清理后的 snapshot 仍与冻结值相同，protected diff、索引、HEAD、分支变化均为空；未重新生成 bytecode，见 `post-cleanup-snapshot.json`。后续 Python 检查须设置 `sys.dont_write_bytecode=True` 或 `PYTHONDONTWRITEBYTECODE=1`。

五工具 parent 尝试均已完成，各有阻断或证据限制，完整验收无 PASS。原首批、首错误、停止收据及公开脱敏事件未重写。没有运行额外测试、构建、安装、登录、配置/模型/权限更改、hosted 写入或 child。等待主会话独立评审和后续 GO。
