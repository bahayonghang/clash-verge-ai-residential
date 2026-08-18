# C0 技术设计：性能与基础能力验证

## 设计目标

C0 用独立、可丢弃的实验面回答「C1 能否安全开始，以及应按什么边界开始」。所有输出必须是可复现证据和批准后的契约，不是演示性质的 happy path。

## C0 拥有的技术边界

```text
WorkloadSpec
  ├─► FixtureGenerator ──► disposable candidate database
  ├─► ReplayDriver ──────► 10k / 1Hz / 30m input
  └─► MetricsCollector ──► EvidenceBundle ──► DecisionRecord

BindingProbe ────────────► SQLite capability decision
CredentialProbe ─────────► CredentialStore port decision
TransportProbe ──────────► TCP / pipe compatibility profiles
WindowsProbe ────────────► NSIS / identity / OS capability decision
ProjectBaseline ─────────► specs / commands / CI contract
```

C0 拥有：

- 最小 Tauri/Vite/Rust 骨架和构建链；
- 性能数据生成器、回放驱动、指标采集与证据格式；
- 可丢弃的候选 schema 和 SQLite binding 比较；
- Credential、HTTP/TCP/pipe、NSIS 的最小隔离 spike；
- 子项目规范、质量命令与 CI 聚合；
- C1 输入决策清单。

C0 不拥有：

- 业务级 `ControllerSession` 和 `AccountingEngine`；
- 正式 core schema、migration ID 或长期兼容承诺；
- 生产 writer、报告、告警、桌面 UI 或恢复流程；
- 用户数据库和任何发布数据迁移。

## 性能先于 schema 的流程

### 1. 固定工作负载，而非固定表结构

`WorkloadSpec` 是基准的稳定输入，至少包含：

- `A`、持续时间、采样频率、会话时长分布和非零分钟比例；
- 域名、进程路径、规则和 chain 的基数与长度分布；
- 首帧、消失、重启、计数回退、缺口和跨分钟质量事件；
- 固定随机种子和生成器版本。

候选 schema 只是实验参数。所有候选必须消费同一 `WorkloadSpec`，避免通过更容易的数据分布获得虚假优势。

### 2. 完整装载三档 30 天库

`A=50/250/1000` 都实际生成 43,200 分钟范围内的 session、chain 和稀疏 minute 事实。每档固定 `A / L / C / q`、维度基数和每帧变化比例。生成器还输出 13 个月精确高基数 rollup 与长期 core daily 的 schema-neutral fixture、期望总量和可复现装载 adapter，供 C3 / C5 在自己的正式 schema 上使用；C0 不借此冻结 C3 schema。基准报告拒绝把小库测量线性外推成大库结果。

每次装载后：

1. 关闭并重开数据库，排除仅在 page cache 中成立的结果。
2. 记录主库、WAL、SHM、freelist 和表/索引页。
3. 用 `dbstat`、`sqlite3_analyzer` 或 binding 等价能力计算每表/索引 B/row。
4. 对固定 Query Corpus 运行冷/热测量和 statement status。

### 3. 运行独立峰值回放

峰值回放保持 10,000 活跃连接、1 Hz、全部连接计数每帧变化、至少 30 分钟。回放同时记录：

- frame 接收至核算批次完成；
- prepared batch 的排队、执行和 `FULL` commit；
- 队列深度、CPU、RSS、DB/WAL 增长；
- deadline、错误、暂停和丢帧计数。

该实验验证瞬时吞吐和有界资源，不扩大 30 天容量承诺。

### 4. 冻结支持范围

`DecisionRecord` 对每个档位写明：

```text
subject
environment
inputs
observations
threshold_source
decision: adopt | reject | fallback
fallback_trigger
constraints_for_c1
evidence_paths
approved_by
```

若父任务初始预算不成立，C0 只能基于原始证据提出新预算并请求批准，不能静默放宽 gate。

## SQLite binding spike

### 候选适配层

`SqliteProbePort` 只为比较暴露 C1 必需能力：

```text
openBundled()
prepareCached()
interrupt()
installProgressHandler()
backupStep()
checkpoint()
statementStatus()
mapError()
```

该 port 不是产品 Repository，也不隐藏 SQLite 语义。比较结果必须能追溯到实际 SQLite C API 能力。

### 必选矩阵

- 实际 bundled `sqlite_version()` 及 WAL 修复来源；
- WAL、STRICT、foreign keys、`FULL`；
- 单 writer 与短 read transaction 并发；
- interrupt 和 progress handler 的取消延迟；
- Online Backup 分页、进度、取消与繁忙重试；
- PASSIVE/FULL/RESTART/TRUNCATE checkpoint 返回值；
- FULLSCAN_STEP、SORT、VM_STEP 或可核验等价指标；
- prepared statement 缓存、bind/step/reset；
- busy、disk full、I/O、corruption 和取消错误映射；
- license、维护状态和 Windows 构建可重复性。

