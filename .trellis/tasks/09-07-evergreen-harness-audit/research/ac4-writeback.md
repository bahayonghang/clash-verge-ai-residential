# Research: AC4 回写对照

- Query: 每个已批准子任务的回写落点、适用工具、实际文件、检查记录与残留限制是否一致
- Scope: 父任务关闭对照；产品文件只读核对；不改产品、不归档父任务、不提交
- Date: 2026-09-07
- Planning baseline: `dev` / `5578576787dba73ba1b96985a2fc408fc246bfed`
- Closeout HEAD: `0500a14da473564e30eabcec196a6744e8750038`（`chore(task): archive 09-07-evergreen-branch-governance`）

## 结论

五个子任务均为 `completed`，目录在 `.trellis/tasks/archive/2026-09/`。回写落点与适用工具与实际仓库文件一致。

静态规则与本地检查已完成。hosted CI 为 **UNVERIFIED**（无推送）。五客户端 live smoke 为 **UNVERIFIED**。不得宣称五工具运行时对齐。

## 子任务状态

| 项 | 归档目录 | task.json status | completedAt |
|---|---|---|---|
| C1 CI gates | `.trellis/tasks/archive/2026-09/09-07-evergreen-ci-gates/` | completed | 2026-09-07 |
| C2 project contract | `.trellis/tasks/archive/2026-09/09-07-evergreen-project-contract/` | completed | 2026-09-07 |
| C3 harness adapters | `.trellis/tasks/archive/2026-09/09-07-evergreen-harness-adapters/` | completed | 2026-09-07 |
| C4 skill delivery | `.trellis/tasks/archive/2026-09/09-07-evergreen-skill-delivery/` | completed | 2026-09-07 |
| C5 branch governance | `.trellis/tasks/archive/2026-09/09-07-evergreen-branch-governance/` | completed | 2026-09-07 |

## 逐项对照

### C1 CI gates

| 字段 | 内容 |
|---|---|
| 回写落点 | `.github/workflows/ci.yml`；`tests/sync-monitor-version.test.js` |
| 适用工具 | 五工具经 GitHub Actions 共用同一 workflow。不按客户端分叉 CI。 |
| 实际文件 | `ci.yml` 将 monitor 六条原生命令拆为独立 `shell: pwsh` step（install、frontend check、fmt、clippy、workspace tests、secret scan）。新增 Ubuntu / Node 22 `docs` job（`npm --prefix docs ci` 后 `npm --prefix docs run build`）。`required-checks` 的 `needs` 与 `needs.*.result == success` 含 `test` / `monitor` / `docs`。`git ls-files` 跟踪上述两文件。 |
| 检查记录 | 子任务：`node --test tests/sync-monitor-version.test.js` 11/11（两次）；pwsh 负向探针（多命令 exit 7→0 且输出 `SURVIVED_AFTER_EXIT_7`；单命令 exit 7）；`just ci` 退出 0；`just docs-build` 退出 0。父任务发布门：`just ci` 退出 0，`just docs-build` 退出 0。 |
| 残留 | 无推送。hosted 矩阵与 aggregate 为 **UNVERIFIED**。 |

### C2 project contract

| 字段 | 内容 |
|---|---|
| 回写落点 | `AGENTS.md`（五工具共享合同）；`CLAUDE.md`（Claude 单向 `@AGENTS.md`）；README「本地验证」；`.trellis/spec/residential-monitor/index.md`；`.trellis/spec/frontend/quality-guidelines.md` 的 Validation Gate |
| 适用工具 | 五工具共享 `AGENTS.md`。Claude 另读 `CLAUDE.md`。`AGENTS.md` 不导入 `CLAUDE.md`。 |
| 实际文件 | `AGENTS.md` 在 Trellis managed block 外自包含产品地图、按路径工具链、验证组合、边界与 spec 导航。`CLAUDE.md` 含 `@AGENTS.md`，无反向导入。README「本地验证」写明 `just ci` 范围、docs 独立、`Required checks` 需要 `test`/`monitor`/`docs`。monitor 包索引指向 backend/frontend/storage。Validation Gate 记录六条独立 pwsh step、docs job、aggregate `needs`。Main Branch Protection 段由 C5 回写，C2 未改该段合同值。 |
| 检查记录 | 子任务：`python .trellis/scripts/get_context.py --mode packages` 退出 0；`npm run check:secrets` 退出 0；`npm --prefix docs run build` 退出 0；`git diff --check` 退出 0。 |
| 残留 | 五工具动态加载归 C3，为 **UNVERIFIED**。 |

