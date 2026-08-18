# 家宽监控桌面外壳与实时监控（C2）

## 目标与用户价值

在 C1 已交付可靠采集、核算、持久化、实时投影与恢复接口的前提下，完成 Windows 11 桌面产品外壳和实时监控体验。用户关闭窗口、重建 WebView 或登录后后台启动时，采集仍持续运行；用户可以安全配置控制器、观察实时流量和健康状态，并对单条当前连接发出关闭请求。

本任务独立交付可验收的桌面与实时能力，不扩展历史报告、导出、保留或告警数据模型。

## 前置条件与边界

- C2 严格依赖 `08-18-monitor-collector-storage`（C1）。只有 C1 的回放核算、单 writer、实时投影、原子 `bootstrap/baseSeq/resync`、存储健康和最小 `RecoveryFacade` 通过独立验收后，C2 才能进入实施。
- C2 只消费 C1 冻结的业务接口和 DTO，不直接读取 SQLite、不解释 mihomo 原始 payload、不复制核算或分类算法。
- C2 不新增或修改历史报告、导出、保留、告警及 notification outbox schema；这些能力分别由后续 C3、C4 交付。
- 当前任务保持 `planning`；本轮未授权执行 `task.py start`。

## 范围内需求

### C2-R1 Windows 桌面生命周期

- 应用保持单实例；第二实例不得启动第二套 collector 或 writer，只聚焦或唤起既有窗口。
- 单实例保护必须先于其他后台服务注册。
- 点击主窗口关闭按钮只隐藏到托盘，首次发生时说明应用仍在采集；明确“退出”才执行完整 shutdown。
- 托盘提供打开窗口、暂停 / 继续采集、立即重连、权威健康摘要和明确退出。
- 登录自启动必须由用户在引导或设置中主动选择；后台启动使用 `--background`，不得主动弹出主窗口。
- 系统休眠 / 恢复、暂停 / 继续、立即重连和退出必须调用 C1 生命周期接口，正确形成 coverage，不把未采集时段写为零。
- WebView 隐藏、销毁或重建不得停止采集、写入或健康状态维护。

### C2-R2 凭据、设置与首次引导

- 实现 C0 冻结的 `CredentialStore` Windows Credential Manager adapter。TCP secret 只在 Credential Manager 与短暂 Rust 内存中出现，永不返回前端，也不进入 URL、日志、SQLite、Channel、错误、诊断或导出。
- v1 不实现 DPAPI 或明文文件 fallback。Credential Manager 不可用时，只允许用户输入当前进程有效的临时 secret，并明确提示退出后失效；进程退出时必须清除。
- 设置保存需要补偿语义：稳定 credential 引用、机器设置和 UI 偏好各自有明确所有者；部分失败不得留下指向不存在凭据的有效配置，也不得误删仍在使用的旧凭据。
- 首次引导覆盖控制器发现与测试、重点目标选择与排序、登录自启动确认、保留与本地隐私说明、通知能力预检。
- TCP 只接受 loopback；named pipe 不发送 secret。手动设置优先，连接状态沿用 C1 的 TCP 鉴权、管道 ACL、busy timeout、端点不存在和协议不兼容分类。

### C2-R3 原子实时 IPC 与前端状态

- 前端只通过版本化 Commands 和一个有序 Channel 消费稳定 DTO。
- 每次订阅的第一条消息必须是原子 `bootstrap { snapshot, baseSeq }`；后续只应用 `seq > baseSeq` 的消息。
- 前端检测到序号缺口、重复或 schema 不兼容时停止应用增量，并通过 `resync` 获取新的原子 bootstrap / watermark；不得继续猜测状态。
- WebView 重建后重新订阅，不复用旧游标。前端 store 只保存视图选择和 DTO cache，不持有第二套账本、分类或统计实现。
- 实时摘要与托盘摘要来自同一份 Rust 权威 health / live projection。

### C2-R4 实时概览、连接表与连接控制

- 概览分别展示 controller meter、attributed observed、重点主分类、其他连接、正向未归因 gap、over-attributed 异常、活跃连接数、最后采样时间、coverage 和健康状态；不得把 meter 与可归因事实混称为同一“全局”口径。
- 活跃连接支持按分类、域名、进程、规则、链路、网络类型和时间筛选，并支持稳定排序、keyset 接口、虚拟化列表和详情查看。
- 连接详情显示 C1 提供的完整规范化元数据、累计字节、当前速率、持续时间、主分类和全部命中标签；缺失字段保持未知，不填伪默认。
- 用户只能关闭单条当前连接。控制器返回 `204` 时显示“已发送关闭请求”，只有后续快照确认连接消失后才显示已关闭；不提供关闭全部连接。
- 连接中、断线、鉴权失败、管道拒绝、协议不兼容、无数据、coverage 缺口、存储故障和迁移失败均有独立中文状态与恢复动作。

### C2-R5 10k 短时实时性能

- 10,000 条活跃连接、1 Hz、至少 30 分钟仅作为短时实时峰值，不代表 10,000 条连接持续 30 天的容量承诺。
- 连接更新使用 keyed delta；Rust 侧在完成 C1 生命周期和 delta 核算后，才允许按连接键 latest-only / coalescing。不得在原始 snapshot 或核算前丢帧。
- Channel 不持续发送全量 10k snapshot；列表使用虚拟化和稳定 keyset 分页，只渲染可见范围。高频更新不得造成内存随运行时间或结果总量无界增长。
- WebView 不存在时可以停止构造 UI payload，但不得停止 C1 collector、writer、coverage 或健康维护；窗口恢复后用新 bootstrap 重建视图。

