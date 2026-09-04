# 互斥锁中毒与 secret 清零

## Goal

`Mutex<AppFacade>` 中毒后 IPC 返回 `storage_failure` 而不是 panic。凭据在进程内以 `Secret` 持有，Windows `CredWriteW` 后清零明文缓冲。

## Background

证据：`lib.rs` 大量 `lock().expect("state")`；`credential.rs:108-163` `ProcessLocalStore` 存 `String`；`collector.rs:59-67` 与 `lib.rs:481-490` 拷贝为 `String`；`CredWriteW` 后 blob 未 `fill(0)`。建议在子任务 2 之后改凭据路径。

## Requirements

- R1 IPC 命令对 facade 锁使用 `unwrap_or_else(p.into_inner)` 或映射为 `AppErrorDto` `storage_failure`，禁止 `expect("state")` 再让命令 panic。
- R2 `ProcessLocalStore` 存 `Secret`；Drop 清零。
- R3 Windows adapter `CredWriteW` 后对写入缓冲 `fill(0)`。
- R4 collector / close_connection 尽量缩短 `String` 寿命；能改成借用则借用。

## Acceptance Criteria

- [ ] AC1 测试：poison 后下一条 command 返回错误码而不是 unwind。
- [ ] AC2 `ProcessLocalStore` 源码不再 `Mutex<Option<(String, String)>>` 持有明文 `String` secret。
- [ ] AC3 日志与错误详情仍无 secret。
- [ ] AC4 子任务 2 的 401/超时测试仍绿。

## Out of scope

- 换成 `parking_lot`（若 `into_inner` 足够则不换依赖）。
- 真机 Credential Manager。
- 关闭第二实例 `CloseHandle`（低优先级，可顺手但不作为验收门）。
