# AC4 关闭对照

日期：2026-09-07。父任务仍在 `.trellis/tasks/09-07-evergreen-harness-audit/`。五个子任务已 `completed`，目录在 `.trellis/tasks/archive/2026-09/`。本轮只改父任务材料。不归档父任务。不提交。

## 结论

静态规则与本地检查已完成。hosted CI 为 UNVERIFIED。五客户端 live smoke 为 UNVERIFIED。不得宣称五工具运行时对齐。

逐项文件、检查记录、SHA/环境与残留见 [research/ac4-writeback.md](research/ac4-writeback.md)。

## 回写表

| Item | Write-back | Applicable tools | Evidence |
|---|---|---|---|
| C1 CI gates | `.github/workflows/ci.yml`，`tests/sync-monitor-version.test.js` | 五工具经 GitHub Actions | 本地 `just ci` / `just docs-build` / pwsh 负向探针。hosted UNVERIFIED |
| C2 project contract | `AGENTS.md`（共享），`CLAUDE.md`（Claude 单向 `@AGENTS.md`），README 本地验证，`.trellis/spec/residential-monitor/index.md`，quality-guidelines Validation Gate | 五工具共享 AGENTS | `get_context --mode packages`；secrets；docs build；`git diff --check` |
| C3 harness adapters | `.trellis/workflow.md`，`docs/agents/harnesses.md`，`.gitignore` 四个 override，`.codex/config.toml` 注释，三份 Kimi role skill | 按 `harnesses.md` 分平台 | 隔离 trellis init 哈希；`git check-ignore`；`task validate`；live smoke UNVERIFIED |
| C4 skill delivery | `skills/residential-rule-tuning/` 源库，`docs/agents/residential-rule-tuning.md`，installer 测试；ignored 七目录副本已在本机同步 | Claude/Codex/Grok/Kimi/OMP 以及 `.agents` / `.cursor` | `node --test` installer；`npm run ci`；`--check` 退出 0 |
| C5 branch governance | GitHub `main` 保护 `required_linear_history=true`；quality-guidelines Main Branch Protection live GET 2026-09-07 | 五工具共享该 spec | 独立 GET；仅该字段变化；下次 PR UNVERIFIED |

## 核对结果

- 上表五个回写落点均在仓库中存在且内容与子任务实施记录一致。`git ls-files` 跟踪除 ignored 七目录副本与 GitHub 远端保护以外的全部落点。
- Check 2026-09-07：只读复验回写落点、归档子任务、`git check-ignore -v`、四 override SHA 与 `--check`。结论不变。hosted CI 与五客户端 live smoke 仍 UNVERIFIED。
- C1：monitor 六条 pwsh step 独立；`docs` job 存在；`Required checks` 需要 `test`/`monitor`/`docs`；测试含 `SURVIVED_AFTER_EXIT_7` 与单命令 exit 7。
- C2：`AGENTS.md` 自包含；`CLAUDE.md` 单向 `@AGENTS.md`；README「本地验证」与 Validation Gate 命令表与 C1 结果一致。
- C3：四个 override 被跟踪；`.codex/hooks.json` 等仍被 ignore；`harnesses.md` 五行 live smoke 均为 UNVERIFIED。
- C4：源库与安装说明写明七目录与 `--check` 作用域；本机 `--check` 退出 0；副本不提交。
- C5：`protection-after.json` 与规范 Live verification 均为 `required_linear_history.enabled=true`；其它保护字段保持批准值。

## 残留限制

- hosted CI UNVERIFIED（无推送）。
- 五客户端 live smoke UNVERIFIED。
- 下次普通 PR 的 exact-head `Required checks` UNVERIFIED。
- 父任务 `task.json` 的 `children` / `relatedFiles` 仍写归档前路径。本轮不改 `task.json`。`design.md` 链接已改到 `../archive/2026-09/`。

## 规划阶段检查（历史）

- [x] 6个task.json关系正确，全部planning，父任务仍是当前planning指针。
- [x] 6套prd/design/implement完整；12个JSONL均为真实spec/research条目。
- [x] task.py validate逐项通过；相关本地链接存在，计划新文件明确标注。
- [x] 强模型独立审查已处理的建议与剩余UNVERIFIED记录清楚。
- [x] git diff --check和模板安全扫描通过；实际diff只在本次6个新任务目录。

详细结果见 [planning-validation.md](research/planning-validation.md)。

## 原执行计划（已执行）

1. 用户批准最新设计后，按实际授权范围激活具体子任务；父任务通常不作为产品实施目标。默认推荐C1–C4，C5单独远端批准。
2. C1先修CI退出传播；C4可在独占测试/业务skill文件范围并行。C2可先起草共享合同，按C1最终门禁收敛并独立验收。
3. C3在C2后完成平台条件路由、窄tracked overrides与隔离bootstrap；C4同步完成后检查业务skill发现。通过现有代理/CLI能力执行，不假设所有客户端都支持相同hooks。
4. 每子任务由强模型审查最终diff及其要求的失败/成功用例。低价执行者不能改AC、绕过门禁或处理远端治理。Trellis默认委派只用于获批的实施/检查；当前审查代理均为强模型。
5. 运行子任务implement.md中的最小检查；涉及共同质量门时运行root、monitor与docs组合。需要干净安装、Node矩阵或GitHub runner时保留对应证据层，不能用单一本机成功替代。
6. 只有获得PR/推送授权后获取对应SHA的hosted run。未授权或没有证据时记录UNVERIFIED，保持缺失的AC，不报告全交付完成。
7. 按C2/C3/C4回写项目说明/skills并标注适用工具。跨任务链接和JSONL在路径变更/归档前后检查；有效源文件不能因归档而成为死链接。
8. C5继续planning直到精确main保护变更获批；不修改其他保护字段或历史。
9. 父任务整体验收：每个批准项有实际文件、检查记录、SHA/环境、适用工具和残留限制；五工具动态smoke未全部完成时只报告静态规则对齐，不声称五工具运行一致。

## 回退

每子任务仅回退其负责文件；保留他人修改与本地skill备份。运行证据不能通过删除失败记录或改写历史“修绿”。用户未批准的产品、全局、安装和远端状态不在回退/修改范围内。
