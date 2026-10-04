# T06 基本读取准备结果

日期：2026-10-02，America/Chicago。状态：**PREPARED_NOT_RUN / STOP_WRITING**。本轮没有启动模型客户端，没有冻结候选，没有派发 child，没有安装或联网操作。

## 本轮修改

- 新增 `research/runtime-capture-operational.py`，以独立模块无副作用导入原 `runtime-capture.py`。原脱敏器、credentialId 识别、输出限量、Windows 自有进程 Job、前后保护快照与严格 GO 检查均复用。
- 新增 `research/operational-parent-prompt.txt`。只读 `AGENTS.md` 和 `CONTEXT.md`，明确 UTF-8 读取，输出英文 ASCII 三行。仅覆盖基本读取，不替代原规划、hook、模型、权限和 child 验收。
- 新增 `research/operational-kimi-acp.py`。Kimi 原生入口为 `kimi.exe acp`。固定执行 `initialize` → `session/new` → `session/set_mode(plan)`，收到 set_mode 响应和本次 `current_mode_update=plan` 后才发送 `session/prompt`。完成后执行 `session/close` 并排空尾部输出。权限请求返回 `cancelled`、发送 `session/cancel` 并停止；不发模型、账户、自动批准或信任设置请求。
- 新增 `research/operational-kimi-source.json`，记录本机 2.0.0 help 及同一 exe 内嵌公开 JS 的 byte-offset、摘录长度与 SHA-256。另有 Python kimi_cli 1.33.0，已识别为不同安装资产，其源码结论未移用。
- 新增 `research/operational-verification.py` 及本轮准备收据。主会话另行创建 `research/operational-bindings.json`；本轮入口使用该独立绑定，原绑定保持历史身份。

## 协议与保护

Kimi 源码证据：`Cannot combine --prompt with --plan.` 位于 byte offset 110852079 附近；`acpModeToToggles` 位于 105627970，plan 映射为 `plan=true, permission=manual`；`setMode` 位于 105705692；`setSessionMode` 位于 105721173；mode 更新结构位于 105646860。准确匹配位置与 byte 区间以 `operational-kimi-source.json` 为准。协议字段、JSONL UTF-8 framing、权限取消结构与 end_turn 均来自同一 exe。exe SHA-256 为 `f630b2a1cd40c275c5d2895252dbaf0c9f1879c79014aa225362f062e86573c0`。

ACP client 只声明受限文件读取 capability；writeTextFile、terminal、terminal auth 均为 false。文件回调仅允许仓库根 `AGENTS.md` 和 `CONTEXT.md`，路径不匹配时先拒绝，再停止。native plan 限制和这些 callback 限制尚未在真实客户端运行验证，不声明 OS sandbox PASS。

driver 在转发事件前复用原脱敏器，秘密识别即停止，外层合并敏感停止状态。原始事件字节不落盘。外层 stream hash 仅表示 driver 转发字节；native Kimi exit、ACP 协议状态、driver exit 和外层 exit 分别保留。Windows Job 清理保留原来的 post_Popen_attach_race 与 best-effort 边界。

保护快照排除 `operational-runs/` 与 `operational-candidate.json`，保留原历史 output 排除。新 capture、prompt、driver、verification、公开源码依据、独立绑定均在 ASSETS；私有/数据库路径沿用原排除，不读取或 hash。execute 只允许 parent；同工具已有本轮 invocation 时拒绝自动重试。

## 检查结果

| 检查 | 结果 | 收据 |
|---|---|---|
| 三份 Python AST 语法 | PASS | preparation-receipt.json |
| 必要合成回归 | 51/51 PASS；verification native exit=0 | verification.stdout.json |
| 五入口最终 dry-run | 5/5 PREPARED_NOT_RUN；native exit=0 | refreshed-plan.stdout.jsonl、final-interpretation.json |
| 新文件空白检查 | 5/5 无诊断；no-index exit=1 表示新增文件差异 | preparation-receipt.json、final-interpretation.json |

合成回归覆盖 prompt 整参数、四工具原权限参数、Kimi 参数互斥修复、UTF-8、双确认顺序、权限取消、mode 丢失、写回调拒绝、私有读取拒绝、无 GO 拒绝、secret/credentialId 停止、超时、自有进程清理以及独立退出。native fixture 仅为临时自有 Python 进程；模型客户端启动数为 0。

保留两个准备诊断：首次扩大到 ACP 停止 fixture 时断言要求 native exit 非零，但 Windows Job 终止可返回 native exit=0，driver exit=1 和协议 BLOCKED 仍明确记录失败。已纠正断言，保留首失败 `../preparation-20261002/`。最终准备聚合曾把 no-index 新增差异 exit=1 判为检查失败；五份诊断均为空，已在 `final-interpretation.json` 单独纠正，原聚合收据未覆盖。

## 主会话下一步

先独立审查，再按新候选执行 freeze。每工具只运行一次基本读取；GO 必须匹配 `operational-candidate.json` 的 source_snapshot_id。默认外层超时 180 秒，Kimi native 超时 165 秒。执行期停止保护文件写入。实际读取成功、认证/API/额度失败、权限阻断、敏感停止与超时分别记录。Kimi 首次 native ACP、其它四工具基本读取、实际模型与权限、hook/extension、完整 child、原完整验收和 hosted 均仍未验证。

当前 Codex 独立绑定由主会话核对为 0.160.0。旧 0.159.2 候选/失败收据继续作为历史身份；版本相同或更新不提供模型/V1/V2 runtime 证明。OMP credentialId 首失败和其它原始客户端收据保持。

所有新脚本与 prompt 的 SHA-256、字节数见 preparation-receipt.json。实现代理到此停止写入。
