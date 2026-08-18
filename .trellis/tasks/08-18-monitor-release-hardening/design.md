# C5 技术设计：发布硬化、最终性能门与供应链

## 设计目标

1. 用一个可追溯的候选提交、构建环境和证据包完成 go / no-go，不把分散的开发态成功误当成可发布状态。
2. 在真实规模数据库和完整并发组合下证明 ingestion、查询、维护与告警可共存。
3. 通过 C0 冻结基线验证首次 v1 手动升级，而不是只验证全新安装。
4. 让安装、升级、卸载、显式删除、签名和 Release 资产具有清晰且可回滚的状态转换。
5. 将视觉、无障碍、故障矩阵、文档和供应链与同一候选版本绑定。

## 依赖与发布候选边界

C5 只在 C4 独立验收后开始。候选输入包含：

```text
C0 选型、Windows 能力、性能 harness、早期安装包与 schema fixture
C1 采集、核算、SQLite writer / migration 与 ingestion SLO
C2 桌面生命周期、实时 UI、Credential Manager 与 Recovery Shell
C3 ReportService、导出、retention、backup / restore 与查询 corpus
C4 AlertEngine、notification outbox、告警中心与脱敏诊断
```

输出是一个不可变候选集合：

```text
source commit
  + lockfiles / toolchain / build metadata
  + canonical NSIS installer
  + signature or explicit exception
  + SHA-256 + SBOM / dependency inventory
  + migration / upgrade fixtures
  + automated and Windows installation evidence
  + performance / soak raw metrics
  + release notes / docs / rollback runbook
```

同一轮验收中的代码、installer、测试结果和文档必须指向同一个候选提交。任何代码、migration、依赖、capability、安装配置或性能相关参数变化都会使受影响证据失效。

## Gate 编排

发布编排使用有向 gate，而不是一条只在最后报错的长脚本：

```text
dependency evidence
  → clean build and static quality
  → migration / upgrade
  → installed end-to-end
  → fault matrix
  → dataset and query-plan validation
  → concurrent performance
  → 24h soak
  → signing / supply chain
  → docs and release review
  → go / no-go
```

- 每个 gate 记录输入版本、命令、环境、原始输出、判定和回滚点。
- 后续 gate 失败不覆盖前序结果；修复后按影响范围重新执行，24 小时 soak 的零容忍不变量失败必须从干净基线重跑。
- C5 不通过调低门限、隐藏指标或临时跳过功能获得绿色结果。父任务预算若被实测证明不可达，必须回到规划层以测量证据重新评审。

## 完整集成数据流审查

最终 code / contract review 沿以下主链逐段核对：

```text
controller frame
  → AccountingEngine
  → CommitBundle
  → SQLite facts + coverage + alerts + outbox
  → LiveProjection / ReportService / AlertEngine
  → AppFacade DTO
  → UI / CSV / JSON / HTML / Windows notification
```

审查不变量：

- controller meter 与 attributed observed 不混称；分类 + 其他守恒。
- coverage gap 不写成零，报告、导出和告警证据使用同一 coverage。
- 周期告警只复用 C3 ReportService / rollup。
- report snapshot token 不持有长期 read transaction。
- WebView、托盘和通知都不拥有第二套账本。
- secret 不进入 SQLite、日志、Channel、诊断、导出或 Release 资产。

发现跨层分叉时回到所属子任务修复；C5 只负责验证整合结果。

## 视觉与无障碍设计验收

建立页面 × 状态 × 输入方式矩阵：

- 页面：首次引导、概览、实时连接、报告、告警、设置 / 数据管理、Recovery Shell。
- 状态：loading、empty、healthy、degraded、disconnected、unauthorized、pipe denied / busy、protocol incompatible、coverage gap、storage / migration / restore failure、notification unavailable、capability expired。
- 输入：键盘、鼠标、系统缩放、高对比与正常主题。

每个矩阵项检查：

- 标题、原因、影响和恢复动作；
- 焦点进入、顺序、可见性和返回位置；
- 文本与背景对比、非颜色编码、图标 / 状态文字；
- 动态更新不造成焦点丢失或无界播报；
- 数字单位、等宽排版、时间范围、coverage 与数据表；
- 窄窗口、高 DPI 和打印 HTML。

自动检查只作为辅助，最终包含键盘和 Windows 真机走查证据。

## 故障矩阵设计

每个故障 fixture 使用统一记录：

