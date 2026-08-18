# C1 剩余 gate 状态

| Gate | 结果 |
|---|---|
| ControllerSession TCP secret 三态 | `session::controller_session_tcp_secret_states` 通过 |
| named pipe 不发送 secret，错误码可区分 | `controller_pipe_does_not_send_secret` 通过 |
| 核心身份变化发出 Restarted | `controller_reconnect_emits_restart_on_identity_change` 通过 |
| checksum mismatch fail closed | `storage_migration_rejects_checksum_mismatch` 通过 |
| 隔离子进程 kill 前 / 不确定 / commit 后 | `tests/kill_gate.rs` 3 通过 |
| verify-design-db A=250 | 行数与期望一致，C1 重开 watermark=1 |
| replay --profile c1 30m | 1800 帧；p50 19ms / p95 21ms / p99 43ms / max 184ms；1800 commit；零未解释丢帧 |
| just monitor-check | 退出码 0；lib 57 通过 + kill_gate 3 通过 |

C2 未启动。
