# 五套 Harness 的启动与委派

本页记录 Claude Code、Codex、Grok Build、Kimi Code、OMP 在本仓库的入口、委派、权限与 fallback。共享阶段规则在仓库文件 `.trellis/workflow.md`。

三层分开读：

- **原生能力**：官方产品支持的规则、skill、子代理、hook 或 extension。
- **项目配置**：本仓库当前文件与本项目的路由选择。
- **运行证据**：本机客户端在新会话中实际发现并执行。静态文件存在不能代替新会话握手。

适用版本（审查日 2026-09-07，见 `.trellis/tasks/09-07-evergreen-harness-audit/research/harness-audit.md`）：Claude Code `2.1.263`，Codex `0.153.4`，Grok Build `1.0.22`，Kimi Code `0.41.0`，OMP `18.1.12`，Trellis `0.7.0-beta.3`。

从**仓库根**启动客户端。Codex 的 `.codex/hooks.json` 命令使用仓库相对路径；从子目录启动时 hook 可能无法解析。

## 能力、配置与委派

| Harness | 原生能力 | 本项目配置 | 入口 | 委派 | 权限 / 约束 | Fallback |
|---|---|---|---|---|---|---|
| Claude Code | `CLAUDE.md` 每会话加载；项目 `.claude/skills/`、`.claude/agents/`、lifecycle hooks | 根 `CLAUDE.md` 引用共享 `AGENTS.md`。`.claude/settings.json` 配置 SessionStart、UserPromptSubmit、任务委派与读写后上下文 hook。`.claude/agents/trellis-*` 为原生角色 | 项目 hook 注入 workflow-state | 主会话派发项目 `trellis-*` 子代理。Dispatch 首行 `Active task: <path>` | 项目 hook 按客户端策略执行。本页不代替用户批准 | hook 未触发时显式读 `task.py current`、JSONL、`prd.md` / `design.md` / `implement.md` 与 spec |
| Codex | 发现并拼接选中的 `AGENTS.md`；项目 `.agents/skills/`、`.codex/agents/`、`.codex/hooks.json` | `.codex/config.toml` 设 `agents.max_depth = 1`。`.codex/hooks.json` 配置 SessionStart、UserPromptSubmit、SubagentStart、PreToolUse。项目 config 不写用户级 feature flag，不自动批准 hook | 项目 hook 注入。仓库必须 trusted；每个 hook 须经 `/hooks` 精确 hash 审查 | 主会话派发项目 `trellis-*`。`SubagentStart` 可注入上下文；子代理侧仍保留 pull | 先信任项目，再审查 hook hash。从仓库根启动 | hook 未生效时显式读 task / JSONL / spec。不为未复现的子目录启动问题改 wrapper |
| Grok Build | 项目 `.grok/skills`、`.grok/agents`、`.grok/hooks`；`spawn_subagent` 可按自定义 agent 类型委派 | 本项目不用自动 SessionStart。`.grok/commands/trellis-start.md` 要求显式调用 `get_context.py`。`.grok/agents/trellis-*` 通过 `Active task:` 首行与 JSONL pull 取上下文 | 显式 pull | `spawn_subagent`，`subagent_type` 为 Trellis 角色名（如 `trellis-implement`）。Dispatch 首行 `Active task: <path>` | 按项目 agent 定义执行。Claude-compatible hooks 在本项目为 disabled | 继续 `get_context.py`、`task.py current --source`、JSONL 与任务产物 |
| Kimi Code | 官方同时提供 built-in `coder` / `explore` / `plan` 与项目自定义 agent（`.kimi-code/agents/` 或 `.agents/agents/`）；也支持 skill 与 hook | 平台支持项目自定义代理。本项目仍选择 built-in `coder` + `.kimi-code/skills/trellis-{implement,check,research}/SKILL.md` pull，不创建 `.kimi-code/agents/` | 显式 pull | 主会话把 built-in `coder`（研究角色同样用 `coder`，因为 `explore` 只读）与对应 role skill 一起派发。Dispatch 首行 `Active task: <path>`。禁止递归 implement/check，也不派发项目 `trellis-*` agent 类型 | 子代理自行读 JSONL / task / spec。能力不足或边界变化时交回规划/审查模型 | `task.py current --source`，再读 jsonl、`prd.md`、`design.md`、`implement.md`。`kimi doctor` 不能证明项目 skill 已加载 |
| OMP | `AGENTS.md` context provider；项目 `.omp/agents` 可定义 task agent；`.omp/skills` 提供流程；extension API 提供生命周期事件 | `.omp/extensions/trellis/index.ts` 从当前目录向上找 `.trellis` 根并注入上下文。`.omp/agents/trellis-*` 是项目 task agent。`model: pi/task` 是会话模型继承选择器 | 项目 Trellis extension 注入 | 主会话派发项目 `trellis-*` task agent。Dispatch 首行 `Active task: <path>` | 按 extension 与 task agent 定义执行 | extension 未注入时显式读 task / JSONL / spec |

