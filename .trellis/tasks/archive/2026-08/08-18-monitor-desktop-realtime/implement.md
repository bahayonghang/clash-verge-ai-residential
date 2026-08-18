# 实施计划：桌面外壳与实时监控（C2）

## 启动前门禁

- [ ] 任务仍为 `planning`；先由用户审阅本任务 PRD、design、implement 和 manifests。
- [ ] 未经用户在规划审阅后的独立消息明确授权，不运行 `task.py start`。
- [ ] C1 已完成、独立验收并冻结 AppFacade、live projection、Channel bootstrap / resync、生命周期、存储健康、连接控制与 `RecoveryFacade` 接口。
- [ ] C0 已冻结 Tauri、Credential Manager、autostart、single-instance 和 NSIS 的 adapter / plugin 选择。
- [ ] 若 C1 或 C0 gate 未通过，停止 C2，不以直接 SQL、私有状态或临时替代实现绕过依赖。

## 执行顺序

### 1. 冻结 C1 → C2 契约

- [ ] 建立 C2 所消费的 C1 interface / DTO 清单和版本矩阵。
- [ ] 为正常 AppFacade 与 `RecoveryFacade` 明确互斥启动结果、稳定错误码和脱敏字段。
- [ ] 确认 `subscribe` 第一条原子 bootstrap、`baseSeq`、后续 `seq`、resync 和 subscription identity 契约。
- [ ] 确认连接身份、列表 sort key / keyset cursor、详情和 close request 的输入输出。
- [ ] 写契约测试，证明 C2 不需要 SQLite schema、原始 mihomo payload 或报告 / 告警表。

**Gate 1**：C1 fixtures 可驱动全部 C2 facade 测试；没有未冻结的跨层字段。

**回滚点**：仅有契约测试与 DTO 适配；删除 C2 adapter 不影响 C1。

### 2. 实现 Windows 桌面生命周期

- [ ] 在后台服务启动前注册 single-instance；第二实例只唤起现有窗口。
- [ ] 实现托盘菜单、close-to-hide、首次说明、打开窗口、暂停 / 继续、立即重连和明确退出。
- [ ] 实现 opt-in autostart、OS 实际状态回读和 `--background` 无窗口启动。
- [ ] 把 sleep / resume 与桌面命令映射到 C1 生命周期接口，不在 C2 生成 coverage。
- [ ] 实现幂等 shutdown coordinator，并覆盖并发退出与超时错误。
- [ ] 验证窗口隐藏、销毁和重建时 collector / writer / coverage 持续。

**Gate 2**：Windows 真机与自动化生命周期矩阵通过；第二实例、窗口和 WebView 均不能创建第二套采集内核。

**回滚点**：可禁用托盘与 autostart，回到显式窗口启动；C1 后台内核和数据库保持不变。

### 3. 实现凭据、设置与首次引导

- [ ] 实现 C0 `CredentialStore` port 的 Windows Credential Manager adapter。
- [ ] 实现 credential target 轮换、读取验证、设置保存和旧凭据清理的补偿流程。
- [ ] 实现 Credential Manager 不可用时的 session-only secret handle，并验证退出 / 替换后清除。
- [ ] 在 Rust 入口校验 loopback TCP、字段长度、目标排序和 command 参数。
- [ ] 实现控制器发现 / probe、重点目标、autostart、保留 / 隐私说明和通知能力预检向导。
- [ ] 对向导前进、返回、取消、重试和部分失败建立状态机测试。
- [ ] 扫描日志、错误、Channel、设置和 fixtures，确认没有 secret。

**Gate 3**：Credential CRUD、轮换、补偿和临时 secret 真机测试通过；任何部分失败都保留一个可用的旧配置或明确的未配置状态。

**回滚点**：停用持久凭据写入，保留临时 secret 与旧稳定引用；只删除确认未引用的临时 credential。

### 4. 实现原子 IPC 与实时状态

- [ ] 定义并生成 / 手写共享的版本化 DTO、TypeScript decoder 和 exhaustive reducer。
- [ ] 实现 bootstrap 首帧、subscription identity、`lastSeq` 校验、重复消息处理和 gap fail closed。
- [ ] 实现可取消 resync；resync 成功前冻结旧增量，失败时显示可恢复状态。
- [ ] WebView 重建时取消旧订阅、丢弃迟到消息并以新 bootstrap 重建 cache。
- [ ] 让托盘与 UI 读取同一 Rust live projection / health。
- [ ] 大列表、详情和后续报告不经 Channel；Channel 维持有界实时消息。

**Gate 4**：确定性竞态测试覆盖订阅与首帧并发、gap、重复、乱序、schema 不兼容、旧订阅迟到消息和连续 resync。

**回滚点**：停用增量订阅并仅保留有界低频 bootstrap 诊断模式；不得回退为高频全量 10k snapshot。

### 5. 实现实时候选投影与界面

