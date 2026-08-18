# 实施计划：residential-monitor 完整 Windows 11 v1

## 执行策略

当前任务作为父任务保存完整产品需求、总体设计、子任务映射与最终集成验收，不直接承载一次性“大爆炸”实现。用户批准本规划后再创建并逐个启动子任务；每个子任务都需独立 PRD / design / implement、质量检查和回滚点。

```text
C0 基础与风险验证
  └─► C1 采集、核算与 SQLite
        └─► C2 桌面外壳与实时监控
              └─► C3 历史报告、导出与数据管理
                    └─► C4 告警与诊断
                          └─► C5 发布硬化与集成
```

建议子任务：

| 顺序 | slug | 交付物 | 依赖 |
|---|---|---|---|
| C0 | `monitor-foundation-spike` | Tauri / Vite 骨架、依赖选型、协议与系统能力 spike、子项目 specs / CI 基线 | 无 |
| C1 | `monitor-collector-storage` | ControllerSession、AccountingEngine、SQLite schema / writer / migration、回放测试 | C0 |
| C2 | `monitor-desktop-realtime` | 托盘、自启动、单实例、设置向导、实时 UI、连接关闭 | C1 |
| C3 | `monitor-reporting-data` | 历史查询、报告、导出、分层保留、备份恢复 | C2 |
| C4 | `monitor-alerting-diagnostics` | 告警规则、Windows 通知、告警中心、诊断与健康 | C3 |
| C5 | `monitor-release-hardening` | 视觉 / 无障碍、安装升级、CI、性能 / soak、最终集成与文档 | C4 |

## C0：基础与风险验证

### 目标

在大量业务代码出现前消除协议、存储、安全和 Windows 集成的阻断风险，并建立本子项目自己的规范。

### 工作项

1. 建立 `residential-monitor/`：
   - Vanilla TypeScript + Vite；
   - Tauri 2 Rust 后端；
   - `withGlobalTauri: false`、显式 CSP 与 Windows-only capability；
   - 稳定 identifier / product name / binary name；
   - npm lockfile 与 Cargo lockfile。
2. 建立 `.trellis/spec/residential-monitor/`：
   - frontend：DTO、状态、可访问性、视觉与测试；
   - backend：模块接口、错误、异步任务、取消与日志；
   - storage：schema、UTC、迁移、备份、保留和查询。
3. 选型并用最小 spike 验证：
   - Rust SQLite binding：bundled SQLite 版本满足修复要求，支持 WAL、STRICT、Backup API；
   - Windows Credential Manager binding：generic credential CRUD、错误映射、升级语义；
   - HTTP over TCP / named pipe 的成熟组合：chunked、大 `/proxies`、WebSocket、取消和 frame limits。
   - 冻结 `CredentialStore` port；C1 使用 fake resolver，Windows Credential Manager adapter 留给 C2，v1 不实现 DPAPI fallback。
4. 控制器兼容矩阵：
   - TCP secret 正确 / 错误 / 为空；
   - Verge v2.5.2 固定 pipe；
   - 当前动态 sidecar / service pipe；
   - ACL 拒绝、server PID 不符、`ERROR_PIPE_BUSY` 超时；
   - `/connections`、`/proxies`、单条 DELETE 的协议 fixture。
5. Windows 系统能力 spike：
   - NSIS current-user 安装；
   - tray、close-to-hide、single-instance；
   - autostart `--background`；
   - 安装态普通用户通知；
   - Credential target 和 LocalAppData 路径稳定。
6. 工程基线：
   - package scripts 固定 `typecheck` / `lint` / `test` / `build` / `tauri:dev` / `tauri:build`；
   - `just monitor-dev`、`monitor-check`、`monitor-build`；
   - GitHub CI 增加 Windows Rust / Node 子项目检查，不改变根 Required checks 名称；
   - secret scan 覆盖 `.rs`、`.ts`、lock / config 中可能泄露的字段；
   - fixture 和录制 payload 全部脱敏。
7. 冻结性能 harness：
   - Windows 11、4 核 x64、8GB RAM、当前 WebView2 基准机；
   - `A=50 / 250 / 1000` 且固定 `L / C / q`、维度基数、每帧变化比例的完整 30 天 raw、13 个月高基数 rollup 与长期 core daily 数据生成器；
   - 10k 活跃、1Hz、全部连接计数每帧变化、至少 30 分钟的短时峰值回放；
   - 真实 / 恶意高基数域名、长进程路径、会话时长与 chain 分布；
   - CPU、RSS、队列、commit、query、UI latency、DB / WAL / freelist、B/row 与 24 小时 soak 采集脚本；
   - binding 必须暴露 interrupt、progress、paged backup、checkpoint 和 statement status 等价能力。
