# 五套 Harness 的启动与委派

本页记录 Claude Code、Codex、Grok Build、Kimi Code、OMP 在本仓库的入口、委派、权限与 fallback。共享阶段规则在仓库文件 `.trellis/workflow.md`。

三层分开读：

- **原生能力**：官方产品支持的规则、skill、子代理、hook 或 extension。
- **项目配置**：本仓库当前文件与本项目的路由选择。
- **运行证据**：本机客户端在新会话中实际发现并执行。静态文件存在不能代替新会话握手。

本机版本快照（审查日 2026-09-30，见 `.trellis/tasks/09-30-evergreen-five-harness-audit/research/audit.md`）：Claude Code `2.1.285`，应用内 Codex CLI `0.159.2`，Grok Build `1.0.45`，Kimi Code `2.0.0`，OMP `18.4.4`。项目 Trellis 模板为 `0.7.0-beta.3`；PATH 中的 CLI 为 `0.6.17`，尚不满足本页 bootstrap 前提。PATH 首位 Codex npm 包装器缺少 Windows 可选二进制，应用内入口可运行。版本发现不能替代新会话执行。历史审查在 `.trellis/tasks/archive/2026-09/09-07-evergreen-harness-audit/research/harness-audit.md`。

从**仓库根**启动客户端。Codex 的 `.codex/hooks.json` 命令使用仓库相对路径；从子目录启动时 hook 可能无法解析。

## 能力、配置与委派

| Harness | 原生能力 | 本项目配置 | 入口 | 委派 | 权限 / 约束 | Fallback |
|---|---|---|---|---|---|---|
| Claude Code | `CLAUDE.md` 每会话加载；项目 `.claude/skills/`、`.claude/agents/`、lifecycle hooks | 根 `CLAUDE.md` 引用共享 `AGENTS.md`。`.claude/settings.json` 配置 SessionStart、UserPromptSubmit、任务委派与读写后上下文 hook。`.claude/agents/trellis-*` 为原生角色 | 项目 hook 注入 workflow-state | 主会话派发项目 `trellis-*` 子代理。Dispatch 首行 `Active task: <path>` | 项目 hook 按客户端策略执行。本页不代替用户批准 | hook 未触发时显式读 `task.py current`、JSONL、`prd.md` / `design.md` / `implement.md` 与 spec |
| Codex | 发现并拼接选中的 `AGENTS.md`；项目 `.agents/skills/`、`.codex/agents/`、`.codex/hooks.json` | `.codex/config.toml` 设 `agents.max_depth = 1`。`.codex/hooks.json` 配置 SessionStart、UserPromptSubmit、SubagentStart、PreToolUse。项目 config 不写用户级 feature flag，不自动批准 hook | 项目 hook 注入。仓库必须 trusted；每个 hook 须经 `/hooks` 精确 hash 审查 | 主会话派发项目 `trellis-*`。`SubagentStart` 可注入上下文；子代理侧仍保留 pull | 先信任项目，再审查 hook hash。从仓库根启动 | hook 未生效时显式读 task / JSONL / spec。不为未复现的子目录启动问题改 wrapper |
| Grok Build | 项目 `.grok/skills`、`.grok/agents`、`.grok/hooks`；`spawn_subagent` 可按自定义 agent 类型委派 | 本项目不用自动 SessionStart。`.grok/commands/trellis-start.md` 要求显式调用 `get_context.py`。`.grok/agents/trellis-*` 通过 `Active task:` 首行与 JSONL pull 取上下文 | 显式 pull | `spawn_subagent`，`subagent_type` 为 Trellis 角色名（如 `trellis-implement`）。Dispatch 首行 `Active task: <path>` | 按项目 agent 定义执行。Claude-compatible hooks 在本项目为 disabled | 继续 `get_context.py`、`task.py current --source`、JSONL 与任务产物 |
| Kimi Code | 官方同时提供 built-in `coder` / `explore` / `plan` 与项目自定义 agent（`.kimi-code/agents/` 或 `.agents/agents/`）；也支持 skill 与 hook | 平台支持项目自定义代理。本项目仍选择 built-in `coder` + `.kimi-code/skills/trellis-{implement,check,research}/SKILL.md` pull，不创建 `.kimi-code/agents/` | 显式 pull | 主会话把 built-in `coder`（研究角色同样用 `coder`，因为 `explore` 只读）与对应 role skill 一起派发。Dispatch 首行 `Active task: <path>`。禁止递归 implement/check，也不派发项目 `trellis-*` agent 类型 | 子代理自行读 JSONL / task / spec。能力不足或边界变化时交回规划/审查模型 | `task.py current --source`，再读 jsonl、`prd.md`、`design.md`、`implement.md`。`kimi doctor` 不能证明项目 skill 已加载 |
| OMP | `AGENTS.md` context provider；项目 `.omp/agents` 可定义 task agent；`.omp/skills` 提供流程；extension API 提供生命周期事件 | `.omp/extensions/trellis/index.ts` 从当前目录向上找 `.trellis` 根并注入上下文。`.omp/agents/trellis-*` 是项目 task agent。`model: pi/task` 是会话模型继承选择器 | 项目 Trellis extension 注入 | 主会话派发项目 `trellis-*` task agent。Dispatch 首行 `Active task: <path>` | 按 extension 与 task agent 定义执行 | extension 未注入时显式读 task / JSONL / spec |

