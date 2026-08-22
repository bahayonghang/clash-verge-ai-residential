# 脚本：顶层查找进程 always

父任务：`08-22-process-attribution`。

## Goal

扩展脚本生成的配置让 Mihomo 实际执行查找进程，同时保持默认不进程路由。

## Requirements

- `ensureProcessLookup` 在 `ENABLE_AI_PROCESS_FALLBACK` 为 false 时也运行。
- 把 `config["find-process-mode"]` 设为 `always`。若仅 `config.profile["find-process-mode"]` 为 always，仍写出顶层 `always`。
- 不因本任务注入 `PROCESS-NAME` / `PROCESS-PATH`。进程路由仍只由 `routing.ai_process_fallback` 控制。
- 回归：「AI-only 不强制进程匹配」改为断言无进程路由规则，且顶层查找进程为 `always`（输入 `off` 也一样）。
- `docs/configuration.md`、`docs/local-configuration.md` 写明顶层键与 `profile:` 嵌套的区别。`CHANGELOG.md` 用英文记一条。

## Out of scope

- 新 TOML 开关。
- 修改 Clash Verge GUI。
- 监控代码。

## Acceptance Criteria

- [x] AC1：`configFixture({ findProcessMode: "off" })` 经 `main` 后 `output["find-process-mode"] === "always"`，且规则不含 `PROCESS-NAME` 与 `PROCESS-PATH`。
- [x] AC2：输入仅有 `profile: { "find-process-mode": "always" }` 时，输出顶层为 `always`。
- [x] AC3：`ENABLE_AI_PROCESS_FALLBACK` 为 true 的既有进程路由回归仍通过。
- [x] AC4：`npm test` 与 `just ci` 通过；公开模板无真实凭证。
