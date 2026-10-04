# T06 捕获执行器准备与待执行清单

日期：2026-09-30。状态：PREPARED, WAITING FOR GO。T05 候选仍在修改；本轮没有运行 freeze、五客户端推理或原生 child。

本文件记录主会话已批准的准备范围。GO 字符串只是执行器的操作防错条件；用户授权仍由 AGENTS 与 T06 合同确定。源码或任务状态不能自行产生授权。

## 文件

- runtime-capture.py：Python 标准库一次性捕获器，默认 plan。
- runtime-parent-prompt.txt：相同公开输入、纯输出、负向语言判断，不派 child。
- runtime-child-prompt.txt：相同公开输入、既定原生 research 角色、单个 child、纯输出合成协议。角色持久化明确未覆盖。
- omp-native-entry-review.md：本机 18.4.4 源码与 schema 依据，区分默认原生 child 路线和有限工具 parent 路线。
- runtime-runs/preparation-f3e7d09a/verification.json：17 个最终准备检查及 3 个合成 Python 进程的捕获结果。不是客户端运行证据。

先前 runtime-scope-review.md 与 runtime-preflight-review.md 均保留。两份报告中此前未准备 OMP 正常入口的状态由本文件及 omp-native-entry-review.md 补充；没有将未运行状态改写为 PASS。

## 固定命令

执行器从父审计 tool-versions.json 读取已核验的绝对入口；Codex 只选择 codex-app-binary，保持 PATH 包装器失败单独记录。prompt 作为最后一个完整 argv 元素传入，Popen 使用 shell=False。OMP 的 ps1 由现有 pwsh 的 -File 入口调用，禁用 PowerShell profile，不使用拼接的 -Command。

| Harness | Parent argv（执行器路径与 prompt 除外） | Child argv 与角色 |
| --- | --- | --- |
| Claude | --restricted --strict-mcp-config --permission-mode plan --permission-prompts none --tools Read,Glob,Grep --no-session-persistence --output-format stream-json --verbose --include-hook-events -p | 原候选增加 Agent 至 tools，并使用已核验 --forward-subagent-text；Agent + trellis-research |
| Codex | exec --sandbox read-only -c approval_policy="never" --ephemeral --json | 同一命令；当前原生 spawn_agent + trellis-research，V1 非 full-history，不切 backend/model |
| Grok | --permission-mode plan --disable-web-search --output-format streaming-messages-json --max-turns 8 -p | 同一命令；spawn_subagent 的既定 trellis-research；actual plan 以事件为准 |
| Kimi | --plan --output-format stream-json -p | 同一命令；Agent + built-in coder + research role skill；拒绝时不退出 plan、不换 explore |
| OMP | --mode json --no-session --no-title --no-prewalk --no-lsp --no-pty --no-extensions --tools read,grep,glob --approval-mode write -p | 独立预选正常路线：--mode json --no-session --no-title --no-prewalk --no-lsp --no-pty -p；原生 task + trellis-research，当前 schema 决定单次/batch 参数形状 |

OMP child 路线沿用现有 tools、审批与 extension 配置，没有增加 yolo/plan-yolo、trust 或 model 参数。该路线从准备时就单独定义；不会由 parent 失败触发参数放宽。动态 schema 可能与静态类型不同，父会话必须按本次实际工具 schema 调用，且总计只派一个 child。

Claude restricted 的 settings/model 解析与正常启动可能不同。记录实际元数据，不能声称通常模型或项目 hooks 已验证。OMP parent 的 no-extensions 不覆盖项目 Trellis extension。OMP child 的正常 discovery 也需要本次事件，不能凭默认行为宣称 extension 已运行。常规 Claude hooks 路线还没有接入本捕获器；已有 trust 证据不足时保持未验证，不通过 -p 跳过对话构造信任证明。

## freeze 与 execute 条件

所有准备文件先由主会话审查。等候 T05 源码冻结和正式执行信号后，才运行：

    python .trellis/tasks/09-30-evergreen-runtime-validation/research/runtime-capture.py plan
    python .trellis/tasks/09-30-evergreen-runtime-validation/research/runtime-capture.py freeze

plan 只打印 argv，不创建最终 manifest、不启动客户端。freeze 写 runtime-candidate.json，使用排他创建；已存在时拒绝覆盖。历史 manifest 必须保留，不能自动覆盖失败候选。

