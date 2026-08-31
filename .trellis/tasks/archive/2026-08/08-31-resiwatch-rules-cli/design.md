# Design：家宽历史库 CLI 与规则优化 Skill

## 1. 设计目标与边界

- 一条命令让 Agent 在本机读到家宽流量证据，读取路径复用桌面端已发布的家宽口径，不产生第二份归属判定。
- CLI 的知识边界停在 residential-monitor 与其数据库。脚本侧的规则清单与开关映射由 skill 指导 Agent 生成为 JSON 后传入，CLI 只做模式匹配、集合运算与求和。
- 维护写操作只包装既有服务，不新增 DDL、不改已发布 migration 文本、不解除 `AUTO_DELETE_ENABLED`。
- 与桌面端共存：CLI 不污染报告 token 池、不触发数据目录迁移；写操作以独占连接与离线前置条件替代「探测 writer 是否存在」的不可证明主张（D9）。

## 2. 变更清单（TPR-11）

按可独立回退单元列出全部计划编辑。Pass 7 以本表判断实现漂移。

| 单元          | 文件                                                                                                                                    | 动作                                                                                     | 回退方式                                        |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ----------------------------------------------- |
| U1 CLI 入口   | `residential-monitor/src-tauri/Cargo.toml`                                                                                              | 增加 `[[bin]] monitor-db`；把 `regex` 从传递依赖提升为直接依赖（`Cargo.lock` 包集不变）  | 删除该 bin 段与依赖行                           |
| U1 CLI 入口   | `residential-monitor/src-tauri/src/bin/monitor-db.rs`                                                                                   | 新增                                                                                     | 删除文件                                        |
| U1 CLI 入口   | `residential-monitor/src-tauri/src/lib.rs`                                                                                              | 增加 `pub mod dbcli;`                                                                    | 删除该行                                        |
| U2 CLI 逻辑   | `residential-monitor/src-tauri/src/dbcli/`（`mod.rs`、`resolve.rs`、`envelope.rs`、`patterns.rs`、`audit.rs`、`maint.rs`、`render.rs`） | 新增                                                                                     | 删除目录                                        |
| U3 查询层复用 | `residential-monitor/src-tauri/src/c3/service.rs`                                                                                       | 抽出 `run_uncached`；`attach_cancel` 提升为 `pub(crate)`                                 | 内联回原实现并恢复私有可见性                    |
| U3 查询层复用 | `residential-monitor/src-tauri/src/c3/share.rs`                                                                                         | 抽出 `build_share` 的公开入口 `query_residential_share_on(&Connection, …)`，原函数委托它 | 内联回原实现                                    |
| U4 新增 SQL   | `residential-monitor/src-tauri/src/c3/sql.rs`                                                                                           | 新增常量 `AUDIT_RESIDENTIAL_HOST_RULE_PROCESS` 并在 `lookup` 注册                        | 删除常量与注册项                                |
| U5 skill 源   | `skills/residential-rule-tuning/`（`SKILL.md`、`reference.md`、`scripts/build-inputs.js`）                                              | 新增                                                                                     | 删除目录                                        |
| U6 安装器     | `scripts/install-agent-skills.js`、`tests/install-agent-skills.test.js`                                                                 | 新增                                                                                     | 删除文件；已安装副本用 `--check` 定位后手工删除 |
| U6 安装器     | `package.json`                                                                                                                          | `check` 增加 `node --check`，`test` 增加新测试文件                                       | 还原两处脚本串                                  |
| U6 安装器     | `justfile`                                                                                                                              | 增加 `install-skills` 与 `monitor-db` 配方                                               | 删除两个配方                                    |
| U7 规范与文档 | `.trellis/spec/residential-monitor/storage/sqlite-contract.md`、`.trellis/spec/residential-monitor/backend/index.md`                    | 追加条款                                                                                 | 还原追加段落                                    |
| U7 规范与文档 | `docs/agents/residential-rule-tuning.md`、`CLAUDE.md`                                                                                   | 新增一页并加索引                                                                         | 删除该页与索引行                                |

U1–U4 只影响 residential-monitor 子项目；U5–U6 只影响根仓库；U7 是文档。任一单元可单独回退，回退后其余单元仍能通过各自质量门。

bin 名沿用既有 `monitor-bench` 的 `monitor-` 前缀，`-db` 表明它管的是 residential-monitor 的数据库，不是 Clash 脚本。bin 只做 clap 解析与退出码映射；全部逻辑放在 `residential_monitor_lib::dbcli`，使 `cargo test --workspace` 能覆盖。

