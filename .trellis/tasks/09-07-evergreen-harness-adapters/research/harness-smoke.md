# Research: harness smoke

- Query: 五工具 fresh-session smoke 与隔离 bootstrap 记录
- Scope: mixed（静态适配结果、隔离 init、未跑 live client）
- Date: 2026-09-07

## 验收分层

静态适配完成。五工具动态对齐未完成。未用其他产品代跑冒充 PASS。

## Fresh-session smoke

本轮没有启动五个客户端的只读新会话。下表全部 **UNVERIFIED**。

| Harness | 适用版本 | 已读 AGENTS/spec | 任务 planning 状态 | 委派与 Active task 前缀 | hooks / extension / pull | 权限 | 结果 | 缺口 |
|---|---|---|---|---|---|---|---|---|
| Claude Code | `2.1.263` | 未跑 | 未跑 | 未跑 | 未跑 UserPromptSubmit | 未跑 | **UNVERIFIED** | 未开 live 新会话 |
| Codex | `0.153.4` | 未跑 | 未跑 | 未跑 | 未跑 `/hooks` 与 SubagentStart | 未跑 | **UNVERIFIED** | 未开 live 新会话；未做 hook hash 审查 |
| Grok Build | `1.0.22` | 未跑本轮 fresh-session | 未跑 | 未跑只读 `spawn_subagent` | 未跑 | 未跑 | **UNVERIFIED** | 本轮是实施会话，不记为 smoke PASS。审计阶段 `grok inspect --json` 只证明发现元数据 |
| Kimi Code | `0.41.0` | 未跑 | 未跑 | 未跑 built-in `coder` + role skill | `doctor` 未代替 | 未跑 | **UNVERIFIED** | 未开 live 新会话 |
| OMP | `18.1.12` | 未跑 | 未跑 | 未跑 task-agent child | 未跑 extension | 未跑 | **UNVERIFIED** | 未开 live 新会话 |

适用工具与日期：上表版本来自 `.trellis/tasks/09-07-evergreen-harness-audit/research/harness-audit.md`，审查日 2026-09-07。Trellis CLI 与 `.trellis/.version` 均为 `0.7.0-beta.3`。

## 隔离 bootstrap

候选目录：`C:\Users\lyh\AppData\Local\Temp\grok-goal-9d32661c612a\implementer\c3-init-candidate`（scratch，不在仓库内）。

完整命令输出：`C:\Users\lyh\AppData\Local\Temp\grok-goal-9d32661c612a\implementer\c3-init.log`。

命令：`trellis init --claude --codex --grok --kimi --omp --skip-existing -y`。未在当前工作树执行广泛 init。未手改 `.trellis/.template-hashes.json`。

Init 日志明确跳过四个覆盖：

- `Skipped: .codex\config.toml (already exists)`
- `Skipped: .kimi-code\skills\trellis-check\SKILL.md (already exists)`
- `Skipped: .kimi-code\skills\trellis-implement\SKILL.md (already exists)`
- `Skipped: .kimi-code\skills\trellis-research\SKILL.md (already exists)`

### 四文件 SHA-256（init 前后相同）

| 文件 | SHA-256 |
|---|---|
| `.codex/config.toml` | `76f59e49be4e6cc240aee0716c7dec8f90c9c7f94bda57c8d047c4e780e9b32a` |
| `.kimi-code/skills/trellis-implement/SKILL.md` | `61013b2aaef14b1da347553da645f5343f73ae17c8ec9a2cb0ad3fdc44cc444c` |
| `.kimi-code/skills/trellis-check/SKILL.md` | `afb3f28ebae69d0f42998e36777111ebb3b77a98c728a72eb87a1ba51711bbe9` |
| `.kimi-code/skills/trellis-research/SKILL.md` | `bc8926d4db30746d28e2847c8592a0b46f5550f17216a6b6d234af2ef157f0c2` |

Init 当时候选文件字节与工作树相同。Check 之后只改了工作树里的 research skill（拒绝代码编辑时交回主会话 `coder` + implement skill，不再建议 spawn 项目 `trellis-implement`）。该文件当前工作树 SHA-256 为 `646f070f6ed7598bccc91a0275ed1f3fa78783c043db5ff16caab587b0cbbb61`。未再对当前工作树或候选目录执行 init。其余三文件仍与上表及候选字节相同。

### 五平台生成资产

| 平台 | 标记文件 | 存在 |
|---|---|---|
| Claude Code | `.claude/agents/trellis-implement.md` | 是 |
| Codex | `.codex/hooks.json` | 是 |
| Grok Build | `.grok/agents/trellis-implement.md` | 是 |
| Kimi Code | `.kimi-code/skills/trellis-start/SKILL.md` | 是 |
| OMP | `.omp/agents/trellis-implement.md` | 是 |

候选内 `get_context.py --mode packages` 与 `--mode phase` 退出码 0。

### template-hashes.json

当前工作树 manifest **未改**（323 keys）。四个覆盖仍指向上游模板 hash，不等于上表本地文件 hash：

| 文件 | 工作树 template hash（上游） |
|---|---|
| `.codex/config.toml` | `9f2d20e28f0bc9c886312eca3ad3bba41533ef4615aaaafe25e98152302267bb` |
| `.kimi-code/skills/trellis-implement/SKILL.md` | `fb1fa4c36fea58437d20f8a43f0028d1976333bc5bbb4487de1e9c32c48e6e8d` |
| `.kimi-code/skills/trellis-check/SKILL.md` | `882c7a2e5fe966de7511c30bab4d5409e7c9dcf2872ea48d1a8f14e6bc6f2bdb` |
| `.kimi-code/skills/trellis-research/SKILL.md` | `b5c0e7a7282d0b133e3700d2815f893e38adc89c817c61e49f7025c66b58a995` |

候选 checkout 的 manifest 被 init 按本次 `--claude --codex --grok --kimi --omp` 重写为 243 keys，四个被 skip 的覆盖从候选 manifest 中消失。未把该文件拷回仓库。后续 `trellis update` 仍应能靠工作树里的上游 hash 暴露本地差异。

Init 还打印了过时的 Codex 提示（要求用户级 `features.hooks = true`）。项目 `.codex/config.toml` 注释已按当前默认-on / 信任 / `/hooks` hash 审查更正。生成器帮助文案不在本任务修改范围。

## 结论

- 静态：workflow 分平台说明、gitignore 四文件 allowlist、Codex 注释、Kimi 三 skill、`docs/agents/harnesses.md`、隔离 init 补齐五平台且四文件字节不变。
- 动态：五行 live smoke 均为 **UNVERIFIED**。
