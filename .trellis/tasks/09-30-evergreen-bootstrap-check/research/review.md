# T04 独立实施审查

审查日期：2026-09-30。审查角色：`trellis-check`，代理 `/root/t03_review`。当前审查任务为 `.trellis/tasks/09-30-evergreen-bootstrap-check`。主会话已明确确认本次任务切换；未使用前序 T03 注入作为 T04 验收依据。

## 结论

T04 的只读诊断、受控隔离 bootstrap 和项目内文档回写通过本轮审查。固定版本 Trellis `0.7.0-beta.3` 的实际 `--version` 与 `init` 均退出 0，使用同一 Node 和 CLI 入口。四个项目覆盖文件保持逐字节一致，五平台的 7 个检查资产均生成。正式 `just ci` 与 `just docs-build` 均通过。

本机默认入口诊断仍为退出 1、`bootstrap.status=BLOCKED`。Codex 首选 npm 包装器缺少 Windows 可选二进制；默认 Trellis `0.6.17` 与项目 `0.7.0-beta.3` 不匹配。隔离执行器的通过不修复默认 PATH。五客户端新会话、hook/extension、权限与 hosted CI 仍须按 T06 单独验收。

## 审查范围与授权

- 已核对 T04 的 PRD、design、implement、JSONL、bootstrap 前提及所列规范。用户已批准项目内实施。审查修正限定于当前脚本、对应 fixture 和研究证据。
- 已复核 `scripts/check-harness-environment.js`、`tests/check-harness-environment.test.js`、`package.json`、`justfile`、`docs/agents/harnesses.md`、`docs/agents/residential-rule-tuning.md`。前序 T01/T02/T03 改动保持原样。
- 修正前与实施代理协调文件所有权。正式门运行前已释放产品文件；正式门期间未并发修改产品、运行测试或构建。最终审查复用同一文件版本的正式收据。
- 未调用全局安装、upgrade、信任或权限修改，未修改生产路由、真实凭据或数据库。未提交、归档、push 或创建 PR。

## Findings (fixed)

### R1：Windows 入口发现与版本探针损坏 Unicode 输出

- File：`scripts/check-harness-environment.js:40`、`:63`；`tests/check-harness-environment.test.js:42`。
- Issue：Windows PowerShell 默认输出编码与 Node 的 UTF-8 解码不一致。真实临时路径 `中文's tool/trellis.ps1` 被解析成乱码，发现结果对应的文件不存在。错误使可用入口被错误分类。
- Fix：仅在两个 PowerShell 子进程内设置 `[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)`。新增原生 Windows fixture，覆盖中文、单引号路径和中文 stderr。fixture 脚本使用 UTF-8 BOM，避免 Windows PowerShell 5 的脚本源解码干扰测试。未修改全局编码或 PATH。
- Evidence：`review-discovery-before.json` 的 `samePath` 和 `selectedExists` 为 false；`review-discovery-after.json` 两项均为 true。`probe-windows-discovery.js` 保留复现方法。修正后聚焦测试 19/19 通过，0 skipped。

### R2：显式入口路径分隔符差异导致替代入口重复

- File：`scripts/check-harness-environment.js:149`；`tests/check-harness-environment.test.js:158`。
- Issue：命令行传入的正斜杠绝对路径与 Windows 发现结果的反斜杠路径不相等，同一 executable 可能被重复探测。
- Fix：`--entry` 与 `--alternative` 都使用 `path.resolve` 规范化绝对路径，并添加参数测试。修正维持显式首选入口与替代证据的语义；替代入口成功仍不能覆盖首选失败。
- Evidence：聚焦 19/19 与正式根测试 201/201 均通过。最终真实诊断中应用内 Codex 二进制仅保留一次。

### R3：首次隔离获取收据错误记录为重试参数