freeze 记录 HEAD、branch、tracked/index 状态、公开新增文件及逐文件 SHA256；另显式包含四覆盖、研究角色、hook/extension 与七目标业务 skill 的 21 个 payload。执行器、两 prompt、研究合同也在保护范围。外部工具记录入口 SHA256；OMP 另记录 pwsh、实际 bun 和 cli.js 的身份。版本字符串来自已保存版本收据，不能替代本次 runtime model/version 事件。

保护扫描只排除本次 runtime-runs 输出目录与 runtime-candidate.json；其余 .trellis 文件继续参与。另遵守私有读取边界，按名称跳过 local TOML/JS/YAML、SQLite/数据库附件、认证文件、私钥、环境秘密文件、native 历史/会话路径、构建/依赖目录。私有文件名识别同时覆盖常见备份、带日期副本及压缩后缀，包括 .bak、.bak-*、.old、.backup、~、.gz、.tar.gz、.zip 等。已知公开 *.local.toml.example 以及普通源码后缀继续参与保护。跳过项只计数量，不读取内容。显式保护文件越界或为 symlink 时停止并交回审查。

2026-09-30 的私有副本边界修订仅作静态编辑。合成路径清单位于 runtime-private-path-cases.json，状态 PREPARED_NOT_RUN；没有搜索、读取、哈希或创建这些路径对应的文件。当前 T05 正式独占测量，修订后的语法、路径判定及回归验证均等待主会话结束正式负载后执行。此前准备检查只证明此前版本，不能转移为本次修订通过。按名称过滤不提供任意秘密内容、重命名文件或无明确名称归属的压缩包识别保证。

收到主会话明确 GO 后，使用 manifest 中的 source_snapshot_id：

    python .trellis/tasks/09-30-evergreen-runtime-validation/research/runtime-capture.py execute --phase parent --harness all --go GO:<source_snapshot_id>

每个客户端前后按同一保护范围重算哈希，检查新添/删除、index、HEAD、branch。与冻结清单不一致则不启动下一工具。只生成独立时间戳目录，不覆盖旧收据。五工具顺序固定为 Claude、Codex、Grok、Kimi、OMP，每次新进程、无 resume、无模型 override。

GO 不自动授权 child 阶段。主会话按已有 T06 授权审核 child 准备依据后，在 runtime-runs/control/child-readiness.json 写以下结构。该准备依据可以来自已安装同版本源码、已审核本地前置研究或实际 runtime schema；不要求无 Agent/task 的 parent-only 会话先产生不可能存在的 child 调用事件。

    {
      "source_snapshot_id": "<冻结值>",
      "reviewed_by_main": true,
      "harnesses": {
        "omp": {
          "approved_route": "task + 项目 trellis-research；字段以本次实际 schema 为准",
          "basis_kind": "installed_source_schema",
          "basis_reference": {
            "path": ".trellis/tasks/09-30-evergreen-runtime-validation/research/omp-native-entry-review.md",
            "locator": "本机来源表与预先选定的命令"
          }
        }
      }
    }

basis_kind 允许 installed_source_schema、runtime_schema_event、reviewed_local_prerequisite。basis_reference 指向 T06 research 内真实存在的报告或去敏事件并提供定位。其他工具按 runtime-capture.py 的 ROLES 原文填写 approved_route。执行器检查路径和准备记录，不把该记录视作动态权限或调用证明。child 的实际工具发现、调用和模型仍由本次事件验收。

    python .trellis/tasks/09-30-evergreen-runtime-validation/research/runtime-capture.py execute --phase child --harness all --child-readiness .trellis/tasks/09-30-evergreen-runtime-validation/research/runtime-runs/control/child-readiness.json --go GO:<source_snapshot_id>

## 捕获、退出和时间预算

- 默认每进程 180 秒，允许显式选择 30–300 秒。五工具单阶段进程累计预算默认 15 分钟；文件哈希、启动和清理时间另计。parent/child 是两个独立阶段，任何阶段均不自动启动下一阶段。
- stdin 为 DEVNULL。认证、额度、trust、权限或不支持参数的明确机器错误保留；给自然退出最多 5 秒，然后只终止本次进程树。普通客户端非零退出可继续记录下一独立工具；保护文件改变、捕获故障、超时、输出上限或敏感输出时停止整批。
- Windows Job 仅用于本次进程生命周期清理。Popen 后 AssignProcessToJobObject 存在短竞态；清理是 best-effort，不能宣称 OS 沙箱或绝对无残留。attach 失败保留具体错误、PID 与已知退出状态，不尝试修改系统权限。
- 每条记录上限 1 MiB，每流捕获上限 16 MiB。截断后只对已捕获字节提供 SHA256，不能称完整流哈希。超时或主动终止会独立记录原因；exit 0 不覆盖这些失败。
- 保存标准化去敏事件 JSONL、每流原始字节的内存 SHA256、事件顺序/流序号、接收 UTC、退出码和 redaction 字段位置。不保存未审查的原始 stdout/stderr 字节，不把去敏文件称原始流。
- stdout 与 stderr 的全序是接收器观测顺序；每流内部顺序保留。结构化 secret 字段、认证头、密钥形状、带凭据 URL 与账户 home 路径去敏。检测到秘密模式则停止。去敏规则不提供任意私有内容的完整识别保证；prompt 首先限制公开输入，出现意外私有读取时必须人工停止并保留去敏事故记录。