8. 生成首版升级基线：
   - 冻结一个早期 NSIS 测试安装包、初始 schema fixture 和 credential / autostart 标识；
   - 后续 C5 用它模拟“上一版本 → v1”；v1 发布后改用上一正式 Release。

### Gate

- 每个 spike 都有脱敏 fixture / 命令、真机证据、`adopt | reject | fallback` 结论和研究路径；仅有口头结论不通过。
- SQLite：实际 bundled version、WAL、FULL、STRICT、Online Backup、busy / I/O error 和 concurrent read / write 证据通过；不满足则重新选型。
- Credential：generic credential 保存 / 读取 / 轮换 / 删除、普通用户、升级后读取与日志脱敏通过。v1 不做 DPAPI fallback；Credential Manager 不可用时仅允许当前进程临时 secret，并验证退出后清除 / 失效。
- HTTP / pipe：chunked、大 body、WebSocket、取消、frame limit、ACL denied、busy timeout 与 PID mismatch fixture 通过。
- 控制器 profile 矩阵逐项标明 `supported | best-effort | incompatible`。私有 pipe 不兼容不阻塞 C1，但必须证明 TCP 回退和 UI 状态可完成 PRD AC1。
- NSIS 安装态 tray、single-instance、autostart `--background`、普通权限通知和稳定数据 / credential 路径通过。
- `npm --prefix residential-monitor run build` 与 `npm --prefix residential-monitor run tauri:build` 在显式 CSP 下均能加载本地 Vite 产物。
- 性能 harness 可重复运行并输出机器信息、原始指标和门限判定。
- `dbstat` / `sqlite3_analyzer` 或等价测量产出每表 / 索引 B/row；不以 cache / mmap 魔法参数掩盖 schema 问题。
- 所有 spike 结论写入 research / 子任务设计；任一必选 adapter 没有 adopt / fallback 结论时不得开始 C1。

### 回滚点 C0

仅有骨架、spec、CI 与 spike；可删除子目录和 CI 入口，不涉及用户数据库。

## C1：采集、核算与 SQLite

### 工作项

1. 实现版本化 controller 原始模型与 normalizer，未知字段容忍、字段上限和脱敏错误。
2. 实现 `ControllerSession`：
   - 手动配置、自动发现、probe、TCP / pipe adapter；
   - TCP 只接受 loopback；凭据通过 C0 冻结的 `CredentialStore` port / fake resolver 注入；
   - 成熟 HTTP + WebSocket；
   - 取消、超时、退避、重新发现、capability / version；
   - 稳定状态码和 replay adapter。
3. 实现 `AccountingEngine` 纯状态机：
   - epoch、首帧 baseline、连接生命周期；
   - 非负 delta、计数器回退、ID 重用；
   - 单调速率、UTC 分桶、跨分钟比例分配；
   - controller meter、attributed observed、未归因差额；
   - coverage interval；
   - 有序 target set、唯一主分类、全部标签和策略版本。
4. 实现 SQLite schema 与 migrations：
   - 只创建 core migration / data version、bundle epoch / recent receipt、machine / controller epoch / policy / coverage / session / chain / minute 与 backup manifest；
   - STRICT、FK、索引和版本 metadata；
   - newer-than-app、checksum mismatch fail closed；
   - 在首个 schema 起提供 migration 前 Online Backup 原语；C3 再增加用户备份 / 恢复与 retention。
5. 实现 single writer 和只读 query 基础：
   - cached prepared statements、1 秒 / 有界批量 transaction、同 minute key 合并；
   - 连续 `(writer_epoch, bundle_seq)`、有界 recent receipt retry window、durable watermark 与不确定 commit 重试；窗口外 duplicate 拒绝执行；
   - WAL、FULL、按 deadline 冻结的 busy timeout、PASSIVE checkpoint；
   - 稀疏 `connection_minute`，零流量连接不每分钟写零；
   - 有界队列、storage backpressure、degraded coverage 与恢复；
   - storage health、磁盘满和 I/O failure；
   - 正常 shutdown flush。
6. 实现基础 live projection 和原子 Channel 水位 contract，不做完整 UI：
   - `seq` 为进程级全局单调序号；
   - `subscribe_monitor` 第一条消息原子携带 snapshot + base seq；
   - seq gap 通过 `resync_monitor` 取得新 snapshot / watermark。
7. 实现最小 `RecoveryFacade` 后端：在正常 schema 不可用时仍可读取版本 / 诊断、列出 migration backup 并验证候选；实际恢复命令由 C3 交付。

