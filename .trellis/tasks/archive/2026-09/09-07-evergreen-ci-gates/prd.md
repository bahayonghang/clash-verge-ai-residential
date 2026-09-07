# 修复 CI 失败传播并补齐文档检查

## Goal
让 Windows 中间检查失败时 CI 必然失败，并让文档构建进入托管必需检查。

## Evidence
- `.github/workflows/ci.yml:77-83` 把六条 native commands 放进同一 pwsh step；错误偏好为 Stop 仍不能使默认 native exit code 自动抛出，末命令可覆盖失败状态。
- 父任务 research/ci-audit.md 记录 exit 7→exit 0 探针、历史失败与已修复提交。
- 当前缺少 docs build job；本轮 docs 本地构建通过，这一项是预防性 P2 改善，不是已复现文档故障。

## Requirements
- R1 Windows 每条原生命令的失败均阻止后续步骤；保留所有现有检查及稳定 Required checks 名称。
- R2 docs 按已有 lockfile / Node22 构建，并进入 Required checks，不增加工具依赖。
- R3 区分本机 Node26、托管 Node18/20/22 和历史绿色 run。

## Acceptance Criteria
- [x] AC1 / R1：六条命令拆成独立 steps；任一失败使 monitor job 与 Required checks 失败，后续成功不能覆盖。
- [x] AC2 / R1：隔离失败命令复现旧语义并验证新单命令 step 非零退出；不只用 YAML 字符串断言证明行为。
- [x] AC3 / R2：docs job 干净安装后构建成功；Required checks 的 needs 与结果判断均包含 docs。
- [x] AC4 / R1–R3：根ci、monitor前端/Rust、docs本地通过；对应提交 hosted run 才能标 hosted PASS，否则 UNVERIFIED。本地 `just ci` 与 `just docs-build` 退出 0。无推送授权，hosted 为 UNVERIFIED。

## Out of scope
产品逻辑、发布部署、升级依赖、自动推送/重跑、分支保护。

## Dependencies
无前置。C2 最终命令说明应采用本项结果。PR/推送须有对应交付授权。
