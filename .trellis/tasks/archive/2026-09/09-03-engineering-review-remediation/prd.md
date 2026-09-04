# 工程级 Review 整改

## Goal

把 2026-09-03 全库工程级 Review 的必须修复项与建议修改项，拆成可独立完成、可独立验收的子任务。母任务只拥有问题分类、任务地图与最终对照；不改产品代码。

## Background

仓库含两块第一方产品：Clash Verge 全局扩展脚本，以及本机 ResiWatch（Tauri + SQLite）。Review 覆盖正确性、安全、性能、可读性、测试与架构。证据在 `research/audit-findings.md`。已归档 `09-03-test-coverage-gaps` 的缺测不重复。

## Requirements

- R1 子任务按严重程度优先、同等严重程度下成本低的先做。
- R2 每个子任务有可观察验收：错误码、锁持有时间、回滚后库状态、分页行为或 CI 失败条件。禁止只改注释。
- R3 子任务之间不是隐式依赖。若 B 必须等 A，写在 B 的 `prd.md`。
- R4 本母任务不启动产品实现。实现只在用户批准某个子任务的规划摘要后，对该子任务 `task.py start`。
- R5 不提交 `*.local.toml` / `*.local.js`。公开模板凭据仍为占位。

## 任务地图（严重程度 → 成本）

| 序 | 子任务 | 严重 | 成本 | 独立 |
| --- | --- | --- | --- | --- |
| 1 | `09-03-sqlite-writer-txn-fail-closed` | high | S | 是 |
| 2 | `09-03-controller-http-auth-timeout` | high | S | 是 |
| 3 | `09-03-restore-reopen-hot-db` | high | S | 是 |
| 4 | `09-03-c3-report-unlock-cancel` | high | M | 是 |
| 5 | `09-03-script-fail-closed-atomic` | high | M | 是 |
| 6 | `09-03-dimension-rank-backend-sort` | high | S | 是 |
| 7 | `09-03-live-pagination-channel-cache` | high | M | 是 |
| 8 | `09-03-frontend-dto-decode-settings` | high | M | 是 |
| 9 | `09-03-public-safety-docs-ci` | medium | S | 是 |
| 10 | `09-03-storage-tick-and-alert-txn` | medium | M | 等 1（同 `storage.rs`） |
| 11 | `09-03-fail-closed-logging-redact` | medium | S | 建议在 4 之后（同 facade 错误路径） |
| 12 | `09-03-mutex-poison-secret-zeroize` | medium | S | 建议在 2 之后（同凭据/collector） |

## Acceptance Criteria

- [ ] AC1 上表 12 个子任务目录存在，且 `task.py list` 显示为母任务的 children。
- [ ] AC2 每个子任务 `prd.md` 含 Goal、In/Out of scope、可观察 AC、证据 `file:line`。
- [ ] AC3 复杂子任务（4、5、7、8、10）另有 `design.md` 与 `implement.md`。
- [ ] AC4 母任务不包含产品代码 diff。对照完成时，high 项要么已在子任务验收，要么明确延期并写原因。

## Out of scope

- 拆分 `AppFacade` / `lib.rs` 上帝对象。
- 合并 Rust `i18n.rs` 与前端 `i18n/zh.ts` 为单一表。
- 根脚本拆模块（Clash Verge 要求单文件粘贴）。
- `npm audit` / `cargo audit`（需网络）。
- 30 天库、24 小时 soak、NSIS 真机、Credential Manager 真机。
- 已归档 `09-03-test-coverage-gaps` 的缺测清单。
