# 家宽规则优化 Skill

源文件（真源）：`skills/residential-rule-tuning/`。

项目级副本（gitignore，干净 clone 默认没有）：

- `.agents/skills/residential-rule-tuning/`
- `.claude/skills/residential-rule-tuning/`
- `.codex/skills/residential-rule-tuning/`
- `.cursor/skills/residential-rule-tuning/`
- `.omp/skills/residential-rule-tuning/`
- `.grok/skills/residential-rule-tuning/`
- `.kimi-code/skills/residential-rule-tuning/`

适用工具：Claude Code、Codex、Grok Build、Kimi Code、OMP，以及 `.agents` / `.cursor` 共享目录。

## 安装

Skill-only 路径：

```bash
just install-skills
just install-skills --create
just install-skills --platforms .agents,.claude
just install-skills --check
just install-skills --force
node scripts/install-agent-skills.js --create --platforms .agents,.claude,.codex,.cursor,.omp,.grok,.kimi-code
```

`just install-skills` 与 `node scripts/install-agent-skills.js` 接受同一组参数。

- 默认只写入已经存在的平台根目录，不创建缺失平台，不改其它 skill。
- `--create` 创建缺失的平台根目录。干净 checkout 必须显式创建目标根（`--create`，或先建目录再安装），否则不会写入任何副本。
- `--platforms` 限制目标根目录。
- 目标存在同名不同内容时默认 fail closed。覆盖前先 `--check`，确认差异仅为已审查旧副本后再 `--force`；`--force` 先写成 `<name>.bak-<UTC>` 再替换。额外用户文件保留。
- `--check` 只比较当前已存在的平台根与源文件。零个平台根时 `--check` 退出码为 0。这不证明五个工具已安装。

不要把 `just install-all` 当作 skill-only 路径。`just install-all`（Windows）有操作系统/全局副作用：静默安装桌面应用到 `%LOCALAPPDATA%\ResiWatch`、把 `monitor-db` 装进 cargo 用户 bin，并把 skill 写入当前项目的 `.agents/skills` 与 `.claude/skills`（目录不存在则创建）。

配套 CLI：`just monitor-db --help`。查询默认 JSON；贴出前加 `--redact`。
