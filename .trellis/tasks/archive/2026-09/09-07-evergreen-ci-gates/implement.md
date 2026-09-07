# Implementation

1. 批准后重新核对基线，加载父任务 CI 证据。
2. 拆 steps，补 docs job 和 aggregate 判断，保留所有现有命令与 Node 矩阵。
3. 在临时目录用 exit7→exit0 复现旧语义，并验证新单命令 step 非零退出。无需修改产品代码造失败；runner 跨步骤行为由 hosted 补证。
4. `node --test tests/sync-monitor-version.test.js`、`npm run ci`、monitor check、Rust fmt/clippy/workspace tests、docs build；干净安装由 CI 按现有 lockfile 执行。
5. 强模型审查 failure/cancel/skip 传播；保存验证 SHA。没有获授权的 PR/hosted run 则标 UNVERIFIED，获授权后继续验证对应提交。
6. 向 C2 回写质量门事实，注明五工具适用；父任务核对实际与计划差异。

分工：Codex/Claude Code 强模型审查 shell 与 CI 语义；较便宜模型仅按已定方案拆 YAML 与调整指定断言，不降低门禁。

## C1 实施记录（2026-09-07）

- `.github/workflows/ci.yml`：monitor 六条原生命令拆为独立 `shell: pwsh` step；新增 `docs` job（Ubuntu / Node 22，checkout@v7 + setup-node@v7）；`required-checks` 的 `needs` 与 `needs.*.result == success` 判断包含 `test`/`monitor`/`docs`。未改 Node 矩阵、timeout、concurrency、permissions、既有命令文本，也未加 paths 过滤。
- `tests/sync-monitor-version.test.js`：直接解析仓库 `.github/workflows/ci.yml`。pwsh 探针在临时目录复现 GitHub 包装下的 exit 7→exit 0 假绿，并验证单命令脚本退出 7。
- 本地已跑：`node --check tests/sync-monitor-version.test.js`、`node --test tests/sync-monitor-version.test.js`（11/11，两次一致）、负向 pwsh 探针（多命令 exit 7→0 且输出 SURVIVED_AFTER_EXIT_7；单命令 exit 7）、`just ci` 退出 0、`just docs-build` 退出 0。
- 未做：hosted run、PR。hosted 仍为 UNVERIFIED。
