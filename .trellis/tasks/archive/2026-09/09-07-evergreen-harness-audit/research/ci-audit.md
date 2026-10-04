# GitHub Actions 与托管门禁审计

审计日期：2026-09-07。当前本地基线为 `dev@5578576787dba73ba1b96985a2fc408fc246bfed`；GitHub `dev` 为 `15f45f59d31156aa1da86b6bb9275b98c5064037`，GitHub `main` 为 `caa580650d6d1bcb5b338f5e1c9ccd4afff3e7ea`。因此本地相对远端 `dev` 的 31 个提交没有托管 CI 证据，下面严格区分本地当前代码、历史 run 和实时仓库设置。

## 严重度排序发现

### P1 / 当前：monitor 作业可以把中间 native command 失败报告为成功

`.github/workflows/ci.yml:75-83` 把前端安装、前端完整检查、Rust fmt、clippy、tests 和凭据扫描放在同一个 `shell: pwsh` step。GitHub 对内置 PowerShell shell 只在脚本末尾把最后一次 `$LASTEXITCODE` 作为 step 退出码；`$ErrorActionPreference = 'stop'` 不会让普通 native executable 的非零退出自动终止这个脚本。官方说明见 [Workflow syntax for GitHub Actions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)。

只读负向探针复现了该行为：

```text
$ErrorActionPreference = "stop"
cmd /c exit 7
Write-Output "SURVIVED_AFTER_EXIT_7"
cmd /c exit 0
exit $LASTEXITCODE

输出：SURVIVED_AFTER_EXIT_7
最终退出：0
```

这意味着 `.github/workflows/ci.yml:78-82` 任一命令失败而最后的 `npm run check:secrets` 成功时，`Windows / residential-monitor` 仍可能变绿；随后 `.github/workflows/ci.yml:85-96` 的 `Required checks` 也会接受这个假绿。当前本地各门由主审线程独立运行并通过，所以没有证据表明当前代码已被这条缺陷掩盖；缺陷影响的是 hosted gate 的可信度和未来回归。

最小改造：把六条 native command 拆成六个独立 step，保留同一个 `monitor` job、现有命令、timeout 和聚合检查名。不要新增通用 CI 执行框架。可在 `tests/sync-monitor-version.test.js:202-208` 的既有 workflow 合同检查旁加入回归，证明六条命令是独立 `run:` step，且 `required-checks.needs` 仍包含 `monitor`。

必须通过：

- `node --test tests/sync-monitor-version.test.js`
- `just ci`
- 精确绑定 PR head SHA 的 hosted `Windows / residential-monitor` 与 `Required checks`
- PR 合入后精确绑定 `main` 新 SHA 的同名检查

规划/审查应由能同时读取 repo、PowerShell 退出语义和 GitHub run 的强模型完成；命令拆 step 与小型文本回归可交给便宜执行模型，最终仍由强模型审查实际 hosted run。

### P2 / 当前：本机七套 harness 的 ignored skill 副本已漂移

`scripts/install-agent-skills.js:9-16` 把 `.agents`、`.claude`、`.codex`、`.cursor`、`.omp`、`.grok`、`.kimi-code` 作为七个投递根；`:33-45` 和 `:121-155` 已实现无写入的 `--check`。这些根全部被 `.gitignore:30-36` 忽略，不会出现在干净 checkout；投递器在目标根不存在且未传 `--create` 时会跳过（`scripts/install-agent-skills.js:92-100`）。因此把裸 `node scripts/install-agent-skills.js --check` 接进 hosted CI，只会检查零个目标并退出 0，不能证明七份副本存在或一致。

当前开发者工作区中七个根实际存在，以下只读命令真实失败：

```text
rtk node scripts/install-agent-skills.js --check
exit 1
skill 与已安装副本不一致：7 个平台各有 SKILL.md、reference.md、scripts/build-inputs.js 不一致，共 21 项
```