## 授权与审查范围

五工具共同遵循根 `AGENTS.md`。只读审查可以运行检查，并写入明确获准的研究产物；不得修复产品代码或配置。任何 `trellis-check` 角色或 skill 的 self-fix 都只适用于用户已批准的实施，并受任务和文件范围限制。创建任务、`in_progress` 状态和可用写工具均不扩展授权。需要额外文件、行为、权限或验收门变更时，先交回规划/审查模型。不得自动信任项目或 hook，不得绕过权限。

业务 skill 的个人调优默认输出建议，由用户应用到 `*.local.toml`。agent 手改 TOML 的例外必须来自用户明确授权，并限定改动范围。生成的 `*.local.js` 始终由渲染器生成。

Codex 的 `agents.max_depth = 1` 保留 V1 agent threads 的深度限制。官方当前 schema 和 `rust-v0.159.2` schema 均注明 V2 忽略该字段。本轮可用 CLI 的 `features list` 返回 `multi_agent_v2=false`；该结果不证明另一个应用会话的后端。角色提示禁止 implement/check 递归，但提示词不提供硬沙箱或 V2 深度保证。不要为验证合同而切换用户级 feature、信任或权限。

## 按问题分工

共享合同、需求与验收标准、根因、权限策略、最终审查由用户指定或当前可用的强模型承担。

| 问题类型 | 优先工具 |
|---|---|
| 共享合同、复杂规划、根因、最终审查 | Claude Code 或 Codex |
| 独立文档或路由反证审查 | Grok Build |
| 显式多 provider 的独占执行 | OMP |
| 目标、文件所有权和验收标准已封闭的单一实现、资料持久化或检查 | Kimi Code（built-in `coder` + role skill） |

较便宜的执行模型只接收单一角色、独占文件范围、明确输入/输出、可运行的验收和升级条件。出现上下文缺失、任务边界变化、授权扩大、测试相互矛盾或需要降低验收标准时，执行者停止扩张并交回审查模型。

## 只读环境诊断

适用 Claude Code、Codex、Grok Build、Kimi Code、OMP，以及 Trellis CLI。运行 `just check-harness-environment` 或 `node scripts/check-harness-environment.js`，输出 JSON。默认读取当前仓库的 `.trellis/.version`，不在脚本中固定工具版本。Windows 使用不加载 profile 的 PowerShell 查询入口，保留发现顺序；其它平台按 PATH 查找可执行文件。

- 每个工具分别记录首选入口、版本、退出码、原始输出和已发现替代入口。包装器失败不会被另一入口成功覆盖。入口缺失、进程失败、权限错误和未知版本分别呈现。
- `project` 记录项目版本和四个本机覆盖文件的存在性与哈希。覆盖不进入版本控制，缺失只报告、不阻断。`permissions` 保持 `UNVERIFIED`；版本输出不证明项目或 hook 已获准。诊断不读取全局信任配置，不修改 PATH、权限或信任。
- `bootstrap.status` 只判断所选 Trellis 的成功退出，以及明确版本与项目版本匹配。缺失、失败、未知或不匹配均为 `BLOCKED`。升级警告中的版本不能作为实际版本。
- 总退出码还包含五客户端首选入口的可用性，因此客户端失败可与 `bootstrap.status=READY` 同时出现。READY 仅表示已检查的前提满足；诊断始终不获取工具、不执行 init，也不授予执行权限。

