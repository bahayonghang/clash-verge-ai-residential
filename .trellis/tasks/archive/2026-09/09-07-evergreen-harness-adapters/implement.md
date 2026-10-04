# Implementation

1. 批准后用trellis-meta核对现有四文件、workflow、gitignore及官方版本证据；保护任何新增用户修改。
2. 修shared workflow的按平台语义；修Kimi三skill与Codex配置注释，不修改hook代码。
3. 写harnesses.md，标明五工具能力/配置/实证与问题分工，由本项负责的workflow提供导航。
4. 加极窄.gitignore例外，`git check-ignore -v`确认非白名单仍被忽略，`git ls-files`/候选diff确认没有私有settings或cache进入交付。
5. 在隔离候选checkout运行已核实init --skip-existing；比对四文件hash前后相同、五平台必要资产存在，执行Python get_context的phase/packages只读加载。保存.template-hashes.json前后快照并审查差异：不得把四个本地override的新内容伪记为upstream基线；允许生成器为新生成资产增加正常元数据，不要求整个manifest字节不变。若现有init不保留本地覆盖的来源语义，标bootstrap失败并修正方案，禁止手改hash“修绿”；不在当前工作树试错。
6. 执行 `python .trellis/scripts/task.py validate <本任务>`、`npm run check:secrets`、docs build、`git diff --check`；强模型审查5条路由及无自动审批/递归。
7. 按可用且获授权的客户端做fresh-session smoke，仅要求读取项目合同/任务、报告能力边界，不执行产品写入。命令/客户端不可用、权限阻断或需额外付费则记录UNVERIFIED与准确缺口，不通过其他产品代跑冒充。
8. 保存 `research/harness-smoke.md` 的五行结果；项目说明注明适用工具与日期。静态验收和动态验收分别报告。

建议强模型：Claude Code/Codex做共享合同与跨平台审查；Grok Build可作独立文档/路由反证审查；OMP适合显式多provider执行分工；Kimi适合边界已定的单子任务。低价模型可整理固定文案和重复测试fixture；子代理上下文/权限策略仍由强模型决定。

## C3 实施记录（2026-09-07）

- 隔离 `trellis init --skip-existing`（0.7.0-beta.3）在 scratch 候选目录执行，四 override 在 init 前后 SHA-256 相同。
- `git add --dry-run` 可加入四 override，不能加入 `.codex/hooks.json`。
- `task.py validate` 退出 0。五客户端 live smoke 为 UNVERIFIED。静态完成，动态未完成。
