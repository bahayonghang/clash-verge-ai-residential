# 家宽历史库 CLI 与规则优化 Skill

## Goal

给本仓库交付两件互相衔接的东西：一个只读查询 + 受控维护写入的本地 CLI，让 Agent 直接从 ResiWatch 历史库读出家宽流量证据；一个安装到本仓库各平台目录的 skill，指导 Agent 把这些证据转成 `clash-verge-ai-residential.js` 路由范围的收窄决定，减少不必要的家宽流量损耗。CLI 只管 residential-monitor 与其数据库，规则判读与配置改动由 skill 指导 Agent 完成。

## Background and Confirmed Facts

- ResiWatch 是 Tauri 应用，Rust 侧已有 rusqlite 与 C3 报告层。现有 bin 只有 `residential-monitor` 与 `monitor-bench`，clap 已是依赖；见 `residential-monitor/src-tauri/Cargo.toml:12-22,36`。
- 主库是 `<安装目录>\data\monitor.sqlite3`，current-user 安装目录为 `%LOCALAPPDATA%\ResiWatch`；`RESIDENTIAL_MONITOR_DATA_DIR` 可覆盖且覆盖时不迁移；见 `residential-monitor/docs/data-directory.md:3-12,19`。
- 数据目录解析函数 `prepare_data_dir` 与 `resolve_and_migrate` 会执行旧目录迁移；见 `residential-monitor/src-tauri/src/data_dir.rs:12,33,45`。CLI 触发迁移会在桌面端之外产生第二个迁移发起方。
- 只读连接入口已存在：`open_interruptible_reader` 以 `SQLITE_OPEN_READ_ONLY` 打开，执行 `apply_required_pragmas` 后注册 `last_chain_hop` 与 `chain_identity`；见 `residential-monitor/src-tauri/src/storage.rs:813-823` 与 `c3/rule_name.rs:72-95`。
- 报告入口 `ReportService::run` 已实现能力规划、cancel、deadline、read transaction 关闭与快照写入；见 `residential-monitor/src-tauri/src/c3/service.rs:29-60`。
- 可用分组维度是 Category / Host / Process / Rule / Chain / Network；过滤字段是单值 `Option<String>`；见 `residential-monitor/src-tauri/src/c3/query.rs:147-154,183-190`。
- 家宽核算的唯一判据是 `primary_category_id`，配合 `RESIDENTIAL_ACCOUNTING_FILTER = "__residential__"` 与 `RESIDENTIAL_RAW_MEMBERSHIP_SQL`；缺失 identity 的哨兵是 `__unknown__`；见 `residential-monitor/src-tauri/src/c3/sql.rs:10,13,20`。
- 家宽历史归属已由归档任务 `08-31-residential-history-address-stats` 修复，raw 保留期内 `connection_chain` 证据可恢复历史归属。本任务不重做该口径，只读取它。
- **`ReportQuery.top_n` 的合法区间是 1..=100（`TOP_N_MAX`），`validate_query` 强制该上限，`RANK_RAW` 把它直接绑定为 `LIMIT`**；见 `c3/query.rs:18,755-757` 与 `c3/sql.rs:54-67`。UI 口径的 Top N 排名因此不能充当全量审计输入（TPR-02）。
- **`query_residential_share` 已经确立了本仓库的空值语义：无采集覆盖时四个字节字段为 `None`，有覆盖但家宽字节为零时是 `Some(0)`**；见 `c3/share.rs:13-26,66-78` 与其单测 `covered_zero_residential_returns_some_zeros`。未知由 coverage 决定，不由数值是否为零决定（TPR-04）。
- **`plan_capability` 对 raw 保留期外的查询直接返回 `ReportError::CapabilityUnsupported`，不返回结果**；见 `c3/query.rs:814-828`。
- **`query_residential_share` 自行打开连接且签名没有 cancel / deadline 参数；`run_user_vacuum` 只接受 path 与 `SpaceBudget`**；见 `c3/share.rs:28-38` 与 `c5/vacuum.rs:9-20`。两者都不能继承 `ReportService` 的可中断性（TPR-12）。
- **桌面端 `StorageCoordinator` 长期持有一条写连接，但只在每次提交时开 immediate 事务**；见 `storage.rs:264-299,321-355`。因此短命 `BEGIN IMMEDIATE` 探测成功只证明探测瞬间没有写事务，不能证明桌面 writer 不存在（TPR-05）。
- **既有维护预览不含字节数**：`RetentionPreview` 只有 `raw_rows` / `hourly_rows` / `daily_dim_rows` / `daily_core_rows`（`c3/retention.rs:22-33`）；`DeleteItem` 只有 `id` / `kind` / `path` / `exists` / `note_zh`（`c5/purge.rs:8-16`）；`run_user_vacuum` 没有预览 API（`c5/vacuum.rs:9-20`）（TPR-09）。
- 已有维护能力：`RetentionService::preview` / `run`（`c3/retention.rs:40,60`）、`BackupRestoreService::create_backup` / `restore` / `validate_candidate`（`c3/backup.rs:26,78,124`）、`run_user_vacuum`（`c5/vacuum.rs:9`）、`preview_delete` / `confirm_delete`（`c5/purge.rs:45,54`）、`SpaceBudget::check`（`c3/space.rs:22`）。
- 过期 DELETE 不可达：`AUTO_DELETE_ENABLED` 是 `false` 常量，`RetentionService::run` 用它短路删除分支，并有两处编译期断言钉死；见 `c3/query.rs:21,1204` 与 `c3/retention.rs:81,989`。存储契约同时要求「自动 DELETE 保持关闭，直到守恒门通过；不自动 VACUUM」；见 `.trellis/spec/residential-monitor/storage/sqlite-contract.md:22`。
- 存储契约要求单 writer，且未来 schema 或 checksum mismatch 必须 fail closed；见同文件 `:4,14`。
- 脚本侧 `buildInjectedRules()` 已导出，返回当前配置下注入的完整规则字符串清单；见 `clash-verge-ai-residential.js:1746-1799`。
- **`module.exports.constants` 不是完整的开关真源。本机 node 比对（2026-08-31）：`scripts/sync-local-config.js` 的 `routing` 表有 21 个开关，其中 11 个开关常量未出现在 `constants`（共享依赖、Antigravity 三项、`claude_code_auxiliary`、进程与 IP fallback、实时基础设施与端口、公共加密 DNS）；`CORE_SUFFIX_DOMAINS` / `CORE_EXACT_DOMAINS` 也未导出**；见 `clash-verge-ai-residential.js:1224-1267,1747-1797` 与 `scripts/sync-local-config.js:24-53`（TPR-01）。
- **开关到域名不是划分而是覆盖**：`intercom.io` 与 `intercomcdn.com` 同时出现在 `OPENAI_SHARED_SUFFIX_DOMAINS` 与 `CLAUDE_SHARED_SUFFIX_DOMAINS`；见 `clash-verge-ai-residential.js:384-390,407-412`。因此按开关求和必须先定义唯一归属规则（TPR-01）。
- **当前活动规则含一条 `DOMAIN-REGEX`**（Vertex 区域端点 `^[a-z0-9-]+-aiplatform\.googleapis\.com$`）；见 `clash-verge-ai-residential.js:317-320,1261-1267,1348-1353`。`residential-monitor/src-tauri/Cargo.toml:26-44` 没有 regex 直接依赖，但 `regex 1.13.1` 与 `regex-automata`、`regex-syntax`、`aho-corasick`、`memchr` 已由 tauri 传递引入并锁在 `Cargo.lock`。把它提升为直接依赖不新增 lockfile 包，`c5-supply` 的依赖清单只按 lockfile 统计（`c5/supply.rs:28-53`），包数不变；本机复核 2026-08-31，cargo 包数 515（TPR-13）。
- Mihomo 的 `DOMAIN-REGEX` 用 Go `regexp` 做部分匹配，Go `regexp` 与 Rust `regex` 同属 RE2 语义族，`Regex::is_match` 与 `regexp.MatchString` 都是部分匹配，锚点由模式自身的 `^` `$` 决定。
- **数据库只 intern 控制器给出的 rule 类型，不保存 `rulePayload`**；见 `storage.rs:742-775` 与既有断言 `storage.rs:1745`（值为 `"IPCIDR"`）。数据库因此无法还原一条完整的规则文本（TPR-07）。
- 本地 TOML 的 `routing.*` 开关到脚本常量的映射在 `scripts/sync-local-config.js:25-45`；`clash-verge-ai-residential.local.toml` 与 `*.local.js` 均被 gitignore，`*.local.js` 是生成物。
- `.agents/`、`.claude/`、`.codex/`、`.cursor/`、`.omp/`、`.grok/`、`.kimi-code/` 全部在 `.gitignore:26-32`。skill 若只写进这些目录，克隆仓库的人拿不到，因此源文件必须放在被跟踪目录，再安装到平台目录。这些目录当前都已存在且可能含用户本地内容（TPR-10）。
- **根仓库的 Node 脚本、测试与零依赖边界由 `.trellis/spec/frontend/` 拥有**；见 `.trellis/spec/frontend/index.md:20-48` 与 `hook-guidelines.md`、`quality-guidelines.md`（TPR-14）。
- 仓库既有 agent 文档约定是 `docs/agents/*.md` 加 `CLAUDE.md` 的 Agent skills 索引；见 `CLAUDE.md` 的 Agent skills 段与 `docs/agents/`。
- 质量门：根仓库 `just ci`（`npm run check` + `npm test` + `check:secrets`），子项目 `just monitor-check`（版本对齐、前端 check、`cargo fmt --check`、`clippy -D warnings`、`cargo test`）；见 `justfile`。

