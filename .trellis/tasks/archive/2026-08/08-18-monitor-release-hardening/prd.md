# C5：家宽监控发布硬化与最终集成

## 目标与用户价值

把 C0–C4 的独立交付物整合为可安装、可升级、可卸载、可诊断且性能边界有真实证据的 Windows 11 v1。发布候选必须在普通用户安装态完成端到端、故障、无障碍、数据库并发、24 小时稳定性和供应链验收，不能以开发态 smoke、小库线性外推或发布说明中的免责声明替代技术 gate。

用户最终获得一个不会因窗口隐藏而停止采集、不会把缺口写成零、不会因通知失败丢失告警、升级后保留历史与凭据引用、卸载时不会误删数据的桌面应用。

## 前置条件与边界

- 本任务是 C5，严格依赖 C4 `08-18-monitor-alerting-diagnostics` 完成并通过独立验收；同时要求 C0–C3 的 gate、fixture、基准和回滚证据齐全。
- C5 是集成与发布硬化，不重新定义 C1 核算、C3 报告 / retention / backup 或 C4 告警语义。发现语义缺陷时退回所属子任务修复并重新验收。
- 首次 v1 手动升级必须使用 C0 冻结的早期 NSIS 安装包、schema fixture、稳定 identifier、AUMID、credential target 和 autostart 标识；v1 之后改用上一正式 Release。
- 当前状态保持 `planning`。本次规划不授权运行 `task.py start`、发布 Release、签名资产或编辑产品代码。

## 功能需求

### R1. 完整安装态集成

- 在 Windows 11、普通用户、NSIS current-user 安装态完成首次引导、TCP / 尽力兼容 named pipe、实时连接、报告、导出、retention、备份恢复、健康 / 流量告警、Windows 通知、托盘、自启动、单实例和明确退出。
- 所有实时、历史、导出和告警数字继续来自同一事实、coverage 与后端投影；集成修复不得在前端添加第二套统计逻辑。
- 开发态通过不替代安装态验证；Windows Service、应用内自动更新和非 Windows 平台不进入 v1 验收。

### R2. 视觉、状态与无障碍

- 完成统一设计令牌、深色主题、响应式窗口、中文文案、数字等宽排版和固定状态层级。
- 连接中、断线、TCP 鉴权失败、管道访问拒绝、管道忙超时、协议不兼容、无数据、coverage 缺口、存储背压 / 故障、迁移 / 恢复失败、通知不可用和数据能力过期均有专门状态与恢复动作。
- 所有核心流程支持键盘操作、可见焦点、合理焦点顺序、高对比文本、非颜色唯一编码和系统缩放。
- 每个图表提供同口径数据表、单位、时间范围和缺口说明；打印 HTML 不依赖颜色表达唯一含义。

### R3. 故障矩阵

- 覆盖 Verge / mihomo 重启、运行模式切换、端点变化、网络变化、系统睡眠 / 恢复、应用 kill 和 Windows restart。
- 覆盖磁盘满、低空间、DB busy、I/O error、WAL checkpoint starvation、数据库 corruption、future schema、migration / restore / backup 失败。
- 覆盖 Credential Manager 不可用、Windows 通知不可用、Focus Assist 和 outbox 持续失败。
- 每个场景记录用户可见状态、coverage 语义、自动恢复、手动动作、数据不变量、诊断和回滚结果；未知时段不得写成零。

### R4. NSIS current-user 与生命周期

- 使用稳定 identifier、product name、binary name、publisher、AUMID、安装目录、数据目录、credential target 和 autostart 参数。
- 安装包不要求管理员权限；Start Menu、WebView2 bootstrapper、托盘、自启动、单实例和普通用户通知在安装态通过。
- 手动升级前明确退出托盘应用；升级完成后数据库、备份、机器设置、UI 偏好、Credential Manager 引用、autostart 和历史告警保持完整。
- v1 不注册 updater plugin；About 页只打开固定 GitHub Releases 地址。

### R5. C0 基线到 v1 的手动升级

- 从 C0 冻结安装包和 schema fixture 执行覆盖安装 / 手动升级，不用当前代码临时重新制作「旧版本」。
- 验证迁移前 Online Backup、forward migration、checksum、重复启动、升级中断、future schema 和恢复界面。
- 升级后执行采集、报告、导出、告警、通知、backup / restore smoke，并核对数据守恒和 coverage。
- 二进制回滚必须先验证 schema compatibility；数据回滚只使用经验证备份，不以安装旧 binary 猜测兼容。

### R6. 签名与显式例外

