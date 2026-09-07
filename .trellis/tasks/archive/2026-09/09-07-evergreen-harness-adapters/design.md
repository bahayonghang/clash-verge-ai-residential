# Design

## Files
- `.trellis/workflow.md`：纠正注释与walkthrough中的平台分支，保留phase tags/审批语义及当前任务上下文协议。
- `docs/agents/harnesses.md`（新建）：五工具边界、按问题分工、项目根启动、bootstrap、手动pull fallback、运行证据表和适用版本。
- `.gitignore`：不再用整目录排除吞掉例外；按目录层级建立极窄例外，只纳入下面4文件，其他settings/hooks/runtime继续忽略。
- `.codex/config.toml`：只更新已核实的hooks默认/信任/表面差异注释；不改用户全局feature、不自动批准hook。
- `.kimi-code/skills/trellis-{implement,check,research}/SKILL.md`：事实更新为“平台支持项目代理，但本项目选择built-in coder + role skill/pull”；保留职责与禁止递归。
- C2 的AGENTS合同已导航到workflow；本项在workflow中链接harnesses.md，不反向要求C2再改AGENTS，避免形成任务依赖环。

## Bootstrap
使用已安装且已核实的 Trellis0.7.0-beta.3：
`trellis init --claude --codex --grok --kimi --omp --skip-existing -y`。
实际验证必须在隔离的候选工作树副本中进行，不对当前项目执行广泛init。保留既有tracked overrides；不手改.template-hashes.json以假装它们是upstream原稿。更新版本时需先审查本地覆盖是否仍需保留。

这不是保证生成器的所有说明都最新；四个override和shared workflow修复已知矛盾，其他能力仅按本项目配置描述。Grok/Kimi当前pull是项目选择，不等于平台没有hooks/custom agents。

## Runtime boundary
Codex从项目根启动，遇到hook未生效显式读task/JSONL/spec；不为未复现的子目录启动问题重构wrapper。Claude采用原hook与原生子代理；Grok沿用.grok/agents；OMP沿用.omp extension；Kimi沿用coder。每次能力不足/任务边界变动交回强模型，低价执行模型不得自行降低AC。
