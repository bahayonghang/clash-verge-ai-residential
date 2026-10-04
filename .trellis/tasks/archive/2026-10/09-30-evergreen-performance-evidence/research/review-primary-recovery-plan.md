# Primary 恢复方案独立审查

日期：2026-10-01。审查代理：`/root/t02_review`。审查范围仅包括 `primary-recovery-plan-20261001.md`、T05 `prd.md`、正式 matrix 证据、原 primary 中断收据、最终保留身份和已审 runner/wrapper。没有运行负载，没有打开或 hash corpus，没有修改产品代码或 planning artifacts。

## 结论

**UNVERIFIED；当前方案不支持立即启动，补齐启动前字段后可在已批准范围内执行。** 恢复方向正确：保留中断批次，使用新输出目录，重新完成 3 对 baseline/candidate、维持原参数和身份，primary 完成后再执行容量。用户于 2026-09-30 的项目内实施批准仍然有效。

当前 primary 不能计入 AC5：原批仅完成 3/6 native，`primary-r2-candidate` 没有 native、driver 或外层退出证据，r3 未启动。证据见 `formal-primary-20260930.interruption.json`。

## 已通过的方案边界

- 保留 `formal-primary-20260930/` 全部原始 receipts、partial data 和中断说明，不覆盖、不拼接为新结果。
- 新批次使用新 output/data 目录，避免读取或复用 `primary-r2-candidate` 的 partial SQLite/WAL。
- 参数与 T05 PRD 一致：A250、counters、complete、1 Hz、30 秒 warmup、300 秒 measurement、`query_every_frames=`、`metadata_change_percent=10`、seed `20260919`、start `180000180`、无 virtual-time。
- primary 完成前不启动容量 106 次。该顺序符合 `prd.md` 的正式对照依赖和 `primary-recovery-plan-20261001.md` 第 6 步。
- 计划区分 native、driver 和外层 PowerShell exit。该区分符合原 matrix 审查对外层 exit=1、native 22×0、driver 未实采的更正边界。

## Findings

### R1 — 阻断：允许“只补中断对与 r3”，但没有定义可接受的收口规则

- 位置：`primary-recovery-plan-20261001.md` 的恢复方案第 4 步。
- 事实：T05 primary 是 3 对、6 次 native 的固定门，`run-formal-replay.py` 在 `args.set == "primary"` 时固定生成 `primary-r1`、`primary-r2`、`primary-r3`，并按 baseline → candidate 顺序写入 `expected_native_invocations=6`。证据：`run-formal-replay.py:152-165`、`formal-matrix-20260930/execution-contract.json` 的 `variant_order` 字段。
- 问题：计划优先要求完整六次重跑，但又允许在强模型批准后只补中断对与 r3。计划没有定义旧 r1/r2 收据与新收据能否合并、如何重算三轮和何时拒绝合并。旧批与新批分属不同 run；将两批合并会把不同运行时段、过程状态和 driver 证据混成一个 primary gate。
- 要求：恢复实施只能选择“完整 3 对六次重跑”。若保留局部补跑作为诊断，必须单列为 `UNVERIFIED`，不能参与 AC5 或 primary summary。

### R2 — 应修：缺少具体身份、runner 和 wrapper 的启动前绑定字段

- 位置：`primary-recovery-plan-20261001.md` 第 3、5 步。
- 事实：最终保留身份已固定：baseline exe SHA256 `0BAC94FD88E37DCD8971893BF79C91A05E9775CEED4A06AB7E2F10B690F3DD9`，candidate exe SHA256 `9604FBE6ED24C6DC4E40E556A6AD7F3338477582B223E15D20B795DDF73A7A5E`；baseline source manifest SHA256 `1120B9D2B5C457F856BCD5F51ED9C88E0612E978AB4C7B76B4816640E1C6678F`，candidate source manifest SHA256 `860165F3183AC8BFAAC2D74AF6F29E793575A91A82B1BB23D86EE975BE5C0462`。证据：`candidate-retained-20260930/executable-identity.json`。
- 事实：已审 replay runner SHA256 为 `6c88acca5994c02266f262fbc185ebf9d09de1ab9ec96baa302993a0cbf381e9`；live wrapper 需要单独绑定 SHA256 `88550bb4fca23d98a5e74abe7aeedab18d1ce7731cf5d890638a0e8f36219f5c`。证据：`review-formal-runner.md`、`run-live-recorded.py`。
- 问题：方案只写“相同最终源码与 executable identity”，没有要求把上述 hash、manifest before/after 相等性、runner/wrapper hash 写入新的 preflight 和 execution contract。
- 要求：启动前生成小型 `recovery-preflight.json`，写入四份身份 hash、runner/wrapper hash、固定 argv、source_revision、variant order 和输出路径。任一不匹配时禁止启动。

