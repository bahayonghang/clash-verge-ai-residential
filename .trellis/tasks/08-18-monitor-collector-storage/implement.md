# C1 实施计划：采集核算与 core SQLite

## 启动前置条件

- 当前任务保持 `planning`；本轮不得运行 `task.py start`。
- C0 必须已完成、验收并由用户批准其最终决策。
- 启动 C1 前，把 C0 最终性能、binding、limits、协议和 CredentialStore 决策证据加入 C1 manifests，再请求用户审阅。
- 若 C0 没有可执行 adopt/fallback，停止 C1，不在实现中自行换 binding、降低 `FULL` 或放宽 SLO。
- 所有 kill、WAL、busy 和磁盘故障测试只针对合成临时数据库。

## 基础验证命令

每个步骤运行相关局部测试；最终至少执行：

```powershell
cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check
cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
just monitor-check
just ci
npm run check:secrets
```

性能和 fault gate 复用 C0 的 release harness：

```powershell
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- replay --active 10000 --hz 1 --duration 30m --profile c1
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- verify-design-db --average-active 250 --days 30 --profile c1
```

命令名在实现时可以按 C0 已冻结入口调整，但不得缩短 30 分钟峰值或用小库替代完整 30 天设计库。

## 有序实施步骤

### 1. 导入 C0 决策并锁定 C1 边界

操作：

1. 读取 C0 决策记录，生成 C1 使用的 binding、durability、batch/queue、deadline、capacity、transport 和 CredentialStore 配置。
2. 断言运行配置只能使用获批值；缺项时 fail closed。
3. 固定 C1 schema table allowlist 和 C3/C4 禁止表清单。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml c0_contract
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml schema_allowlist
```

证据：

- C0 决策散列与 C1 配置映射；
- 缺失/未批准决定会阻止启动；
- core 表 allowlist 不含报告、rollup、retention、alert 或 notification。

回滚：

- 映射不完整时只撤销 C1 配置，不修改 C0 决策。停止后续步骤。

### 2. 建立版本化控制器模型与 normalizer

操作：

1. 定义原始 payload decoder、版本化 `ControllerInput` 和稳定错误 taxonomy。
2. 支持 unknown 字段、nullable 字段、数组乱序、字段长度和 frame/body limits。
3. 建立脱敏 `/version`、`/connections`、`/proxies` 和 DELETE fixture，以及 replay adapter。
4. 证明 secret 类型不可序列化和不可记录。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml controller_model
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml controller_normalizer
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml controller_redaction
npm run check:secrets
```

证据：

- golden fixture 解码；
- unknown/missing/oversize/乱序输入结果；
- 错误码和脱敏快照；
- secret scan 零命中。

回滚：

- decoder 变更只影响版本化 boundary；删除未采用字段，不让下游直接读取 raw JSON。

### 3. 实现 `ControllerSession`

操作：

1. 实现 TCP loopback、CredentialStore fake resolver、HTTP/WebSocket、取消、deadline 和退避。
2. 实现 C0 批准的 named pipe profile、server PID 验证、busy 有界重试和无 secret probe。
3. 实现手动配置优先、核心变化后重新发现和 capability/version normalization。
4. 将 protocol/transport 错误映射为稳定 session 状态。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml controller_session -- --nocapture
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml controller_tcp
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml controller_pipe
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml controller_reconnect
```

证据：

- TCP secret 正确/错误/为空；
- pipe ACL、busy、not found、PID mismatch、取消和协议错误；
- 大 frame、chunked 和 WebSocket；
- pipe 请求无 Authorization，TCP 只接受 loopback。

回滚：

- pipe adapter 可 feature-disable 并回退 TCP；不得扩大 ACL、缓存永久管道名或允许远程明文 TCP。

### 4. 以 replay 驱动实现 `AccountingEngine`

操作：

1. 先写首帧、乱序、消失、回退、重启、ID 重用和断线测试。
2. 实现 epoch、baseline、非负 delta、session 生命周期和稀疏 minute accumulator。
3. 实现单调时间速率、UTC 分桶、跨分钟比例分配和 coverage。
4. 实现 controller meter、attributed、gap/over-attributed 和 target 优先级分类。
5. 增加属性测试与守恒 oracle。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml accounting_replay
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml accounting_properties
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml accounting_coverage
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml accounting_policy
```

证据：

- 每个生命周期 fixture 的 AccountingBatch；
- 分类、方向和跨桶守恒；
- reset 首帧未知、gap 不写零；
- `chains` 正序/逆序结果一致。

