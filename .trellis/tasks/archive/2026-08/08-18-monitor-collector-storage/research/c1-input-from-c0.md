# C1 输入：已批准的 C0 决策

用户于 2026-08-18 批准 C0 并授权启动 C1。C1 不得改写这些值。

| 项 | 冻结值 | 证据 |
|---|---|---|
| SQLite binding | rusqlite 0.40 bundled | `08-18-monitor-foundation-spike/research/evidence/sqlite-binding.json` |
| sqlite_version | 3.53.2 | 同上 |
| journal / synchronous | WAL / FULL | 同上 |
| busy_timeout | 5000 ms | `c0_contract::BUSY_TIMEOUT_MS` |
| writer batch | 1000 ms 或 10000 行 | `WRITER_BATCH_MS` / `PREPARED_BATCH_ROWS` |
| queue | 最多 8 个待写 batch | `QUEUE_MAX_BATCHES` |
| frame/body | 8 MiB；字符串 4096 | `FRAME_BODY_LIMIT` / `STRING_LIMIT` |
| retry window | 100000 receipt 或 24h 较大者 | `RETRY_WINDOW_*` |
| 设计档 | A=250，L=5，C=3，q=1 | `generate-a250-d30.json` |
| 回归档 | A=50 | `generate-a50-d30.json` |
| 压力档 | A=1000；3904s；4.35 GB；行数与期望一致 | `generate-a1000-d30.json` |
| 峰值 | 10000 / 1Hz / 30m；1800 帧；p95 37ms；max 3321ms；零未解释丢帧 | `peak-10k-30m.json` |
| CredentialStore | C1 用 Fake；Windows CM 真机 CRUD 已通过 | `credential-windows.json` |
| TCP | supported，loopback + Bearer | `controller-profiles.json` |
| named pipe | best-effort，不发送 secret，TCP fallback | 同上 |
| identifier | io.github.bahayonghang.residential-monitor | identity.rs |

C1 缺项 fail closed。不得把 `FULL` 降为 `NORMAL`，不得自行换 binding。
