# Implement：家宽历史库 CLI 与规则优化 Skill

顺序执行。每步末尾的命令必须先跑通再进下一步。除非另行说明，工作目录是仓库根。步骤后的括号标注它交付的 R 与 AC。

## 1. CLI 骨架、库路径解析与 just 入口（U1、R1、R2 → AC1）

- [ ] `residential-monitor/src-tauri/Cargo.toml` 增加 `[[bin]] monitor-db`，路径 `src/bin/monitor-db.rs`；把 `regex` 从传递依赖提升为直接依赖，并确认 `Cargo.lock` 的包集不变。
- [ ] 新建 `src/dbcli/mod.rs`，在 `lib.rs` 注册 `pub mod dbcli;`。
- [ ] `dbcli::resolve_existing_db`：`--db` → `RESIDENTIAL_MONITOR_DATA_DIR` → 默认安装目录，取第一个存在的 `monitor.sqlite3`。不得调用 `prepare_data_dir` / `resolve_and_migrate`；库缺失返回 `DbCliError::DatabaseMissing`。
- [ ] `dbcli::DbCliError` 与退出码映射：`2` 参数、`3` 库不存在、`4` 独占锁冲突或忙、`5` fail closed、`6` 能力不支持、`7` 取消。
- [ ] bin 只做 clap 解析与退出码映射，逻辑全部在 lib。
- [ ] `justfile` 增加 `monitor-db` 配方，转发参数到 `cargo run --quiet --bin monitor-db --`。
- [ ] 单测：库不存在时不创建文件、不迁移目录；三种来源的优先级。
- [ ] AC1 验收：`--help` 与 `just monitor-db --help` 都列出全部子命令。

```bash
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib dbcli::
just monitor-db --help
```

## 2. 查询层复用与 envelope（U3、R4、R5、R11 → AC11、AC6 前置）

- [ ] `c3/service.rs` 抽出 `run_uncached`，`ReportService::run` 改为委托它再写 store；`attach_cancel` 提升为 `pub(crate)`。
- [ ] `c3/share.rs` 抽出 `query_residential_share_on(&Connection, …)`，原 `query_residential_share` 委托它。
- [ ] `dbcli::envelope`：`schemaVersion` / `command` / `generatedUtc` / `window` / `dataVersion` / `capability` / `coverage` / `attributionQuality` / `truncation` / `namedSql` / `result` / `notes`。
- [ ] 实现 design 6.2 的状态字段矩阵：未知走 `null` 加状态，已知空集合走 `0` 加 `zeroFlow`；`CapabilityUnsupported` 转成 `capability.supported=false` 的完整 envelope 并返回退出码 6。
- [ ] `--format json|table`、`--since/--until/--last/--tz` 时间窗参数与校验；表格渲染与 JSON 同源。
- [ ] 取消与 deadline：Ctrl-C 置位 `Arc<AtomicBool>`，查询路径统一 `attach_cancel`。
- [ ] 回归：`ReportService::run` 与 `query_residential_share` 既有测试全部通过，证明两处抽取无行为变化。
- [ ] AC11 验收：至少一个命令的 `--format table` 与 JSON 逐字段比对。

```bash
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c3::
```

## 3. rank 与 share（R3 → AC2）

- [ ] `monitor-db rank --by host|process|chain|rule|network|category [--residential] [--top N]`，走 `run_uncached`。
- [ ] `monitor-db share`，走 `query_residential_share`。
- [ ] `rank` 输出中不含份额字段；两条命令不互相承诺一致性。
- [ ] `__unknown__` 单独成行并标注，不与真实 identity 合并。
- [ ] AC2 验收：fixture 中 `rank` 返回非空 Top N 且无关节点不进入结果；`share` 同窗口返回份额；断言 `rank` 输出无份额字段。

## 4. audit 全量投影（U4、R3、R11 → AC4 前置、AC6）

- [ ] `c3/sql.rs` 新增 `AUDIT_RESIDENTIAL_HOST_RULE_PROCESS` 与 `AUDIT_MAX_ROWS = 200_000`，在 `lookup` 注册；窗口与行数上限走绑定参数，家宽谓词用 `render_residential_membership_sql`。
- [ ] `dbcli::audit` 在一条 `open_interruptible_reader` 连接、一个 `begin deferred` 内依次执行 `plan_capability`、`COVERAGE_RAW`、`query_residential_share_on`、投影 SQL，并回显 `namedSql`。
- [ ] 截断处理：返回行数等于 `AUDIT_MAX_ROWS` 时置 `truncation.status="truncated"`，守恒相关字段写 `null`。
- [ ] 断言「投影字节之和 = 份额分子」（未截断时）。
- [ ] AC6 验收：无覆盖、零流量、能力不支持、raw 期外、截断五种情形逐一断言字段矩阵与退出码。