### 自动化验证

- replay fixtures：首帧、乱序、消失、负增量、重启、ID 重用、断线、休眠、跨小时 / DST、target 变化。
- 属性测试 / 守恒：每批 `分类 + 其他 = attributed observed`；分桶总和等于输入 delta；缺口不产生零记录。
- 差额 fixture：同 frame / epoch / direction 计算正向 gap 与 over-attributed；reset 首帧未知；跨分钟分配守恒且负差不抵扣历史。
- migration fixtures：空库、重复启动、上一 schema、迁移中断、checksum mismatch、future schema。
- 并发压力：writer + readers + checkpoint + migration backup；验证实际 SQLite 版本。
- crash / kill / Windows restart 后 reopen、integrity 和 FULL durability。
- kill 点覆盖 commit 前、commit 结果不确定、commit 后未回执；重启后零重复 bundle、零已确认漏账。
- prepared statement 复用、transaction 内 per-key merge、queue saturation / storage gap、checkpoint starvation 和 WAL 回落有自动 gate。
- `subscribe` / first snapshot 并发、seq gap、窗口重建和 resync 无丢失 / 重放错误。
- 运行 `just monitor-check`；证据写入 C1 子任务的 check 记录。

### 回滚点 C1

尚未发布时可重建开发数据库；一旦生成候选 Release，schema 只能前向迁移，变更必须新增 migration。

## C2：桌面外壳与实时监控

### 工作项

1. Rust app lifecycle：
   - tray 菜单、关闭隐藏、明确退出；
   - single-instance；
   - autostart opt-in 与 `--background`；
   - sleep / resume、暂停 / 继续、立即重连；
   - shutdown coordinator。
2. Credential Manager 与设置补偿事务：
   - secret 不返回前端；
   - 实现 C0 `CredentialStore` 的 Credential Manager adapter；不可用时只支持当前进程临时 secret，不持久化 DPAPI 文件；
   - machine settings / UI preferences 分离；
   - 只允许 loopback TCP；controller probe、目标列表、目标排序与策略版本。
3. TypeScript IPC：
   - DTO / decoder / discriminated Channel messages；
   - 处理首条原子 bootstrap / base seq、seq gap resync 与窗口重建重新订阅；
   - 仅视图状态 store，不复制业务统计。
4. 首次引导：
   - 控制器发现与测试；
   - 重点目标；
   - 登录自启动；
   - 保留与隐私说明；
   - 通知能力预检。
5. 实时页面：
   - 概览分别呈现 controller meter、attributed observed、分类 / 其他、覆盖与未归因差额；
   - 连接分页 / 虚拟化、完整详情、筛选排序；
   - 单条关闭请求及后续确认；
   - 各类 health / empty / failure 状态。
6. 托盘状态与 UI 状态共用 Rust 权威 health。
7. 交付不依赖正常业务 schema 的 Recovery Shell：迁移 / integrity 失败时仍能显示脱敏诊断、备份列表和候选验证；C3 接入实际 restore command。

### 验证

- 10k 模拟连接，Channel 顺序、coalescing、内存和 UI 响应。
- 隐藏 / 销毁 / 重建窗口不影响 collector；第二实例只聚焦。
- autostart 状态读取失败、设置失败和后台启动。
- 关闭不存在 ID 返回 204，不误报成功。
- 正常数据库不可用时 Recovery Shell 可启动，且不调用普通 ReportService / query schema。
- 键盘、焦点、屏幕缩放、高对比与图表数据表走查。
- 运行 `just monitor-check` 与安装态 smoke；证据写入 C2 子任务。

### 回滚点 C2

可保留 C1 无头采集内核，临时停用 UI / OS capability；不回滚数据库 schema。

## C3：历史报告、导出与数据管理

### 工作项

1. 实现统一 `ReportQuery` / `ReportResult`：
   - UTC range、本地时区、粒度；
   - 分类 / 标签、域名、进程、规则、链路、网络类型 filters；
   - totals、series、Top N、连接数、时长、period comparison、coverage；
   - raw 可下钻 / 历史策略边界；
   - keyset 分页、排序、参数校验、data version 与短期不可变 report snapshot token；
   - 命名 SQL corpus、可取消连接、deadline、EQP / statement-status 回归。
2. 历史与报告 UI：
   - 小时、日期、7 / 30 日、月、自定义区间；
   - 趋势、占比、Top 域名 / 进程 / 规则 / 链路；
   - 缺口与策略版本明确展示；
   - 连接会话下钻。