选择已核验的绝对入口或补充替代入口时，直接使用 Node 命令；以下路径须替换为实际路径：

```bash
node scripts/check-harness-environment.js --entry "trellis=<固定版本执行器的绝对路径>"
node scripts/check-harness-environment.js --alternative "codex=<已核验的应用内可执行文件绝对路径>"
```

`--entry` 明确选择入口，不回落 PATH；`--alternative` 只补充证据，不替换首选入口。Node `.js` 入口由当前 Node 执行。根质量门只运行此脚本的语法检查和独立 fixture，不把开发者安装态或客户端可用性设为 hosted CI 条件。

## Bootstrap

在**缺少被忽略的 harness 资产**的新隔离目录中，放入当前项目合同、`.trellis/.version` 和 `.gitignore`；如需保留本机覆盖行为，再从本机工作树复制下列四个文件并记录初始 SHA256。必须先明确选择 Trellis 入口，再用同一入口检查版本并执行 init；所选 CLI 的退出码必须为 0，明确版本必须与项目 `.trellis/.version` 完全一致。缺少可用匹配入口时记录 `BLOCKED`，不得以 `--help`、包元数据或历史成功替代实际验证。默认 PATH 入口不匹配时停止，不自动升级全局包或修复 PATH。

获准使用隔离固定版本执行器时，把工具依赖与 npm cache 放入本任务新建的临时目录，候选项目使用另一目录。包版本从项目 `.trellis/.version` 取得，禁止使用未固定版本或 `latest`。优先 `--ignore-scripts`；保留本地 lockfile、实际解析的 CLI/core/依赖版本、Node 与 CLI 绝对入口及命令退出码。若使用隔离 npm 配置，userconfig 与 globalconfig 必须为两个不同文件。临时获取不扩大到全局工具或应用安装。

版本检查与 init 必须调用同一已核验入口。命令名 `trellis` 只适用于已确认该名称解析到所选入口的场景；隔离 Node 包应使用该包的绝对 `bin/trellis.js` 路径。前置检查失败时不要继续 init，也不要回落旧全局版本。仅在新候选目录中执行：

```bash
trellis init --claude --codex --grok --kimi --omp --skip-existing -y
```

隔离 Node 包的等价形式：`node "<临时工具目录>/node_modules/@mindfoldhq/trellis/bin/trellis.js" init --claude --codex --grok --kimi --omp --skip-existing -y`。两种形式均不适用于当前工作树的广泛初始化。

`--skip-existing` 保留已经存在的文件。下列四个覆盖是本机资产，不再随仓库进入新 checkout；init 候选目录若由使用者复制了这些文件，init 不得改写其字节：

- `.codex/config.toml`
- `.kimi-code/skills/trellis-implement/SKILL.md`
- `.kimi-code/skills/trellis-check/SKILL.md`
- `.kimi-code/skills/trellis-research/SKILL.md`

本节适用 Claude Code、Codex、Grok Build、Kimi Code、OMP。选用匹配 CLI 且 init 退出 0 只证明初始化。四个可选覆盖缺失不阻断 init；固定 `0.7.0-beta.3` 的五平台 init 会实际生成默认覆盖。init 后必须在候选目录另跑 `node scripts/check-agent-contract.js`，校验共享合同与已存在覆盖。不得自动复制本机覆盖掩盖失败或放宽 checker；默认文件存在不证明原生角色已加载。

2026-10-02 的缺失覆盖隔离复验使用 67 个明确公开文件，结果为 **INIT_PASS / CONTRACT_FAIL**：init 退出 0，init 前 checker 退出 0，init 后 checker 退出 1。生成的 Kimi check 默认内容缺少 5 项 AUTH/DISPATCH 授权与目标一致性标记，Codex config 缺少 2 项 DEPTH 适用范围说明。首失败与默认字节保持原样；该候选不代表完整 fresh checkout 副本。

