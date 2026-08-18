# C0 实施计划：基础与风险验证

## 启动前置条件

- 本文件只描述后续执行。当前任务保持 `planning`，本轮不得运行 `task.py start`。
- 用户必须先审阅 C0 的 PRD、design、implement 和 manifests，并在后续消息中明确授权启动 C0。
- 执行前重新读取父任务研究和 C0 manifests；不得使用真实 secret、真实住宅代理信息或用户数据库。
- 每个 spike 先写通过条件，再运行实验。失败结果也必须保留，不得只记录成功候选。

## 计划中的稳定命令面

实现阶段应建立以下命令；最终以锁定后的脚本为准，但语义不得缩水：

```powershell
npm --prefix residential-monitor ci
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check
cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace
npm --prefix residential-monitor run tauri:build
just monitor-check
just ci
npm run check:secrets
```

性能工具建立统一入口，完整 gate 使用 release 构建：

```powershell
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- generate --average-active 50 --days 30 --profile full
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- generate --average-active 250 --days 30 --profile full
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- generate --average-active 1000 --days 30 --profile full
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- replay --active 10000 --hz 1 --duration 30m --profile peak
```

如果实现阶段调整命令名，必须同步更新规范、CI、帮助文本和证据；不得保留两个语义不同的入口。

## 有序实施步骤

### 1. 固定实验协议与证据格式

操作：

1. 定义 `WorkloadSpec`、固定随机种子、`A / L / C / q`、维度基数、每帧变化比例、三档 30 天数据分布、13 个月高基数 + 长期 core daily 输出、10k 峰值输入和 Query Corpus。
2. 定义机器信息、工具版本、原始指标、判定、脱敏状态和 artifact 校验信息的统一输出格式。
3. 为每类 spike 建立 `adopt | reject | fallback` 决策模板。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml workload_spec
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml evidence
```

证据：

- 相同 seed 的清单散列和各表预期行数一致；
- 输出包含 OS、CPU、RAM、Rust、Node、Tauri、binding 和 SQLite 版本；
- 决策模板能链接原始结果并标记批准状态。

回滚：

- 在任何数据库候选落地前修改协议；协议一旦用于比较候选，变更版本号并重跑全部候选，不混用旧结果。

### 2. 建立最小子项目与快速质量 gate

操作：

1. 建立 Tauri 2、Vanilla TypeScript、Vite 和 Rust 最小骨架。
2. 固定 lockfile、本地资源、显式 CSP、`withGlobalTauri: false` 和最小 Windows capability。
3. 建立 `typecheck/lint/test/build/tauri:build`、`just monitor-check` 和根 `just ci` 聚合。
4. 扩展 secret scan 覆盖新增文本、配置、fixture 和证据摘要。

验证命令：

```powershell
npm --prefix residential-monitor ci
npm --prefix residential-monitor run build
npm --prefix residential-monitor run tauri:build
just monitor-check
just ci
npm run check:secrets
```

证据：

- 开发和 production 构建只加载本地资源；
- CI 与本地命令一一对应；
- 根扩展原有检查仍通过。

回滚：

- 骨架和 CI 入口可整体撤销；不得修改根扩展运行契约或用跳过项让 gate 变绿。

### 3. 先实现性能生成器，再冻结 schema

操作：

1. 实现与物理 schema 解耦的工作负载生成器和候选 schema adapter。
2. 实际生成 `A=50/250/1000` 完整 30 天数据库。
3. 生成 13 个月精确高基数 rollup + 长期 core daily 的 schema-neutral fixture、期望总量和可复现装载 adapter。
4. 在关闭重开数据库后采集表/索引 B/row、DB/WAL、freelist、写放大、冷/热查询和 statement status。
5. 记录生成时间、完整 workload tuple、输入分布、行数守恒和磁盘临时空间。

验证命令：

```powershell
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- generate --average-active 50 --days 30 --profile full
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- generate --average-active 250 --days 30 --profile full
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- generate --average-active 1000 --days 30 --profile full
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- analyze --all-generated
```

证据：

- 三份完整数据库及输入 manifest；
- 每表/索引 B/row、DB/WAL 峰值、freelist 和 Query Corpus 指标；
- `A=250` 设计点与 `A=1000` 压力档的明确判定。

回滚：

- 候选 schema 均为可丢弃实验库；删除失败候选并保留报告。不得把候选 migration 复制为 C1 正式 migration。

### 4. 验证 prepared batch、FULL 与 10k 峰值

操作：

1. 比较 prepared statement 缓存和多个有界 batch 档位。
2. 使用 WAL + `synchronous=FULL` 运行所有正式指标。
3. 运行 10,000 活跃连接、1 Hz、全部连接计数每帧变化、至少 30 分钟峰值。
4. 记录 frame、核算、排队、commit、CPU、RSS、队列、DB/WAL 和错误。

验证命令：

```powershell
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- compare-batches --synchronous full
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- replay --active 10000 --hz 1 --duration 30m --profile peak --synchronous full
```

证据：

- bind → step → reset 的复用计数；
- p50/p95/p99/max 与资源曲线；
- 队列峰值和是否出现未解释丢帧；
- batch 上限、writer 周期候选和支持范围的决策。

回滚：

- 不通过时先减少索引、调整有界 batch 或降低声明的支持档位；不得以 `NORMAL` 作为静默回滚。

### 5. 比较并冻结 SQLite binding

操作：

1. 对每个候选运行同一 capability suite。
2. 验证实际 bundled 版本、WAL 修复、license 和锁定来源。
3. 实际调用 interrupt、progress、paged backup、checkpoint 和 statement status。
4. 注入 busy、I/O error、disk full、取消和并发读写。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml sqlite_probe -- --nocapture
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml sqlite_fault -- --nocapture
cargo run --release --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- binding-capabilities
```

