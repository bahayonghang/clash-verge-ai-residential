# T01 独立审查

日期：2026-09-30（America/Chicago）。角色：已派发的 `trellis-check`，继承主会话模型配置，没有调低审查模型或递归委派。任务：`.trellis/tasks/09-30-evergreen-dependency-gate`。用户已批准项目内实施。审查结论：修正 1 项审计覆盖缺陷；本地 AC1–AC4 通过。父任务的集成验收、T03 说明同步和 T06 运行证据仍按原任务边界处理。

已读取完整 hook 保存文件、`check.jsonl`、PRD、设计、执行清单、`CONTEXT.md`、适用 spec、审计与实施收据。审查使用 `trellis-check` skill。共享工作树中的其它任务内容保持原状。

## Findings (fixed)

### R1：完整审计可能受开发依赖排除配置影响

- 文件：`justfile:23–24`、`.github/workflows/ci.yml:81,122`。
- 问题：新增命令仅指定 `--audit-level=high`。当调用环境使用 `NODE_ENV=production` 或 npm 的 `omit=dev` 配置时，开发依赖可能不进入审计。该行为违反本任务的完整 npm 审计要求。
- 复现：只将修改前的公共 `package.json` 与 `package-lock.before.json` 复制到临时目录。未安装依赖，也未改仓库或用户配置。仅对 npm 子进程设置 `NODE_ENV=production`、`npm_config_omit=dev`。真实 npm 注册表审计结果如下。

| 观测时间（UTC） | 命令后缀 | 退出码 | high / total |
| --- | --- | --- | --- |
| 2026-09-30 12:35:26 | `audit --json --audit-level=high` | 0 | 0 / 0 |
| 2026-09-30 12:35:28 | `audit --json --audit-level=high --include=dev` | 1 | 2 / 2 |

- 修正：monitor/docs 的 recipe 和 CI 命令都显式加入 `--include=dev`。独立生产依赖审计继续使用 `--omit=dev`。
- 回归：`tests/sync-monitor-version.test.js:538` 起的两个 job 各有 7 个负向变体，共 14 个。新增缺失 `--include=dev` 的变体；排除开发依赖的变体改为用 `--omit=dev` 替换 include。该变体避免把同时 include/omit 的 npm 优先级误记为实际排除开发依赖。
- 回写：`AGENTS.md`、`README.md`、monitor frontend spec、T01 `implement.md` 和 `research/implementation.md` 同步最终命令及环境边界。五工具共同适用。旧失败收据与无 include 参数的原验收收据均保留。

## 依赖与 workflow 核对

独立结构比较确认：修改前锁文件与 Git HEAD 的锁内容一致；最终锁文件 SHA256 为 `fd0dc69d35e7d8c78d486a53fe3651df19220bbe35ee55a04a55647edc5fd723`，与实施收据一致。仅以下 3 个节点有变化，每个节点仅改 `version`、`resolved`、`integrity`。新版本的 tarball、integrity、dependencies、engines、bin 与保存的 registry 元数据一致。根 manifest、父节点、其它 lock 节点、ESLint/TypeScript major、lint 规则与产品源码没有变化。

| 节点 | 版本变化 | 保持的父范围 |
| --- | --- | --- |
| 根 `brace-expansion` | 1.1.18 → 1.1.21 | minimatch 3.1.5 的 `^1.1.7` |
| typescript-estree 下的 `brace-expansion` | 5.0.9 → 5.0.12 | minimatch 10.2.6 的 `^5.0.8` |
| 根 `js-yaml` | 4.3.1 → 4.3.2 | eslintrc 3.3.6 的 `^4.3.0` |

三个更新均在原有父范围内。已完成的两次 monitor `npm ci` 和当前 lint/typecheck 支持锁文件可安装和工具行为兼容结论。没有使用 `npm audit fix --force`。