## Product Decisions

- D1（用户于 2026-08-31 确认）：CLI 用新增 Rust bin 实现，放在 `residential-monitor/src-tauri`，复用现有查询层与家宽口径 SQL，不另起一份实现。
- D2（用户于 2026-08-31 确认，purge 于同日单独确认）：CLI 除只读查询外还要支持维护写操作，且确认覆盖 `maint purge`——它会删除主库、WAL、SHM、报告 spool 与日志目录，不可中断、无回滚。该子命令按 D9 要求 `--offline-confirmed`，并沿用 `c5::purge` 既有的确认短语。
- D3（用户于 2026-08-31 确认）：CLI 的职责边界是 residential-monitor 与其数据库；CLI 不读、不解析、不改写 `clash-verge-ai-residential.js`、`*.local.toml` 与 `*.local.js`。
- D4（用户于 2026-08-31 确认）：首版分析包含家宽 host 排名与占比、死规则检测、越界流量检测、开关级聚合四项。
- D5（TPR-01 修订）：规则清单由 `buildInjectedRules()` 导出，是完整且权威的模式全集；开关映射不是完整真源，因此首版只承诺**已声明的受支持开关集合**，其余开关一律以 `unsupportedSwitch` 状态出现，不写成 0。模式属于多个开关时归入 `shared` 并只计一次。
- D6：过期 raw 与维度层的 DELETE 不在本任务范围。它由 `AUTO_DELETE_ENABLED` 与两处编译期断言钉死，解除需要先过守恒门，属于独立任务。
- D7（用户于 2026-08-31 确认）：首版支持三种模式语法——`DOMAIN`（exact）、`DOMAIN-SUFFIX`（按标签边界的后缀）、`DOMAIN-REGEX`（`regex` crate）。`regex` 从传递依赖提升为直接依赖，不新增 lockfile 包。匹配对小写 host 用 `Regex::is_match`，与 Mihomo 的 `regexp.MatchString` 同为部分匹配。编译失败的模式与未知规则类型进 `unsupportedPattern` 桶，不判为 dead 也不判为 uncovered。
- D8（TPR-07）：审计的真源是「期望模式集合 × 观测 host 字节」。数据库不保存 rulePayload，无法还原完整命中规则文本，因此规划中不存在「观测规则集合」这一概念，也不使用 `unexpected` 这个名字。
- D9（TPR-05）：CLI 不承诺检测桌面 writer 是否存在。它承诺两件可证明的事：对同库写操作在整个操作期间持有 `PRAGMA locking_mode = EXCLUSIVE` 的连接并在冲突时 fail closed；`restore` / `vacuum` / `purge` 是离线命令，要求用户先退出 ResiWatch 并以 `--offline-confirmed` 显式声明，CLI 明示自己不验证该前置条件。
- D10（TPR-04）：数值语义对齐 `c3/share.rs`——未知（无覆盖、能力不支持）用 `null` 加状态字段；已知空集合用 `0`。零流量开关与零命中模式是已知空集合，数值为 `0` 且带 `zeroFlow` 标记，可参与守恒求和。