### C3 harness adapters

| 字段 | 内容 |
|---|---|
| 回写落点 | `.trellis/workflow.md`；`docs/agents/harnesses.md`；`.gitignore` 四个 override；`.codex/config.toml` 注释；`.kimi-code/skills/trellis-{implement,check,research}/SKILL.md` |
| 适用工具 | 按 `docs/agents/harnesses.md`：Claude/Codex 为项目 hook；OMP 为 Trellis extension；Grok/Kimi 为本项目显式 pull。Kimi 保持 built-in `coder` + role skill，不新增 `.kimi-code/agents/`。 |
| 实际文件 | `workflow.md` 指向 `docs/agents/harnesses.md`，并按 hook / extension / pull 分平台。`harnesses.md` 运行证据五行均为 UNVERIFIED。`.gitignore` 只放行 `.codex/config.toml` 与三份 Kimi role skill。Codex 注释写 hooks 默认开启、需信任与 `/hooks` hash 审查、从仓库根启动。三份 Kimi skill 写明平台支持项目代理，本项目仍选 built-in `coder` + pull。`git ls-files` 跟踪上述四 override。 |
| 检查记录 | 子任务：隔离 `trellis init --claude --codex --grok --kimi --omp --skip-existing -y`（Trellis `0.7.0-beta.3`）；四 override init 前后 SHA-256 相同；`git add --dry-run` 可加入四 override，不能加入 `.codex/hooks.json`；`task.py validate` 退出 0。AC4 关闭时 `git check-ignore -v`：`.codex/config.toml` 与三份 Kimi skill 无 ignore 规则；`.codex/hooks.json` 命中 `.gitignore:38:.codex/*`；`.kimi-code/config.json` 命中 `.kimi-code/*`；`.claude/settings.json` 命中 `.claude/`；`.agents/skills/residential-rule-tuning/SKILL.md` 命中 `.agents/`。 |
| 残留 | 五客户端 live smoke 为 **UNVERIFIED**。静态适配完成。运行时对齐未完成。 |

### C4 skill delivery

| 字段 | 内容 |
|---|---|
| 回写落点 | `skills/residential-rule-tuning/` 源库；`docs/agents/residential-rule-tuning.md`；`tests/install-agent-skills.test.js`；本机 ignored 七目录副本（不提交） |
| 适用工具 | Claude Code、Codex、Grok Build、Kimi Code、OMP，以及 `.agents` / `.cursor` 共享目录 |
| 实际文件 | 源 `SKILL.md` / `reference.md` / `scripts/build-inputs.js` 跟踪在仓库。安装说明写明 `--create` / `--platforms` / `--check` / `--force`，并写明零平台根时 `--check` 退出 0 不能证明五平台已安装。七目录副本被 gitignore。 |
| 检查记录 | 子任务：`node --test tests/install-agent-skills.test.js` 两次 14/14；临时仓库真实三文件 payload 覆盖七平台根，二次安装 `written=0`；本机 `node scripts/install-agent-skills.js --force --platforms .agents,.claude,.codex,.cursor,.omp,.grok,.kimi-code` 后 `--check` 退出 0；`npm run ci` 退出 0。生成器合同仍为 24/12/12，三 core，`grok_web_assets` → `GROK_STRICT_EXACT_DOMAINS`。 |
| 残留 | ignored 副本不随 clone 交付。`--check` 只比较已存在的平台根。客户端加载 skill 为 **UNVERIFIED**。 |

### C5 branch governance