## 3. 读取路径

### 3.1 命令与事务所有权（TPR-02、TPR-06）

| 命令    | 数据源                                                                | 事务所有权                                                | 守恒承诺                                                                   |
| ------- | --------------------------------------------------------------------- | --------------------------------------------------------- | -------------------------------------------------------------------------- |
| `rank`  | `run_uncached`，`grouping` 可选，`filters.category="__residential__"` | `ReportService` 自己的只读事务                            | 无。它是 Top N 诊断视图，受 `top_n ≤ 100` 约束（`c3/query.rs:18,755-757`） |
| `share` | `query_residential_share`                                             | 自己的只读连接                                            | 无                                                                         |
| `audit` | `dbcli::audit` 全量投影                                               | **一条只读连接、一个 `begin deferred`、一个 dataVersion** | 有，见 4.3                                                                 |

`rank` 不返回份额，`share` 不返回排名。两条命令之间不做一致性承诺；需要一致数据的分析全部由 `audit` 在单事务内产出。

### 3.2 不使用 ReportSnapshotStore

`ReportService::run` 在返回前会把结果写进快照 store（`c3/service.rs:29-60`），而活动 token 上限是 `MAX_ACTIVE_TOKENS = 8`，满了按 `last_access_utc` 淘汰（`c3/query.rs:8`、存储契约 `:19`）。CLI 走这条路径会淘汰桌面端正在用的报告 token 并向 `report-spool` 写文件。

因此在 `c3::service` 内把 `run` 的读取段抽成 `run_uncached(db_path, query, now_utc, raw_retain_days, cancel, deadline)`：打开 `open_interruptible_reader`、`attach_cancel`、`begin deferred`、`build_result`、`commit`、校验读事务已关闭。`ReportService::run` 改为调用它再插入 store，行为不变。CLI 只调 `run_uncached`。

### 3.3 audit 全量投影（TPR-02、TPR-03）

新增 named SQL 常量 `AUDIT_RESIDENTIAL_HOST_RULE_PROCESS`，在 `sql::lookup` 注册，家宽谓词用既有 `render_residential_membership_sql`（`c3/sql.rs:304`）注入，窗口与行数上限走绑定参数：

```sql
select
  case when coalesce(h.value,'') = '' then '__unknown__' else h.value end,
  coalesce(r.value, '__unknown__'),
  case when coalesce(p.value,'') = '' then '__unknown__' else p.value end,
  coalesce(sum(m.upload), 0), coalesce(sum(m.download), 0),
  count(distinct m.session_pk)
from connection_minute m
join connection_session s on s.session_pk = m.session_pk
left join connection_session_attr a on a.session_pk = m.session_pk
left join dimension_dict h on h.dimension_kind = 'host'    and h.dimension_id = a.host_id
left join dimension_dict r on r.dimension_kind = 'rule'    and r.dimension_id = a.rule_id
left join dimension_dict p on p.dimension_kind = 'process' and p.dimension_id = a.process_id
where m.utc_minute >= ?1 and m.utc_minute < ?2 and {residential_membership}
group by 1, 2, 3
order by 4 + 5 desc
limit ?3
```

- 一行同时携带 host、规则类型与进程 identity 的**联合键**，所以模式判定与越界展开都在同一份数据上做，不需要把 Rust 侧算出的 host 集合再塞回 SQL（TPR-03）。`ReportFilters` 是单值字段（`c3/query.rs:183-190`），本投影不使用它。
- 行数上限常量 `AUDIT_MAX_ROWS = 200_000` 作为第三个绑定参数。返回行数等于上限时置 `truncation.status = "truncated"`，并把全部守恒字段按 D10 写成 `null` 加状态——截断窗口不谎称守恒。
- 同一事务内另外执行 `COVERAGE_RAW` 与 `SHARE_RESIDENTIAL_RAW`，得到窗口覆盖度与家宽/可归因总量。`SHARE_RESIDENTIAL_RAW` 的分子与本投影用同一个家宽谓词，因此「投影字节之和 = 份额分子」是可断言的等式。
- 取消与 deadline：投影在 `open_interruptible_reader` 打开的连接上执行，执行前调用 `attach_cancel`（提升为 `pub(crate)`），与 `run_uncached` 同一套机制（R11、TPR-12）。
- 能力：调用前先用既有 `plan_capability` 判定 raw 可用性；返回 `CapabilityUnsupported` 时按 6.2 的矩阵转成 envelope，不向上抛裸错误（TPR-04）。
- 份额复用：在 `c3/share.rs` 抽出 `query_residential_share_on(&Connection, …)`，`audit` 在自己的连接上调用它，从而与投影共享事务、覆盖度语义与 `None` / `Some(0)` 约定；原 `query_residential_share` 委托新函数，行为不变（TPR-06、TPR-12）。