## Requirements

### CLI 通用

- R1：CLI 是 `residential-monitor/src-tauri` 下的新增 bin，`cargo run` 与新增的 `just` 配方均可调用。它复用既有 `ReportService`、`open_interruptible_reader`、维护服务与家宽口径 SQL，不新增第二份家宽归属判定、不新增 DDL、不改写已发布 migration 文本。
- R2：库路径按 `--db` 显式参数、`RESIDENTIAL_MONITOR_DATA_DIR`、默认安装目录的顺序解析。CLI 不得触发数据目录迁移，也不得在库缺失时创建空库。
- R4：所有查询默认输出机器可读 JSON，并可切换为人读表格。JSON 顶层带 schema 版本、命令名、实际窗口起止、数据版本、能力状态、覆盖度与字段归因状态。表格是同一结果的渲染，不另算数值。
- R5（TPR-04 修订）：输出用逐状态的字段矩阵表达确定性。未知——无采集覆盖、能力不支持、raw 保留期外——用 `null` 加状态字段；已知空集合用 `0` 并带 `zeroFlow` 标记。`__unknown__` 哨兵单独成行，不混入真实 identity。能力不支持时仍返回完整 envelope，不以裸错误代替结果。
- R8：CLI 不输出凭据；对 host、进程 identity 提供脱敏开关，脱敏模式下只输出哈希前缀与计数。完整进程路径任何模式下都不进入输出。
- R11（TPR-12）：所有可能长运行的查询共享一条取消与 deadline 传播路径，并在 SQLite 层实际可中断。不能中断的维护操作必须显式声明为不可中断，给出有界前置条件、失败后的明确状态与人工恢复步骤，不得继承查询路径的可中断性声明。

