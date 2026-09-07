# Research: 五套 Harness 的项目发现、委派与可恢复性

- Query: 对照 Claude Code、Codex、Grok Build、Kimi Code 与 OMP 的规则发现、skill、子代理、hook/extension、权限和检查能力，并给出可在新 checkout 恢复的最小改造边界
- Scope: mixed（仓库静态配置、本机只读元数据、第一方文档与上游源码）
- Date: 2026-09-07
- Baseline: `dev` / `5578576787dba73ba1b96985a2fc408fc246bfed`

## 结论

五套客户端都能承载 Trellis 工作流，但它们没有一个统一的自动注入入口。Claude Code 与 Codex 在本项目配置了生命周期 hook，OMP 使用项目 extension，Grok Build 与当前 Kimi 路由通过角色文件或 skill 主动读取任务上下文。共享 `.trellis/workflow.md` 应只描述这些实际分支，不能再把 `UserPromptSubmit` hook 写成所有平台的共同机制。

新 checkout 还存在第二个边界：`.agents/`、`.claude/`、`.codex/`、`.grok/`、`.kimi-code/` 与 `.omp/` 当前被整目录忽略（`.gitignore:30-36`），因此 tracked `.trellis/` 不能独自恢复五个平台。最小完整方案是固定并核对 Trellis `0.7.0-beta.3`，用 `trellis init --claude --codex --grok --kimi --omp --skip-existing -y` 生成常规适配器，同时只跟踪四个必须覆盖当前生成器错误的文件：

- `.codex/config.toml`
- `.kimi-code/skills/trellis-implement/SKILL.md`
- `.kimi-code/skills/trellis-check/SKILL.md`
- `.kimi-code/skills/trellis-research/SKILL.md`

`.trellis/workflow.md`、`.gitignore` 和新文档 `docs/agents/harnesses.md` 本来就在（或应进入）正常仓库表面，不属于上述四个生成器覆盖。不要手改 `.trellis/.template-hashes.json` 来把本地覆盖伪装成上游原稿；`init` 可以为新生成文件合法增加元数据，但四个覆盖项仍须保留上游模板 hash，使后续 `trellis update` 能暴露本地差异并要求审查。

## 五工具能力与路由矩阵

表中“原生能力”只表示官方产品支持；“项目配置”表示当前文件存在；只有“运行证据”列能证明本机客户端实际发现。静态文件存在不能替代新会话握手。