- [ ] 实现概览卡片和数据表，分别显示 controller meter、attributed observed、分类 / 其他、gap、over-attributed、coverage 与健康。
- [ ] 在 C1 核算后实现按连接 identity 的 UI delta coalescing，设置键数、消息大小和时间上限。
- [ ] 实现稳定 sort tuple + identity 的 keyset 列表接口，不使用深 OFFSET。
- [ ] 实现虚拟化列表、有限 overscan、筛选 / 排序重置游标和详情按需查询。
- [ ] 实现各类连接 / 存储 / 协议 / coverage 状态与恢复动作。
- [ ] 实现单条关闭请求：`204` 只进入“已发送”，后续 remove 才进入“已关闭”；超时进入“未确认”。
- [ ] 验证所有视图只格式化 DTO，不重新累计、分类或计算 coverage。
- [ ] 实现五段稳定导航与 view registration seam；reports / alerts 尚未交付时显示明确禁用占位。
- [ ] 冻结按用途收敛的 `FileDialogPort`，不向前端开放通用文件系统。
- [ ] 冻结 `OperationProgress` / cancel DTO 和窗口重建后的重订阅；用 C3 fixture 验证 seam，不实现实际报告或备份 operation。

**Gate 5**：随机连接顺序、连接 churn、未知字段和关闭不存在 ID fixtures 通过；UI 与托盘摘要一致；C3 fixture 能只经稳定 route / file / progress seam 完成模拟流程。

**回滚点**：可停用实时连接表和 close command，保留 C1 采集及最小健康概览。

### 6. 交付数据库无关 Recovery Shell

- [ ] 在正常 schema 初始化前确定 `NormalReady | RecoveryOnly` 分支。
- [ ] 实现 Recovery Shell 的独立入口、最小 DTO 和脱敏状态。
- [ ] 接入 C1 `RecoveryFacade` 的版本 / 诊断、migration backup 列表、候选验证和打开数据目录。
- [ ] 保留 C3 restore command 的 capability 位；C2 不实现恢复写操作。
- [ ] 注入 future schema、checksum mismatch、migration / integrity failure，断言不创建普通 read pool、不调用 ReportService 或业务表。

**Gate 6**：所有正常数据库不可用 fixtures 均能打开 Recovery Shell；恢复按钮明确不可用或标记待 C3 接入。

**回滚点**：候选验证可 feature-disable，仅保留版本、诊断与备份列表；不得尝试启动普通 UI。

### 7. 性能、安全与可用性收口

- [ ] 使用 10k 活跃、1 Hz、至少 30 分钟短时回放，记录 Channel 消息量、coalescing 键数、UI 交互 p50 / p95 / p99 / max、CPU、RSS 和队列水位。
- [ ] 验证筛选 / 排序可见交互 p95 小于 150 ms，虚拟化只渲染可见范围，内存不随运行时长无界增长。
- [ ] 在回放中重复隐藏、销毁和重建 WebView，确认采集 / durable commit SLO 不退化且新窗口能原子 resync。
- [ ] 报告中明确 10k 是短时实时峰值，不是 30 天容量或存储支持范围。
- [ ] 检查生产 CSP、最小 capability、本地资源、command 参数校验与日志白名单。
- [ ] 完成键盘、可见焦点、高对比、高 DPI、空 / 错误 / 恢复状态走查。
- [ ] 完成 Windows 安装态 tray、single-instance、autostart、Credential Manager 和 shutdown smoke。

**Gate 7**：C2-AC1 至 C2-AC11 均有自动化或真机证据，且根项目行为未变化。

## 计划验证命令

实施完成后，以 C0 冻结的实际脚本为准，至少运行：

```text
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

另需保存 Windows 11 安装态 smoke、10k 短时峰值和 Recovery Shell 故障矩阵证据。

## 独立验收

- [ ] 逐项映射 C2 PRD 的十一项验收标准，不借用父任务最终集成结论代替。
- [ ] 审查 C2 diff，确认没有 C3 报告 / retention 或 C4 告警 schema。
- [ ] 审查所有 DB 使用点，确认只经过 C1 interface。
- [ ] 记录 C1 依赖版本、Windows / WebView2 / adapter 版本和未解决限制。
- [ ] 用户审阅独立验收证据后，C2 才可归档并允许 C3 进入启动审查。

## 整体回滚方案

1. 停用 C2 UI、close command 和 Windows capability，保留 C1 无头采集。
2. 不删除数据库、不回退 migration、不修改 C1 durable watermark。
3. 恢复旧设置引用，只清理确认未引用的临时 credential；Credential Manager 不可用时清除进程 secret。
4. 若 Recovery Shell 有问题，降级为只读版本 / 诊断 / 备份列表，不尝试打开正常 schema。
5. 回滚后重新验证 C1 collector、writer、coverage 与 shutdown，确认 C2 故障未污染采集账本。