### 查询命令

- R3（TPR-01 / TPR-02 / TPR-03 / TPR-06 / TPR-07 修订）：查询命令覆盖 D4 的四项分析，并按下列边界交付：
  - `rank`：窗口内 `filters.category="__residential__"` 下按选定维度的 Top N 排名。它是 UI 口径的诊断视图，受 `top_n` 上限 100 约束，**不承诺全窗口守恒，也不返回份额**。
  - `share`：窗口内家宽字节占全部可归因观测的份额，沿用 `query_residential_share` 的空值语义。
  - `audit`：单条命令、单个只读事务、单个 dataVersion 内完成死规则、越界与开关级聚合三项分析，并在同一 envelope 内返回该窗口的家宽总量、覆盖度与份额。它不使用 `top_n`，改用带上限的全量 host 投影；投影的行数上限、超限行为、取消与 deadline 必须显式声明。
  - `audit` 的模式分桶是 `covered` / `dead` / `unsupportedPattern` 三类模式桶加一个 `uncovered` host 桶。模式桶之间互斥，三者并集等于期望模式全集；`uncovered` 是 host 集合，与模式桶不同类，不参与模式集合等式。
  - `audit` 的越界展开必须在同一投影里保留 host、规则类型与进程 identity 的联合键，只对 `uncovered` 子集展开，不把 `covered` 流量计入。
  - `audit` 的开关聚合按 D5 输出 `mapped` / `shared` / `unmapped` / `unsupportedSwitch` 四类，并对未在受支持集合内的开关返回不支持状态而不是 0。
