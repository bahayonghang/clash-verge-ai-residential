# C5 实施计划：发布硬化与最终集成

## 启动前 Gate

- [ ] C0–C4 均已完成独立验收；C4 告警、outbox、诊断和 ingestion 性能证据完整。
- [ ] C0 冻结的 Windows 11 harness、数据生成器、早期 NSIS 安装包、schema fixture、identifier / AUMID / credential / autostart 标识可用且 checksum 已核验。
- [ ] C5 的 PRD、design、implement 和 manifests 已由用户审阅。
- [ ] 用户在审阅后另行明确授权启动；在此之前保持 `planning`，不得运行 `task.py start`。

## 实施顺序

### 1. 建立候选与证据索引

- [ ] 选择单一候选提交，记录 Node、Rust、Tauri、WebView2、SQLite、Windows 和构建环境版本。
- [ ] 收集 C0–C4 的 gate、fixture、benchmark、migration、回滚和已知限制。
- [ ] 建立 C5 AC → 命令 → fixture → 原始输出 → 判定 → 资产的证据索引。
- [ ] 确认 lockfiles、构建配置、CSP、capability 和 identifier 已冻结。
- [ ] 定义任何改动后必须重跑的影响矩阵。

Gate：所有输入都能追溯到同一候选；缺失 C4 或 C0 升级基线时停止。

### 2. 完成跨层集成审查

- [ ] 从 controller frame 追踪到核算、CommitBundle、SQLite、实时投影、报告、导出、告警、诊断和 UI。
- [ ] 核对 controller meter / attributed observed、分类守恒、coverage、data version 和 policy version。
- [ ] 核对 C4 周期告警只复用 C3 ReportService / rollup。
- [ ] 核对 report snapshot token、Channel seq / resync、outbox 和 Recovery Shell 的生命周期。
- [ ] 扫描 secret、远程资源、宽泛 capability、前端 SQL 和重复统计逻辑。

Gate：发现语义分叉时退回所属 C1–C4 子任务修复并重新验收，不在 C5 叠加补丁口径。

### 3. 完成视觉、状态与无障碍

- [ ] 统一设计令牌、排版、间距、状态层级、等宽数字和响应式窗口。
- [ ] 完成页面 × 状态矩阵中的专门中文状态、影响说明和恢复动作。
- [ ] 验证键盘操作、焦点顺序 / 可见性 / 恢复、高对比、非颜色编码和系统缩放。
- [ ] 为所有图表提供同口径数据表、单位、时间范围与 coverage / gap 表达。
- [ ] 验证动态 Channel 更新、窗口重建和错误提示不造成焦点丢失。
- [ ] 验证可打印 HTML 在彩色和灰阶下都可解释。

Gate：自动检查和 Windows 真机走查都通过；截图不能替代键盘 / 焦点操作记录。

### 4. 执行故障矩阵

- [ ] 控制器：重启、模式切换、端点变化、TCP secret 正确 / 错误 / 为空、pipe ACL / busy / incompatible。
- [ ] 生命周期：窗口隐藏 / 重建、应用 kill、Windows restart、睡眠 / 恢复、网络变化、明确退出。
- [ ] 存储：DB busy、I/O error、磁盘满、WAL starvation、corruption、future schema。
- [ ] 维护：migration、backup、restore、retention、VACUUM 中断与低空间。
- [ ] 系统能力：Credential Manager、notification、Focus Assist、autostart、single-instance。
- [ ] 每个 case 核对 UI 状态、health、coverage、durable watermark、守恒、诊断、恢复和回滚。

Gate：任何未知时段写成零、静默丢账、不可解释状态或恢复后不守恒均阻止后续发布。

### 5. 生成并验证最终规模数据集

- [ ] 使用真实 binding 与 C0 冻结生成器分别创建 `A=50 / 250 / 1000` 完整 30 天库。
- [ ] 每档加入真实 / 恶意高基数、稀疏分钟事实、长进程路径、多 chain、coverage、策略版本、告警和 outbox。
- [ ] 创建 13 个月精确高基数 hourly / daily rollup 和更长期 core daily。
- [ ] 记录 seed、版本、时间范围、每表行数、逻辑 checksum、DB / WAL / freelist 与生成耗时。
- [ ] 用 `dbstat`、`sqlite3_analyzer` 或 binding 等价能力记录每表 / 每索引 B/row。
- [ ] 对全部命名查询运行冷 / 热 corpus、EQP / statement-status 和结果守恒检查。

Gate：三档均为实际完整规模，不允许小库线性外推；查询 capability 与保留层承诺一致。

### 6. 执行 10,000 活跃短峰