```text
FaultCase {
  id, environment, precondition, injection,
  expected_health, expected_coverage,
  expected_data_invariant,
  automatic_recovery, manual_action,
  diagnostic_fields, rollback,
  actual_result, evidence
}
```

故障按边界分组：

1. 控制器：重启、模式切换、端点变化、TCP 401、pipe ACL / busy / incompatible。
2. 时间与进程：睡眠恢复、应用 kill、Windows restart、时钟变化。
3. 存储：busy、I/O error、磁盘满、WAL starvation、corruption、future schema。
4. 维护：migration、backup、restore、retention、VACUUM 中断或空间不足。
5. 系统能力：Credential Manager、通知、Focus Assist、autostart、single-instance。

所有 case 都核对 coverage 和 durable watermark，不只看 UI 文案。

## 性能数据集

### 数据集族

使用 C0 冻结生成器和真实 SQLite binding 生成：

- `A=50`、`A=250`、`A=1000` 三档完整 30 天 raw 库；
- 每档包含 session、chain、稀疏 minute、coverage、策略版本、告警和 outbox 的代表性分布；
- 13 个月精确高基数 hourly / daily dimension；
- 跨越 13 个月后的长期 core daily：attributed total、历史主分类与 coverage；
- 大量唯一域名、进程、规则、链路、长文本、冷热分布和恶意边界值；
- 独立 10,000 活跃连接、1 Hz、至少 30 分钟峰值流。

生成后记录 seed、生成器版本、schema / app / SQLite 版本、每表行数、时间范围、逻辑 checksum 和物理大小。三档均实际运行；不从 `A=50` 或 `A=250` 线性外推更大档位。

### 查询与操作 corpus

至少包含：

- 连接列表常用筛选 / 排序 / keyset 翻页；
- 30 天 raw totals、趋势、Top N、组合过滤和下钻；
- 13 个月 hourly / daily 单维趋势、Top N 和周期对比；
- 长期 core daily 总量、分类和 coverage；
- 同 snapshot 的 CSV / JSON / HTML 流式导出；
- alert evidence 打开同口径报告；
- backup、retention、checkpoint、取消和低空间路径。

每个公开查询记录冷 / 热执行、EQP 守卫、FULLSCAN_STEP、SORT、VM_STEP 和 p50 / p95 / p99 / max。

## 并发负载模型

稳态 writer 作为最高优先级负载，其上同时运行：

```text
1 × collector/accounting/writer
N × bounded report readers
1 × streaming export
1 × paged Online Backup
1 × retention chunk worker
1 × PASSIVE checkpoint coordinator
1 × alert/outbox worker
```

- 并发数、查询组合和启动偏移由 harness 固定并记录。
- report、export、backup、retention 和 checkpoint 必须与 writer 真正重叠，不能串行运行后声称并发通过。
- 长操作响应取消；reader deadline / interrupt 后及时释放 transaction。
- backup 无法收敛时可按既有设计短暂停采，但必须产生 coverage gap 并计入结果。
- 记录每个操作自身延迟以及对 frame / durable commit 的影响。

## 指标采集与判定

统一时钟和 trace ID 关联 frame、bundle、commit、query、maintenance、alert 和 outbox：

- 延迟：p50 / p95 / p99 / max；
- ingestion：frame → `CommitBundle`、frame → durable commit、queue depth / oldest age；
- 查询与 UI：命令执行、投影、前端可见交互；
- 存储：每表 / 索引 B/row、DB / WAL 峰值与回落、freelist、page count、写放大、checkpoint；
- 进程：CPU、RSS、线程 / task、句柄和预热后增长；
- 维护：backup / export 吞吐、retention 批次、取消响应；
- 告警：evaluation、outbox backlog、lease、attempt、stuck age。

报告保留原始时间序列与汇总，不能只保留通过 / 失败。采样本身的 CPU / I/O 开销要单独说明。

## 24 小时 soak

soak 顺序：

1. 预热 1 小时，建立稳定 cache、WAL 和 RSS 基线。
2. 运行 24 小时 C0 批准发布设计点（初始 `A=250`）的完整 workload tuple：固定 `A / L / C / q`、维度基数和每帧变化比例，持续 ingestion；不得以未声明轻载替代。
3. 执行运行前版本化的固定日程：至少每 5 分钟报告、每小时导出与 retention / checkpoint、全程至少 2 次 Online Backup、每小时告警和可恢复通知失败。若 C0 要求不同频率，必须在执行前随证据批准。
4. 在预定窗口注入睡眠 / 恢复、控制器重连和读取消，不破坏可解释性。
5. 结束后 flush、checkpoint、reopen、integrity、守恒、bundle 幂等、coverage 和 outbox 检查。

