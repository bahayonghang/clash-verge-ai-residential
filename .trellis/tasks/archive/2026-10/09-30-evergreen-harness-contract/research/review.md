# T03 独立审查

日期：2026-09-30（America/Chicago）。任务：`.trellis/tasks/09-30-evergreen-harness-contract`。角色：已派发的 `trellis-check`，沿用主会话模型配置，未降低审查模型或递归委派。用户已批准项目内实施。结论：修正 1 项上下文核对缺陷；T03 的 AC1–AC5 通过本地验收。首次产品门的 Rust 中断仍保留为未关闭的失败证据。

已读取 hook 保存文件、任务 JSONL 与各条目、PRD、设计、执行清单、领域说明、适用 spec 和 T01 独立审查。派发路径、注入路径及 `task.py current --source` 均指向 T03。审查使用 `trellis-check` 与常青 harness 审查方法。

## Findings (fixed)

### R1：Kimi check 角色在派发路径存在时跳过路径一致性核对

- 文件：`.kimi-code/skills/trellis-check/SKILL.md:18`。
- 问题：Step 1 仍要求按顺序查找并在第一个路径处停止。收到 `Active task:` 首行后，角色不会执行 `task.py current --source`。该流程无法发现派发路径与当前任务路径不一致，与本次共享 workflow 增加的确认流程冲突。
- 修正：记录派发路径后仍读取当前任务路径；同时记录实际存在的注入路径。非空路径冲突时向主会话确认，随后显式加载确认目标；不得改共享活动任务指针。缺少路径时交回主会话，不猜测任务。
- 回归：`scripts/check-agent-contract.js:97` 增加两条 Kimi 标记检查。`tests/check-agent-contract.test.js:76` 增加“跳过派发路径核对”和“忽略任务路径冲突”两个负例。原有删除全部派发前缀的 fixture 处理仅作用于 harness 文档，防止该处理掩盖新负例的实际原因。
- 结果：`review-focused.result.json` 记录 39 个测试通过，0 失败、0 skipped。修正后才执行最终完整产品门。

审查仅修改上述 3 个获批文件及本任务研究证据。修正前已与实施者协调；最终正式检查期间未改产品或合同文件。业务 skill 源未被审查再次修改，已向主会话释放给 T02。

## 合同与实现核对

- `AGENTS.md` 自包含五工具共享合同；`CLAUDE.md` 只单向导入 AGENTS。frontend index 指向共享入口。T01 的 `--include=dev --audit-level=high`、独立审计门及七条 Windows native step 已同步到 root frontend quality spec。
- 共享 AGENTS、workflow、harness 文档、业务 skill 源和 Kimi check override 明确区分只读审查与已批准实施。任务状态、skill、写工具及原生能力均不扩展授权。默认由用户应用个人 TOML 建议；agent 的 TOML 例外需要用户明确授权和范围。生成的 local JavaScript 仍禁止手改。
- `.codex/config.toml` 的修改只涉及注释。用 Python `tomllib` 比较 HEAD 与当前文件，解析结果完全相同：`project_doc_fallback_filenames = ["AGENTS.md"]`、`agents.max_depth = 1`。没有新增 feature、权限或信任设置。
- V1/V2 说明与本轮审计已保存的两份第一方 schema 摘录一致：`max_depth` 限制 V1，V2 忽略该字段。当前 CLI 的 V2=false 被限定为该入口的观测；提示词没有被描述为硬沙箱。来源证据为父任务 `research/codex-schema.json`，本审查未重新进行联网抓取。
- 四个覆盖由 `git ls-files` 独立确认已跟踪；检查器读取固定的项目文件清单，不读取用户全局配置、真实 TOML、数据库或 ignored 安装态。检查器不联网。负例使用临时公开 fixture。路径检查拒绝词法越界或缺失的任务引用。
- 新 checker 接入 root check/test。检查覆盖入口、门名称、四个覆盖、具体任务引用、成对阶段标记和授权标记。机械检查没有被宣称为语义或客户端执行证明。
- `scope-integrity.json` 记录阶段标记序列保持不变；CLAUDE、Kimi implement/research、`storage_lifecycle.rs`、template hashes 均保持原状。独立 `review-structure.json` 保存审查文件 SHA256、Codex 解析比较及四覆盖跟踪结果。

## Findings (not fixed)

