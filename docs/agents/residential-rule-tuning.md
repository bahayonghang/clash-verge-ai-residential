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
- `--check` 只比较当前已存在的平台根与源文件。零个平台根时 `--check` 退出码为 0。这不证明 skill 已交付或被客户端发现。

不要把 `just install-all` 当作 skill-only 路径。`just install-all`（Windows）有操作系统/全局副作用：静默安装桌面应用到 `%LOCALAPPDATA%\ResiWatch`、把 `monitor-db` 装进 cargo 用户 bin，并把 skill 写入当前项目的 `.agents/skills` 与 `.claude/skills`（目录不存在则创建）。

配套 CLI：`just monitor-db --help`。查询默认 JSON；贴出前加 `--redact`。

## 源更新后的交付检查

只修改 `skills/residential-rule-tuning/` 单一源；先完成共享合同等前置变更，再同步副本。以下流程适用于 Claude Code、Codex、Grok Build、Kimi Code、OMP，以及 `.agents` / `.cursor` 投递。交付范围须来自用户授权。

1. 运行 `node --test tests/install-agent-skills.test.js`。测试在临时目录写入真实源文件，检查七目标字节一致、额外文件保留、幂等，以及七份生成器的 26 个 routing 键、13 supported、13 unsupported。`extra_anyrouter` 必须映射 `anyrouter.top`，`extra` 保持 unsupported。
2. 记录实际存在的目标目录和数量；逐文件比较 `SKILL.md`、`reference.md`、`scripts/build-inputs.js`，记录源与副本哈希以及额外用户文件。先运行 `--check`，保留非零退出的差异记录。缺失目标须报告，不能用零目标的成功退出补足数量。
3. 审查差异。仅在差异属于已审查旧副本且交付已获批时，对限定目标使用 `--force`。安装器在覆盖前生成 `<name>.bak-<UTC>`；确认备份与覆盖前哈希相同，额外文件保持不变。出现未知改动时，停止覆盖并提交审查。
4. 重跑相同目标的 `--check`，确认退出 0；再运行一次相同目标的默认安装，确认写入数为 0。记录实际目标数量、文件一致结果和每条命令的退出码。源文件再次变化后重新执行交付检查。

当前七目标都已存在且全部获准交付时，可使用以下限定范围的命令；不要附加 `--create`，也不要以 `just install-all` 替代：

```bash
node scripts/install-agent-skills.js --check --platforms .agents,.claude,.codex,.cursor,.omp,.grok,.kimi-code
# 完成差异审查且交付获批后才执行下一条。
node scripts/install-agent-skills.js --force --platforms .agents,.claude,.codex,.cursor,.omp,.grok,.kimi-code
node scripts/install-agent-skills.js --check --platforms .agents,.claude,.codex,.cursor,.omp,.grok,.kimi-code
node scripts/install-agent-skills.js --platforms .agents,.claude,.codex,.cursor,.omp,.grok,.kimi-code
```

## 证据边界

| 证据 | 可证明的范围 | 仍需另行验证 |
| --- | --- | --- |
| 临时七目标真实 payload fixture | 干净 checkout 中的安装器、字节复制、映射与幂等行为 | 当前机器的实际副本 |
| 实际目标数量、逐文件一致、备份、`--check=0`、二次写入数 0 | 已记录目录在检查时与源文件一致 | 客户端发现、加载和执行 |
| 客户端发现记录及获准的新会话调用记录 | 已记录客户端版本、模型、权限下的发现或实际调用 | 其它客户端、版本与未执行场景 |

客户端仅发现 skill 时，不得报告实际执行通过。运行记录应注明客户端、模型、权限、加载路径和实际调用。五工具运行证据见 [harness 适配说明](./harnesses.md)。

项目级副本受 gitignore 管理，不提交副本或备份。Hosted CI 使用临时 fixture；不得把开发者机器的 ignored 目录检查作为 hosted PASS 条件。

Trellis 的[隔离 bootstrap](./harnesses.md#bootstrap)只生成平台流程资产，不交付本业务 skill。初始化通过后，仍须按本页记录获准目标数量、同步源文件和验证副本；不得用 bootstrap PASS 替代业务 skill 的 `--check` 或客户端调用记录。