### 项目公开模板与隔离入口

项目提供 `scripts/bootstrap-harnesses.js` 和 `scripts/harness-templates/` 下的四份公开默认模板。公开模板不含本机配置、凭据或模型选择。入口在系统临时目录内建立明确指定的隔离候选，临时目录下的首级目录名须以 `trellis-` 开头。入口复制具名公开合同，在 init 前补齐缺失的四份覆盖。已存在的覆盖逐字保留；不合合同的已有内容由原 checker 报错，入口不覆盖修复。

```bash
node scripts/bootstrap-harnesses.js --root "<系统临时目录中的新候选>" --entry "<匹配版本的绝对 bin/trellis.js 或 exe 路径>"
```

入口先用所选程序核对 `.trellis/.version`，再用同一程序执行原五平台 init 参数，最后运行候选的合同 checker。JS 入口由当前 Node 执行；不接受 `.cmd` 或 `.ps1` 包装器，不通过 shell 拼接参数。原 `check-harness-environment` 继续只读。入口不获取工具、不改全局 PATH、信任、账户或权限，也不在当前工作树 init。

最小隔离候选用于初始化与合同验证，不能当作完整 fresh checkout。生成完成后再次调用入口应拒绝且保持字节不变。历史缺失覆盖首失败仍保留；公开模板路线的实际 init 与 checker 结果须独立记录。

2026-10-02 的公开模板路线已使用同一 `0.7.0-beta.3` JS 入口完成一次真实隔离初始化：version、init 与入口均退出 0，初始化前后合同检查通过，四份公开覆盖字节保留，五平台必要资产存在。当前工作树的四本机覆盖、原模板 hash、Git 索引、HEAD 与分支保持。该结果验证基本初始化；客户端运行与正式性能结果仍分别记录。

不要手改 `.trellis/.template-hashes.json`，把本地覆盖伪装成上游原稿。`trellis init --skip-existing` 会跳过已存在的四个覆盖并保留其字节。同一次 init 可能按本次平台集合重写该 checkout 的 `.trellis/.template-hashes.json`，并把被跳过的覆盖从 manifest 里删除。不要把候选 checkout 的 hash 文件拷回本仓库。本仓库继续保留现有上游 hash，以便后续 `trellis update` 能标出本地差异。升级 Trellis 时先审查这四个覆盖是否仍需保留。不要在当前工作树做广泛 `trellis init`。

`.claude/`、`.codex/`、`.grok/`、`.kimi-code/`、`.omp/`、`.agents/` 全部由 `.gitignore` 忽略，依赖上述 init 在新 checkout 生成。干净 clone 中没有 `.codex/` 与 `.kimi-code/`，仓库也不持有任何本机 harness 覆盖。