回滚：

- 状态机无 I/O，可回退单个 transition；fixture 作为不可丢失回归保留。

### 5. 实现 core schema、migration 与 backup primitive

操作：

1. 按 C0 页成本结论定义 C1 core 表、索引、STRICT/FK 和 UTC 字段。
2. 实现 `user_version`、migration history/checksum 和 future schema fail closed。
3. 在 collector 启动前执行 migration。
4. 实现分页 Online Backup 至 `.partial`、校验、原子 rename 和 backup manifest。
5. 建立 schema allowlist 测试，禁止 C3/C4 表。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml storage_schema
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml storage_migration
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml migration_backup
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml schema_allowlist
```

证据：

- 空库、重复启动、上一版本、中断、checksum mismatch 和 future schema fixture；
- backup 分页/取消/校验记录；
- 实际 `sqlite_version()`、journal/synchronous/FK 状态；
- schema 清单只有 core 表。

回滚：

- 候选发布前可删除临时开发库并重建；一旦 migration 进入候选 Release，只追加新 migration，不修改旧文件。

### 6. 实现 prepared single writer 与幂等 commit

操作：

1. 缓存固定 prepared statement set，加入 prepare/bind/step/reset 可观测计数。
2. 定义连续 `bundle_epoch / bundle_seq`、payload hash、有界 `committed_bundle` 和 `CommitReceipt`。
3. 在一个 `FULL` transaction 内原子写 bundle、facts、coverage、data version 和 durable watermark。
4. 冻结至少覆盖最近 24 小时与 100,000 receipts 中较大集合的 retry window；小批量 prune 只处理已被 highest contiguous sequence 覆盖且无进程引用的旧 receipt。
5. 窗口内相同 bundle 返回原 receipt；同 ID 不同 hash fail closed；窗口外 duplicate 返回 `RetryWindowExpired`，绝不重执行。
6. 只在 commit 成功后返回 receipt。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml storage_prepared
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml storage_bundle_idempotency
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml storage_watermark
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml storage_bundle_retention
```

证据：

- prepared statement 只在预期阶段 prepare；
- per-key merge 每 transaction 最多一行；
- 重复 bundle 的表行数、字节、data version 和 receipt 不变；
- 不同 hash 复用 ID 被拒绝；
- 持续超过窗口后 ledger 行数达到平台，recent duplicate 可重放 receipt，过期 duplicate 被拒绝且不重新累计。

回滚：

- writer 尚未发布时可回退实现并重建临时库；不得通过删除 committed bundle ledger 规避幂等测试。

### 7. 加入有界队列、backpressure、WAL 与 shutdown

操作：

1. 应用 C0 queue/batch/deadline limits。
2. 只在 Accounting 后合并相同 minute key；禁止原始 frame latest-only。
3. 实现 storage_backpressure、storage_failure 和 gap 开闭。
4. 实现 PASSIVE checkpoint、长读 interrupt/expiry、WAL 回落检测。
5. 实现停接帧 → flush → 结束 coverage → checkpoint/close 的正常 shutdown。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml storage_backpressure
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml storage_checkpoint
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml storage_wal_fallback
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml storage_shutdown
```

证据：

- 队列最大深度和内存上限；
- saturation/busy/disk-full 的 health 与 coverage interval；
- checkpoint starvation 后 WAL 可继续收敛且未被删除；
- WAL 回落时停止 healthy durable 声明；
- shutdown 后最后 receipt 和 coverage 边界可重开验证。

回滚：

- backpressure 策略失败时停止采集并保留显式 gap；不得改成无界队列或静默 drop。

### 8. 实现 durable projection 与原子 Channel

操作：

1. 只在 `CommitReceipt` 后应用 projection delta。
2. 用单一锁/原子临界区拥有 snapshot、seq、watermark 和订阅者登记。
3. 保证首条 bootstrap 携带原子 `baseSeq`，后续 seq 严格更大。
4. 实现 seq gap 检测所需的 resync bootstrap。
5. 限制 Channel 队列，只合并可重建 projection 通知。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml live_projection
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml channel_atomic_subscribe
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml channel_resync
```

证据：

- 并发 subscribe/update 调度测试；
- 第一条恒为 bootstrap；
- 后续 `seq > baseSeq`；
- 注入 gap 后 resync 不遗漏 durable snapshot，也不重放旧 delta。

回滚：

- 可暂时停用 Channel 发布而保留采集/writer；不得让投影绕过 receipt 读取未提交 batch。