## 4. 脚本知识的注入方式

### 4.1 输入契约（TPR-01）

CLI 不读 `clash-verge-ai-residential.js`。skill 附带的生成器 `skills/residential-rule-tuning/scripts/build-inputs.js` 产出两份 JSON：

```jsonc
// rules.json —— 模式全集，来自 buildInjectedRules()，权威且完整
{ "schemaVersion": 1, "group": "AI-家宽",
  "rules": ["DOMAIN-SUFFIX,claude.ai,AI-家宽", "DOMAIN,api.anthropic.com,AI-家宽", "..."] }

// switches.json —— 开关映射，显式声明支持范围
{ "schemaVersion": 1,
  "supported":   { "openai_core": ["chatgpt.com", "oaiusercontent.com", "api.openai.com"] },
  "unsupported": ["openai_shared_dependencies", "claude_shared_dependencies", "..."] }
```

`rules.json` 由 `buildInjectedRules()` 生成，是当前配置下实际注入规则的完整清单，可作为模式全集。

`switches.json` 不是完整真源：21 个 routing 开关中有 11 个的常量未出现在 `module.exports.constants`，`CORE_SUFFIX_DOMAINS` / `CORE_EXACT_DOMAINS` 也未导出。因此生成器按 D5 显式分成两个清单，并对照 `scripts/sync-local-config.js` 的 `routing` 表做完整性检查：`supported` 的键数加 `unsupported` 的长度必须等于该表开关数，否则生成器以非零码退出（R12）。CLI 侧只信任这两个清单，不推断缺失开关。

### 4.2 模式语法与匹配语义（TPR-13）

| 规则前缀                                              | CLI 处理       | 语义                                                                                  |
| ----------------------------------------------------- | -------------- | ------------------------------------------------------------------------------------- |
| `DOMAIN`                                              | exact          | host 与模式全等（小写比较）                                                           |
| `DOMAIN-SUFFIX`                                       | suffix         | host 等于模式，或以 `.` + 模式结尾。按标签边界匹配，`notclaude.ai` 不命中 `claude.ai` |
| `DOMAIN-REGEX`                                        | regex          | 用 `regex` crate 对小写 host 做 `is_match`；Go `regexp.MatchString` 与它同为 RE2 语义族的部分匹配，锚点由模式自身的 `^` `$` 决定 |
| 编译失败或类型未知的条目                              | 单列一桶       | 进 `unsupportedPattern`，不参与匹配，不误报为 dead 或 uncovered                       |
| `PROCESS-*`、`IP-CIDR`、`IP-CIDR6`、`DST-PORT`、`AND` | 非域名兜底机制 | 不参与 host 匹配，登记为兜底机制清单供越界段引用                                      |

`regex 1.13.1` 与其传递依赖已由 tauri 锁在 `Cargo.lock`，提升为直接依赖不新增 lockfile 包，`c5-supply` 按 lockfile 统计的包数不变（`c5/supply.rs:28-53`）。当前活动清单里的 Vertex 区域端点正则（`clash-verge-ai-residential.js:317-320`）因此进入 `covered`，不落兜底桶。

匹配优先级 exact > 最长 suffix > regex > 输入顺序。**该优先级只是本工具的字节归属规则，不模拟 Mihomo 的首个规则命中**；报告解释的是「这些字节可以归到哪条模式」，不是「内核当时选了哪条规则」。

### 4.3 分桶与守恒（TPR-07、TPR-01）

模式桶（三者互斥，并集 = `rules.json` 中的域名模式全集）：

- `covered`：窗口内有 host 命中的模式，带命中 host 数与上下行字节。
- `dead`：窗口内零命中的模式，字节为 `0` 且带 `zeroFlow`（已知空集合，D10）。
- `unsupportedPattern`：编译失败或类型未知的模式条目；其字节按 4.2 单列。

host 桶：