3. ExportService：
   - CSV、JSON、可打印 HTML；
   - 字段选择和脱敏预览；
   - 流式写、临时文件、原子完成；
   - 消费 UI 当前 report snapshot token，metadata 与应用内查询一致。
   - token 绑定物化结果 / 有界 spool，不持有长期 SQLite read transaction；TTL 与 quota 经基准冻结。
4. RetentionService：
   - 新增 C3 前向 migrations：primary category + 单一 dimension 的 hourly / daily、daily coverage、retention state；
   - raw → hourly dimension → daily dimension；
   - 30 天 raw 支持任意组合；精确高基数 hourly / daily 最多 13 个月；长期 daily 只保留总量 / 历史主分类 / coverage；
   - watermark、幂等续跑、守恒核对；
   - 自动与手动清理共用服务；
   - 占用估算和可取消维护；不自动 VACUUM。
5. BackupRestoreService：
   - 复用 C1 的 migration Online Backup 原语；
   - 分页、可取消、节流、空间预检的用户手动 Online Backup；
   - checksum / manifest / integrity；
   - maintenance restore、当前库保护和 forward migration；
   - 接入 C2 Recovery Shell 的实际恢复命令。
6. C3 migration 只追加新版本，不修改 C1；migration ID 按串行合并顺序分配。
7. 大型数据变化使用 expand → checkpointed backfill → contract，可跨启动续跑；不以单个长事务阻塞 collector / 首次窗口。

### 验证

- golden query fixtures 覆盖时区、DST、空区间、部分 coverage、策略变化和 raw 过期。
- UI / CSV / JSON / HTML 对同一 report snapshot token 总计逐项一致；token 过期要求重新运行报告。
- retention 中断、重试、并发采集、raw / hourly / daily 守恒。
- raw 过期 fixture 证明老数据只开放分类 + 单维查询，不错误承诺任意跨维筛选。
- live backup、坏 checksum、坏 integrity、旧 schema restore、残留 WAL / SHM。
- 大数据分页与流式导出不造成内存峰值。
- 全部深分页为 keyset；页面 / 报告查询可 deadline / interrupt；EQP 无意外大表 SCAN / TEMP B-TREE / 自动索引。
- writer、report、export、backup、retention、checkpoint 并发时仍满足 durable commit SLO，WAL 不无限增长。
- 运行 `just monitor-check`；证据写入 C3 子任务。

### 回滚点 C3

报告 / 导出可 feature-disable；retention 未通过守恒 gate 前不得启用自动删除。恢复功能未通过全矩阵前只允许创建备份，不允许覆盖恢复。

## C4：告警与诊断

### 工作项

1. 实现 AlertEngine：
   - 新增 C4 前向 migrations：alert rule / instance / event / notification outbox；
   - health、60 秒滚动速率与滚动 1 小时 / 自然日 / 自然月用量规则；
   - 周期用量与证据链接复用 C3 ReportService / rollup，不创建第二套窗口聚合；
   - 活动、恢复、冷却、静默和去重状态机；
   - 告警历史、rule version、data version 与证据查询。
2. NotificationSink：
   - facts、coverage、alert state / event、notification outbox 在同一个 writer transaction 提交；
   - commit 成功后才通知，失败更新 outbox 并可重试；
   - 启动与周期 pending 扫描、原子 lease / stale lease 回收、attempt / next-at、指数退避和 sent / failed 状态；
   - 权限 / 系统禁用 / Focus Assist 说明；
   - 测试通知；
   - 点击最小行为为打开应用。
3. 告警中心：
   - 活动 / 历史、规则编辑、恢复时间、数据证据链接；
   - 规则输入校验和预览。
4. 诊断：
   - app / schema / SQLite / mihomo / transport 版本；
   - last frame、coverage、reconnect、queue、DB / WAL、commit / checkpoint / backup、retention；
   - 脱敏日志与诊断导出。

### 验证

- 状态机时间测试：60 秒滚动平均、连续 3 次、滚动小时、本地自然日 / 月、DST、重复事件、恢复、冷却、静默跨午夜、规则变更。
- 通知发送失败不丢告警；重复 health flapping 不轰炸。
- facts / coverage / alert / outbox 注入 transaction failure 和 hard reset 后保持一致。
- 在 commit 后、首次发送前 kill 进程；重启后 stale / pending outbox 被重新认领并最终 sent 或明确 failed，不永久卡住。
- 安装态普通用户测试通知；开发态不作为品牌 / AUMID 验收。
- 诊断产物 secret 和完整敏感字段扫描为零。
- 运行 `just monitor-check` 与安装态通知 smoke；证据写入 C4 子任务。

### 回滚点 C4

可关闭 NotificationSink 保留应用内告警；不能删除已持久化告警历史或回退 schema。

