# T01 实施与本地验收

日期：2026-09-30（America/Chicago）。基线：`dev@d3a25b4164343f5cbeab8a51efe1a1d3974cdf4a`。用户已批准项目内实施。当前状态：实施、本地验证与独立强模型审查完成，等待主会话集成验收。审查修正和最终证据见 `review.md`。未提交、归档、push、PR 或触发远端 workflow。

审查交接：独立 reviewer 随后发现 `NODE_ENV=production` / `npm_config_omit=dev` 可使未显式 include 的审计忽略开发依赖。reviewer 已复现旧锁 high=2 被隐藏，并接管 CI、recipe、测试和说明中的 `--include=dev` 修正。本文记录首次实施及其环境下的本地收据；后续 review 收据才用于最终状态判断。原实施者不覆盖 reviewer 的产品文件，不并发重跑产品构建。

## 变更范围

- `residential-monitor/package-lock.json`：只改三个节点的 `version`、`resolved`、`integrity`。根 manifest、父链、其它节点、lint 规则和产品源码均未改。
- `justfile`：新增独立 `dependency-audit`。依次执行 monitor 完整 npm audit、docs 完整 npm audit、RustSec audit。每条原生命令单独执行。
- `.github/workflows/ci.yml`：monitor/docs 安装后新增独立 high 阈值审计 step。Windows monitor 现有七条独立 pwsh 原生命令。`Required checks` 仍依赖 `test`、`monitor`、`docs`。
- `tests/sync-monitor-version.test.js`：新增审计合同、14 个负向变体、recipe 命令合同、实际 pwsh 包装器保留审计失败退出码。审查补上缺失 `--include=dev` 的负向用例。没有访问真实控制器或数据库。
- `AGENTS.md`、`README.md`、`.trellis/spec/residential-monitor/frontend/index.md`：持久记录依赖门、工具与网络前提、漏洞/网络失败/警告分类，以及 Claude Code、Codex、Grok Build、Kimi Code、OMP 的共同验收边界。

## 依赖链与公告

实施前重新运行完整 npm audit：exit 1，high=2。`npm-audit-monitor-before.log` 保留 npm 返回的四项公告及受影响区间：GHSA-q2hr-2g5m-vwhr、GHSA-qhr7-859c-m2p7、GHSA-6j4f-fj2g-mc7p、GHSA-2883-xcg3-v3hh。版本与 tarball/integrity 均从 npm registry 的 `npm view <name>@<version> ... --json` 实际读取，未使用 `npm audit fix --force`。

| 现有父链 | 修复前 | 修复后 | 父依赖范围 | registry 元数据 |
| --- | --- | --- | --- | --- |
| eslint-plugin-react@7.37.5 → minimatch@3.1.5 | brace-expansion@1.1.18 | brace-expansion@1.1.21 | `^1.1.7` | brace-expansion-1.1.21.json |
| typescript-eslint@8.69.0 → typescript-estree@8.69.0 → minimatch@10.2.6 | brace-expansion@5.0.9 | brace-expansion@5.0.12 | `^5.0.8` | brace-expansion-5.0.12.json |
| eslint@9.39.5 → @eslint/eslintrc@3.3.6 | js-yaml@4.3.1 | js-yaml@4.3.2 | `^4.3.0` | js-yaml-4.3.2.json |

`dependency-chain-before.json`、`dependency-chain-after.json` 保存完整相关父链。`lock-delta.json` 验证只有上述三个节点变化，且每个节点仅三个字段变化；dependencies/engines/bin 与新版本 registry 元数据一致。`package-lock.before.json` 和 `lock-before-hash.json` 保留修改前锁文件及 SHA256。两次全新 `npm ci` 成功，验证当前锁定元数据可安装；第二次包含在 `just ci` 中。

## 必需检查

每项 `.result.json` 记录实际命令、开始/结束时间、退出码和日志名。修复前失败与后续通过分开保存。完整 npm 审计最终命令显式使用 `--include=dev`；原有无该参数的收据保留为审查修正前证据。

可读 `.log` 已归一化行尾及尾随空白；13 份原始日志保存在同名 `.log.gz`。`evidence-normalization.json` 记录原始/归一化 SHA256。新增文件第一次空白检查发现 `dependency-audit.log` 的末尾空行；移除可读副本的末尾空白后，45 个任务文本文件全部通过，见 `new-files-whitespace-check.json`。原始捕获未删除。