init 成功后检查五平台必要资产：Claude `.claude/agents/trellis-implement.md`；Codex `.codex/hooks.json`；Grok `.grok/agents/trellis-implement.md`；Kimi `.kimi-code/skills/trellis-start/SKILL.md`；OMP `.omp/agents/trellis-implement.md`。候选目录若携带四个覆盖，核对其 SHA256 前后相同。用 `git check-ignore -v` 核验这些 harness 资产，并确认当前仓库 `.trellis/.template-hashes.json` 未变。这些结果只证明隔离初始化与静态资产；五客户端运行验收仍单独记录。业务 skill 的交付另按[源更新后的交付检查](./residential-rule-tuning.md#源更新后的交付检查)执行。

## 基本客户端读取

基本调用可在已记录的竞争负载下运行。每工具使用一个新 parent 会话，读取公开 `AGENTS.md` 与 `CONTEXT.md`，按 UTF-8 读取并返回简短结果。基本调用不派发 child，不修改文件、账户、模型、信任或权限；成功只证明本次入口与公开读取可用。正式性能、hook、有效权限和完整子代理验收继续单独记录。

Kimi Code 2.0.0 的 `--plan` 与 `-p` 互斥。本轮任务内基本调用使用同版本原生 ACP 接口，依次执行 `initialize`、`session/new`、`session/set_mode(plan)`；确认响应和 `current_mode_update=plan` 均到达后才发送 prompt。权限请求取消并停止；文件回调仅允许两份具名公开文件。原生退出、协议结果、driver 退出、外层退出和敏感停止分别保存。该路线不改变本机配置，也不据此声明 OS 隔离已验证。

## 手动 pull fallback

任一 hook、extension 或自动注入未触发时，从仓库根执行：

```bash
python ./.trellis/scripts/task.py current --source
python ./.trellis/scripts/get_context.py
python ./.trellis/scripts/get_context.py --mode packages
python ./.trellis/scripts/get_context.py --mode phase --step <step>
```

先核对派发首行 `Active task: <path>`、注入头部的任务路径和 `task.py current --source`。路径不一致时，停止依赖该注入内容并向主会话确认派发目标；不得仅凭 `trellis-hook-injected` marker 宣称上下文正确。主会话确认目标后显式 pull 目标任务，无需改动共享活动任务指针。

然后读已确认目标的 `implement.jsonl` 或 `check.jsonl`、各条目、`prd.md`、`design.md`（若存在）、`implement.md`（若存在）。Dispatch 提示的第一行仍必须是 `Active task: <path>`。

## 运行证据

下表记录 fresh-session smoke。未实际启动对应客户端的新会话时，状态为 **UNVERIFIED**。静态适配完成不等于五工具动态对齐完成。

| Harness | 适用版本 | 已读 AGENTS/spec | planning 状态 | 委派前缀 | hooks / extension / pull | 权限 | 结果 |
|---|---|---|---|---|---|---|---|
| Claude Code | `2.1.285` | 未跑 live smoke | 未跑 | 未跑 | 未跑 UserPromptSubmit | 未跑 | **UNVERIFIED** |
| Codex | `0.159.2` | 未跑独立 fresh-session smoke | 未跑 | 未跑完整验收 | 当前任务注入不替代新会话验收 | 未跑完整验收 | **UNVERIFIED** |
| Grok Build | `1.0.45` | 未跑 live fresh-session smoke | 未跑 | 未跑 | 未跑本轮只读 `spawn_subagent` | 未跑 | **UNVERIFIED** |
| Kimi Code | `2.0.0` | 未跑 live smoke | 未跑 | 未跑 | `doctor` 不能代替 | 未跑 | **UNVERIFIED** |
| OMP | `18.4.4` | 未跑 live smoke | 未跑 | 未跑 | 未跑 extension 与 task-agent child | 未跑 | **UNVERIFIED** |

审计阶段的 `grok inspect --json` 只证明配置发现元数据，不构成本页的 fresh-session PASS。

当前结论：静态适配可单独验收。五工具动态对齐未完成。

## 结构检查与第一方依据

`npm run check:agents` 使用 Node 标准库检查共享入口、质量门、任务引用和约束标记；本机存在的四个覆盖同时受检，干净 clone 缺失时跳过。根 `npm run check` 运行该检查，`npm test` 包含正反 fixture。检查不读取忽略的安装态，不联网，也不证明语义无冲突或五工具运行成功。语义由强模型审查；安装态由 skill 同步任务验证；新会话证据单独记录。

以下地址由 2026-09-30 审查读取，摘录与抓取结果保存在上述本轮审查目录。原生能力、项目选择和运行结果分别记录。

- Claude Code：<https://code.claude.com/docs/en/sub-agents>
- Codex AGENTS、subagents、hooks：<https://learn.chatgpt.com/docs/agent-configuration/agents-md>、<https://learn.chatgpt.com/docs/agent-configuration/subagents>、<https://learn.chatgpt.com/docs/hooks>
- Codex schema：<https://developers.openai.com/codex/config-schema.json>；版本对照：<https://raw.githubusercontent.com/openai/codex/rust-v0.159.2/codex-rs/core/config.schema.json>
- Grok Build subagents、project rules：<https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/16-subagents.md>、<https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-pager/docs/user-guide/12-project-rules.md>
- Kimi Code agents：<https://www.kimi.com/code/docs/en/kimi-code-cli/customization/agents.html>
- OMP task discovery、context：<https://github.com/can1357/oh-my-pi/blob/main/docs/task-agent-discovery.md>、<https://github.com/can1357/oh-my-pi/blob/main/docs/context-files.md>
