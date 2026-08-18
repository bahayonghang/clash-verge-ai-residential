# C0：家宽监控基础与风险验证

## 目标

在业务 schema 和采集产品代码出现前，用可复现证据消除 `residential-monitor` 的性能、SQLite、控制器传输、凭据与 Windows 分发阻断风险，并冻结 C1 可直接执行的工程契约。

C0 交付基础骨架、测试夹具、基准工具、项目规范、CI 基线和选型决策，不交付可供最终用户使用的监控产品。

## 背景与已确认约束

- 父任务把 TCP External Controller 定义为受支持路径，把 Clash Verge Rev 私有 named pipe 定义为版本化、尽力兼容路径。
- 数据库性能必须先于业务 schema 冻结。C0 可以使用可丢弃的候选 schema 做实验，但不得把实验 schema 当作 C1 的已发布 migration。
- SQLite 默认耐久策略为 WAL + `synchronous=FULL`。若实测不达标，应调整批量、索引或支持范围；未经重新评审，不得把 `FULL` 降为 `NORMAL`。
- 10,000 活跃连接是 1 Hz、连续 30 分钟的短时峰值，不是 30 天持续容量承诺。
- Windows 11 是 v1 唯一正式支持平台。应用以普通用户、NSIS current-user、登录后托盘常驻形态运行，不使用 Windows Service。
- secret 只用于 TCP；named pipe 不发送 secret。凭据最终由 Windows Credential Manager 保存，v1 不提供 DPAPI 文件回退。
- 本任务状态保持 `planning`。本轮未授权运行 `task.py start`，也未授权实现产品代码。

## 交付范围

### R1. 性能优先的可复现基准

1. 先定义负载模型、生成器、指标格式和通过规则，再选择 C1 的物理 schema。
2. 生成并实际装载 `A=50`、`A=250`、`A=1000` 三档完整 30 天数据库。每档冻结完整 `WorkloadSpec`：`A / L / C / q`、维度基数、每帧变化比例；数据覆盖真实与恶意高基数域名、长进程路径、会话时长分布、多个 chain、稀疏分钟事实和质量事件。
3. 同一生成器还必须产出 13 个月精确高基数 rollup 与长期 core daily 的 schema-neutral fixture、期望总量和可复现装载适配器，供 C3 / C5 使用；该 fixture 不冻结 C3 正式 schema。
4. 运行 10,000 活跃连接、1 Hz、全部连接计数每帧变化、连续至少 30 分钟的短时峰值。不得用空闲连接或短样本线性外推替代该运行。
5. 每档记录机器与软件版本、行数、每表和每索引 B/row、主库与 WAL 峰值、freelist、写放大、批量大小、提交延迟、冷/热查询的 p50/p95/p99/max、CPU、RSS 与队列深度。
6. 比较 prepared statement 复用与候选有界批量。逐行路径必须是 bind → step → reset；不得每行重新 prepare，也不得拼接无界多值 SQL。
7. 使用 `FULL` 完成基准，并把性能不达标归因到 schema、索引、批量或支持范围。非 `FULL` 结果只能作为诊断对照，不能成为默认结论。
8. 输出 `A=250` 发布设计点、`A=50` 开发回归档和 `A=1000` 压力档的 adopt/reject/fallback 结论。最终支持上限和 busy deadline 必须来自测量，不得沿用未验证估算。

### R2. SQLite binding 能力验证

候选 binding 必须用实际捆绑版本和 Windows 构建证明：

- SQLite 版本满足父设计要求的 WAL 修复基线，或有可核验的官方 backport；
- WAL、STRICT、foreign keys、`synchronous=FULL`、并发短读与单 writer 可用；
- interrupt、progress handler、分页 Online Backup、checkpoint 和 statement status 存在可调用的直接能力或等价封装；
- prepared statement 可缓存复用，批量 transaction 可控；
- busy、I/O error、磁盘满、取消和 backup 中断可映射为稳定错误；
- license、维护状态、锁文件和供应链来源可接受。

不满足必选能力的 binding 必须标记 `reject`；只有具备完整恢复路径的替代方案才能标记 `fallback`。

### R3. CredentialStore 与敏感信息边界

- 冻结最小 `CredentialStore` port。port 只暴露保存、读取、轮换、删除和稳定错误，不让 secret 进入序列化配置、日志或前端 DTO。
- 用最小 spike 验证 Windows Credential Manager generic credential 在普通用户下的 CRUD、升级后读取、删除和失败映射。
- 验证日志、错误、数据库、Channel 和测试证据中无 secret。
- Credential Manager 不可用时，只允许当前进程临时 secret；进程退出后必须失效。C0 不实现 DPAPI fallback，也不交付完整产品 adapter。

### R4. HTTP、TCP 与 named pipe 兼容性验证

- 固定脱敏 `/version`、`/connections`、`/proxies` 和单条 `DELETE` fixture。
- 验证统一异步流上的 HTTP/1.1 `Content-Length`、chunked、connection-close framing、大响应体、响应上限、取消和超时。
- 验证 `/connections` WebSocket、1 Hz frame、大 frame、未知字段与数组乱序。
- TCP 覆盖 secret 正确、错误、为空和 loopback 限制。
- named pipe 覆盖 Verge v2.5.2 固定管道、当前动态 sidecar/service 候选、ACL 拒绝、PID 不符、端点不存在和 `ERROR_PIPE_BUSY` 有界可取消重试。
- 每个 profile 标记 `supported | best-effort | incompatible`。私有 pipe 不兼容时，必须证明 TCP fallback 可用，不得通过猜测永久管道名或发送 secret 绕过。

### R5. Windows 桌面与安装能力 spike