- `uncovered`：走了家宽但不匹配任何受支持模式的 host。它是 host 集合，**与模式桶不同类，不参与模式集合等式**，也不叫 `unexpected`——数据库不保存 rulePayload，无法还原「实际命中的完整规则」，因此规划中不存在「观测规则集合」（D8）。

开关桶（基于 `covered` 的 per-pattern 字节再聚合）：

- `mapped`：模式唯一属于某个受支持开关。
- `shared`：模式同时属于两个及以上受支持开关（例如 `intercom.io` 同时在 OpenAI 与 Claude 的共享依赖清单里），字节只在此桶计一次，并列出涉及的开关名。
- `unmapped`：模式命中但不属于任何受支持开关，例如始终启用的 Claude 核心域。
- `unsupportedSwitch`：`switches.json` 的 `unsupported` 清单，只列名与状态，不给数值。

守恒式（未截断且能力可用时断言）：

```
Σcovered(=Σmapped + shared + unmapped) + ΣunsupportedPattern + Σuncovered = 窗口内家宽总字节
```

`dead` 恒为 0 不影响等式。截断或能力不支持时，等式两侧字段按 D10 置 `null` 并标状态，不做断言。

### 4.4 越界展开（TPR-03）

`audit` 的越界段只对 `uncovered` 的 host 集合展开，数据直接取自 3.3 投影中这些 host 的行，按规则类型与进程 identity 的联合键分组，输出三类来源：进程规则（规则类型以 `Process` 开头）、IP 规则（`IPCIDR` / `IPCIDR6`）、其余（含 `__unknown__`）。`covered` 的行不进入该段。

## 5. 维护写路径

### 5.1 子命令与既有服务的对应

| 子命令            | 包装                                                                               | 写库                   | 离线要求                                    |
| ----------------- | ---------------------------------------------------------------------------------- | ---------------------- | ------------------------------------------- |
| `maint status`    | schema `user_version`、库/WAL 大小、freelist、`retention_watermark`、`SpaceBudget` | 否                     | 否                                          |
| `maint retention` | `RetentionService::preview` / `run`（`c3/retention.rs:40,60`）                     | 是（物化与修复）       | 否，但全程持独占连接                        |
| `maint backup`    | `BackupRestoreService::create_backup`（`c3/backup.rs:26`）                         | 否（只写目标文件）     | 否                                          |
| `maint restore`   | `validate_candidate` + `restore`（`c3/backup.rs:78,124`）                          | 是                     | 是                                          |
| `maint vacuum`    | `run_user_vacuum`（`c5/vacuum.rs:9`）                                              | 是                     | 是                                          |
| `maint purge`     | `preview_delete` / `confirm_delete`（`c5/purge.rs:45,54`）                         | 是（删除全部本地数据） | 是；另需 `c5::purge` 既有确认短语        |

`maint retention` 的输出必须显式打印 `auto_delete_enabled=false` 与既有说明「DELETE 后的 freelist 不是已释放文件空间」。过期 DELETE 不实现，见 PRD D6。

### 5.2 独占与离线前置条件（TPR-05）

桌面端 `StorageCoordinator` 长期持有写连接但只在提交时开 immediate 事务（`storage.rs:264-299,321-355`），因此短命 `BEGIN IMMEDIATE` 探测既有 TOCTOU，也无法证明桌面 writer 不存在。改为两条可证明的机制：

1. **全程独占**：`maint retention` 由 CLI 自建 `StorageCoordinator`，在 `apply_required_pragmas` 之后、任何写事务之前设置 `PRAGMA locking_mode = EXCLUSIVE`，并在整个 `RetentionService::run` 期间持有该连接。取不到独占锁时以退出码 4 fail closed，不做重试。`RetentionService::run` 内部的多个 immediate 事务因此都在同一把独占锁下执行。
2. **离线声明**：`restore`、`vacuum`、`purge` 涉及文件重命名、删除或不可中断的整库重写，SQLite 锁不覆盖其全部生命周期。它们要求 `--offline-confirmed`，并在输出中明示「CLI 不验证 ResiWatch 是否已退出，该前置条件由执行者保证」。`vacuum` 在打开连接后同样先取独占锁，取不到即 fail closed。

R7 与 AC7 按此收窄：CLI 不再声称「检测 writer 是否存在」。

### 5.3 预览字段与字节口径（TPR-09）