### U1：首次完整产品门发生 SQLite 硬期限中断

`product.result.json` 记录首轮 `just ci` 退出 1：Rust 单元测试 541 通过、1 失败、6 ignored。失败位置为 `residential-monitor/src-tauri/src/storage_lifecycle.rs:646`，测试 `storage::lifecycle::tests::session_fanout_yields_a_durable_prefix_before_hard_deadline` 在 reopen 后 cleanup 的 `unwrap()` 收到 SQLite `OperationInterrupted`。

实施者已按真实调用路径记录取消标记与 250ms 硬预算。该次墙钟预算耗尽的具体原因未查明。产品 Rust 源码不在 T03 范围，审查没有修改源码、超时或测试阈值。一次同版本定向复测通过，随后在无其它测试或构建竞争时的正式重跑通过。两次成功不关闭首次中断的根因调查。详见 `failure-investigation.md`、`product.*`、`rust-targeted.*` 和 `product-rerun.*`。

### U2：安装态与客户端运行证据仍分属后续任务

T02 负责将稳定业务源同步到实际 skill 目录。T04 负责隔离 bootstrap。T06 负责五客户端 fresh-session 与 hosted 证据。T03 只完成共享说明及本地结构验证。没有将 `--version`、inspect、配置文件存在或本次注入记为五客户端动态 PASS。

## Verification

最终完整门由实施者运行，审查读取原始结果、日志及范围哈希后复用。审查没有重复运行完整门。

| 检查 | 结果 | 收据 |
| --- | --- | --- |
| 聚焦说明检查 fixture | PASS：39 通过、0 失败、0 skipped | `review-focused.*` |
| 首轮 `just ci` | FAIL：exit 1；Rust 541/1/6；随后 root 门未执行 | `product.*` |
| 同版本 Rust 定向复测 | PASS：1 通过、547 filtered；首轮 durable prefix 105.9169ms | `rust-targeted.*` |
| 最终 `just ci` | PASS：exit 0；2026-09-30 12:55:54–12:56:55 UTC | `product-rerun.*` |
| Lint / TypeCheck | PASS：monitor ESLint、TypeScript、Rust fmt/clippy；root 语法及说明检查通过 | `product-rerun.log` |
| 最终产品测试 | Node 182；Vitest 73 文件/300 测试；Rust 单元 542、kill_gate 集成 3；Rust ignored 6；doc tests 0 | `product-rerun.log` |
| `just docs-build` | PASS：exit 0；2026-09-30 12:57:41–12:57:44 UTC | `docs-build.*` |
| phase/packages 与 Codex/Kimi 平台 phase | PASS：使用 `--platform kimi-code`；先前 `kimi` 参数只取到公共块的情况已记录 | `phase.*`、`packages.*`、`phase-codex.*`、`phase-kimi-code.*` |
| 静态范围核对 | PASS：Codex 配置语义不变；四覆盖 tracked；保留文件及阶段标记未改 | `review-structure.json`、`scope-integrity.json` |

根脚本无独立 linter 或 TypeScript 工程。以上结果来自本机 Node 26.7.0；Node 18/20/22 hosted 矩阵仍需相应运行证据。6 个 ignored 未记为通过。T01 的依赖审计收据继续保留，本次合同复核没有修改锁文件或审计行为。

## Acceptance Criteria

| AC | 状态 | 依据 |
| --- | --- | --- |
| AC1 共享权威与单向入口 | PASS | AGENTS 自包含，CLAUDE 单向导入，frontend 共享入口；正反 fixture |
| AC2 个人 TOML 授权边界 | PASS | AGENTS 与业务源一致；用户应用建议，例外显式授权；负例覆盖 |
| AC3 只读与 self-fix 范围 | PASS | 共享合同、workflow 执行块及 Kimi override；路径冲突修正；未新增自动信任 |
| AC4 V1/V2 保证范围 | PASS | 注释和文档对应保存的 schema；TOML 解析值保持不变；深度负例 |
| AC5 确定性结构检查 | PASS | 正常仓库、LF/CRLF fixture、破坏例、CLI 非零结果、root 接线及最终正式门 |

T03 静态交付可供后续任务使用。父任务整体状态、性能验收和五工具运行证据按原责任继续处理。本审查未提交、归档、push、创建 PR 或触发远端 workflow。