## 5. 模式解析、分桶与守恒（R3、D7、D8 → AC3）

- [ ] `dbcli::patterns`：`DOMAIN` → exact、`DOMAIN-SUFFIX` → 按标签边界的 suffix、`DOMAIN-REGEX` → `regex::Regex` 对小写 host 做 `is_match`；编译失败或类型未知的条目 → `unsupportedPattern`；`PROCESS-*` / `IP-CIDR` / `IP-CIDR6` / `DST-PORT` / `AND` 登记为非域名兜底机制。目标组不匹配的条目跳过并计数。
- [ ] 匹配优先级 exact > 最长 suffix > regex > 输入顺序；每个 host 至多归属一个模式。代码注释与输出说明写明这是字节归属规则，不模拟 Mihomo 首个规则命中。
- [ ] `audit` 输出模式桶 `covered` / `dead` / `unsupportedPattern` 与 host 桶 `uncovered`；`dead` 字节为 `0` 且带 `zeroFlow`。
- [ ] 不使用 `unexpected` 这个名字；`uncovered` 是 host 集合，不参与模式集合等式。
- [ ] AC3 验收：fixture 含正常命中、零命中、一条 Vertex 区域端点 host（须进 `covered`）与一条无法编译的正则（须进 `unsupportedPattern`）；断言三个模式桶两两互斥、并集等于期望模式全集。
- [ ] 负测：`notclaude.ai` 不命中 `DOMAIN-SUFFIX,claude.ai`；`us-central1-aiplatform.googleapis.com` 命中 Vertex 正则而 `aiplatform.googleapis.com` 不命中该正则（它由 exact 模式覆盖）。

## 6. 越界展开（R3 → AC4）

- [ ] `audit` 的越界段只对 `uncovered` host 展开，数据取自第 4 步投影中这些 host 的行，按规则类型与进程 identity 的联合键分组。
- [ ] 输出三类来源：进程规则、IP 规则、其余（含 `__unknown__`）。
- [ ] AC4 验收：fixture 覆盖三类来源；断言 `covered` 行不进入该段、无负数、无重复计数。

## 7. 开关聚合（R3、R12、D5 → AC5）

- [ ] `audit --map switches.json` 读入 `supported` / `unsupported` 两个清单。
- [ ] 输出 `mapped` / `shared` / `unmapped` / `unsupportedSwitch` 四桶；跨开关重复的模式只在 `shared` 计一次并列出涉及开关名。
- [ ] 受支持但零流量的开关字节为 `0` 且带 `zeroFlow`；不受支持的开关只给状态不给数值。
- [ ] AC5 验收：断言 `Σmapped + shared + unmapped + ΣunsupportedPattern + Σuncovered = 窗口内家宽总字节`；fixture 含一个跨开关重复模式（如 `intercom.io`）。

## 8. 脱敏（R8 → AC8）

- [ ] `--redact`：host 与进程 identity 输出 `sha256` 前 8 位加长度；完整进程路径任何模式都不输出。
- [ ] AC8 验收：脱敏前后差异测试。

## 9. 维护子命令（R6、R7、R13 → AC7、AC12）

- [ ] `maint status`：schema `user_version`、库 / WAL 大小、freelist、`retention_watermark`、空间预算。只读。
- [ ] `maint retention` / `backup` / `restore` / `vacuum` / `purge`，全部包装既有服务，不新写业务 SQL。
- [ ] 预览适配层按 design 5.3 的表逐命令给出对象清单与字节口径：`retention` 的 `bytes` 为 `null` 并给行数；`backup` 给上界估计；`restore` 与 `purge` 用 `fs::metadata` 给精确值；`vacuum` 给 `freelist_count × page_size` 上界。
- [ ] 独占：`retention` 与 `vacuum` 在写前设置 `PRAGMA locking_mode = EXCLUSIVE` 并全程持有连接，冲突退 `4`。
- [ ] 离线：`restore` / `vacuum` / `purge` 要求 `--offline-confirmed`，输出明示 CLI 不验证 ResiWatch 是否已退出。
- [ ] 不可中断声明：`vacuum` 与 `purge` 在输出中标注不可中断，给出失败后状态与人工恢复步骤；`vacuum` 超过声明的库大小阈值时要求 `--allow-long`。
- [ ] `user_version` 高于已知版本时全部子命令 fail closed。
- [ ] `maint retention` 输出显式声明 `auto_delete_enabled=false` 与 freelist 说明。
- [ ] `maint purge` 除 `--confirm` 与 `--offline-confirmed` 外，还要求输入 `c5::purge` 的既有确认短语（`DELETE_CONFIRM_PHRASE`）。
- [ ] AC7、AC12 验收：缺 `--confirm` 不写库；独占冲突 fail closed 且原库可用；缺 `--offline-confirmed` 拒绝；低空间下备份 / 恢复 / VACUUM 拒绝且原库可用；取消信号在 deadline 内返回取消错误。