既有预览结构不含字节数，因此在 `dbcli::maint` 内建一层预览适配，口径逐命令声明，不给无口径数字：

| 子命令      | 对象清单来源                                                              | 字节字段                     | 口径                                                                           |
| ----------- | ------------------------------------------------------------------------- | ---------------------------- | ------------------------------------------------------------------------------ |
| `retention` | `RetentionPreview` 的四个行数（`c3/retention.rs:22-33`）                  | `bytes: null`                | 未知。物化与修复的写入量不可预估，显式为 `null` 并给出行数                     |
| `backup`    | 目标路径                                                                  | `estimatedBytes`             | 上界估计 = 主库文件大小 + WAL 文件大小                                         |
| `restore`   | 候选文件与将被替换的 live 库、WAL、SHM                                    | `bytes`                      | 精确值，来自 `fs::metadata`                                                    |
| `vacuum`    | 主库路径                                                                  | `reclaimableUpperBoundBytes` | 上界估计 = `freelist_count × page_size`                                        |
| `purge`     | `preview_delete` 的 `DeleteItem` 列表（`c5/purge.rs:8-16`，无 size 字段） | `bytes`                      | 精确值，CLI 对每个 `path` 做 `fs::metadata`，目录递归求和；不存在的项为 `null` |

`maint status` 复用同一批口径给出只读快照。

### 5.4 取消与不可中断声明（TPR-12）

- 查询路径（`rank`、`share`、`audit`）统一 `attach_cancel` + deadline，SQLite 层实际可中断。
- `retention` 内部使用既有 `cancel: &Arc<AtomicBool>` 参数，CLI 接 Ctrl-C 信号置位。
- `run_user_vacuum` 与 `confirm_delete` 无取消参数，**声明为不可中断**。`vacuum` 在执行前按主库大小给出预计时长区间；超过声明阈值时需要 `--allow-long`。两者的运行时输出与文档都必须写明：中途终止进程后的状态与人工恢复步骤（`vacuum` 失败保留原库，`purge` 按 `DeleteReport` 的分项结果逐项确认）。

### 5.5 库路径解析

`dbcli::resolve_existing_db(explicit: Option<&Path>) -> Result<PathBuf, DbCliError>`：按 `--db` → `RESIDENTIAL_MONITOR_DATA_DIR`（`data_dir.rs:12`）→ 默认安装目录顺序取第一个存在的 `monitor.sqlite3`。不调用 `prepare_data_dir` / `resolve_and_migrate`（`data_dir.rs:33,45`），因此不会触发迁移；库不存在时返回明确错误而不是创建空库。

## 6. 输出契约

### 6.1 envelope

```jsonc
{
  "schemaVersion": 1,
  "command": "audit",
  "generatedUtc": 1756600000,
  "window": {
    "startUtc": 0,
    "endUtc": 0,
    "timezone": "Asia/Shanghai",
    "granularity": "hour",
  },
  "dataVersion": 42,
  "capability": { "layer": "raw", "supported": true, "reason": null },
  "coverage": { "observedSec": 0, "gapSec": 0, "status": "covered" },
  "attributionQuality": {
    "status": "partial",
    "knownBytes": 0,
    "missingBytes": null,
  },
  "truncation": { "status": "complete", "rowCap": 200000, "rows": 1234 },
  "namedSql": [
    "coverage_raw",
    "share_residential_raw",
    "audit_residential_host_rule_process",
  ],
  "result": {},
  "notes": [],
}
```

`--format json`（默认）与 `--format table`。表格是同一 `result` 的渲染，不另算数值。`namedSql` 回显实际执行的常量名，符合存储契约 `:25`。

### 6.2 状态字段矩阵（TPR-04）

对齐 `c3/share.rs` 已确立的语义：未知看 coverage 与 capability，不看数值是否为零。

| 情形                              | `capability.supported`                            | `coverage.status`     | 数值字段                                             | `zeroFlow` | 退出码 |
| --------------------------------- | ------------------------------------------------- | --------------------- | ---------------------------------------------------- | ---------- | ------ |
| 有覆盖且有流量                    | `true`                                            | `covered` / `partial` | 实际值                                               | `false`    | 0      |
| 有覆盖但家宽零流量（已知空集合）  | `true`                                            | `covered` / `partial` | `0`                                                  | `true`     | 0      |
| 无采集覆盖（未知）                | `true`                                            | `uncovered`           | `null`                                               | 不适用     | 0      |
| raw 保留期外 / 能力不支持（未知） | `false`，`reason` 填 `plan_capability` 的错误文本 | 按实际                | `null`                                               | 不适用     | 6      |
| 投影截断（未知）                  | `true`                                            | 按实际                | 守恒相关字段 `null`，`truncation.status="truncated"` | 不适用     | 0      |

