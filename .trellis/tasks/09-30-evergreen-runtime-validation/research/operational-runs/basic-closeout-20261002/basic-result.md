# T06 本轮基本运行结果

日期：2026-10-02，America/Chicago。状态：**BASIC_PARTIAL / STOP_WRITING**。冻结候选 `4be0cfb58c637129fefb99c750a5baed0fb0fb6378cdce86f6a0ee90a1877e4b`；五路线各一次，未重试、放宽读取、修改模型/账户/信任/权限。

| 路线 | 基本读取 | tool outer | native / driver | 直接证据与原因 |
|---|---|---|---|---|
| Kimi | BLOCKED | 1 | native 0 / driver 1 | 原生 plan 双确认通过；新会话计划文件请求在 open 前拒绝，READ_PATH_NOT_ALLOWED。两个目标文件未读。 |
| Codex | PASS | 0 | native 0 | AGENTS/CONTEXT UTF-8 实读 exit0；正确 ASCII 三行。额外公开 skill 和 CONTEXT 转义读取。model UNKNOWN。 |
| Grok | PASS | 0 | native 0 | 两份 read_file/tool_result 与正确三行；runtime plan。init/assistant grok-4.6；usage grok-4.6-build 分别保留。 |
| Claude | BLOCKED | 1 | native 1 | version2.1.288；plan，Glob/Grep/Read；选择 claude-opus-5-5 后 API400，未 Read。未按供应方建议切模型。 |
| OMP | STOPPED | 1 | launch pwsh 0 / Bun UNKNOWN | credentialId 事件6按原识别器脱敏并停止；未观察完成读取。message_start 模型/提供方仅作启动元数据。 |

3618项保护文件在各次前后及最终快照均保持；index、HEAD、branch保持；全部入口/driver绑定一致。精确已捕获PID最终均不存在，见 outerobserver.json。Kimi native child及OMP Bun未输出独立PID，保持UNKNOWN；自有Job清理保留post_Popen_attach_race及best-effort边界。

本轮竞争负载按用户要求不阻断基本调用，结果不作为正式性能验收。PASS只覆盖基本公开文件读取；原完整T06模型、权限执行、hook/extension/pull、child、负向案例与hosted缺口保持各自UNVERIFIED。三个失败完整保留于各自独立收据，native exit0不覆盖stop或协议失败。

所有结果、事件locator和独立退出字段见basic-result.json；最终保护及绑定核验见final-protection.json。产品、docs、spec、任务meta、Git索引未修改。实现代理到此停止写入。