- 验证 Tauri 2 + Vanilla TypeScript + Vite 的最小本地资源构建，`withGlobalTauri: false`、显式 CSP 和最小 Windows capability 生效。
- 验证稳定 identifier、product name、binary name、Credential target 与 LocalAppData 路径。
- 使用 NSIS current-user 测试安装态 tray、close-to-hide、single-instance、autostart `--background` 和普通用户通知。
- 冻结一个早期 NSIS 测试安装包、初始 schema fixture 与稳定标识，供 C5 执行首次 v1 升级测试。
- spike 只证明能力和标识稳定性，不实现完整桌面体验。

### R6. 子项目规范与 CI 基线

- 建立 `residential-monitor` 自有 frontend、backend、storage 规范，明确 DTO、错误、取消、UTC、schema、迁移、备份、隐私和测试约束。
- 建立可重复的 Node、Rust、Tauri 和根仓库聚合命令；锁文件必须固定，fixture 必须脱敏。
- Windows CI 必须运行等价检查，并保持根 Required checks 聚合名称稳定。
- secret scan 必须覆盖 Rust、TypeScript、配置、锁文件、fixture 和证据输出。
- 性能工具至少提供快速回归模式和完整手工发布 gate；CI 不得伪装执行耗时 30 天库与 30 分钟峰值的完整 gate。

### R7. 决策输出

每项 spike 都必须留下：

- 固定版本、环境和复现命令；
- 原始证据位置及脱敏说明；
- 通过/失败条件和观察值；
- `adopt | reject | fallback` 结论；
- fallback 的触发条件、能力差异和对 C1 的约束；
- 决策负责人确认状态。

C1 只能消费已批准的决定。任一必选项没有 `adopt` 或可执行 `fallback` 时，C1 保持 `planning`。

## 可观察验收标准

- [ ] AC1：同一台基准机可从空目录通过记录的命令生成固定 `A / L / C / q`、维度基数和变化比例的 `A=50/250/1000` 三档完整 30 天库，并生成 13 个月精确高基数 rollup + 长期 core daily fixture；证据显示实际时间跨度、输入分布和各表行数，不使用线性外推替代。
- [ ] AC2：每档报告列出每表/索引 B/row、DB/WAL 峰值、freelist、写放大、冷/热查询和 p50/p95/p99/max；`A=250` 有明确的发布设计点判定。
- [ ] AC3：10,000 活跃连接、1 Hz、全部连接计数每帧变化、连续至少 30 分钟峰值完成，输出 frame、Accounting、prepared batch、durable commit、CPU、RSS 和队列指标；失败也必须形成可复现的 reject/fallback 证据。
- [ ] AC4：真实 binding 在 `FULL` 下通过 WAL、STRICT、并发读写、prepared statement 复用、busy/I/O error 测试，并实际调用 interrupt、progress、paged backup、checkpoint 和 statement status。
- [ ] AC5：候选 binding 的实际 SQLite 版本、修复来源、license、维护状态和锁定方式可核验；不满足必选项的候选已标记 `reject`。
- [ ] AC6：Credential Manager spike 在普通用户下完成保存、读取、轮换、删除、升级后读取和不可用场景；secret 扫描对日志、错误、SQLite、Channel、fixture 和证据输出为零。
- [ ] AC7：TCP 与 named pipe 兼容矩阵覆盖 R4 的全部场景；pipe busy 可取消且有总时限，ACL/PID/协议错误分类正确，pipe 不发送 secret，TCP fallback 有真机证据。
- [ ] AC8：NSIS current-user 安装态的 tray、single-instance、autostart `--background`、通知、稳定数据路径和 credential target 有普通用户真机证据。
- [ ] AC9：本地构建在显式 CSP 下只加载打包资源；Node/Rust/Tauri 检查和根 `just ci` 可由一个稳定聚合 gate 执行。
- [ ] AC10：frontend/backend/storage spec 与 Windows CI 基线已建立；快速 CI 和耗时手工性能 gate 的边界清楚，secret scan 覆盖新增文件类型。
- [ ] AC11：每个性能、SQLite、Credential、HTTP/pipe、NSIS 和 CI spike 都有 `adopt | reject | fallback` 决策；没有口头结论或未链接证据。
- [ ] AC12：C1 输入清单明确冻结 binding、SQLite 版本、FULL、batch/queue 上限、busy deadline、支持档位、协议 profile、CredentialStore port、稳定标识和 fallback；批准前 C1 未启动。

## 非目标

- 不实现 `ControllerSession`、`AccountingEngine`、持久化 writer、业务查询、实时投影或 `RecoveryFacade`。
- 不创建 C1 的正式 core migration，不冻结未经完整基准验证的业务 schema。
- 不实现报告、导出、保留、恢复、告警、通知中心或最终界面。
- 不修改 Clash / mihomo 配置、ACL 或用户连接，不发送真实 `DELETE` 到用户正在使用的控制器。
- 不支持 macOS、Linux、Windows Service、远程控制器、DPAPI fallback 或自动更新。
- 不宣称监控数据具有代理商账单精度。

## 依赖、风险与停止条件

- 前置依赖：无。研究依据来自父任务的控制器兼容性审计、实施上下文和 SQLite 性能预算。
- 后置依赖：C1。C1 必须等待 C0 的必选决策获批，并在自己的上下文清单中引用最终证据。
- 如果 `FULL`、必需 binding 能力、TCP fallback 或普通用户安装态存在无可执行 fallback 的失败，C0 不得标记可进入 C1。
- 私有 pipe 可以判定 `incompatible`，但必须保留 TCP 受支持路径。
- 本任务当前没有阻塞性产品问题；执行期测得的支持上限和 deadline 属于 C0 决策，不得在规划阶段编造。