- 正式资产优先使用 Windows Authenticode code signing 和可信 timestamp；签名前后均校验 canonical installer 哈希。
- 若首版无法取得证书，必须由发布负责人对具体版本和具体资产显式批准未签名例外，并在 Release notes 清楚说明 SmartScreen 风险。
- 不得把未签名构建描述为已验证签名体验，也不得在 CI 或文档中静默降级签名要求。

### R7. 卸载与本地数据

- 普通卸载默认保留 LocalAppData 中的数据库、备份和机器设置，以及 Credential Manager 凭据，避免误删历史。
- 应用内提供「删除全部本地数据与凭据」操作，列出删除范围、要求二次确认、先停止 collector / writer，并验证失败后的可恢复状态。
- Release 文档列出卸载后手动清理位置和凭据删除方式；不得把保留数据误报为卸载失败。

### R8. 文档与支持信息

- 完成子项目 README、安装、首次配置、控制器兼容、隐私、数据目录、备份恢复、报告口径、coverage / 观测下界 / 尾差、告警、故障排查、卸载和升级说明。
- 根 README 提供入口；文档明确 Windows 11 支持范围、TCP 稳定路径、named pipe 尽力兼容、无应用内自动更新和无遥测。
- 完成 Release checklist、故障矩阵、已知限制和回滚 runbook；命令、路径、状态文案与实际产品一致。

### R9. 发布供应链

- 锁定并审查 npm / Cargo 依赖、许可证、Tauri capability、CSP、远程资源、构建脚本和 secret scan。
- 发布 canonical NSIS installer、SHA-256、Release notes 和可审计的依赖清单 / SBOM；构建环境、源提交和资产哈希可追溯。
- GitHub CI 执行等价质量门并保持根 Required checks 聚合名称稳定。
- 已发布 tag 下不得替换同名资产，不得修改已发布 migration。发布事故通过撤回 Release 或发布新版本处理。

### R10. 数据库最终性能数据集

- 使用 C0 冻结且由真实 binding 生成的 `A=50 / 250 / 1000` 三档完整 30 天库，不得用小库结果线性外推。
- 每档同时包含真实分布的会话时长、稀疏分钟事实、多 chain、长进程路径和恶意 / 高基数维度。
- 历史层必须包含 13 个月精确高基数 hourly / daily rollup，以及长期 attributed total、历史主分类和 coverage 的 core daily。
- 10,000 活跃连接、1 Hz、至少 30 分钟只作为独立短时峰值，不宣称为 30 天持续容量。

### R11. 数据库并发最终门

- 在持续 writer 下并发运行 report、export、Online Backup、retention 和 checkpoint；覆盖冷 / 热查询、长导出、backup 收敛、retention 批次和 checkpoint starvation。
- 所有页面 / 报告查询可 deadline / interrupt；导出、backup、retention 和用户主动 VACUUM 可取消并显示进度。
- 读取不得长期持有 transaction；report snapshot token 不得导致 WAL 无界增长。
- 低空间下 backup、migration 和 VACUUM 必须 fail closed，不破坏当前可用数据库、不留下被误认成成功的 `.partial` 资产。

### R12. 指标与性能门限

- 所有延迟记录 p50 / p95 / p99 / max，不只记录平均值。
- 记录每表 / 每索引 B/row、DB / WAL 峰值与回落、freelist、写放大、writer queue、outbox backlog、CPU 和 RSS。
- 10,000 活跃短峰：frame → `CommitBundle` 计算 p95 小于 500 ms；frame → durable commit p95 小于 1.5 s，正常最大值小于 3 s；collector 输入队列不持续超过 2 帧。
- 稳态应用总 CPU 平均小于 15%，RSS 小于 500 MB；预热 1 小时后 24 小时 RSS 净增长小于 10%。
- 活跃连接可见交互 p95 小于 150 ms；30 天 raw 常用报告 p95 小于 2 s；13 个月 hourly 常用报告 p95 小于 3 s。
- 性能报告必须同时给出基准机、WebView2、SQLite、应用 / schema 版本、数据集种子和冷 / 热条件。

### R13. 24 小时 soak 与零容忍不变量

- 在 C0 冻结基准机或已批准的等价 Windows 11 环境上执行至少 24 小时真实或回放 soak。负载固定为 C0 批准的发布设计点（初始 `A=250`）及其完整 `A / L / C / q`、维度基数和每帧变化比例，不能使用未声明轻载。
- soak 使用版本化固定日程：至少每 5 分钟报告、每小时导出与 retention / checkpoint、运行期至少 2 次 Online Backup，并每小时注入可恢复告警 / 通知失败；实际频率若由 C0 证据调整，必须在运行前冻结而非运行后放宽。
- soak 期间要求零崩溃、零守恒失败、零重复 bundle、零静默 coverage gap、零无限 WAL、零不可取消 query、零永久 stuck outbox。
- 任一零容忍不变量失败都必须修复并从干净基线重新执行相应 soak；不能以平均表现或手工清理后继续累计时长。