| Harness | 原生规则、skill 与子代理能力 | 当前项目路由 | 适合承担的工作 | 运行证据与边界 |
|---|---|---|---|---|
| Claude Code `2.1.263` | `CLAUDE.md` 为每会话上下文并支持 `@path` 导入；项目 `.claude/skills/`、`.claude/agents/` 和 lifecycle hooks 均为原生表面。官方将 skill 定义为按需知识/流程，将 subagent 定义为隔离上下文，将 hook 定义为确定性生命周期自动化。[扩展能力总览](https://code.claude.com/docs/en/features-overview)、[hooks](https://code.claude.com/docs/en/hooks)、[subagents](https://code.claude.com/docs/en/sub-agents)、[skills](https://code.claude.com/docs/en/slash-commands) | 根 `CLAUDE.md` 是业务合同；`.claude/settings.json:5-90` 配置 SessionStart、UserPromptSubmit、任务委派和读写后的上下文 hook；`.claude/agents/trellis-*` 为原生角色。 | 共享合同、复杂规划、根因判断和最终审查；范围已定时可把独占实现或检查交给 `trellis-*`。 | 本轮未启动付费新会话，hook、skill 和子代理的实际加载为 **UNVERIFIED**。现有文件与官方能力只构成静态证据。 |
| Codex `0.153.4` | 从用户层到仓库层发现并拼接选中的 `AGENTS.md`；项目 `.agents/skills/`、`.codex/agents/` 与 `.codex/hooks.json` 为相应扩展表面。[AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md)、[subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents)、[hooks](https://learn.chatgpt.com/docs/hooks) | `.codex/config.toml` 设 `agents.max_depth=1`；`.codex/hooks.json` 配置 SessionStart、UserPromptSubmit、SubagentStart、PreToolUse。命令使用仓库相对路径（`.codex/hooks.json:9,20,32,44`），因此当前项目约定从仓库根启动；失效时显式读取 task/JSONL/spec。 | 跨平台合同、复杂规划、根因判断和最终审查；原生 `trellis-*` 子代理可做边界清楚的实现/研究/检查。 | `codex features list` 仅证明本机二进制把 hooks 与 multi-agent 标为 stable/default；不证明当前项目 hook 已获信任并执行。官方要求先信任项目，再按精确 hook hash 审查；本轮没有新的 `/hooks` 和 child-session 握手，故为 **UNVERIFIED**。原生 Windows 曾有 project `SubagentStart/Stop` 未触发的公开报告，实施后仍要以当前版本实测为准：[openai/codex#33097](https://github.com/openai/codex/issues/33097)。 |
| Grok Build `1.0.22` | 第一方仓库说明项目 `.grok/skills`、`.grok/agents`、`.grok/hooks` 与模型覆盖；`spawn_subagent` 可按自定义 agent 类型委派。[Grok Build README](https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-shell/README.md)、[subagents guide](https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-shell/user-guide/16-subagents.md) | 本项目刻意不用自动 SessionStart：`.grok/commands/trellis-start.md:3-44` 要求显式调用 `get_context.py`；`.grok/agents/trellis-*` 通过 `Active task:` 首行和 JSONL pull 获取上下文。 | 独立文档/路由反证审查；边界固定时可将实现、研究和检查委派给项目 `trellis-*`。 | `grok inspect --json` 是本轮唯一完成的客户端元数据握手：`projectTrusted=true`，分别发现根 `AGENTS.md` 与 `CLAUDE.md`、三个项目 Trellis agent、Trellis skills 和业务 skill；Claude-compatible hooks 被配置为 disabled。它证明发现元数据，不证明模型实际遵守内容或完成任务。 |
| Kimi Code `0.41.0` | 官方同时提供 built-in `coder`/`explore`/`plan` 与项目自定义 agent；项目 agent 可位于 `.kimi-code/agents/` 或 `.agents/agents/`。也支持 skill 与 hook 配置。[agents](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/agents.html)、[hooks](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/hooks.html)、[config files](https://www.kimi.com/code/docs/en/kimi-code-cli/configuration/config-files.html) | 本项目暂不迁移到自定义 agent：主会话用 built-in `coder`，把相应 `.kimi-code/skills/trellis-{implement,check,research}/SKILL.md` 作为角色合同传入；首行必须是 `Active task: <path>`，子代理自行读取 JSONL/task/spec，并禁止递归派发。 | 目标、文件所有权和 AC 已封闭的单一实现、资料持久化或检查任务；能力不足或边界变化时交回规划/审查模型。 | `kimi doctor` 只核对用户配置，不能证明项目 agent/skill 发现。本轮没有 Kimi 新会话或子代理握手，故为 **UNVERIFIED**。现有三个 skill 声称“Kimi Code 不支持项目自定义代理”（implement/check `:49`、research `:22`），与第一方资料冲突。 |
| OMP `18.1.12` | 第一方 `AGENTS.md` context provider 支持向上发现和 `@` 导入；项目 `.omp/agents` 可定义 task agent，`.omp/skills` 提供流程，extension API 提供生命周期事件。[context files](https://github.com/can1357/oh-my-pi/blob/main/docs/context-files.md)、[task agent discovery](https://github.com/can1357/oh-my-pi/blob/main/docs/task-agent-discovery.md) | `.omp/extensions/trellis/index.ts:11-19` 从当前目录向上找 `.trellis` 根并注入上下文；`.omp/agents/trellis-*` 是项目 task agent。`model: pi/task` 是 OMP 的会话模型继承选择器，不应因 `omp models find pi/task` 无普通模型条目而判错。 | 显式多 provider 的独占执行任务；复杂规划和跨平台最终审查仍需固定强模型，不允许执行者自行降低 AC。 | 本轮没有 OMP 新会话、extension event 或 task-agent child 握手，故为 **UNVERIFIED**。结论来自第一方源码/文档与本机已安装源码，未把文件存在当作运行成功。 |

## 关键发现与所有权

### P1：Codex 的共享合同发现存在跨客户端兼容缺口（C2 所有）

根 `AGENTS.md:23` 只有 `@CLAUDE.md`。Claude Code 与 OMP 明确支持这种导入；Codex 官方文档只承诺发现并拼接选中的 `AGENTS.md`，没有声明扩展 `@file`。上游仍以功能请求形式追踪这一能力：[openai/codex#17401](https://github.com/openai/codex/issues/17401)、[openai/codex#28739](https://github.com/openai/codex/issues/28739)。这构成 fresh session 的兼容风险：仅按文档行为时，Codex 可能只看到字面量 `@CLAUDE.md`，遗漏命令、secret 边界、语言约定和业务 skill 路由；本轮当前会话确实收到了聚合后的项目说明，因此不能把风险写成已发生的运行故障。修复应由 `09-07-evergreen-project-contract` 把必要共享合同直接放到 canonical `AGENTS.md`，再让 `CLAUDE.md` 以 Claude 兼容入口引用共享合同；C3 不重复维护这份内容。

### P2：共享 workflow 把不同自动化机制写成同一个 hook（C3 所有）

`.trellis/workflow.md:102-107` 把 breadcrumb 称为“every supported AI platform's UserPromptSubmit hook”读取。实际只有 Claude/Codex 走这里的 hook 配置；OMP 走 `.omp/extensions/trellis/index.ts`；Grok/Kimi 当前走显式 pull。应将 source-of-truth 说明改为“各平台适配器读取同一 `[workflow-state:*]` block”，再逐平台说明 hook、extension 或 pull。

### P2：Kimi 的工作流和角色说明互相冲突（C3 所有）

`.trellis/workflow.md:223` 已正确规定 Kimi 用 built-in `coder`/`explore` 配合 role skill；`:226` 又说不存在这些 skill；`:356-364` 与 `:490-502` 又把 Kimi 放进必须派发 `trellis-*` 自定义 agent 的通用分支。三个 Kimi role skill 还把本项目的选择误写成平台不支持自定义 agent。修复应统一为：平台原生支持项目 agent；本项目这一版刻意保留 built-in `coder` + role skill + `Active task:` + pull，不创建 `.kimi-code/agents/`。

### P2：被忽略的平台目录使本地修正无法进入新 checkout（C3 所有）

仅改 ignored 文件会形成当前机器有效、克隆后消失的修复。C3 应修改 `.gitignore`，只放行上述四个覆盖文件，并把其它 settings、hooks、agents、skills、cache 和本机状态继续排除。由于 Git 不能重新包含仍被忽略目录中的子文件，规则必须逐层打开父目录后再收紧，例如：

```gitignore
.codex/*
!.codex/config.toml

.kimi-code/*
!.kimi-code/skills/
.kimi-code/skills/*
!.kimi-code/skills/trellis-implement/
!.kimi-code/skills/trellis-check/
!.kimi-code/skills/trellis-research/
.kimi-code/skills/trellis-implement/*
.kimi-code/skills/trellis-check/*
.kimi-code/skills/trellis-research/*
!.kimi-code/skills/trellis-implement/SKILL.md
!.kimi-code/skills/trellis-check/SKILL.md
!.kimi-code/skills/trellis-research/SKILL.md
```

上例是预期语义，不要求照抄排版。验收必须证明四文件均可跟踪，同时 `.codex/hooks.json`、`.codex/agents/`、`.kimi-code/config*`、其它 Kimi skills 和所有其它 harness 目录仍被忽略。

### P2：Codex hook 注释已过时，执行路径仍有边界（C3 所有）

`.codex/config.toml:12-16` 声称必须在用户配置显式启用 `[features].hooks=true`。当前官方 hooks 文档和本机 `codex features list` 均表明 hooks 默认启用；项目仍必须 trusted，且每个项目 hook 的精确 hash 必须经 `/hooks` 审查。C3 只更新注释，不写用户全局配置、不代替用户信任或批准 hook。

`.codex/hooks.json` 的四条命令都从 cwd 使用 `.codex/...` 相对路径。官方建议 hook 自行解析 Git root，因为客户端可能从子目录启动；本轮没有复现失败，且路径 wrapper 被明确排除。文档应把“从仓库根启动”写成当前约束，并保留 hook 失效时显式 pull 的 fallback，不能声称子目录已支持。

### P2：除 Grok 元数据外，动态发现仍未验证（C3 记录）

Claude、Codex、Kimi、OMP 的版本命令、静态文件或 `doctor` 不足以证明新会话加载。C3 可先完成静态适配；只有五行真实 fresh-session 证据齐全后，才可宣称“五工具动态对齐”。任何收费、权限或客户端可用性阻断都应保留准确的 **UNVERIFIED**，不能用另一产品的执行结果代替。

## C3 最小实施路径

### 精确改动范围

1. `.trellis/workflow.md`：修正 breadcrumb 机制和 Kimi research/implement/check 分支，保留 phase tag、审批和 `Active task:` 协议，并导航到 `docs/agents/harnesses.md`。
2. `docs/agents/harnesses.md`：新增本报告矩阵的稳定项目版，记录版本日期、仓库根启动、bootstrap、fallback、任务分工和动态证据边界。
3. `.gitignore`：只放行四个覆盖文件所需的父目录；其它平台表面仍忽略。
4. `.codex/config.toml`：只修 hooks 默认/信任/hash review/根目录启动注释，不改行为配置。
5. `.kimi-code/skills/trellis-implement/SKILL.md`、`trellis-check/SKILL.md`、`trellis-research/SKILL.md`：承认 native project agents，说明本项目仍选 built-in `coder` + role skill/pull；保持职责、首行与递归禁止。

不改 `.codex/hooks.json`、`.kimi-code/agents/`、`.grok/`、`.omp/` 或 `.claude/`。`.trellis/.template-hashes.json` 只允许 Trellis 初始化器产生与新增生成资产一致的元数据变化，实施者不得手改或把四个本地覆盖的现值写成上游模板 hash。

### 新 checkout/bootstrap

实施验收在隔离的候选 checkout 进行，不对当前工作树执行广泛 init：

1. 确认 `trellis --version` 与 `.trellis/.version` 都是 `0.7.0-beta.3`；不一致即停止并记录工具链差异。
2. 记录四个 tracked override 的 SHA-256。
3. 运行 `trellis init --claude --codex --grok --kimi --omp --skip-existing -y`。本机 `trellis init --help` 已证明这五个开关、`--skip-existing` 与 `-y` 存在；本轮研究代理没有在临时 clone 执行写入式 init。
4. 再次计算四个 override hash，必须逐字节不变；检查五个平台的必要生成资产存在。
5. 对 `.trellis/.template-hashes.json` 做 init 前后语义比较：允许初始化器为新生成资产增加合法项；四个覆盖项不能被改写为本地修改后内容的 hash。升级 Trellis 时先运行 dry-run/差异审查，重新判断覆盖是否仍需保留。

### 静态验收

- `git check-ignore -v` 能解释 `.codex/hooks.json`、非白名单 Kimi skill/配置及其它 harness 资产仍由预期规则忽略；四个覆盖文件不再命中 ignore。
- `git ls-files` 和候选 diff 只包含批准的文档、workflow、gitignore 与四个覆盖，不含 settings、cache、credential 或本机状态。
- `python ./.trellis/scripts/get_context.py --mode phase` 与 `--mode packages` 能从候选 checkout 读取共享工作流和 spec 索引。
- C3 的 `implement.jsonl` / `check.jsonl` 已删除 `_example`，只引用本报告和实际相关 spec；`task.py validate` 通过。
- `npm run check:secrets`、docs build 与 `git diff --check` 通过；强模型复核五条路由、授权边界与递归禁止。

### 动态验收

每个 fresh session 只执行只读 smoke，输出一行结构化记录：客户端及版本、实际读取的 `AGENTS.md`/spec、任务仍为 `planning`、委派类型与 `Active task:`、hook/extension/pull 是否触发、实际工具权限、PASS/FAIL/UNVERIFIED 和原始缺口。建议入口：

- Claude Code：新会话报告合同和 planning task；若获准，再派一个只读 `trellis-research` child 验证上下文。
- Codex：先在 `/hooks` 核对 exact hash/trust，再用新会话及 `trellis-research` child 验证 `SubagentStart`；根目录启动约束必须保留。
- Grok Build：先保存 `grok inspect --json` 摘要，再以只读 `spawn_subagent(subagent_type="trellis-research", prompt="Active task: ...")` 验证 pull。
- Kimi Code：built-in `coder` 接收角色 skill 和 `Active task:`，回报读取的 JSONL/task/spec；`doctor` 不能替代此项。
- OMP：新会话回报 extension 注入，再派只读 task agent 验证项目 agent 与 `pi/task` 继承。

静态验收可以独立完成。动态记录缺任一行时，任务只能报告“静态适配完成，五工具动态对齐未完成”。

## 强模型与执行模型边界

本轮没有可靠、同口径的跨产品价格或模型能力基准，因此不写虚构排名。项目只规定职责：共享合同、需求/AC、根因、权限与最终审查由用户指定或当前可用的强模型承担；较便宜的执行模型只接收单一角色、独占文件范围、明确输入/输出、可运行的验收和升级条件。出现上下文缺失、任务边界变化、授权扩大、测试相互矛盾或需要降低 AC 时，执行者停止扩张并交回审查模型。

## Caveats / Not Found

- 除 `grok inspect --json` 外，本轮没有完成五客户端的真实新会话发现；Claude/Codex/Kimi/OMP 均保持 **UNVERIFIED**。
- 没有执行付费模型实验、用户全局设置写入、hook 信任/批准、plugin 安装或远端操作。
- 没有在隔离 clone 实际运行写入式 Trellis init；命令表面和版本已经只读核实，bootstrap 结果属于 C3 实施验收。
- `grok inspect --json` 证明配置发现，不证明模型遵循全部规则；它还报告一个用户配置 `privacy` unknown-field warning，与本项目 C3 改造无关。
- Kimi native project agent 能力已确认，但迁移到 `.kimi-code/agents/` 明确不在本任务范围。
- Codex 子目录 hook wrapper、Grok 新 hook、全局 Trellis 升级和 `.template-hashes.json` 手工同步均不在本任务范围。