monitor 的 audit 独占一条 `pwsh` step，位于安装之后、frontend check 之前。docs 的 audit 独占一条默认 shell step，位于安装之后、build 之前。审计步骤没有 `if` 或 `continue-on-error`。`Required checks` 仍依赖 `[test, monitor, docs]`，并在任何依赖结果不是 success 时退出 1。静态合同测试及实际 pwsh 包装器的 exit 7 用例均通过。

另用当前仓库 justfile、临时 PATH 中的 `npm.cmd` / `cargo.cmd` 和临时调用记录进行 recipe 失败注入。stub 只记录命令和返回模拟退出码，不访问网络或真实工具。结果：

| 模拟失败位置 | stub 退出码 | just 退出码 | 实际执行命令数 |
| --- | --- | --- | --- |
| monitor audit | 7 | 1 | 1 |
| docs audit | 7 | 1 | 2 |
| cargo audit | 7 | 1 | 3 |
| 全部成功 | 0 | 0 | 3 |

Windows recipe 使用 `powershell.exe -Command`，该探针保留非零结果并停止后续行；进程层退出码折叠为 1。没有宣称 recipe 原样保留 7。首次探针脚本因 Python 字符串换行转义错误退出 1，未启动命令；修正探针后取得上表结果。

## Findings (not fixed)

- `.trellis/spec/frontend/quality-guidelines.md` 仍描述 6 条 Windows native step 和无审计的 docs 流程。主会话已接收同合同同步并归属 T03；该文件不在本次独占范围，审查没有修改。T03 必须保持最终 `--include=dev --audit-level=high` 命令。
- RustSec 仍有 6 个 unmaintained、1 个 unsound 警告。T01 没有改 Cargo.lock。现有 Windows target 证据没有 glib 路径；其它 target 未由该证据验证。警告未记为漏洞已修复。
- 当前修改没有 hosted CI 和五客户端 fresh-session 的新证据。T06 负责记录；本次未进行提交、归档、push、PR 或远端触发。

## Verification

以下复测均在 R1 修正后执行；`.result.json` 记录真实命令、时间与退出码。

| 检查 | 结果 | 收据前缀 |
| --- | --- | --- |
| `npm run ci` | PASS，143 tests，0 fail，0 skipped；语法和秘密扫描通过 | review-root-ci |
| `npm --prefix residential-monitor run typecheck` | PASS / exit 0 | review-typecheck |
| `npm --prefix residential-monitor run lint` | PASS / exit 0 | review-lint |
| `actionlint .github/workflows/ci.yml` | PASS / exit 0 | review-actionlint |
| `npm --prefix residential-monitor audit --include=dev --audit-level=high` | PASS / exit 0，0 vulnerabilities | review-monitor-audit |
| `npm --prefix docs audit --include=dev --audit-level=high` | PASS / exit 0，0 vulnerabilities | review-docs-audit |
| `just dependency-audit`，子进程设置 `NODE_ENV=production`、`npm_config_omit=dev` | PASS / exit 0；两个 npm audit 均为 0 vulnerabilities；RustSec 警告保留 | review-dependency-audit-production |

审查未修改 lock、产品源码、frontend 源码或 docs 页面。完整产品证据复用 `product.result.json` / `product.log`：`just ci` 退出 0，Vitest 73 文件 / 300 测试，Rust 单元 542 通过 / 6 ignored，另 3 个 kill_gate 集成测试通过，Rust doc tests 0。更新后的根测试已单独重跑。全新 docs 安装与构建复用 `docs-install`、`docs-build` 收据。未把复用记录标为审查重新运行。

AC1：通过。完整 npm audit 使用显式 dev 参数并退出 0；原 high=2 的失败保留。AC2：通过。三个 lock 节点的精确差异与原 semver 边界核对完成。AC3：通过。独立步骤、聚合、pwsh 与 recipe 失败行为已验证。AC4：通过。本地产品门、docs、审计、lint/typecheck 与失败传播均有证据，适用五工具的说明已回写。父任务未由本报告整体关闭。