- [ ] 运行 10,000 活跃连接、1 Hz、至少 30 分钟回放。
- [ ] 同时启用代表性速率 / 周期规则、实时 UI 订阅与正常 writer。
- [ ] 记录 frame → `CommitBundle` 和 frame → durable commit 的 p50 / p95 / p99 / max。
- [ ] 记录 queue depth / oldest age、CPU、RSS、DB / WAL、告警评估和 Channel seq / resync。
- [ ] 验证计算 p95 小于 500 ms、durable p95 小于 1.5 s、正常 max 小于 3 s，输入队列不持续超过 2 帧。

Gate：该峰值只证明短时能力；报告不得把它写成 30 天持续支持范围。

### 7. 执行 writer / report / export / backup / retention / checkpoint 并发门

- [ ] 在持续 writer 下重叠运行 bounded reports、流式 export、分页 Online Backup、retention chunk 和 PASSIVE checkpoint。
- [ ] 在 30 天三档、13 个月高基数和长期 core daily 上运行对应查询 corpus。
- [ ] 验证页面 / 报告 deadline / interrupt，export / backup / retention 取消与进度。
- [ ] 验证 report snapshot token 不保留长期 read transaction，reader 结束后 WAL 可回落。
- [ ] 记录所有操作自身指标及其对 ingestion、queue、CPU、RSS、DB / WAL / freelist 的影响。
- [ ] 验证 30 天报告 p95 小于 2 s、13 个月 hourly 报告 p95 小于 3 s、可见交互 p95 小于 150 ms。

Gate：出现无限 WAL、不可取消 query、backup 不收敛却无 coverage、retention 抢占 writer 或 ingestion SLO 失败时停止。

### 8. 执行低空间 fail-closed

- [ ] 对 backup 在预检、`.partial`、分页中段和最终 rename 前注入空间不足。
- [ ] 对 migration 在备份、DDL / backfill 和验证阶段注入空间不足。
- [ ] 对用户主动 VACUUM 在预检和执行阶段注入空间不足。
- [ ] 对 restore 临时目标和受控 swap 前注入空间不足。
- [ ] 每次失败后执行当前库 reopen、integrity、schema / checksum、smoke query 和 coverage 检查。

Gate：当前可用数据库受损、伪成功 manifest / 资产、不可恢复半迁移或静默 gap 任一出现都失败。

### 9. 执行 24 小时 soak

- [ ] 预热 1 小时并冻结 RSS、cache 和 WAL 基线。
- [ ] 连续运行至少 24 小时 C0 批准发布设计点（初始 `A=250`）的完整 `A / L / C / q`、维度基数、每帧变化比例，不使用未声明轻载。
- [ ] 按运行前冻结日程执行：至少每 5 分钟 report、每小时 export 与 retention / checkpoint、全程至少 2 次 backup、每小时告警与通知失败。
- [ ] 按计划注入控制器重连、睡眠 / 恢复和查询取消。
- [ ] 持续采集 p50 / p95 / p99 / max、CPU、RSS、queue、DB / WAL / freelist、coverage、bundle 和 outbox 指标。
- [ ] 结束后 flush、checkpoint、reopen、integrity、守恒、幂等、coverage 和 outbox 检查。
- [ ] 验证预热后 RSS 净增长小于 10%，总 CPU 平均小于 15%，RSS 小于 500 MB。

Gate：零崩溃、零守恒失败、零重复 bundle、零静默 gap、零无限 WAL、零不可取消 query、零永久 stuck outbox。任一失败都从干净基线重新运行。

### 10. 验证 NSIS current-user 安装、升级和卸载

- [ ] 构建普通用户 NSIS current-user installer，验证稳定 identifier、AUMID、Start Menu、WebView2、数据 / 日志路径。
- [ ] 安装态验证托盘、关闭隐藏、自启动 `--background`、single-instance、普通权限通知和明确退出。
- [ ] 安装 C0 冻结基线，生成代表性历史、设置、备份、credential 和 autostart 状态。
- [ ] 退出托盘应用后手动升级到 v1 候选，验证 migration backup、数据、设置、credential 引用、autostart 和历史告警。
- [ ] 注入升级中断、future schema 和 checksum mismatch，验证 fail closed 与 Recovery Shell。
- [ ] 普通卸载验证数据与 Credential Manager 保留。
- [ ] 应用内二次确认删除验证全部声明对象、部分失败状态和再次启动行为。

Gate：不得用开发态或重新生成的伪旧包代替 C0 基线；首次 v1 通过后才把基线策略改为上一正式 Release。

### 11. 完成签名与发布供应链

- [ ] 审查 npm / Cargo lock、许可证、构建脚本、CSP、capability、远程请求和 secret scan。
- [ ] 从冻结候选构建 canonical installer，生成并核对签名前哈希。
- [ ] 使用 Authenticode 和可信 timestamp 签名，验证签名链、timestamp 与安装；或取得针对具体资产哈希的发布负责人显式未签名例外。
- [ ] 生成最终 SHA-256、SBOM / 依赖清单、许可证清单和构建 metadata。
- [ ] 验证 GitHub CI 等价 gate 与根 Required checks 聚合稳定。
- [ ] 创建但不发布 Release draft，核对资产名、哈希、签名状态、已知限制和不可变策略。