### 9. 实现 `RecoveryFacade` backend

操作：

1. 将正常启动失败导向独立 RecoveryFacade。
2. 实现版本/诊断、migration backup 列表和候选 checksum/integrity/schema 验证。
3. 证明该路径不初始化普通 writer、collector、查询、报告或告警模块。
4. 保持 restore/swap 命令不可用。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml recovery_facade
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml recovery_future_schema
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml recovery_bad_backup
```

证据：

- future/checksum/migration failure 下的最小 DTO；
- backup 清单和候选验证；
- 脱敏诊断；
- 普通 schema service 未启动的断言。

回滚：

- RecoveryFacade 失败时保持 fail closed 并输出最小启动错误；不得尝试覆盖当前数据库。

### 10. 执行三类 kill gate

操作：

1. 在隔离子进程中注入 commit 前 kill。
2. 注入 commit 结果不确定时 kill。
3. 注入 commit 已成功但 receipt 未送达时 kill。
4. 每次重启后重放相同 bundle，检查 ledger、facts、watermark、receipt 和 coverage。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml crash_before_commit -- --ignored --nocapture
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml crash_commit_unknown -- --ignored --nocapture
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml crash_after_commit_before_receipt -- --ignored --nocapture
```

证据：

- commit 前无伪 receipt，重启 gap 明确；
- 不确定/未回执场景均只有一个 committed bundle；
- bytes/data version/watermark 只推进一次；
- 已返回 receipt 的事实重启后完整。

回滚：

- 任一 kill gate 失败，停止性能放行并回到步骤 6；保留失败数据库和操作日志的脱敏校验信息。

### 11. 执行性能与存储 fault gate

操作：

1. 运行 10k/1Hz/30m、全部连接计数每帧变化的峰值。
2. 在 C0 固定完整 workload tuple 的 `A=250` 完整 30 天设计库上运行写入、重开和 core query smoke。
3. 注入 checkpoint starvation、WAL 回落、busy deadline、磁盘满和 I/O error。
4. 核对 C0 SLO、DB/WAL 预算、队列、CPU、RSS、bundle 和 coverage。

验证命令：

```powershell
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- replay --active 10000 --hz 1 --duration 30m --profile c1
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- verify-design-db --average-active 250 --days 30 --profile c1
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml storage_fault_gate -- --ignored --nocapture
```

证据：

- 10k 峰值 p50/p95/p99/max、CPU、RSS、queue、DB/WAL；
- A=250 完整设计库的行数、B/row、重开和查询 smoke；
- starvation/WAL/busy/disk-full 的 health、gap 和恢复；
- 零重复 bundle、零重复字节、零已确认漏账、零静默 gap。

回滚：

- 不达标时回到 C0 批准的 fallback 或重新评审 C0；不得缩短测试、删除 WAL、降低 FULL 或隐瞒 gap。

### 12. 全量质量与边界审查

操作：

1. 对照 PRD AC1–AC15 和 design 检查全部代码与证据。
2. 检查 schema 中没有 C3/C4 表，前端/报告/告警功能没有进入 C1。
3. 运行全量 Rust、TypeScript、Tauri、根 CI 和 secret scan。
4. 记录 C2 可消费的接口、schema version、health、Channel 和 RecoveryFacade backend。

验证命令：

```powershell
cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check
cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
just monitor-check
just ci
npm run check:secrets
```

证据：

- AC 对照表和命令日志；
- schema allowlist；
- kill/performance/fault 报告；
- C2 handoff 契约。

回滚：

- 全量 gate 未通过时不归档 C1、不启动 C2。回到拥有缺陷的最早步骤修复并重跑受影响 gate。

## 最终检查

- [ ] C0 决策已批准并实际注入，未自行改写。
- [ ] ControllerSession 与 AccountingEngine 各有唯一边界和 replay 证据。
- [ ] core schema 只含采集、幂等、coverage、migration backup 所需表。
- [ ] prepared、bundle_id、durable watermark、retry window / 有界 ledger 和三类 kill gate 全部通过。
- [ ] 有界队列、backpressure、checkpoint、WAL、busy 和磁盘满均产生可观察结果。
- [ ] 全部连接计数变化的 10k 峰值与固定 workload tuple 的 A=250 完整 30 天设计库均实际运行。
- [ ] 原子 bootstrap/baseSeq、seq gap/resync 和 RecoveryFacade backend 通过。
- [ ] 零重复 bundle、零重复字节、零已确认漏账、零静默 gap。
- [ ] C2、C3、C4 范围未提前实现。
