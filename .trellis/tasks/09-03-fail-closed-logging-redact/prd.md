# fail-closed 日志与脱敏

## Goal

自动档案与报告失败在本机日志留下稳定 `class`，不含 secret。`map_report` 使用当前 `ui_locale`。export 禁止子串与 `redact::FORBIDDEN` 对齐。

## Background

证据：`lib.rs:276-341` archive 失败 `let _ =`；`facade.rs:1783-1797` `map_report` 无日志且 locale 固定 Zh；`export.rs` reject 列表缺 `authorization:` / `credential`。建议在子任务 4 之后改同一错误路径。

## Requirements

- R1 `archive_tick` 的 purge / next_job / persist_outcome 失败 `app_log::emit` Error + class。
- R2 `map_report` 记 class；`message_zh`/`action` 走 `self.ui_locale`。
- R3 export `reject_secret` 使用与 `FORBIDDEN` 相同的子串集。
- R4 日志字段仍走 `sanitize_fields`，不得写入 secret 原文。

## Acceptance Criteria

- [ ] AC1 注入档案 persist 失败：日志文件含 event 名与 class，不含 fixture secret。
- [ ] AC2 `ui_locale=en` 时报告 `invalid_query` 的 IPC `messageZh` 为英文文案。
- [ ] AC3 含 `Authorization:` 的导出结果被拒绝。
- [ ] AC4 `persist_settings` / `save_targets` 失败至少一条 Error 日志。

## Out of scope

- 把嵌套 JSON 日志从丢弃改为递归扫描（保持现策略：非标量变 `null`）。
- 第三方 log 桥。
