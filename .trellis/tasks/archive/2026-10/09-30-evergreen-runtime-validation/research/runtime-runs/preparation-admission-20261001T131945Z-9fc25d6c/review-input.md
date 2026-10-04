# T06 当前入口与捕获器准入收据

日期：2026-10-01（America/Chicago）。状态：PREPARATION_CHECKS_PASS_RUNTIME_NOT_RUN。主会话确认 primary 正式测量已结束，批准在 capacity 尚未启动的间隙核验入口和运行已有定向合成检查。本轮没有 freeze，没有启动任何客户端推理。

## 当前 Codex 入口

旧 `c6fe824d725f02d7/codex.exe` 已不存在。现存 `de8a38d2100ae498/codex.exe` 的实际 `--version` 返回 `codex-cli 0.159.2`，`--version`、`--help`、`exec --help` 均退出 0。当前文件 SHA256 为 `34549ded6e2aee87c911c62d025e52e26c488683d0f489cd68f756baef1a6df6`，大小 324702000 bytes。

本轮实际 help 确认 `exec`、`--sandbox read-only`、`-c key=value`、`--ephemeral` 和 `--json`。既定 argv 无需调整。help 同时列出 never 审批值；本轮不修改账户、模型、全局配置、项目覆盖、信任或权限。

入口元数据收据位于 `../preparation-identity-20261001T131731Z-d5cd2f2a/identity.json`，每条命令另有独立 receipt、stdout 和 stderr 文件。输出均先在内存检查去敏，redactions 为空；文件标为 redacted decoded output，不称原始字节。stdout/stderr 原始字节哈希保留在 receipt。

`../../runtime-bindings-20261001.json` 保留旧入口缺失，并绑定本轮版本/help/hash。verified_for_runtime=true 仅表示入口准备已核验；execute 仍要求冻结候选、匹配 GO 和 child readiness。相同版本字符串不证明二进制与旧版本逐字节相同，也不转移旧源码、V1/V2、模型、权限或 hook 证明。实际模型和 live multi-agent backend 仍为 UNKNOWN。

## 当前捕获器检查

最终捕获器 SHA256 为 `392b2f4c211c03f0bb9479dcbb5e6804660a8b3a82f31c618ef814d28036cf27`；定向验证器为 `1e33d36a0f017dfe71c412c4ddb7b08376975440307b201b6f77b089b8b25a2c`。本轮未修改这两份源码。

- `../preparation-validation-20261001T131842Z-2e0a5b15/verification.receipt.json`：验证器退出 0，91 项合成检查全部通过。包括 50 个 private_path 字符串、5 个去敏案例、10 组单 argv prompt 和对应无模型覆盖检查、4 类逐工具独立阻断场景的 16 项断言。没有真实工作树 snapshot 或客户端。
- 本目录 `admission.json`：两份 Python 源码 compile 通过；parent/child 各 5 行 plan；Popen 和 subprocess.run 均以禁止启动的 mock 核对，10 行均为 NOT_RUN。candidate manifest 不存在。
- `../preparation-native-synthetic-20261001T132036Z-a9ddb080/verification.json`：3 个本机 Python 合成进程验证当前 capture。normal 退出 0、无停止标记；authentication 退出 7、CLIENT_REPORTED_BLOCKER；timeout 退出 0，但独立保留 TIMEOUT 和 termination_requested=true。退出 0 不覆盖超时标记。三个场景均通过对应断言。

本轮所有检查只涉及 T06 准备执行器。没有执行产品测试、全量检查、数据库扫描、构建或模型推理。旧准备收据保持原样，没有用本轮成功覆盖旧失败。

## 待独立复审与执行前提

请复审本轮绑定身份、真实 help 与上述收据。准备检查不关闭 T06 AC1/AC3。原生角色发现、实际模型、权限、planning 状态、child 生命周期、hook/extension/pull、语言拒绝和候选前后差异均待五客户端真实事件。

主会话将等待 raw_fold 回退后的新源码与最终产品门，暂停其它代理对保护文件的写入，再 freeze 当前 dirty candidate。HEAD 不代表未提交候选。parent GO 与 child readiness/GO 分开。没有相应信号不得运行客户端；capacity 开始后暂停其它负载。
