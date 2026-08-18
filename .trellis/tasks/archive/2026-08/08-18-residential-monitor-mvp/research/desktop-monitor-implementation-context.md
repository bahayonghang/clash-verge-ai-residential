# Windows 11 监控应用：实施上下文摘要

> 本文件用于 Trellis 子代理上下文注入。完整一手资料、权衡与来源见 `desktop-monitor-architecture.md`；实现遇到细节争议时以完整研究和 `design.md` 为准。

## 已采用的技术决策

1. 前端采用 Vanilla TypeScript + Vite，不引入 React / Vue / Svelte。
2. `withGlobalTauri: false`；生产只加载打包内本地资源，显式 CSP 与最小 Windows capability。
3. Rust 后端拥有 collector、accounting、SQLite、reports、retention、backup、alerts、tray、autostart、notification 和 credential。
4. 前端通过版本化 Commands 与一个有序 Channel 使用业务 DTO；不接触 mihomo 原始 JSON、SQL、secret 或任意文件权限。
5. SQLite 取代 JSON 账本，位于 `app_local_data_dir`；使用 bundled 已修复版本、WAL、单 writer、短 read transaction、STRICT、前向 migration 和 Online Backup。
6. TCP secret 存 Windows Credential Manager；SQLite / JSON 只存 credential target 与 `hasSecret`。v1 不做 DPAPI fallback；Credential Manager 不可用时只允许当前进程临时 secret。
7. 登录后托盘常驻，不使用 Windows Service；关闭窗口只隐藏，明确退出经过 shutdown coordinator。
8. autostart 在 onboarding 由用户确认，使用 `--background`；single-instance 防止第二个 collector / writer。
9. Windows 告警先落 SQLite，再 best-effort 发系统通知；应用内告警中心是权威记录。
10. NSIS current-user 是正式安装渠道；GitHub Releases 手动升级，v1 不注册 updater plugin。
11. 数据库、备份和机器设置放 LocalAppData；偏好可放 config dir；日志放 app log dir 并轮转脱敏。
12. 根项目的“零 npm / 无构建”规范只适用于可粘贴 Clash 扩展，不适用于独立桌面子应用。

## 必须保持的模块所有权

```text
ControllerSession
  -> AccountingEngine
  -> AlertEngine
  -> StorageCoordinator
  -> ReportService
  -> AppFacade
  -> TypeScript UI
```

- ControllerSession 隐藏 discovery、TCP / pipe、HTTP / WebSocket、鉴权、重连和版本兼容。
- AccountingEngine 是纯状态机，统一处理 epoch、baseline、delta、UTC bucket、coverage 和 policy。
- AlertEngine 生成状态变化；facts、coverage、alerts 与 notification outbox 作为一个 CommitBundle 由同一 SQLite transaction 提交，成功后才发送通知。
- StorageCoordinator 是唯一 SQLite writer / migration / backup / restore / retention owner，不做泛化 Repository。
- CommitBundle 使用连续 `(writer_epoch, bundle_seq)`；recent receipt ledger 只保留冻结的 retry window，epoch summary / durable watermark 长期保留，窗口外重复输入拒绝执行，避免每秒一行无限增长。
- ReportService 是 UI、CSV、JSON、HTML 的唯一统计投影。
- AlertEngine 是冷却、静默、恢复和去重的唯一状态机。
- AppFacade 校验所有前端参数并返回稳定 DTO / 错误码。

## Tauri IPC 约束

- Commands：设置、probe、分页查询、报告、导出、备份恢复、关闭连接、通知测试等 request / response。
- Channel：`snapshot | connectionDelta | healthChanged | alertChanged | dataVersionChanged`。
- `seq` 为进程级全局单调序号；订阅首条消息原子携带 snapshot + base seq，后续只接受更大 seq。
- seq gap 必须 resync 新 snapshot / watermark，不能继续盲目应用 delta。
- 实时摘要约 1Hz；连接使用 delta 或有界 snapshot；Rust 侧 coalescing / latest-only。
- WebView 重建后重新订阅并先取 bootstrap；collector 生命周期不依赖订阅者。
- 禁止高频 `emit` 全量 snapshot，禁止前端 SQL，禁止前端轮询整个数据库。

## SQLite 约束

启动时验证实际 `sqlite_version()`。WAL 多连接使用 SQLite 3.51.3+ 或带官方修复的 backport。

