# 家宽规则优化 Skill

源文件：`skills/residential-rule-tuning/`。

安装：

```bash
just install-all
just install-cli
just install-skills
just install-skills --check
just install-skills --force
```

`just install-all`（Windows）会：静默安装桌面应用到 `%LOCALAPPDATA%\ResiWatch`、把 `monitor-db` 装进 cargo 用户 bin，并把 skill 写入当前项目的 `.agents/skills` 与 `.claude/skills`（目录不存在则创建）。

`just install-skills` 默认只写入已经存在的平台目录，不创建缺失平台，不改其它 skill。目标存在同名不同内容时默认 fail closed；`--force` 先写成 `<name>.bak-<UTC>` 再替换。

配套 CLI：`just monitor-db --help`。查询默认 JSON；贴出前加 `--redact`。