## 审查输出

每工具生成 invocation.json、prompt.txt、before.json、after.json、events.redacted.jsonl、receipt.json。保存传入的完整公开 prompt 与其 SHA256；argv 中的 prompt 以该哈希引用，账户 home 路径去敏。最终 batch.json 只汇总运行状态，不填验收 PASS。

effective_model、rules、planning、permissions、runtime_enforcement、native_child、hook_extension_pull、negative_case 默认 UNKNOWN/UNVERIFIED。强模型审查时逐项引用原生事件位置：模型 metadata、实际文件读取返回、角色调用、child 生命周期和语言拒绝。自然语言自报不能填补字段。源码 schema 准备依据与实际 runtime 结果分开。

product_hash_unchanged 仅指受保护清单的前后内容与 Git 元数据相同。该字段不证明期间从未写入后回滚、全文件系统零写入、仓库外副作用或权限强制。客户端自身既有 cache/session 副作用保持单独未验证。纯输出 child 一律不覆盖角色持久化。

## 已验证与尚缺证据

已完成：脚本语法；10 组 parent/child prompt 单 argv；OMP 正常 child 不新增 tools/approval/model/extension 参数；缺 GO 在进程启动前拒绝；合成正常退出 0、认证错误退出 7、超时退出 0 的独立失败标记。早期另有 36 项静态输入/去敏检查通过。合成收据均明确 synthetic_only=true。

未完成：最终 candidate manifest、真实 CLI 进程输出、实际模型/权限/schema/角色发现、native child、真实 hook/extension、受保护产品范围在客户端运行前后的差异。没有用准备结果关闭任何 T06 AC。T05 未结束，freeze 和所有客户端运行等待主会话 GO。

## 2026-10-01 静态入口复核与定向验证准备

本节保留前述历史收据。旧 `codex-app-binary` 的 `c6fe824d725f02d7/codex.exe` 已不存在。只列本机 app bin 文件元数据后，发现 `de8a38d2100ae498/codex.exe`。该文件的 PE 版本字段为空；当前没有运行 `--version` 或 `--help`，没有哈希该文件，也没有启动推理。

新绑定记录位于 `runtime-bindings-20261001.json`。该记录保留旧入口缺失和旧版本来源；现存入口的版本与哈希为 UNKNOWN。捕获器只接受现有 app bin 内的 `codex.exe`，且在绑定尚未核验时阻断 Codex 的执行。正式负载结束后，先记录新入口的版本、help 及 SHA256，再由主会话审核参数/schema 差异。不得把旧 0.159.2 版本、V1 状态或源码证明转移给新入口。

捕获器现按工具记录入口缺失、依赖缺失和执行器身份变化。单个 Codex 入口阻断不会吞掉 Claude/Grok/Kimi/OMP 的独立收据。OMP 的 pwsh、bun 或 cli.js 缺失仅阻断 OMP。候选源码变化、捕获故障、超时、输出上限或敏感输出仍停止整批。新绑定文件纳入候选保护，历史父任务 `tool-versions.json` 未改。

`runtime-capture-verification.py` 已准备 50 个私有路径字符串、正常授权文本与合成认证头、10 个单 argv prompt、无模型覆盖、入口缺失/身份变化/OMP 依赖缺失/未核验 Codex 的独立收据回归。验证器使用模拟 capture 与 snapshot，不启动客户端、不扫描真实工作树。当前状态 PREPARED_NOT_RUN；待 T05 正式负载结束后执行，并为本轮捕获器版本保存新收据。旧 d44c 与 6ab03f26 的准备通过不覆盖本轮修订。

五工具实际模型仍为 UNKNOWN。既定默认模型、权限、信任和账户保持现状；本轮没有安装、登录、配置变更、产品写入或外部写入。freeze 与两阶段 GO 仍由主会话在候选稳定后下达。运行期间需暂停其它代理对所有保护文件的写入，包括其它任务的研究记录。