### R3 — 应修：没有定义新的输出目录、driver receipt 和外层 exit 收据的确切位置

- 位置：`primary-recovery-plan-20261001.md` 第 2、5 步。
- 事实：`run-live-recorded.py` 要求 receipt 路径不存在，并在 child 完成后先保存 driver exit 再 `sys.exit(code)`。证据：`run-live-recorded.py`。
- 问题：计划只写“新输出目录和新 driver receipt”，没有给出固定名称或拒绝复用的 preflight 检查。执行器将 `OUT` 和 `DATA` 目录直接断言为不存在；缺少显式路径会增加误写旧证据的风险。
- 要求：预先固定例如 `formal-primary-recovery-20261001/`、对应独立 `run-root` data 目录和 `formal-primary-recovery-20261001.driver.json`。保存外层 PowerShell exit 到单独 receipt，并保留 wrapper stdout。

### R4 — 应修：没有明确 driver exit 与指标 gate 的分离判定

- 位置：`primary-recovery-plan-20261001.md` 第 5、6 步。
- 事实：`run-formal-replay.py` 的 `sys.exit` 语义为：执行失败或 gate FAIL 返回 2；存在 UNVERIFIED 返回 3；全部通过返回 0。证据：`run-formal-replay.py:180-203`。
- 问题：计划要求保存 driver exit，但没有规定如何解释 exit=2/3。native 全部 exit 0 仍可能因性能门 FAIL 返回 2，不能把 driver 非零泛称为执行失败；相反，native/validation 失败必须独立列入 execution failures。
- 要求：最终 recovery summary 同时保存 `execution_failures`、每个 gate、`recorded_invocations=6`、driver exit 和 outer exit。判定顺序为：先审 native/validation execution；再审指标 gate；最后按 runner 的 0/2/3 语义记录整体状态。

### R5 — 提示：配对字段应写成不可变合同

- 位置：`primary-recovery-plan-20261001.md` 第 3、4 步。
- 事实：runner 固定 `variant_order=["baseline","candidate"]`，candidate `source_revision` 使用 manifest SHA256，baseline 使用 c278 revision；每轮 native 使用新 case 目录。证据：`formal-matrix-20260930/execution-contract.json`、`run-formal-replay.py:104-142`。
- 要求：恢复计划逐字写出 baseline → candidate 顺序、三轮 case 名、两侧 `source_revision`、每轮新空目录、输出 JSON 在目录外，并禁止跨批拼接。

## 必须补齐的启动前证据

启动前只写 research 收据，不改产品：

1. `recovery-preflight.json`：UTC 时间、无旧 driver/native PID、旧 `formal-primary-2026093` 只读保留、恢复 output/data 路径不存在、可用磁盘空间、四份 exe/manifest hash、runner/wrapper hash、固定 argv、`source_revision`、`variant_order`。
2. `execution-contract.json`：`expected_native_invocations=6`、case 列表、baseline → candidate、seed/start、30/300、query=0、无 virtual-time、失败保留策略。
3. 每条 native receipt：argv、PID、started/finished UTC、exit、validation_errors、stdout/stderr raw hash、report identity、WAL/FULL、traffic conservation、wall seconds。
4. driver receipt：wrapper argv、PID、finished status、driver exit、wall time。
5. 外层 receipt：PowerShell command、started/finished UTC、outer exit。
6. summary：`recorded_invocations=6`、`execution_failures`、每个 gate、driver exit、outer exit、UNVERIFIED/NOT_RUN 边界。

## 状态判定

| 范围 | 状态 | 证据 |
| --- | --- | --- |
| 用户批准 | PASS | T05 `prd.md` Authorization；`task.json` meta approval |
| 恢复方案主线 | PASS | `primary-recovery-plan-20261001.md` 第 1–6 步 |
| 完整 primary 可执行性 | UNVERIFIED | 仍缺六次新 native 与新 summary |
| 局部补跑作为正式 gate | FAIL | R1；没有合法合并规则 |
| 身份/参数边界 | PASS with required preflight | `candidate-retained-20260930/executable-identity.json`、formal matrix execution contract |
| AC5 | UNVERIFIED | `prd.md` AC5；中断收据仅有3/6 native |
| 容量前置条件 | PASS as sequencing rule / NOT RUN | recovery plan 第6步；capacity runner review |

## 审查结论

当前方案支持在既有批准范围内继续工作，但不支持立即启动。先补齐 R1–R4 的计划字段和 `recovery-preflight.json`，并强制完整六次重跑；完成后再开始恢复批。恢复批完成前不得运行 capacity 106。所有性能 FAIL、UNVERIFIED、NOT_RUN 和中断证据必须继续单列。