每个连接显式设置：

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = FULL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;
```

- 单 writer 按约 1 秒或有界行数批量 transaction；FULL 保护 facts / coverage / alerts / outbox 的断电一致性。
- 查询只读、短事务、分页；不能长期持有 read transaction。
- 不允许 live DB 位于 OneDrive、SMB 或 network share。
- 监控 DB / WAL 大小、最后 commit / checkpoint / backup、retention watermark 和 I/O error。
- migration 使用 `user_version` + migration history / checksum，启动 collector 前在 `BEGIN IMMEDIATE` 中前向执行。
- future schema 或 checksum mismatch 必须 fail closed。
- migration 前 Online Backup；不能复制 hot `.sqlite3`，不能删除或遗漏有效 `-wal`。

## 时间与事实模型

- 持久时间统一 UTC integer epoch；本地时区只在查询边界应用。
- 速率使用单调时钟。
- 不保存每秒完整 frame；保存 session metadata、chains 和分钟级 delta。
- 首次观察累计值只做 baseline，不计入历史。
- 采样间隔跨分钟时按单调时间比例分配并保持总字节守恒。
- 显式 coverage：running、disconnected、unauthorized、pipe denied、protocol incompatible、sleep / clock gap、paused、storage failure、app exit。
- 缺口绝不能填 0。
- `attributed_observed` 来自全部连接 delta，满足分类守恒；mihomo controller totals 单独作为核对 meter，不强行分配尾差。
- 同 frame / epoch / direction：`gap=max(0, controller-attributed)`，`over=max(0, attributed-controller)`；reset 首帧未知，跨分钟用同一比例分桶，负差不抵扣历史。

## 分层保留

- session / chain / minute raw：30 天；
- hourly：13 个月；
- daily：长期；
- alert history：180 天；
- raw coverage：13 个月，daily coverage：长期；
- migration / backup manifest：长期。

小时 / 日 rollup 只保存“历史主分类 + 单一分析维度”，分别支持域名、进程、规则、链路和网络类型 Top N；不保存所有维度笛卡尔积。只有 30 天 raw 支持任意跨维过滤和会话下钻。

清理顺序：

1. 按 UTC cutoff 和 watermark 选批次。
2. 同一 transaction 幂等生成上层汇总并核对 bytes / coverage。
3. 成功后推进 watermark。
4. 最后删除已覆盖下层事实。
5. 中断可续跑；manual / scheduled clean 共用服务。

## Windows 生命周期与安全

- 稳定 identifier、productName、binary name、publisher、安装模式和 credential target 发布后不得修改。
- NSIS current-user，不要求 app 以管理员运行；通知只在真实安装态验收。
- 关闭 X：prevent close + hide；托盘包含打开、暂停 / 继续、重连、状态、明确退出。
- 明确退出：停接帧 → flush writer → 结束 coverage → checkpoint / close DB → 删除托盘 → exit。
- secret 不返回前端，不进入 URL、日志、错误、SQLite、Channel、诊断或导出。
- TCP v1 只接受 loopback；非本机控制器直接拒绝。named pipe 不发送 secret。
- capability 只匹配主窗口和 Windows；不授予宽泛 fs、opener、SQL 或 remote URL。
- 所有 command 参数在 Rust 校验；SQL 参数化。
- Windows Credential Manager 只防磁盘明文，不防同用户恶意进程，威胁模型必须说明。

## 发布前验证门

1. 显式 CSP 下 dev / production 均可加载，无 remote request 或 inline exception。
2. 10k 活跃、1Hz、全部连接计数每帧变化的短时峰值下 Channel 有序、内存有界；隐藏 / 重建窗口不影响 collector。
3. writer、query、checkpoint、Online Backup 并发压力通过。
4. migration 覆盖 C0 冻结的首次 v1 schema baseline（后续版本为上一正式 schema）、重复启动、中断、future schema、checksum mismatch。
5. retention 证明 raw → hourly → daily 守恒且缺口不变零。
6. backup / restore 覆盖坏 checksum、integrity failure、旧 schema、残留 WAL / SHM。
7. Credential Manager 覆盖保存、轮换、删除、读取失败和升级；secret 扫描为零。
8. NSIS 安装态覆盖 tray、autostart、single instance、普通权限通知和手动升级。
9. 首次 v1 从 C0 冻结安装包升级（后续版本从上一正式 Release），identifier、AUMID、settings、DB、credential 和 autostart 保持。
10. 正式资产包含 canonical installer、checksum 和诚实的签名 / SmartScreen 说明。

## 报告与告警一致性

- `run_report` 返回短期不可变 report snapshot token；UI 与 CSV / JSON / HTML 导出消费同一 snapshot。
- 速率告警使用 60 秒滚动平均并连续 3 次满足才触发。
- 周期用量窗口只允许滚动 1 小时、用户本地自然日或自然月，并复用 ReportService / rollup。
- 告警证据保存 data version、规则版本、窗口与过滤条件。
- notification outbox 在启动和运行中扫描；原子 lease、stale lease 回收、attempt、退避与 sent / failed 状态保证崩溃后可恢复。
