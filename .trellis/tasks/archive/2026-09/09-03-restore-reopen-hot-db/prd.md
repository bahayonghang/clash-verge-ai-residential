# restore 失败后重开热库

## Goal

restore 失败且热库文件仍可按 schema 打开时，应用回到 `NormalReady` 并恢复 writer。只有热库打不开才保持 Recovery-only。

## Background

证据：`facade.rs:1641-1669` 先把 `storage = None` 且 `branch = RecoveryOnly`，然后 restore。`backup.rs:113-119` 会把 live 文件滚回。restore `Err` 时 facade 不 `reboot_storage()`，UI 停在 Recovery-only 直到进程重启。规格：失败必须保留当前可用库。

## Requirements

- R1 restore `Err` 后，若 `monitor.sqlite3` 仍能 `StorageCoordinator::open`，调用 `reboot_storage()` 并设 `NormalReady`。
- R2 热库无法打开时保持 `RecoveryOnly`、`storage = None`，错误码仍为现有 restore 失败码。
- R3 成功 restore 后仍 `reboot_storage()`（现行为保留）。

## Acceptance Criteria

- [ ] AC1 无效候选 restore：错误返回，随后 `run_report` / `save_targets` 不是 `recovery_only`，SQLite 仍是 restore 前内容。
- [ ] AC2 热库文件损坏无法打开：保持 `recovery_only`。
- [ ] AC3 成功 restore 测试仍通过。
- [ ] AC4 不把损坏热库复制为备份。

## Out of scope

- 改 backup 分页或 checksum 算法。
- 子任务 4 的 cancel 打断 backup。