能力不支持时仍在 stdout 输出完整 envelope，退出码为 6；调用方既能读到原因，也能用退出码分支。零流量的 `dead` 模式与受支持开关是已知空集合，数值为 `0`，参与 4.3 的守恒等式；这与「未知不得写成零」不冲突，因为未知走的是 `null` 分支。

### 6.3 退出码

`0` 成功；`2` 参数错误；`3` 库不存在；`4` 独占锁冲突或库忙；`5` fail closed（空间、schema、校验、离线声明缺失）；`6` 能力不支持；`7` 取消。

## 7. 脱敏

`--redact` 打开后，host 与进程 identity 只输出 `sha256` 前 8 位加长度，字节与计数照常。完整进程路径任何模式下都不进入输出（沿用 `CONTEXT.md` 的进程 identity 定义）。默认不脱敏，因为 Agent 需要真实 host 才能判断域名清单；skill 必须写明：任何要贴进 issue、PR 或对话记录的输出都用 `--redact` 重跑。

## 8. Skill 打包与安装

平台目录 `.agents/`、`.claude/`、`.codex/`、`.cursor/`、`.omp/`、`.grok/`、`.kimi-code/` 全部被 `.gitignore:26-32` 排除，因此源文件放 `skills/residential-rule-tuning/`，用 `node scripts/install-agent-skills.js`（`just install-skills`）安装。

安装器遵循 `.trellis/spec/frontend/` 的 Node CommonJS、零依赖与入口副作用约定（TPR-14）。

冲突策略（TPR-10）：

| 目标状态       | 默认行为                                                    | `--force` 行为                             |
| -------------- | ----------------------------------------------------------- | ------------------------------------------ |
| 文件不存在     | 写入                                                        | 写入                                       |
| 存在且内容一致 | 跳过，不重写                                                | 跳过                                       |
| 存在且内容不同 | **fail closed**，列出差异路径，非零退出，不写入任何目标目录 | 先复制为 `<name>.bak-<UTC 时间戳>`，再写入 |
| 平台目录不存在 | 跳过该平台，不创建                                          | 跳过                                       |

`--check` 只报告差异并以非零码退出，供 CI 使用；它不是正常安装路径的冲突处理手段。安装器不改平台目录中的其它 skill。

skill 内容分三部分：`SKILL.md` 给触发条件、生成器用法、命令顺序、四类结果判读、改动落点与禁止项；`reference.md` 给判读规则细节；`scripts/build-inputs.js` 生成 4.1 的两份 JSON 并执行 R12 的完整性检查。

## 9. 安全、兼容与回滚

- 无 schema 变更、无 DDL、无 migration 改动。数据库侧新增只有一条 named SQL 常量与其 lookup 注册（U4）。
- 桌面端行为不变：`ReportService::run` 与 `query_residential_share` 语义保持，只是内部委托新抽出的函数（U3）。
- CLI 是新 bin，默认构建产物增加一个可执行文件；不进 NSIS 安装包，只在仓库内经 `cargo run` 或 `just` 配方使用。
- 回滚按第 2 节的 U1–U7 逐单元执行。已安装到平台目录的 skill 副本不受 git 回滚影响，用安装器 `--check` 定位后手工删除；`--force` 产生的 `.bak-*` 备份同样需要手工处理。

## 10. Spec changes required

- `.trellis/spec/residential-monitor/storage/sqlite-contract.md`：登记 `AUDIT_RESIDENTIAL_HOST_RULE_PROCESS`、`AUDIT_MAX_ROWS` 截断语义、CLI 的独占锁与离线前置条件、CLI 不使用 `ReportSnapshotStore`。
- `.trellis/spec/residential-monitor/backend/index.md`：登记 `dbcli` 模块边界、`run_uncached` 与 `query_residential_share_on`。
- `.trellis/spec/residential-monitor/backend/secrets-and-cancellation.md`：登记不可中断维护命令的声明义务与脱敏输出规则。
- `CLAUDE.md` 的 Agent skills 段与 `docs/agents/residential-rule-tuning.md`：登记新 skill 与安装命令。