硬失败条件：

- 崩溃或守恒失败；
- 重复 bundle；
- 没有原因的 coverage gap；
- WAL 无界且不能在 reader 结束后回落；
- query / backup / export / retention 无法取消；
- outbox 永久 stuck；
- 预热后 RSS 净增长达到或超过 10%；
- 任一 ingestion 或报告硬门限失败。

失败后保留证据并从干净数据库基线重新执行，不把两段运行拼接为 24 小时。

## 低磁盘空间 fail-closed

分别在预检前、临时文件创建后、操作中段和最终 rename / swap 前注入空间不足：

- backup：当前库不变，`.partial` 不被列为有效备份，manifest 不标成功。
- migration：collector 不启动，保留 migration 前备份，Recovery Shell 可解释并恢复。
- VACUUM：空间预检拒绝启动；执行中失败不删除当前数据库，不自动重试占满磁盘。
- restore：候选或临时目标失败不覆盖当前可用库。

每个 case 在失败后执行 reopen、integrity、schema / checksum 和 smoke query。

## NSIS 安装、升级与卸载

### 安装

- 固定 per-user 安装模式和身份标识。
- 在普通用户环境检查 Start Menu、WebView2、通知、托盘、自启动、single-instance 和 LocalAppData / log 位置。
- 安装包不偷偷启用 autostart；由 onboarding 明确选择。

### 升级

```text
C0 frozen installer/schema
  → create representative data/settings/credential/autostart
  → exit tray app
  → install v1 candidate
  → migration backup + forward migration
  → reopen and verify all invariants
```

升级中断和失败进入 Recovery Shell；不得用 down migration。首次 v1 后，测试基线改为上一正式 Release。

### 卸载与显式删除

- NSIS 普通卸载删除 binary 和快捷方式，保留数据库、备份、设置与 Credential Manager 项。
- 应用内显式删除先进入 maintenance mode，展示对象清单，二次确认后停止运行服务并分阶段删除。
- 删除操作返回逐项结果；部分失败必须可见，不显示「已全部删除」。
- 文档提供卸载后手动删除路径与凭据步骤。

## 签名与发布供应链

构建后按以下顺序处理：

1. 从固定候选提交和 lockfiles 构建 canonical installer。
2. 生成签名前哈希并执行恶意软件 / secret / remote-resource 检查。
3. 使用 Authenticode 与可信 timestamp 签名，再验证签名链、timestamp 和安装执行。
4. 生成最终 SHA-256、SBOM / 依赖清单、许可证清单和构建 metadata。
5. 将 installer、checksum、说明和证据绑定到同一 Release draft。

如果没有证书：

- 生成明确的未签名候选；
- 发布负责人记录版本、资产哈希、原因、风险和批准时间；
- Release notes 显示 SmartScreen 预期；
- 不生成或传播「signed」标记。

发布后资产不可替换。若哈希、签名或内容错误，撤回 Release 或发布新版本。

## 文档一致性

文档由候选行为反向核对：

- 安装 / 升级 / 卸载步骤在干净 Windows 用户上实走；
- 数据目录、备份恢复、显式删除和 credential 语义与实际一致；
- TCP / pipe 支持边界、观测下界、coverage、尾差和告警送达限制完整；
- 所有命令从锁定候选运行；
- Release notes 明确签名状态、已知限制和回滚路径。

## 回滚与发布事故

- 候选未发布：撤下 draft 资产，保留失败证据，修复后生成新候选和新哈希。
- 签名 / checksum 失败：资产不得发布，也不得原地用另一个文件复用 canonical 名称。
- 已发布代码问题：撤回 Release 或发布更高版本；不替换 tag 下资产。
- 已发布 migration 问题：停止分发，提供前向修复；数据恢复只使用经 C3 验证的备份。
- 性能不通过：回到所属模块修复并重跑受影响数据集 / 并发 / soak；不静默降低 durability 或支持范围。

## 设计验收

- 一个证据索引能从每条 C5 AC 追溯到候选提交、资产、命令和原始结果。
- 三档 30 天库、13 个月高基数、长期 core daily、10,000 峰值和并发 workload 均实际运行。
- 24 小时 soak 与低空间矩阵满足零容忍不变量。
- C0 → v1 安装升级、普通卸载保留和应用内显式删除通过。
- 签名或显式例外、checksum、SBOM / 依赖清单、文档和 Release draft 指向同一不可变候选。