这不是纯文本噪声。源 `skills/residential-rule-tuning/scripts/build-inputs.js:9-14` 已支持 `anthropic_core`、`gemini_api_core`、`antigravity_core`，且 `:43` 将 `grok_web_assets` 映射到 `GROK_STRICT_EXACT_DOMAINS`；例如 `.agents` 副本在 `scripts/build-inputs.js:38` 仍把 `grok_web_assets` 映射到 `GROK_EXACT_DOMAINS`，并缺少三个 core builder。`tests/install-agent-skills.test.js:10` 只导入源实现，`:71-79` 只在临时 fixture 验证 `--check` 机制，因而现有单测能绿而真实七份副本仍错。

已选改造放入 `09-07-evergreen-skill-delivery`：在临时仓库 fixture 中用真实 source payload 显式创建全部七个目标，验证完整内容、幂等和关键生成器映射；这项测试可随现有 `npm run ci` 在 clean checkout 执行。获准交付到当前本机时，再同步七个 ignored 目标并运行真实 `--check`。不增加会在 hosted checkout 零目标通过的 `check:skills` 门，也不把 ignored 本机安装态伪装成可由 GitHub checkout 持有的仓库副本。

必须通过：

- 临时 fixture 从真实 source payload 创建全部七个目标，逐文件一致且二次安装 `written=0`
- `node --test tests/install-agent-skills.test.js`，覆盖 24/12/12、三个 core builder 与正确 Grok 映射
- `npm run ci` 与 `just ci`
- 获准本机交付后，`node scripts/install-agent-skills.js --check` 退出 0，并明确输出覆盖七个实际存在的目标
- 五套客户端的真实新会话发现/加载仍单列 `UNVERIFIED`，本机 ignored 副本一致不等于客户端已加载

单一源、adapter 边界与最终审查需要强模型；按已批准映射补临时 fixture、整理文档和同步已审查副本可交给便宜执行模型。

### P2 / 当前：`main` 分支保护关闭了已批准且仍写在规范中的线性历史

实时 `GET /repos/bahayonghang/clash-verge-ai-residential/branches/main/protection` 返回：

- `Required checks` 与 GitHub Actions app id `15368` 绑定，`strict=true`；
- PR 要求开启、审批数 0、管理员约束和对话解决开启；
- force push 与删除关闭；
- `required_linear_history.enabled=false`。

最后一项与 `.trellis/spec/frontend/quality-guidelines.md:98-105` 冲突，其中 `:103` 明确要求 linear history；也与已归档 `07-23-main-branch-protection/prd.md:19,28,39`、`design.md:10-16` 的批准策略和已勾选验收冲突。当前 `main` 最新标题本身也是 `Merge pull request #7 from bahayonghang/dev`。无法从现存事件确定该设置何时被关闭，所以“发生时间/操作者”是 `UNVERIFIED`，但当前漂移已通过 API 确认。

最小改造应放入独立的 `09-07-evergreen-branch-governance`，这是 GitHub 外部写入，实施前需要对精确 repo/branch/完整保护请求取得批准。先读出并保存完整当前结构，只把 `required_linear_history` 改为 `true`，立即完整 GET 回读，证明其他字段未漂移；不得用局部猜测请求覆盖未知设置。仓库已允许 squash/rebase，启用线性历史有可用合并路径。

必须通过：

- 变更前后完整 protection GET 的结构化对比
- `required_linear_history.enabled=true`
- `Required checks` 仍 app-bound、`strict=true`，管理员/PR/对话/force/delete 字段保持批准值
- 下一次普通 PR 不使用管理员绕过，并由 exact-head `Required checks` 放行

此项只适合有 GitHub API、权限与状态新鲜度审查能力的强模型规划、执行和复核，不应下放给便宜模型直接写远端设置。

### P2 / 当前：文档站可在本地构建，但不属于 hosted required gate

`docs/package.json:5-14` 定义 Node 22+ 的 VitePress 构建；`Justfile:20-25` 把它作为独立命令，这是合理的本地工具链边界。可是 `.github/workflows/ci.yml:16-97` 只有 root matrix、monitor 和聚合 job，`:87` 的 required needs 只有 `[test, monitor]`。因此破坏 VitePress 配置、导航或 Markdown 构建的 PR 仍可能通过 branch protection。