| 检查 | 退出码 | 证据前缀 |
| --- | --- | --- |
| `npm --prefix residential-monitor ci` | 0 | monitor-ci |
| `npm --prefix residential-monitor audit --include=dev --audit-level=high` | 0，found 0 vulnerabilities | review-monitor-audit |
| `npm --prefix residential-monitor audit --omit=dev --audit-level=high` | 0 | npm-audit-monitor-prod-after |
| `npm --prefix docs ci` | 0 | docs-install |
| `npm --prefix docs audit --include=dev --audit-level=high` | 0 | review-docs-audit |
| `cargo audit --file residential-monitor/src-tauri/Cargo.lock` | 0，7 个 allowed warnings | cargo-audit |
| `node --test tests/sync-monitor-version.test.js` | 0，15/15，无 skipped | sync-monitor-version-tests |
| `actionlint .github/workflows/ci.yml` | 0 | review-actionlint |
| `just dependency-audit`，子进程设 `NODE_ENV=production`、`npm_config_omit=dev` | 0 | review-dependency-audit-production |
| `just ci` | 0 | product |
| `just docs-build` | 0 | docs-build |
| `git diff --check` | 0 | diff-check |

`just ci` 结果：根 Node 测试 143/143；Vitest 73 文件、300 测试；Rust 单元测试 542 通过、6 ignored；kill_gate 集成测试 3 通过；Rust doc tests 0。六个 ignored 没有算作通过：凭据写入、完整隔离容量库和四个隔离库阶段探针仍保留原边界。

审查修正后另跑 `npm run ci`（143/143，包含更新后的 14 个负向变体）、monitor typecheck、monitor lint，均退出 0。收据前缀分别为 `review-root-ci`、`review-typecheck`、`review-lint`。审查未改 lock、产品源码或 docs 页面，故复用实施阶段的同一产品源码与依赖集的完整 Rust、Vitest、build 和 docs-build 收据；不把复用结果记为重新运行。

## 验收映射

- AC1：完整 npm audit 从 high=2 / exit 1 变为 0 vulnerabilities / exit 0；生产依赖审计另外通过。
- AC2：`lock-delta.json` 的三个节点/三个字段断言通过。所有修复满足原父依赖 semver，未修改 manifest 或产品代码。
- AC3：独立 audit steps、七条 pwsh 命令与三 job 聚合合同通过。负向变体拒绝删除审计、缺失 include=dev、用 omit=dev 替代、阈值改为 critical、continue-on-error、条件跳过、合并 native 命令。pwsh 测试实际返回模拟的 exit 7。独立 recipe 也在三处失败注入中停止，详见 `review.md`。
- AC4：上述本地产品门、全新 docs 安装/构建、安全审计、actionlint 与定向测试均通过。适用五工具的说明已落盘。独立强模型审查已完成；父任务集成验收继续由主会话负责。

## 剩余边界与交接

- RustSec：6 项 unmaintained（proc-macro-error、五项 unic）与 1 项 unsound（glib 0.18.5）警告仍存在。本项未修改 Cargo.lock。父审查的 Windows target 树没有 glib 路径；其它 target 不据此获得安全结论。
- 本机 Node 26.7.0 / npm 12.1.0；本轮没有替代 hosted Node 18/20/22 矩阵或五客户端 fresh-session。远端 CI 仍由 T06 单独记录。
- npm 12 持续提示用户 `.npmrc` 的 allow-scripts 被 package.json 的既有 allowScripts 覆盖。未修改全局配置或安装脚本授权。
- `.trellis/spec/frontend/quality-guidelines.md` 的旧 six-step 描述交给 T03 同步；主会话已确认接收。该文件不在本项独占范围内。
- 生成依赖链证据时，内联 Node 正则转义和 Python 换行转义先后失败。修正后通过 Python 结构校验生成最终 JSON。此类证据工具错误没有改变产品代码，也没有覆盖修复前 audit 失败记录。
- 本项只写获批项目文件与当前任务研究产物，没有修改私有 TOML/JS、凭据、生产控制器、真实数据库、系统安装态或全局权限。