- R12（TPR-01）：受支持开关集合必须是显式声明的清单，skill 侧的生成器要对照 `scripts/sync-local-config.js` 的 `routing` 表做完整性检查：每个 routing 开关要么在受支持集合内，要么出现在不支持清单里，两者数量之和等于该表的开关总数。

### 维护命令

- R6（TPR-09 修订）：维护命令复用既有服务：状态查看、retention 物化与修复、备份、恢复、VACUUM、整体删除本地数据。所有写操作必须显式 `--confirm`；预览必须列出将要改动的对象清单，并按命令声明每项字节数的口径是精确值、上界估计还是未知，不得给出无口径数字。低空间时 fail closed，不覆盖当前可用库。
- R7（TPR-05 修订）：写操作在整个操作期间持有独占连接，冲突时 fail closed 且不改库。`restore` / `vacuum` / `purge` 额外要求 `--offline-confirmed`。CLI 在输出中明示：它不能证明 ResiWatch 已退出，该前置条件由用户保证。
- R13：schema `user_version` 高于本二进制已知版本时，所有子命令 fail closed。

### Skill

- R9（TPR-10 修订）：skill 源文件放在被跟踪目录，由一条安装命令写入本仓库已存在的平台 skill 目录，不创建原本不存在的平台目录，不改其它 skill。目标位置存在同名但内容不同的文件时默认 fail closed；`--force` 才替换，且替换前把原文件备份到带时间戳的副本。
- R10：skill 内容必须覆盖：何时触发；如何生成 CLI 需要的规则清单与开关映射并做 R12 的完整性检查；CLI 子命令的执行顺序；四类结果的判读规则；改动落点——个人调优改本地 TOML 的 `routing.*` 后执行 `just render-local`，公开模板域名清单改动需要官方出处或脱敏 Connections 证据加 negative test 并通过 `just ci`；禁止项——不改 `*.local.js` 生成物、不把真实凭据写进公开模板、不新增宽泛 provider 后缀。

## Acceptance Criteria

每条 AC 后标注它验证的 R。