- File：`research/isolated-paths.json`。
- Issue：首次 npm 获取把 userconfig 与 globalconfig 指向同一 `empty.npmrc`，npm 以 double-loading 错误退出 1。记录对象共用可变参数列表，后续修改把首轮 argv 误写为第二轮的两个独立配置文件。
- Fix：实施代理按已执行命令和首轮日志恢复首轮 argv，增加 `record_correction`，保留首次失败日志。第二轮使用两个独立空配置文件，退出 0。此次证据修正没有重新获取包。
- Evidence：`isolated-install-attempt1.log`、`isolated-install-attempt2.log`、`isolated-paths.json`。通用前提已写入 `docs/agents/harnesses.md:64`。

## Findings (not fixed)

- 默认 Codex 入口 `C:/home/lyh/.npm-global/codex.ps1` 退出 1，原因是缺少 `@openai/codex-win32-x64`。显式记录的应用内 `codex.exe` 可运行，版本 `0.159.2`。全局安装修复不在本任务授权范围；诊断保留首选失败与替代成功。
- 默认 Trellis 入口返回 `0.6.17`，与项目版本不符。脚本正确报告 `BLOCKED`，没有执行 bootstrap。全局升级和 PATH 修改不在授权范围。已批准的独立临时执行器完成本任务隔离验收。
- 五客户端 fresh-session、hook/extension 和权限行为未在 T04 验证。初始化日志中的上游 `features.hooks=true` 提示仅作为原始输出保留，未据此修改用户配置。T06 继续承接运行验收。

## 验收逐项核对

| 条目 | 结果 | 依据与边界 |
| --- | --- | --- |
| AC1 入口、版本、退出码、替代入口与权限分离 | PASS | `environment-default-after-review.json` 记录首选失败、可用应用二进制及四个 override；`permissions` 明确 `UNVERIFIED`。`checks.json` 保留真实退出 1。 |
| AC2 版本不匹配阻断，不改全局入口 | PASS | fixture 覆盖不匹配、缺失、非零退出、未知版本、权限错误、信号与超时。真实默认 Trellis 为 `BLOCKED`。诊断不执行 init 或安装。 |
| AC3 同一匹配入口实际隔离 init，四 override 不变 | PASS | `bootstrap-result.json` 的 version/init 前两个参数完全相同，实际版本均为项目要求。临时 tool/cache/candidate 分离，四个 override 的当前、候选前、候选后 SHA256 相同。 |
| AC4 五平台资产、项目 manifest 与写入边界 | PASS（限定证据范围） | `review-bootstrap-verification.json` 重新核对五平台 7 个资产、9 个当前保护文件和当前 manifest。未复制候选 manifest。命令及写入范围未涉及全局配置、信任和私有文件；未读取这些文件，也未声称完成全盘逐字节前后比较。 |
| AC5 缺少可用匹配 CLI 时明确 BLOCKED | PASS | 默认诊断持续退出 1；隔离方案另有真实固定版本获取、版本门和 init 收据。未用 `--help`、包元数据或文档替代执行证据。 |

## Bootstrap 证据复核

- 获取固定 `@mindfoldhq/trellis@0.7.0-beta.3`，采用任务临时 prefix/cache、两个不同空 npm 配置、`--global=false --ignore-scripts --no-audit --no-fund --save-exact`。首次获取失败与修正后获取成功分开保留。
- `resolved-executor.json` 记录实际 Node 和 CLI、CLI/core 与依赖解析。保存的 `isolated-package-lock.json` 与临时工具目录真实 lock 逐字节一致，共 59 个节点（含根）。CLI 与 core 的版本和 integrity 均与前期 registry 收据一致。
- init 时间：2026-09-30 13:25:54–13:25:55 UTC。命令为同一入口执行 `init --claude --codex --grok --kimi --omp --skip-existing -y`，退出 0。
- 当前 `.trellis/.template-hashes.json` SHA256 保持 `61058f644764515363f2b7d9c5b2877eb167d474a38fc08955e7eef22c3e5f79`。候选 manifest 独立保存，未回写当前仓库。
- 当前仓库和候选仓库的 7 个非白名单平台资产均通过 `git check-ignore -v` 核对；生成资产仍由忽略规则管理。
- 实际七个 skill 目标的三个业务 payload 文件均与源一致，共 21 个逐文件检查。该检查复用并独立复核 T02 交付，不能替代客户端加载或执行证明。