## 验收标准

- [ ] **C5-AC1 依赖完整**：C0–C4 每个子任务的独立验收、fixture、命令、性能和回滚证据可追溯；C4 未通过时 C5 不启动。
- [ ] **C5-AC2 安装态端到端**：普通用户 NSIS 安装态完成 onboarding → controller → 实时 → 报告 / 导出 → retention / backup → 告警 / 通知 → 托盘 / 自启动 → 明确退出，全链路数字和 coverage 口径一致。
- [ ] **C5-AC3 视觉无障碍**：全部规定状态有中文界面和恢复动作；键盘、焦点、对比度、系统缩放、非颜色编码、图表数据表和打印 HTML 走查通过并留存证据。
- [ ] **C5-AC4 故障矩阵**：R3 每个场景均有自动或真机证据，记录状态、coverage、恢复、诊断和回滚；不存在静默缺口或把未知写成零。
- [ ] **C5-AC5 手动升级**：使用 C0 冻结安装包 / schema fixture 升级到 v1，identifier、AUMID、数据库、设置、备份、credential 引用、autostart 和历史告警完整；升级中断可 fail closed 并恢复。
- [ ] **C5-AC6 签名与资产**：installer 有有效签名和 timestamp；若无证书，则存在发布负责人针对该资产的显式例外和诚实 SmartScreen 说明。canonical installer、SHA-256、SBOM / 依赖清单和源提交可相互核验。
- [ ] **C5-AC7 卸载语义**：普通卸载保留数据与凭据；应用内二次确认删除可清理全部声明对象，失败不留下半删除却显示成功的状态；手动清理文档准确。
- [ ] **C5-AC8 性能数据集**：`A=50 / 250 / 1000` 完整 30 天库、13 个月精确高基数 + 长期 core daily 和独立 10,000 短峰均以真实规模运行，未使用线性外推。
- [ ] **C5-AC9 指标与门限**：记录 p50 / p95 / p99 / max、B/row、DB / WAL / freelist、写放大、queue、CPU、RSS；R12 的 ingestion、UI、报告、CPU 和 RSS 门限全部通过。
- [ ] **C5-AC10 并发与取消**：writer / report / export / backup / retention / checkpoint 并发下仍满足 ingestion SLO；所有长操作可取消，WAL 可回落，无不可取消 query 或永久 read transaction。
- [ ] **C5-AC11 24 小时稳定性**：C0 批准发布 workload tuple 与固定维护日程下，至少 24 小时 soak 达到零重复 bundle、零静默 gap、零无限 WAL、零不可取消 query、零永久 stuck outbox 和零崩溃 / 守恒失败。
- [ ] **C5-AC12 低空间 fail closed**：backup、migration 和 VACUUM 在每个关键空间不足点失败时保留当前可用库、产生正确诊断和 coverage，不生成伪成功资产。
- [ ] **C5-AC13 文档与供应链**：安装、配置、隐私、口径、coverage、备份恢复、告警、故障排查、升级卸载、已知限制和 Release checklist 与候选版本一致；CI、secret scan、CSP、capability、依赖与许可证检查通过。
- [ ] **C5-AC14 独立发布 Gate**：实施计划中的全部命令在同一候选提交通过；证据包包含环境、原始输出、门限判定、签名 / 例外、资产哈希、故障与回滚演练，发布负责人可以据此作出 go / no-go。

## 非目标

- 不在 C5 增加应用内自动更新、Windows Service、macOS / Linux 发布、云同步或外发遥测。
- 不把发布硬化变成新业务功能阶段；新需求需另建任务。
- 不通过降低 SQLite durability、隐藏 coverage、缩短保留、关闭失败测试或放宽零容忍不变量让候选通过。
- 不把开发态、未安装 executable 或重新制作的伪旧版本当成 NSIS 升级证据。
- 不替换已发布资产、重写 migration 或以旧 binary 代替数据库恢复。

## 规划结论

需求、依赖、安装升级、签名例外、卸载语义、供应链和最终数据库性能门已明确，无阻塞性开放问题。任务继续保持 `planning`，需在 C4 验收完成且用户审阅本规划后另行明确授权启动。