## C5：发布硬化与最终集成

### 工作项

1. 完成设计系统、所有状态、响应式窗口、键盘 / 焦点 / 对比度 / 数据表和中文文案。
2. 完成性能预算：
   - 使用 C0 冻结的 Windows 11 4 核 / 8GB harness；
   - 10k 活跃、1Hz、全部连接计数每帧变化时，frame → CommitBundle 计算 p95 < 500ms（不含 batch wait），frame → durable commit p95 < 1.5s、正常最大值 < 3s，队列不持续超过 2 帧；
   - 稳态 CPU 平均 < 15%、RSS < 500MB，24 小时预热后净增长 < 10%；
   - UI 可见交互 p95 < 150ms、30 天 raw 报告 p95 < 2s、13 个月 hourly 报告 p95 < 3s；
   - 大型 `/proxies`；
   - `A=50 / 250 / 1000` 的真实完整 30 天库 + 13 个月精确高基数 rollup + 长期 core daily；
   - writer / query / export / backup / retention / checkpoint 并行；
   - 至少 24 小时 soak 使用 C0 批准发布设计点的完整 workload tuple 与运行前冻结的报告 / 导出 / backup / retention / checkpoint / 告警日程，零崩溃、零守恒失败、零未解释缺口、零永久 stuck outbox。
   - 记录 p50 / p95 / p99 / max、每表 / 索引 B/row、DB / WAL / freelist、queue、CPU、RSS；
   - 零重复 bundle、零静默 gap、零无限 WAL、零不可取消查询；
   - backup / migration / VACUUM 在低磁盘空间下 fail closed。
3. 故障矩阵：
   - Verge / mihomo 重启、模式切换、网络变化、睡眠恢复；
   - app kill、Windows restart、磁盘满、DB busy / corruption；
   - migration / restore failure；
   - notification unavailable。
4. NSIS current-user：
   - 稳定 identifier / AUMID；
   - Start Menu、普通权限、WebView2 bootstrapper；
   - tray app 退出后手动升级；
   - 首次 v1 使用 C0 冻结安装包 / schema fixture → v1；后续使用上一正式 Release；
   - 默认卸载保留 LocalAppData 与 Credential Manager；应用内“删除全部数据与凭据”需二次确认并验证。
5. 发布供应链：
   - Windows code signing 与 timestamp；若首版无证书，必须由发布负责人显式批准未签名例外并在 Release notes 说明 SmartScreen，不能静默降级；
   - canonical installer、SHA-256、release notes、必要 SBOM；
   - 不替换已发布同名资产；
   - 发布事故通过新版本或撤回 Release，禁止改 migration 或覆盖旧资产；
   - About 固定 Releases 链接。
6. 文档：
   - 子项目 README、安装、首次配置、隐私、数据目录、备份恢复、报告口径、覆盖 / 尾差、故障排查；
   - 根 README 入口；
   - Release checklist。

### 最终验证命令

以实际 package scripts 为准，至少提供并执行：

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

GitHub CI 必须执行等价检查，并保持根 Required checks 聚合稳定。

### 回滚点 C5

- 候选未发布：撤下草稿资产，修复后重新构建，不复用未经验证的 installer。
- 已发布：不得替换 tag 下同名资产；发布修复版本或撤回有问题的 Release。数据库恢复只使用 C3 验证的备份，不以安装旧 binary 代替数据回滚。

## 父任务最终验收

1. 汇总 C0–C5 的验收证据，对照 PRD AC1–AC15。
2. 运行全量 code / spec review，确认实时、报告、导出和告警都使用同一后端事实与查询。
3. 使用安装包完成 Windows 11 真机端到端：
   - 首次引导 → 自启动 → 托盘；
   - TCP / pipe 接入；
   - 实时连接与策略；
   - 断线 / 恢复与告警；
   - 报告 / 导出；
   - retention / backup；
   - C0 冻结基线（后续为上一正式版本）手动升级；
   - 普通卸载保留数据，以及应用内显式删除全部数据 / 凭据。
4. 记录所有已知限制：观测下界、私有 pipe 兼容、未签名 SmartScreen（若适用）、不支持平台和非目标。
5. 所有子任务完成并归档后，再归档父任务。

## 开始前检查

- 当前仍处于 planning；不得运行 `task.py start`。
- 先由用户审阅本 PRD、design 和任务拆分。
- C0–C5 子任务已经创建且全部保持 `planning`；只有用户在后续独立消息明确授权后才可启动 C0，C1–C5 仍按依赖逐项审阅授权。父任务仅跟踪集成，不作为首个实现目标。