## Verification

| 检查 | 结果 | 收据 |
| --- | --- | --- |
| 聚焦诊断测试 | PASS，19/19，0 skipped；13:20:47–13:20:49 UTC | `review-focused.result.json`、`review-focused.log`、原始 `.log.gz` |
| 正式 `just ci` | PASS，exit 0；13:30:34–13:31:40 UTC | `checks.json`、`just-ci.log`、原始 `.log.gz` |
| Frontend TypeCheck | PASS，`tsc --noEmit` | `just-ci.log` |
| Frontend Lint | PASS，`eslint src` | `just-ci.log` |
| Frontend tests/build | PASS，73 文件、300 测试；构建通过 | `just-ci.log` |
| Rust fmt/clippy | PASS，`cargo fmt --check` 与 `cargo clippy --workspace --all-targets -- -D warnings` | `just-ci.log` |
| Rust tests | PASS，542 单元测试、3 集成测试；6 ignored，doc tests 0 | `just-ci.log` |
| Root syntax/contract/tests | PASS，Node 201/201，0 skipped；无独立根 linter | `just-ci.log` |
| Secret scan | PASS，正式门及 13:38:14 UTC 补充检查均退出 0 | `just-ci.log`、`final-secrets.log`、`checks.json` |
| 正式 `just docs-build` | PASS，exit 0；13:31:40–13:31:42 UTC | `checks.json`、`docs-build.log`、原始 `.log.gz` |
| 产品差异空白检查 | PASS，`git diff --check` exit 0；仅现有 CRLF 提示 | `final-diff-check.log`、`checks.json`；本轮审查复核 |
| 最终业务 skill 一致性 | PASS，`node scripts/install-agent-skills.js --check` exit 0；实际 7 目标数量另有逐文件证据 | `final-skill-check.log`、`checks.json`、`review-bootstrap-verification.json` |
| 默认本机环境诊断 | EXPECTED BLOCKED，exit 1；不能列为客户端运行 PASS | `checks.json`、`environment-default-after-review.json` |

已重新计算六个交付文件当前 SHA256，均与 `formal-state-before.json`、`formal-state-after.json` 一致。已对正式门两个 gzip 原始日志解压后重算 SHA256，并重算两个规范化日志 SHA256，全部匹配 `checks.json`。最终正式门覆盖本轮 Unicode 与参数规范化修正后的版本。

13:38:14 UTC 的最终检查再次确认六个交付文件和项目保护清单哈希不变。日志换行与行尾空白规范化另在 `evidence-normalization.json` 记录，原始字节以 gzip 保留。审查报告写入后已单独检查新文件空白；未发现错误。

前序 T03 曾记录 Rust `OperationInterrupted`。T04 本次正式门未复现；没有修改 Rust、超时或阈值。前序失败保留，具体墙钟预算耗尽原因未查明，不宣称根因已经修复。

## 持久回写与后续边界

`docs/agents/harnesses.md:46` 已说明诊断与状态语义，`:64` 已写入同一固定入口、独立缓存、两个 npm 配置、四 override 与 manifest 的验证方法。`docs/agents/residential-rule-tuning.md:73` 已注明 bootstrap 后仍须单独完成业务 skill 交付。上述合同适用 Claude Code、Codex、Grok Build、Kimi Code、OMP。

本审查报告不推进提交、归档、全局修复、远端 CI 或五客户端新会话验收。任务状态由主会话依据父任务顺序更新。