已归档 `09-03-dependency-upgrade` 明确决定不把 docs 塞进 Node 18+ 的本地 `just ci`，本发现不推翻该决定。最小做法是在 GitHub workflow 增加独立 Ubuntu/Node 22 `docs` job：`npm --prefix docs ci` 后 `npm --prefix docs run build`，并把 `docs` 加入 `Required checks` 的 `needs`。本地仍保留独立 `docs-build`。

必须通过：

- `npm --prefix docs ci`
- `npm --prefix docs run build`
- hosted `docs` job 与聚合 `Required checks`
- 一个临时隔离的负向 fixture 或 workflow 合同测试，证明 docs job failure 会使聚合失败；不得在真实 PR 中故意提交破坏文档的变更

边界决定与最终聚合审查用强模型；增加固定 job 的 YAML 可交给便宜执行模型。

### P3 / 可选加固：Action 引用可变且仓库未启用 SHA pinning

`.github/workflows/ci.yml:40,45,58,63` 使用 `actions/*@v7`，`:68` 使用第三方 `dtolnay/rust-toolchain@stable`；实时 Actions 权限返回 `allowed_actions=all`、`sha_pinning_required=false`。GitHub 的 [Secure use reference](https://docs.github.com/en/actions/reference/security/secure-use) 说明完整 commit SHA 是 action 不可变引用的唯一方式。

当前 workflow 已是 `permissions: contents: read` 且 checkout 使用 `persist-credentials: false`，降低了影响；没有证据显示当前 tag 已被移动或有凭据暴露。这是供应链加固项，不是本轮 must-fix，留待单独批准和维护策略决定。若以后实施，应把三个 action 解析并核对为其官方仓库/可信发布 tag 对应的完整 SHA，同一行保留版本注释；可选增加 GitHub Actions 类型的 Dependabot 周期更新。只有 default branch 已全部使用 full SHA 后，才考虑经批准把仓库 `sha_pinning_required` 设为 true，否则会先把现有 CI 配置阻断。

若实施则验证：

- 每个 SHA 通过对应 action 官方仓库的 tag/ref 反查，不能采信 fork
- workflow 中不存在非本地 action 的 tag/branch 引用
- exact-head hosted 全矩阵和 `Required checks` 成功
- 若启用远端 pinning policy，先合入 pinned workflow，再 PUT 设置并 GET 回读 `sha_pinning_required=true`，最后手工 dispatch 验证

选择/核验 SHA、变更顺序和远端 policy 必须由强模型负责；已核定后的机械替换和版本注释可交给便宜执行模型。

## 历史失败工作流根因

GitHub 可见 31 次 run：`CI` 成功 18、失败 2、取消 2；退役的 `Bootstrap repository contents` 成功 1、失败 7、跳过 1。两个取消项由 `.github/workflows/ci.yml:12-14` 的新 run 取消旧 run 设计解释，不能计为代码失败。

| run / head | 失败 job / step | 根因与修复 | 当前适用性 |
|---|---|---|---|
| [33710198328](https://github.com/bahayonghang/clash-verge-ai-residential/actions/runs/33710198328) / `d3cfb384` | Windows / Node.js 22；`Run checks and regression tests`；`tests/install-agent-skills.test.js:120` | checkout 使用 CRLF，测试只接受 `---\n`，`startsWith` 为 false。`e732de5` 先 `.replace(/\r\n/g, "\n")`，随后 [33710808326](https://github.com/bahayonghang/clash-verge-ai-residential/actions/runs/33710808326) 全部成功。 | 已修复；当前断言见 `tests/install-agent-skills.test.js:116-128`，不重复立项。 |
| [32560684556](https://github.com/bahayonghang/clash-verge-ai-residential/actions/runs/32560684556) / `b01462a1` | Windows / Node.js 22；`Run checks and regression tests`；`tests/sync-local-config.test.js:827` | fixture 用 LF-only 字符串替换删除配置行，CRLF checkout 下没有删除，导致预期异常未发生。`5746354` 改为 `/grok_core = true\r?\n/`，随后 [32561306866](https://github.com/bahayonghang/clash-verge-ai-residential/actions/runs/32561306866) 全部成功。 | 已修复；当前同类断言见 `tests/sync-local-config.test.js:873-885`，保留 Windows matrix。 |
| `29972422612`、`29972539541`、`29972551861` | 退役 bootstrap；`Materialize reviewed repository tree` | 拼接后的 base64 payload 与硬编码 SHA-256 不一致，`sha256sum` fail closed。 | 仅初始化历史；当前 `.github/workflows/` 只剩 `ci.yml`。 |
| `29972665474`、`29972667447`、`29972727956`、`29972729388` | 退役 bootstrap；materialize/commit 后 push | GitHub App token 被远端拒绝更新 `.github/workflows/ci.yml`，日志明确为缺少 `workflows` permission。随后 `d7653d9` 的 bootstrap run 成功，旧 workflow 已退役。 | 历史权限/自修改设计问题；不要恢复自修改 bootstrap。 |

历史失败证明 Windows matrix 有独立价值，也证明只看最终聚合失败文案不能定位根因；必须保留 run SHA、job、step 和最小日志块。

## 托管状态与证据边界

### PASS

- GitHub workflow `CI` 当前 active。
- `main@caa58065` 的 [run 33711400557](https://github.com/bahayonghang/clash-verge-ai-residential/actions/runs/33711400557) 六个检查均成功；`Required checks` 来自 app id `15368`。
- `dev@e732de5d` 的 PR [run 33710808326](https://github.com/bahayonghang/clash-verge-ai-residential/actions/runs/33710808326) 六个检查均成功。
- 当前 branch protection 仍要求 strict/app-bound `Required checks`、PR、管理员约束和对话解决，并禁止 force push/delete。
- workflow 权限为 `contents: read`，checkout 配置 `persist-credentials: false`。

### FAIL

- 当前开发者工作区的 `node scripts/install-agent-skills.js --check` 退出 1：七个 ignored 目标共 21 个文件漂移。
- monitor `pwsh` step 的退出语义不能保证所有列出的命令都成功。
- live branch protection 的 `required_linear_history.enabled=false` 与批准策略/规范冲突。

### SKIPPED

- 本代理按分工没有重跑 root/前端/Rust/docs 本地测试，也没有 `npm ci`；结果由主审线程记录。
- 没有 rerun/dispatch workflow，没有 push、PR、分支保护写入或 Actions policy 写入。
- 没有执行真实 Clash/Mihomo、NSIS、Credential Manager 写入、30 天库或长时 soak。

### UNVERIFIED

- 当前 `dev@55785767` 尚不在 GitHub；其 Actions v7、31 个本地提交以及最新路由/工程修复没有 hosted runner 证据。最新 hosted 绿色只覆盖 `e732de5d`/`caa58065`，当时 workflow 仍是 checkout/setup-node v4。
- 现有历史绿色 monitor run 的日志显示命令被执行，但由于单 step 退出语义，绿色本身不足以作“每条命令都成功”的一般性证明；当前本地独立门通过仅覆盖本机。
- 干净 checkout 不含七个 ignored harness 根；裸 `--check` 会跳过全部目标。五套客户端是否在全新会话发现并加载同步后的 skill，需要各客户端实际证据；CI fixture 只能证明投递器可把真实 payload 正确安装到临时目标。
- branch protection 线性历史何时、由谁关闭不可从当前可访问数据确定。

## 建议子任务依赖

1. `09-07-evergreen-ci-gates` 修复 PowerShell 假绿并增加独立 docs job；不接入 clean checkout 中零目标通过的裸 `--check`。
2. `09-07-evergreen-skill-delivery` 独立完成临时全七目标真实 payload 测试，并在获准交付时同步当前本机 ignored 副本、实际运行 `--check`。
3. `09-07-evergreen-branch-governance` 的远端状态写入单独授权，需要精确 repo/branch/完整保护请求；其 `.trellis/spec/frontend/quality-guidelines.md` 回写与 `09-07-evergreen-project-contract` 串行，避免同文件冲突。实施后完整回读保护状态，下一次普通 PR 再补不绕过的流程证据。
4. Action SHA pinning 降为 P3 可选加固并延期，不阻塞上述任务。
5. 每个子任务的 hosted 证据都绑定实际 PR head SHA；本地门、PR 门和 `main` push 门分别记录，不能互相替代。
