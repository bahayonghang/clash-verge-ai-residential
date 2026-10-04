# Primary 中断后的恢复方案

日期：2026-10-01。状态：INTERRUPTED_UNVERIFIED。适用工具：Claude Code、Codex、Grok Build、Kimi Code、OMP。

## 已确认事实

- 原 session 14594 已无法继续读取：write_stdin 返回 Unknown process id 14594。
- driver receipt formal-primary-20260930.driver.json 仍保留 status=running，但 driver PID 5576 与 native PID 56468 均不再存在。
- 已完成的 3 次 native 为 primary-r1-baseline、primary-r1-candidate、primary-r2-baseline；三次均 exit 0 且 validation_errors=[]。
- primary-r2-candidate 仅有 running receipt，native exit、driver exit、外层 exit 均未记录。其 partial SQLite/WAL 文件保留，不作为测量结果。
- primary-r3-baseline 与 primary-r3-candidate 未启动。
- 原 receipts、driver receipt 和 partial data 未覆盖。未启动重叠测量。

## 恢复方案

1. 保留 formal-primary-20260930/ 全部中断证据和 formal-primary-20260930.interruption.json。
2. 使用新输出目录和新 driver receipt。不得覆盖原目录或复用 partial candidate 数据。
3. 固定使用 candidate-retained-20260930、原 baseline 身份、相同 seed/start、A250、1Hz、counters、complete、metadata_change_percent=100、warmup 30 秒、measurement 300 秒、query_every_frames=0。
4. 完整重新运行 3 轮 baseline/candidate 六次配对，顺序固定为 primary-r1-baseline、primary-r1-candidate、primary-r2-baseline、primary-r2-candidate、primary-r3-baseline、primary-r3-candidate。禁止将原中断批次和恢复批次拼接为正式结果。每次使用新的空 case 目录；JSON 位于 data 目录之外。
5. 每条 native 保存原始 stdout/stderr、native exit、validation_errors、wall time、身份、fixture、WAL/FULL 与守恒字段。driver 另存原始 exit；外层 PowerShell exit 单列，禁止用外层码替代 native/driver 码。
6. primary 完成前不启动 retained capacity。只有在 primary 恢复批次完成并经强模型审查后，才可按已审 capacity runner 使用新输出目录运行容量 106 次。

## 判定边界

- 当前 primary gate 保持 INTERRUPTED_UNVERIFIED，不能计入 AC7 或关闭容量前置条件。
- 中断原因未查明。503 仅说明代理调用失败，不能证明 native 或 driver 的具体终止原因。
- 任何重跑必须保留原中断收据，并使用相同最终源码与 executable identity。不得修改阈值、缓存条件、时长、样本数或数据库语义。

## 启动前绑定与退出码解释（2026-10-01）

本节落实 `review-primary-recovery-plan.md` 的 R1–R5。审查文本中的 baseline hash、query、metadata 和 start 字段存在转写缺失；实际合同取自 canonical `candidate-retained-20260930/executable-identity.json` 与已审 `run-formal-replay.py`。原审查和首份失败 preflight 保留。

| 字段 | 固定值 |
| --- | --- |
| baseline bench SHA256 | `06BAC94FD88E37DCD8971893BF79C91A05E9775CEED4A06AB7E2F10B690F3DD9` |
| candidate bench SHA256 | `9604FBE6ED24C6DC4E40E556A6AD7F3338477582B223E15D20B795DDF73A7A5E` |
| baseline source manifest SHA256 | `1120B9D2B5C457F856BCD5F51ED9C88E0612E978AB4C7B76B4816640E1C6678F` |
| candidate source manifest SHA256 | `860165F3183AC8BFAAC2D74AF6F29E793575A91A82B1BB23D86EE975BE5C0462` |
| replay runner SHA256 | `6C88ACCA5994C02266F262FBC185EBF9D09DE1AB9EC96BAA302993A0CBF381E9` |
| live wrapper SHA256 | `88550BB4FCA23D98A5E74ABE7AEEDAB18D1CE7731CF5D890638A0E8F36219F5C` |
| baseline source_revision | `c278bb7b56603001e32e353d2ee589dccef0bfe9` |
| candidate source_revision | `manifest-sha256:860165f3183ac8bfaac2d74af6f29e793575a91a82b1bb23d86ee975be5c0462` |
| metadata_change_percent / query_every_frames | `100` / `0`；counters 下 metadata 不变 |
| seed / start_utc | `20260919` / `1800001800` |
| expected_native_invocations / variant_order | `6` / `["baseline", "candidate"]` |

Research 根目录为本文件所在目录。新 output 为 `formal-primary-recovery-20261001/`；新 data 为 `build-20260930/state.json` 的 run_root 下同名目录。driver receipt 为 `formal-primary-recovery-20261001.driver.json`；外层收据为 `formal-primary-recovery-20261001.outer.json`；wrapper stdout/stderr 为同前缀 `.wrapper.stdout.log` / `.wrapper.stderr.log`。这些路径均必须不存在。最终 preflight 为 `formal-primary-recovery-20261001.preflight-final.json`；包含 source before/after 相等性、当前每个源文件 hash、全部四个 exe hash、runner/wrapper hash、六次 argv、UTC 时间、空间和实际进程分类。进程短样本为同前缀 `.process-sample.json`。常驻 MCP 的进程名称不构成负载证据；记录真实 command line、父进程和 CPU 增量。不得终止用户程序。

启动前保存计划 execution contract；runner 在新的 output 内生成实际 `execution-contract.json`。wrapper 独立保存 driver exit；外层收据先保存 wrapper exit 和预定 shell exit，再由执行工具的实际完成结果补录 `observed_outer_exit_code`。native exit 和 validation_errors 仍逐次保留。任何身份或 fresh-path 不匹配禁止启动。

恢复 summary 先核对 `recorded_invocations=6` 和 `execution_failures`，再逐项解释指标 gate。driver exit `2` 表示 execution failure 或指标 FAIL；`3` 表示存在 UNVERIFIED；`0` 表示 runner 全部门通过。native 全部 exit 0 不关闭全部应用文件归属缺口。CPU、SQLite 子集与全部应用文件门分别记录；全部应用文件字段为 null 时保持 UNVERIFIED。容量 106 次保持 NOT_RUN，等待主矩阵独立检查。
