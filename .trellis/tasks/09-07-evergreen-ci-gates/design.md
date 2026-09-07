# Design

拆分 Windows fast-gate 为 install、frontend check、fmt、clippy、test、secret scan 六步，每步保留原命令和 shell: pwsh，利用 runner 的单步 native exit code 传播，不引入自定义调度器。

加 Ubuntu/Node22 docs job：checkout → setup-node → npm --prefix docs ci → npm --prefix docs run build。Required checks needs=test/monitor/docs，三者均 success 才成功，不给 required job 加 paths 过滤。

Files:
- `.github/workflows/ci.yml`。
- `tests/sync-monitor-version.test.js`：仅现有 CI 契约断言需要时调整，行为探针仍必做。

`just ci` 保留 monitor+root 范围，`just docs-build` 仍独立；C2 准确说明本地验证组合。不为本项修改 justfile。回滚只回退本项 workflow/test diff。