```bash
cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check
cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace
```

## 10. Skill 源与生成器（U5、R10、R12 → AC10）

- [ ] `skills/residential-rule-tuning/SKILL.md`，六个小节：触发条件、生成器用法、命令顺序、四类结果判读、改动落点、禁止项。
- [ ] `skills/residential-rule-tuning/reference.md`：判读规则细节；改动落点——本地 TOML `routing.*` → `just render-local`，公开模板域名清单 → 官方出处或脱敏证据 + negative test + `just ci`；禁止项——不改 `*.local.js`、不把真实凭据写进公开模板、不新增宽泛 provider 后缀。
- [ ] `skills/residential-rule-tuning/scripts/build-inputs.js`：由 `buildInjectedRules()` 生成 `rules.json`；由 `constants` 与显式受支持清单生成 `switches.json` 的 `supported` / `unsupported`；对照 `scripts/sync-local-config.js` 的 `routing` 表做完整性检查，两个清单之和不等于开关总数时非零退出。遵循零依赖 CommonJS 约定。
- [ ] AC10 验收：`SKILL.md` 六小节齐备的结构测试；生成器测试断言完整性检查覆盖 `routing` 表全部开关。

## 11. 安装器（U6、R9 → AC9）

- [ ] `scripts/install-agent-skills.js`：只写入已存在的平台 skill 目录，不创建缺失目录，不改其它 skill。
- [ ] 冲突策略：不存在则写入；内容一致则跳过；**内容不同默认 fail closed 并非零退出，不写入任何目标目录**；`--force` 时先复制为 `<name>.bak-<UTC 时间戳>` 再写入。
- [ ] `--check` 只报告差异并非零退出，供 CI 使用。
- [ ] `justfile` 增加 `install-skills` 配方；`package.json` 的 `check` 增加 `node --check`，`test` 增加新测试文件。
- [ ] `tests/install-agent-skills.test.js`：幂等、不创建缺失平台目录、同名不同内容默认拒绝、`--force` 生成备份后替换、`--check` 差异时非零退出。
- [ ] AC9 验收：上述测试全部通过；`skills/` 被 git 跟踪。

```bash
just install-skills
node --test tests/install-agent-skills.test.js
```

## 12. Spec、文档与全量门（U7 → AC13）

- [ ] `.trellis/spec/residential-monitor/storage/sqlite-contract.md`：登记 `AUDIT_RESIDENTIAL_HOST_RULE_PROCESS`、`AUDIT_MAX_ROWS` 截断语义、独占锁与离线前置条件、CLI 不使用 `ReportSnapshotStore`。
- [ ] `.trellis/spec/residential-monitor/backend/index.md`：登记 `dbcli` 边界、`run_uncached` 与 `query_residential_share_on`。
- [ ] `.trellis/spec/residential-monitor/backend/secrets-and-cancellation.md`：登记不可中断维护命令的声明义务与脱敏输出规则。
- [ ] `docs/agents/residential-rule-tuning.md` 新增；`CLAUDE.md` 的 Agent skills 段加索引。
- [ ] 全量门：

```bash
just monitor-check
just ci
git diff --check
```

- [ ] AC13 验收：既有基线失败与本任务新增失败分开记录。

## 13. Start 前门（已于 2026-08-31 全部确认）

- [x] PRD、design、implement 三份文档已由用户复核并按审阅报告 TPR-01…TPR-14 修订。
- [x] bin 名 `monitor-db`、skill 名 `residential-rule-tuning`。
- [x] 允许三处既有代码抽取：`c3/sql.rs` 新增 named SQL 与常量、`c3/service.rs` 抽出 `run_uncached` 并提升 `attach_cancel` 可见性、`c3/share.rs` 抽出 `query_residential_share_on`。
- [x] 默认输出不脱敏，贴出前用 `--redact` 重跑，该条写进 skill。
- [x] D2 覆盖 `maint purge`，纳入首版交付。
- [x] D7 改为支持 `DOMAIN-REGEX`，`regex` 从传递依赖提升为直接依赖，lockfile 包集不变。
- [x] D5 保持收窄：只对受支持开关求和，其余返回 `unsupportedSwitch`。

## 回滚点

按 design 第 2 节的 U1–U7 逐单元回退：

- U1、U2 删除新增文件与两处注册行即可。
- U3 把 `run_uncached`、`query_residential_share_on` 内联回原实现并恢复可见性。
- U4 删除新增常量与其 `lookup` 注册项，既有查询不受影响。
- U5、U6 删除 `skills/`、`scripts/install-agent-skills.js`、测试与两个 justfile 配方，并还原 `package.json` 的两处脚本串；已安装到平台目录的副本与 `.bak-*` 备份用安装器 `--check` 定位后手工删除。
- U7 还原追加的 spec 段落与文档页。