## 按问题分工

共享合同、需求与验收标准、根因、权限策略、最终审查由用户指定或当前可用的强模型承担。

| 问题类型 | 优先工具 |
|---|---|
| 共享合同、复杂规划、根因、最终审查 | Claude Code 或 Codex |
| 独立文档或路由反证审查 | Grok Build |
| 显式多 provider 的独占执行 | OMP |
| 目标、文件所有权和验收标准已封闭的单一实现、资料持久化或检查 | Kimi Code（built-in `coder` + role skill） |

较便宜的执行模型只接收单一角色、独占文件范围、明确输入/输出、可运行的验收和升级条件。出现上下文缺失、任务边界变化、授权扩大、测试相互矛盾或需要降低验收标准时，执行者停止扩张并交回审查模型。

## Bootstrap

在**缺少被忽略的 harness 资产**的 checkout 上，使用已安装的 Trellis `0.7.0-beta.3`：

```bash
trellis init --claude --codex --grok --kimi --omp --skip-existing -y
```

`--skip-existing` 保留已经存在的文件。下列四个覆盖必须随仓库进入新 checkout，init 不得改写其字节：

- `.codex/config.toml`
- `.kimi-code/skills/trellis-implement/SKILL.md`
- `.kimi-code/skills/trellis-check/SKILL.md`
- `.kimi-code/skills/trellis-research/SKILL.md`

不要手改 `.trellis/.template-hashes.json`，把本地覆盖伪装成上游原稿。`trellis init --skip-existing` 会跳过已存在的四个覆盖并保留其字节。同一次 init 可能按本次平台集合重写该 checkout 的 `.trellis/.template-hashes.json`，并把被跳过的覆盖从 manifest 里删除。不要把候选 checkout 的 hash 文件拷回本仓库。本仓库继续保留现有上游 hash，以便后续 `trellis update` 能标出本地差异。升级 Trellis 时先审查这四个覆盖是否仍需保留。不要在当前工作树做广泛 `trellis init`。

其余 `.claude/`、`.codex/`（除 `config.toml`）、`.grok/`、`.kimi-code/`（除上述三份 skill）、`.omp/`、`.agents/` 仍由 `.gitignore` 忽略，依赖上述 init 在新 checkout 生成。

## 手动 pull fallback

任一 hook、extension 或自动注入未触发时，从仓库根执行：

```bash
python ./.trellis/scripts/task.py current --source
python ./.trellis/scripts/get_context.py
python ./.trellis/scripts/get_context.py --mode packages
python ./.trellis/scripts/get_context.py --mode phase --step <step>
```

然后读当前任务的 `implement.jsonl` 或 `check.jsonl`、`prd.md`、`design.md`（若存在）、`implement.md`（若存在）。Dispatch 提示的第一行仍必须是 `Active task: <path>`。

## 运行证据

下表记录 fresh-session smoke。未实际启动对应客户端的新会话时，状态为 **UNVERIFIED**。静态适配完成不等于五工具动态对齐完成。

| Harness | 适用版本 | 已读 AGENTS/spec | planning 状态 | 委派前缀 | hooks / extension / pull | 权限 | 结果 |
|---|---|---|---|---|---|---|---|
| Claude Code | `2.1.263` | 未跑 live smoke | 未跑 | 未跑 | 未跑 UserPromptSubmit | 未跑 | **UNVERIFIED** |
| Codex | `0.153.4` | 未跑 live smoke | 未跑 | 未跑 | 未跑 `/hooks` 与 SubagentStart | 未跑 | **UNVERIFIED** |
| Grok Build | `1.0.22` | 未跑 live fresh-session smoke | 未跑 | 未跑 | 未跑本轮只读 `spawn_subagent` | 未跑 | **UNVERIFIED** |
| Kimi Code | `0.41.0` | 未跑 live smoke | 未跑 | 未跑 | `doctor` 不能代替 | 未跑 | **UNVERIFIED** |
| OMP | `18.1.12` | 未跑 live smoke | 未跑 | 未跑 | 未跑 extension 与 task-agent child | 未跑 | **UNVERIFIED** |

审计阶段的 `grok inspect --json` 只证明配置发现元数据，不构成本页的 fresh-session PASS。

当前结论：静态适配可单独验收。五工具动态对齐未完成。