Gate：无有效签名也无显式例外、哈希不匹配、secret 泄露、宽泛 capability 或来源不可追溯时不得发布。

### 12. 完成文档与最终 go / no-go

- [ ] 完成 README、安装、首次配置、控制器兼容、隐私、数据目录、备份恢复、统计口径、coverage / 尾差、告警和故障排查。
- [ ] 完成升级、普通卸载保留、应用内显式删除和手动清理说明。
- [ ] 完成 Release checklist、故障矩阵、性能报告、24 小时 soak 报告、已知限制和回滚 runbook。
- [ ] 在干净 Windows 用户上按文档实走关键流程。
- [ ] 对 C5-AC1 至 C5-AC14 逐项链接候选、命令、原始输出、判定和资产。
- [ ] 由发布负责人依据证据作出 go / no-go；本任务本身不自动发布 Release。

Gate：文档、候选、资产和证据版本完全一致后才可建议发布。

## 最终验证命令

以 C0 冻结的实际 scripts 为准，至少执行：

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

此外必须提供可重复的专项入口：

- C0 → v1 NSIS 手动升级与普通卸载 / 显式删除。
- 三档 30 天库、13 个月高基数、长期 core daily 数据生成和校验。
- 命名查询 cold / hot corpus、EQP / statement-status 与 B/row 分析。
- 10,000 活跃短峰。
- writer / report / export / backup / retention / checkpoint 并发。
- backup / migration / VACUUM / restore 低空间注入。
- 24 小时 soak、post-soak integrity / conservation / idempotency。
- Windows 安装态通知、托盘、自启动、single-instance 和无障碍走查。
- installer 签名验证、SHA-256、SBOM / 依赖与许可证检查。

## 性能 Gate 摘要

- 数据集：`A=50 / 250 / 1000` 真实完整 30 天库；13 个月精确高基数；长期 core daily；独立 10,000 × 1 Hz × 至少 30 分钟。
- 并发：writer、report、export、backup、retention、checkpoint 和 alert / outbox 真正重叠。
- 指标：p50 / p95 / p99 / max、B/row、DB / WAL / freelist、写放大、queue、CPU、RSS。
- 时延：计算 p95 `< 500 ms`；durable p95 `< 1.5 s`、正常 max `< 3 s`；UI p95 `< 150 ms`；30 天报告 p95 `< 2 s`；13 个月报告 p95 `< 3 s`。
- 资源：CPU 平均 `< 15%`；RSS `< 500 MB`；预热后 24 小时 RSS 净增长 `< 10%`。
- 零容忍：零重复 bundle、零静默 gap、零无限 WAL、零不可取消 query、零永久 stuck outbox。
- 低空间：backup、migration、VACUUM 和 restore fail closed，当前可用库不受损。

## 验收证据

每项证据至少包含：

- 候选提交、schema / app / SQLite / WebView2 / Windows / toolchain 版本；
- 数据集 seed、行数、时间范围、逻辑 checksum 和物理占用；
- 完整命令、开始 / 结束时间和原始机器可读输出；
- p50 / p95 / p99 / max 与门限判定；
- fault case 的前置、注入、coverage、恢复、诊断和回滚；
- installer 哈希、签名 / 例外、SBOM / 依赖清单和 Release draft；
- 对应 PRD AC 与通过 / 失败结论。

不得只提交汇总截图、删除失败样本或用多次运行的最佳值替代完整分布。

## 回滚计划

- **候选未发布**：撤下 draft 资产，保留失败证据，修复后生成新候选、哈希和受影响 gate。
- **性能失败**：回退到所属模块修复，不降低 durability、不隐藏 coverage、不缩短保留；重新生成受影响数据库并重跑并发 / soak。
- **安装或升级失败**：停止分发，保留 C0 基线和 migration 前备份；使用 Recovery Shell / C3 restore，不执行 down migration。
- **签名或供应链失败**：资产不得发布；未签名只允许在具体哈希获得显式例外后继续。
- **已发布事故**：撤回 Release 或发布新版本，不替换 tag 下同名资产，不修改已发布 migration。
- **数据回滚**：只恢复经验证备份并安装 schema 兼容 binary，不把安装旧 binary 当成数据恢复。

## 完成条件

- [ ] C5-AC1 至 C5-AC14 全部通过并有可重复、可追溯证据。
- [ ] 视觉无障碍、故障矩阵、三档数据库、并发门、低空间和 24 小时 soak 全部通过。
- [ ] C0 → v1 手动升级、普通卸载保留和应用内显式删除通过。
- [ ] 签名或显式例外、SHA-256、SBOM / 依赖清单、文档和候选一致。
- [ ] go / no-go 结论明确；Release 发布仍需单独授权。
