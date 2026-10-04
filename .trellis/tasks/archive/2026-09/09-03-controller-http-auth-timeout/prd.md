# 控制器 HTTP 超时与鉴权

## Goal

采集循环在控制器挂起或超大 `/connections` 体时能结束该次取帧；错误 secret 显示鉴权失败而不是协议不兼容；探测未成功时不把 secret 写入持久凭据库。

## Background

证据：`transport.rs:137-175` 无超时、无 body 上限；`session.rs:107-109` 对非 2xx（含 401）返回 `ProtocolIncompatible`，而 `connect_tcp` 在 `session.rs:78-80` 已把 401 映射为 `AuthFailed`；`facade.rs:1027` `probe_ok = true`；`lib.rs:579` `test_controller` 在 HTTP 前调用 `save_controller`。

## Requirements

- R1 `fetch_path_method` 对 connect / handshake / 响应体施加超时；超限映射为现有 `EndpointMissing` 或等价可重试状态，不得卡死 collector 循环。
- R2 响应体超过上限视为 `ProtocolIncompatible`，不把整段读进 `String`。
- R3 `fetch_normalized_snapshot` 对 HTTP 401 返回 `AuthFailed`，与 `connect_tcp` 一致。
- R4 `has_secret` 且 `resolve` 失败时 `plan_tick` 不得发无 Authorization 的 GET；记 `AuthFailed` 并打 class 日志。
- R5 `save_secret(..., probe_ok: true)` 只允许在探测成功之后。`save_settings` / `test_controller` 失败探测不得把 pending 提升为稳定凭据目标。

## Acceptance Criteria

- [ ] AC1 单元或 fixture：401 的 GET `/connections` → `AuthFailed`，不是 `ProtocolIncompatible`。
- [ ] AC2 超时或超大体：取帧返回错误状态，后续 tick 仍可调度。
- [ ] AC3 `probe_ok = false` 路径删除 pending，Windows/Fake store 无稳定 target。
- [ ] AC4 `test_controller` 在 HTTP 失败后，凭据库稳定 target 与探测前一致（失败探测不落盘）。
- [ ] AC5 日志与 IPC 不含 secret 原文。

## Out of scope

- `Secret` 堆拷贝清零（子任务 12）。
- named pipe 超时矩阵。
- 真机 Credential Manager。