### C2-R6 数据库无关 Recovery Shell

- 正常业务 schema 不可打开、迁移失败、数据库版本过高、checksum 不符或 integrity 检查失败时，仍能启动最小 Recovery Shell。
- Recovery Shell 只通过 C1 `RecoveryFacade` 读取版本 / 脱敏诊断、列出 migration backup、验证候选和打开数据目录；不得初始化普通 ReportService 或依赖正常查询 schema。
- C2 只交付恢复入口、状态和候选验证界面；实际 restore command 由 C3 接入。

### C2-R7 应用导航与后续能力 seam

- 交付稳定的应用壳和导航注册表：概览、实时连接、分析报告、告警、设置 / 数据管理。C3 / C4 页面尚未实现时使用明确的禁用占位，不伪造数据。
- 冻结 C3 / C4 可扩展的 route / view registration seam，后续任务替换页面内容时不得重写桌面生命周期或实时 store。
- 冻结受限 `FileDialogPort`：只返回用户明确选择的打开 / 保存句柄或路径，前端不获得宽泛文件系统 capability；C2 不读写报告或备份。
- 冻结版本化 `OperationProgress` DTO：`operationId`、phase、current / total、可取消状态、完成 / 失败与脱敏错误。C2 只交付状态呈现与取消入口，C3 提供实际报告 / backup / restore operation。

## 非目标

- 不实现历史报告、导出、retention、用户备份 / restore 执行或告警中心。
- 不实现分析报告 / 告警的业务页面；C2 只交付稳定导航占位、文件选择和 operation progress seam。
- 不新增报告、rollup、retention、alert 或 notification outbox 表。
- 不修改 Clash / mihomo 配置，不提供关闭全部连接，不自动执行控制命令。
- 不支持非 loopback TCP、多控制器、Windows Service、macOS 或 Linux。
- 不把 10k 短时实时峰值外推为 30 天数据库容量或发布支持上限。

## 验收标准

- [ ] **C2-AC1 依赖门**：证据表明 C1 已完成并通过独立 gate；C2 仅经冻结接口消费采集、投影、存储健康和恢复能力，代码审查未发现直接 SQL、原始 payload 解析或报告 / 告警 schema 变更。
- [ ] **C2-AC2 生命周期**：Windows 11 安装态下单实例、托盘、关闭隐藏、明确退出、`--background` 自启动、暂停 / 继续、立即重连和休眠恢复均符合需求；隐藏、销毁、重建 WebView 不影响采集与 durable commit。
- [ ] **C2-AC3 凭据与引导**：Credential Manager 保存、读取、轮换、删除和失败补偿通过；不可用时只有进程内临时 secret，退出后失效；首次引导所有步骤可完成、跳回和重试，secret 扫描结果为零。
- [ ] **C2-AC4 原子订阅**：并发订阅 / 首帧、重复消息、序号缺口、窗口重建和 schema 不兼容测试证明 bootstrap 与 `baseSeq` 原子，gap 会停止增量并 resync，不丢失或错误重放连接状态。
- [ ] **C2-AC5 实时口径**：概览和托盘使用同一 Rust 投影；meter、attributed、分类 / 其他、gap、over-attributed、coverage 与健康状态分开展示，缺口不显示为零。
- [ ] **C2-AC6 连接控制**：随机重排连接数组不影响列表身份；不存在 ID 返回 `204` 时只显示已发送请求，直到后续快照确认消失；没有关闭全部连接入口。
- [ ] **C2-AC7 10k 性能**：10k 活跃、1 Hz、至少 30 分钟短时回放下 keyed delta / coalescing、虚拟化与 keyset 生效；可见筛选 / 排序交互 p95 小于 150 ms，内存有界，无持续队列积压；测试报告明确不代表 30 天容量。
- [ ] **C2-AC8 Recovery Shell**：正常 schema 不可用时 Recovery Shell 仍能打开并完成脱敏诊断、备份列表和候选验证，且没有调用普通报告 / 查询 schema；restore 操作保持不可用并说明由 C3 交付。
- [ ] **C2-AC9 应用壳 seam**：五段导航可稳定注册，未实现页面明确禁用；C3 fixture 可通过冻结 route、`FileDialogPort` 和 `OperationProgress` DTO 完成模拟导出 / 恢复流程，且前端没有宽泛文件权限。
- [ ] **C2-AC10 可用性与安全**：专门状态、键盘操作、可见焦点、高对比和高 DPI 走查通过；生产 CSP、本地资源和最小 capability 生效，前端无法读取 secret、数据库或任意文件。
- [ ] **C2-AC11 独立回滚**：可停用 C2 UI 与 Windows capability 并保留 C1 无头采集内核；回滚不删除数据库、不回退 migration、不破坏 Credential Manager 中仍被有效配置引用的凭据。

## 开放问题

无阻塞性开放问题。最终 adapter 与插件用法以 C0 冻结结果和 C1 实际接口为准；若两者未满足前置 gate，C2 不得启动。