- [ ] AC1（R1、R2）：`cargo run --bin monitor-db -- --help` 与新增的 `just` 配方都能列出全部子命令；在没有库的路径上执行查询子命令时返回「库不存在」错误与退出码 3，不创建文件、不触发目录迁移。
- [ ] AC2（R3 `rank`、R3 `share`）：给定含家宽会话的 fixture 库，`rank` 返回非空 Top N 排名且无关节点不进入结果；`share` 在同窗口返回份额。测试显式断言 `rank` 的输出中没有份额字段，两条命令不互相承诺一致性。
- [ ] AC3（R3 模式桶、D7、D8）：给定 `buildInjectedRules()` 导出的模式清单，`audit` 把零命中模式列为 `dead`、有命中模式列为 `covered`、编译失败或未知类型的条目列为 `unsupportedPattern`；三个桶两两互斥且并集等于期望模式全集。fixture 同时包含正常命中、零命中、一条 Vertex 区域端点 host（须落进 `covered`）与一条无法编译的正则（须落进 `unsupportedPattern`）。
- [ ] AC4（R3 越界展开）：`audit` 的越界段只展开 `uncovered` host，按规则类型与进程 identity 的联合键给出进程规则、IP 规则与无域名规则三类来源；`covered` 流量不出现在该段；无负数、无重复计数。
- [ ] AC5（R3 开关聚合、R12、D5、D10）：给定受支持开关映射，`audit` 输出满足 `Σmapped + shared + unmapped + unsupportedPattern + uncovered = 窗口内家宽总字节`；零流量的受支持开关字节为 `0` 且带 `zeroFlow`；不受支持的开关返回 `unsupportedSwitch` 状态而不是 0；跨开关重复的模式只在 `shared` 中计一次。
- [ ] AC6（R5、D10）：`audit` 与 `share` 在四种情形下返回可区分状态——无采集覆盖与能力不支持返回 `null` 加状态，无命中与零流量返回 `0` 加 `zeroFlow`，raw 保留期外返回 `capability.supported=false` 的完整 envelope 与退出码 6。测试逐情形断言字段矩阵。
- [ ] AC7（R6、R7、R13）：维护子命令在缺少 `--confirm` 时只预览不写库；预览按声明的口径给出对象与字节字段，不可估算项显式为 `null`；独占连接冲突时 fail closed 且原库可用；`restore` / `vacuum` / `purge` 缺少 `--offline-confirmed` 时拒绝执行；低空间时备份、恢复与 VACUUM 拒绝且不破坏当前库；`user_version` 高于已知版本时全部子命令 fail closed；`retention` 输出显式声明过期 DELETE 处于关闭状态。
- [ ] AC8（R8）：脱敏模式下的输出不含完整 host、完整进程路径与凭据；测试覆盖脱敏前后差异。
- [ ] AC9（R9）：安装命令在本机执行后，已存在的平台 skill 目录中出现同一份 skill，重复执行不产生差异；目标存在同名不同内容文件时默认拒绝并以非零码退出，`--force` 时先生成带时间戳备份再替换；不创建缺失的平台目录；skill 源文件被 git 跟踪。
- [ ] AC10（R10、R12）：skill 内容验收——`SKILL.md` 含触发条件、生成器用法、命令顺序、四类结果判读、改动落点与禁止项六个小节；随附的生成器脚本有测试证明它对 `routing` 表 21 个开关做了完整性检查，受支持与不支持两个清单之和等于 21。
- [ ] AC11（R4）：每个查询命令的 JSON 输出通过 envelope 字段校验（schema 版本、命令名、窗口、dataVersion、capability、coverage、attributionQuality 齐备）；同一结果的 `--format table` 渲染与 JSON 数值一致，测试对至少一个命令逐字段比对。
- [ ] AC12（R11）：查询路径在取消信号置位后于 deadline 内返回可识别的取消错误；不可中断的维护命令在文档与运行时输出中都标注为不可中断，并给出失败后状态与人工恢复步骤。
- [ ] AC13（R1、全量门）：`just monitor-check` 与 `just ci` 通过。既有基线失败与本任务新增失败分开记录。

## Out of Scope

- 不解除 `AUTO_DELETE_ENABLED`，不实现过期 raw 或维度层删除，不改守恒门。
- 不修改 `clash-verge-ai-residential.js` 的现有路由清单与开关默认值；也不为本任务扩充它的导出面。本任务只交付得出改动结论的工具与流程。
- 不引入新的 lockfile 包；`regex` 只从既有传递依赖提升为直接依赖。不模拟 Mihomo 的首个规则命中顺序，CLI 的模式优先级只用于解释字节归属。
- 不承诺检测或终止 ResiWatch 进程。
- 不改 ResiWatch 桌面端界面、告警、报告页与 IPC 契约。
- 不新增云同步、遥测、跨机聚合或通用查询 DSL。
- 不把观测量当作运营商账单，不补齐控制器未提供的隧道内域名或采集缺口。
- 不在规划阶段读取本机真实库中的具体 host、IP 或进程路径。