binding 缺少任一必选能力时不做本地脆弱补丁伪装支持。优先比较下一个候选；只有已验证的降级流程才能作为 fallback。

## CredentialStore seam

冻结的 port 不携带 UI 或数据库概念：

```text
put(target, secret)
get(target) -> SecretHandle
replace(target, secret)
delete(target)
```

- `SecretHandle` 只在 Rust 受控作用域内提供请求头所需字节。
- 稳定配置只保存 `target` 和是否已配置，不保存 secret。
- spike adapter 与未来产品 adapter 分离；C0 证明 Win32 generic credential 行为，C2 才交付完整 Windows adapter。
- 临时 secret fallback 只存在于进程内，不能序列化。

## 控制器传输 spike

### 统一流边界

HTTP/WebSocket 客户端作用于统一异步字节流：

```text
TcpLoopbackStream | VerifiedNamedPipeStream
             │
             ▼
HTTP framing + WebSocket + limits + cancellation
```

测试不把 named pipe 当作永久官方协议。pipe 候选先验证 server PID，再用不含敏感信息的 `/version` 探测。ACL、busy、not found、PID mismatch 和协议错误保持独立状态。

### 兼容 profile

每个 profile 固定：

- Verge/mihomo 版本或源码快照；
- 端点发现方法；
- server 身份验证；
- 鉴权行为；
- framing 与 WebSocket 能力；
- 错误分类；
- `supported | best-effort | incompatible`；
- TCP fallback。

私有布局变化只更新 profile，不改变 TCP 的受支持地位。

## Windows 与分发 spike

最小安装态应用只验证 OS seam：

- current-user NSIS；
- 稳定 identifier、binary、AUMID/通知身份和 credential target；
- LocalAppData 与日志目录；
- tray、close-to-hide、single-instance；
- opt-in autostart `--background`；
- 普通用户通知；
- 显式 CSP、本地资源和最小 capability。

冻结的测试安装包和 schema fixture 只用于未来升级链路，不代表 v1 候选版本。

## 规范与 CI

C0 新建 `residential-monitor` 专属规范入口：

- frontend：DTO 解码、视图状态、可访问性和测试；
- backend：模块边界、错误、异步任务、取消、日志和 secret；
- storage：UTC、schema、migration、prepared statement、WAL/FULL、backup、性能和 fault injection。

质量命令分两层：

- 快速 gate：格式、静态检查、单元/集成测试、最小构建和 secret scan，可进入常规 CI。
- 完整证据 gate：三档 30 天库、30 分钟峰值、真机 Credential/pipe/NSIS；由固定 Windows 基准环境手工或受控 workflow 执行。

CI 只报告实际执行内容，不用缩短样本冒充完整 gate。

## C1 消费契约

C0 结束时输出一份批准清单，至少冻结：

- SQLite binding、实际版本与 fallback；
- WAL/FULL、checkpoint 和 backup 能力；
- prepared batch 上限、writer 周期候选和 busy deadline；
- 固定 `A / L / C / q`、维度基数、每帧变化比例的 `A=50/250/1000` 判定，`A=250` 设计预算和“全部连接计数变化”的 10k 峰值结论；
- 13 个月精确高基数 rollup + 长期 core daily 生成器契约；
- DB/WAL/临时空间预算及每表/索引成本；
- CredentialStore port；
- TCP/pipe profile、错误分类与 frame/body limits；
- 稳定安装标识和路径；
- 子项目规范和 CI 命令。

C1 不得覆盖这些值。新证据需要变更时，必须回到 C0 决策评审或建立显式后续决策。

## 安全与证据处理

- fixture、命令输出和报告使用合成或脱敏数据。
- secret、真实住宅代理信息、真实进程路径和本地数据库不进入仓库。
- 决策报告记录工具版本和散列，原始大体积结果可存受控 artifact；仓库内只保存可审阅摘要和校验信息。
- spike 不修改用户 Clash/mihomo 配置，不扩大 pipe ACL，不自动发送控制命令。

## 回滚与失败边界

- C0 没有用户数据库；候选 schema、基准库和 spike 产物均可删除重建。
- 某候选失败时，只回滚该候选适配层和锁文件，不删除其他候选证据。
- CI 基线失败时可以移除新增聚合入口，根扩展运行时必须保持不变。
- 安装 spike 回滚只卸载测试应用；稳定 identifier 一旦作为升级基线冻结，不复用给不同产品。
- 任一必选能力无 adopt/fallback 时停止在 C0，不启动 C1。