证据：

- 候选能力矩阵和实际 API 调用记录；
- SQLite 版本及修复来源；
- 错误分类、取消延迟和 backup/checkpoint 进度；
- 最终 adopt/reject/fallback 与 C1 约束。

回滚：

- 候选失败时移除其 adapter 和依赖；保留 lockfile diff 与失败证据。没有合格候选时停止 C0。

### 6. 冻结 CredentialStore port

操作：

1. 定义不泄露 secret 的 port 和 fake adapter。
2. 用普通用户真机验证 Credential Manager generic credential CRUD、轮换、升级后读取和删除。
3. 验证不可用时仅保留进程内临时 secret，退出后失效。
4. 对日志、错误、SQLite、Channel、fixture 和证据执行 secret 扫描。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml credential_port
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml credential_windows -- --ignored --nocapture
npm run check:secrets
```

证据：

- 普通用户测试矩阵；
- 升级前后同一 target 的读取结果；
- 不可用与删除后的稳定错误；
- secret 扫描零命中。

回滚：

- 清除测试 credential；保留 port 时可移除 Windows spike adapter。不得添加 DPAPI 文件 fallback。

### 7. 验证 HTTP、TCP 与 named pipe

操作：

1. 建立脱敏协议 fixture server 和录制 payload。
2. 验证 framing、大 `/proxies`、WebSocket、frame/body limits、取消和超时。
3. 验证 TCP secret 三态与 loopback。
4. 在真机和注入环境覆盖固定/动态 pipe、ACL denied、PID mismatch、not found 和 busy deadline。
5. 输出兼容 profile 和 TCP fallback。

验证命令：

```powershell
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml transport_fixture -- --nocapture
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml named_pipe_faults -- --nocapture
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml controller_profiles -- --ignored --nocapture
```

证据：

- 每种 framing 和 WebSocket 的通过记录；
- `supported | best-effort | incompatible` profile；
- ACL/PID/busy/not-found/协议错误映射；
- pipe 请求不含 Authorization，TCP fallback 可复现。

回滚：

- 私有 pipe 不兼容时保留 profile 并关闭对应自动发现分支；TCP 路径继续作为受支持 fallback。

### 8. 验证 NSIS 与 Windows 系统能力

操作：

1. 固定 identifier、product/binary name、credential target 和数据路径。
2. 构建 current-user NSIS。
3. 在普通用户安装态验证 tray、close-to-hide、single-instance、autostart `--background` 和通知。
4. 冻结早期测试安装包、schema fixture 与校验和。

验证命令：

```powershell
npm --prefix residential-monitor run tauri:build
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml windows_identity
```

证据：

- 安装、启动、后台启动、第二实例、通知和卸载记录；
- LocalAppData 与 credential target 在升级测试前后稳定；
- 基线安装包和 schema fixture 的校验和。

回滚：

- 卸载测试应用并清理合成测试数据/credential；已冻结标识不得用于另一个产品。

### 9. 建立 residential-monitor specs 与 CI

操作：

1. 把已批准的 DTO、backend、storage、security、test 和性能约束写入子项目 specs。
2. 让 Windows CI 执行快速 gate，保留完整性能/真机 gate 的受控入口。
3. 验证 manifests 可向后续实现和检查代理注入正确研究与 guides。

验证命令：

```powershell
just monitor-check
just ci
npm run check:secrets
```

证据：

- spec 索引的 pre-development 和 quality checklist；
- CI job 与本地命令映射；
- 完整 gate 未被短样本替代的说明。

回滚：

- CI job 可从聚合中移除，但不得删除已形成的失败证据或篡改根 Required checks。

### 10. 汇总决策并执行 C1 放行审查

操作：

1. 汇总性能、binding、Credential、transport、Windows 和 CI 的决策记录。
2. 核对所有必选项都有 `adopt` 或可执行 `fallback`。
3. 生成 C1 输入清单，并由用户审阅。
4. 只有 C0 验收通过并归档后，才允许更新 C1 manifests 指向最终决策证据并请求启动 C1。

验证命令：

```powershell
just monitor-check
just ci
npm run check:secrets
```

证据：

- C0 AC1–AC12 对照表；
- 决策记录无缺项、无未批准必选项；
- C1 输入清单包含 binding、预算、deadline、limits、ports、profiles 和 fallback。

回滚：

- 任一 gate 失败即撤销 C1 放行，不删除已完成实验；回到对应步骤补证据或选择新候选。

## 最终检查

- [ ] 固定 workload tuple 的三档 30 天库、13 个月高基数 + 长期 core daily 生成器，以及全部连接计数变化的 10k/30m 峰值均实际运行。
- [ ] B/row、DB/WAL、FULL、prepared batch 和 binding 五类关键证据齐全。
- [ ] Credential、HTTP/pipe、NSIS、spec 与 CI 均有独立结论。
- [ ] 所有 fixture 和证据已脱敏，secret scan 为零。
- [ ] C0 只交付基础与决策，没有实现 C1 业务产品。
- [ ] C1 尚未启动，且放行条件明确可检查。