| 字段 | 内容 |
|---|---|
| 回写落点 | GitHub `bahayonghang/clash-verge-ai-residential` 的 `main` 保护 `required_linear_history=true`；`.trellis/spec/frontend/quality-guidelines.md` Main Branch Protection「Live verification」 |
| 适用工具 | 五工具共享该 spec 段。远端设置以 GitHub GET 为准，本地文件不是远端设置。 |
| 实际文件 | 规范 Live verification 日期 2026-09-07：独立 GET 记录 `required_linear_history.enabled=true`；strict / app-bound `Required checks`（`app_id` 15368）、PR 计数 0、管理员、对话解决、禁止 force/delete 保持批准值。子任务快照：`research/protection-before.json`、`protection-after.json`、`protection-check.json`、`validation.md`。 |
| 检查记录 | 新鲜 GET：`required_linear_history.enabled=false`。PUT 仅将该项设为 true。独立 GET 与 after 快照一致，其它保护字段未变。rulesets 为空。squash/rebase 仍可用。无 `git push`、无强推、无改写历史。检查代理独立 GET 与 after 快照一致。GitHub `main` SHA 当时为 `caa580650d6d1bcb5b338f5e1c9ccd4afff3e7ea`。 |
| 残留 | 下次普通 PR 的 exact-head `Required checks` 为 **UNVERIFIED**。未开测试 PR。 |

## 计划与实际的差异

- C3 实施记录写的是 `git add --dry-run`。AC4 关闭时补跑 `git check-ignore -v`，结果与 allowlist 一致。
- C2 把 Validation Gate 写入 quality spec；C5 把 Live GET 写入同一文件的 Main Branch Protection 段。两段串行，符合原子任务文件所有权。
- C4 `docs/en/agents/residential-rule-tuning.md` 不在子任务文件清单，未纳入该任务提交。中文 `docs/agents/residential-rule-tuning.md` 为回写落点。
- 父任务 `task.json` 的 `relatedFiles` 与 `children` 仍写归档前路径。本轮不允许改 `task.json`。归档后的有效目录为 `.trellis/tasks/archive/2026-09/09-07-evergreen-*`。`design.md` 的子任务链接已改到归档路径。

## 未验证（不得升级为 PASS）

- 对应提交的 hosted GitHub Actions（Ubuntu Node 18/20/22、Windows Node 22、Windows monitor、Ubuntu docs、`Required checks`）
- Claude Code、Codex、Grok Build、Kimi Code、OMP 的五客户端 live / fresh-session smoke
- 下一张获授权普通 PR 的 exact-head `Required checks`
- Clash/Mihomo 宿主、NSIS 真机、Credential Manager 真写入、30 天库、24 小时 soak

## 非声明

静态规则对齐已完成。本地 `just ci` 与 `just docs-build` 退出 0。不得据此声称 hosted CI 通过，也不得声称五工具运行时对齐。

## Check 核对（2026-09-07）

Check agent 只读复验父任务材料、归档子任务与工作树文件。未改产品文件。未归档父任务。未提交。

- 父任务仍在 `.trellis/tasks/09-07-evergreen-harness-audit/`。`task.json.status=in_progress`。`task.py list-archive` 无本父任务。
- `prd.md` AC4 已勾选。结论仍为 hosted CI UNVERIFIED、五客户端 live smoke UNVERIFIED、不得宣称五工具运行时对齐。
- C1–C5 归档目录存在，`task.json.status=completed`，`completedAt=2026-09-07`。`design.md` 链接指向 `../archive/2026-09/`。
- `git ls-files` 跟踪应跟踪的回写落点。`git check-ignore -v`：四 override 无 ignore 规则（exit 1）。`.codex/hooks.json` 命中 `.gitignore:38:.codex/*`。`.kimi-code/config.json` 命中 `.kimi-code/*`。`.claude/settings.json` 命中 `.claude/`。`.agents/skills/residential-rule-tuning/SKILL.md` 命中 `.agents/`。
- 四 override SHA-256 与 C3 归档记录的当前工作树值一致。research skill 为 check 后的 `646f070f6ed7598bccc91a0275ed1f3fa78783c043db5ff16caab587b0cbbb61`。
- `node scripts/install-agent-skills.js --check` 退出 0。`python .trellis/scripts/get_context.py --mode packages` 退出 0。
- 工作树除本父任务目录外干净。HEAD `0500a14da473564e30eabcec196a6744e8750038`。
- 本轮未重跑 `just ci` / `just docs-build`（产品文件相对该 HEAD 无 diff）。未重跑 hosted CI。未做五客户端 live smoke。
