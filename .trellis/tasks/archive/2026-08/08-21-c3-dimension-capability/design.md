# 设计：C3 维度查询与物化能力

## 1. 派生键的单一实现

新模块 `src-tauri/src/c3/rule_name.rs`：

```rust
/// 顶层策略组：chain_key 形如 "a>b>c"，取最后一段并 trim。
/// 无 '>' 视为单跳，返回 None（调用方回退到 rule(payload)）。
pub fn last_chain_hop(chain_key: Option<&str>) -> Option<String>;

/// 与 neko buildRuleName 等价：
///   chains.len() > 1 && last_hop 非空  -> last_hop
///   rule 非空                          -> rule(payload) 或 rule
///   否则                               -> last_hop 或 "DIRECT"
pub fn build_rule_name(rule: Option<&str>, payload: Option<&str>, chain_key: Option<&str>) -> String;
```

`last_chain_hop` 通过 `Connection::create_scalar_function` 注册为 SQLite 标量函数，名字 `last_chain_hop`，`Deterministic | Innocuous`。raw 层排名、维度层物化、`filters.chain` 匹配三处共用它，满足「判定只有一处」。

注册点：打开连接的地方（`sqlite_probe::apply_required_pragmas` 之后）。所有走 `StorageCoordinator` 的连接都必须注册，否则物化 SQL 会在运行时报 no such function——**这是本子任务最容易漏的一处，必须有一条测试从 `StorageCoordinator` 新建连接后直接执行含该函数的 SQL**。

## 2. filters 注入的统一形状

现有 `TOTALS_RAW` 用「开关 + 值」成对参数（`?3/?4` host、`?5/?6` process…）。这种手工编号在六个维度、五条 SQL 上会难以维护。

改为：在 Rust 侧生成过滤片段与绑定值。

```rust
/// 返回 (sql_fragment, params)。fragment 形如 " and a.process_id = (...)"，无过滤时为空串。
fn filter_clause(filters: &ReportFilters) -> (String, Vec<Box<dyn ToSql>>);
```

`named_sql` 回显的仍是常量名（`totals_raw` 等），片段是运行期拼装。**片段只由枚举驱动**：六个字段各一段固定文本，值走绑定参数，不做任何字符串插值。这保持 `namedSql` 契约不变，也避免注入面。

各 SQL 常量改成带 `{filters}` 占位的模板，由一个 helper 做一次 `replace`。

six 段固定文本：

| 字段     | 片段                                                                                                      |
| -------- | --------------------------------------------------------------------------------------------------------- |
| host     | `and s.host = ?`                                                                                          |
| process  | `and a.process_id = (select dimension_id from dimension_dict where dimension_kind='process' and value=?)` |
| rule     | `and build_rule_name_sql(...) = ?` → 见下                                                                 |
| network  | `and a.network_id = (select ... 'network' ...)`                                                           |
| chain    | `and last_chain_hop(a.chain_key) = ?`                                                                     |
| category | `and a.primary_category_id = (select ... 'category' ...)`                                                 |

`rule` 的匹配需要与派生键一致。派生键依赖 `rule` / `rule_payload` / `chain_key` 三者，而 `connection_session_attr` 只存 `rule_id`（指向 `dimension_dict` 的原始 `rule` 值）与 `chain_key`，**没有 `rule_payload`**。

所以 rule 的 SQL 侧派生只能做到：多跳时 `last_chain_hop(a.chain_key)`；单跳时退回 `dimension_dict` 的原始 `rule` 值。`rule(payload)` 这一形态无法在 SQL 里复原。

**定稿**：SQL 侧的规则键定义为

```sql
coalesce(last_chain_hop(a.chain_key),
         (select value from dimension_dict where dimension_kind='rule' and dimension_id=a.rule_id),
         'DIRECT')
```

即多跳取策略组，单跳取原始 rule 值（不带 payload），皆空取 DIRECT。这与 `build_rule_name` 在「单跳且有 payload」一种情形上不同：Rust 函数会给出 `rule(payload)`，SQL 只给 `rule`。

处理：`build_rule_name` 的 payload 分支只用于**展示**（若未来需要），聚合键统一用上面的 SQL 定义，并把这条差异写进 `rule_name.rs` 的文档注释与本文件。`build_rule_name` 的签名保留 payload 参数以便单测覆盖 neko 语义，但聚合路径不调用它——**避免出现「两处实现看起来一样其实不一样」**。若后续要让聚合键带 payload，必须在 `connection_session_attr` 增列存 payload，属独立任务。

## 3. 五维物化

`c3/retention.rs:103-119` 从一条 insert 改为五条（或一条带 union all）。以 rule 为例：

```sql
insert or replace into traffic_hourly_dimension(
  utc_hour, category_id, dimension_kind, dimension_id, upload, download,
  connection_count, active_duration_sec)
select (m.utc_minute * 60 / 3600) * 3600,
       coalesce(a.primary_category_id, 0),
       'rule',
       coalesce(a.rule_id, 0),
       sum(m.upload), sum(m.download),
       count(distinct m.session_pk), count(distinct m.utc_minute) * 60
  from connection_minute m
  left join connection_session_attr a on a.session_pk = m.session_pk
 where m.utc_minute >= ?1 and m.utc_minute < ?2
 group by 1, 2, 4
```

**chain 维度的物化需要 dimension_dict 里有条目**，否则 `RANK_HOURLY` 的 join 取不到 value。`chain_key` 不在 `dimension_dict` 里。

**定稿**：物化 chain 时先把 `last_chain_hop(a.chain_key)` intern 进 `dimension_dict` 的新 kind `'chain'`，再用其 `dimension_id`。intern 在 Rust 侧先跑一遍 distinct 值，再执行 insert。这样维度层与 raw 层的 chain 键集合一致（AC11 的断言就测这一点）。

同理 rule 维度的物化值改用第 2 节的 SQL 定义后，也需要 intern 进 `'rule_group'` 这个新 kind——**不能覆写既有 `'rule'` kind**，因为 raw 层的 `filters.rule` 仍可能有历史查询依赖原始值，且 `dimension_dict` 是 `(kind, value)` 唯一。用新 kind 隔离，旧 kind 保留不动。

于是维度层的 `dimension_kind` 集合是：`host` / `process` / `rule_group` / `chain` / `network`。`DimensionKind::Rule` 在维度层映射到 `'rule_group'`，在 raw 层用派生 SQL。这个映射写在 `dimension_kind_sql` 的维度层版本里，与 raw 层版本分开命名，避免混用。

### 3.1 category 的维度层排名

`traffic_hourly_dimension` 的 `category_id` 是列而不是 `dimension_kind`。所以 category 排名要一条独立 SQL：

```sql
select coalesce(d.value, ''), sum(h.upload), sum(h.download),
       sum(h.connection_count), sum(h.active_duration_sec)
  from traffic_hourly_dimension h
  left join dimension_dict d
    on d.dimension_kind = 'category' and d.dimension_id = h.category_id
 where h.utc_hour >= ?1 and h.utc_hour < ?2
   and h.dimension_kind = 'host'   -- 固定一种 kind，避免五维重复计数
 group by h.category_id
 order by sum(h.download) desc
 limit ?3
```

`and h.dimension_kind = 'host'` 是必需的去重条件：五维物化后同一份流量在表里有五行，不固定一种 kind 会把总量放大五倍。这条约束要写进 SQL 注释，并有一条单测断言 category 排名之和不超过同区间 totals。

### 3.2 历史区间

**定稿：仅新数据可用 + 明确能力标记。**

理由：回填要重跑全部历史 `connection_minute`，在 30 天库上是一次长事务，且 `verify_layer` 的 checksum 与 watermark 语义按 chunk 推进，回填会与既有水位冲突。

实现：`retention_watermark` 新增一条 `layer = 'hourly_dim_v2'` 记录五维物化的起点。查询落在该水位之前且 grouping 不是 host 时，返回 `CapabilityUnsupported` 带中文原因「该维度的精确层从 <日期> 起可用」。host 不受影响，历史数据本来就有。

这条水位也是 R5 能力报告的判定依据：`plan_capability` 读它来决定 `exact_top_n`。

## 4. 未知维度值

现状 INNER JOIN 丢弃 `dimension_id = 0` 的行（`dimension_dict` 无该行）。

**定稿：改 LEFT JOIN，`coalesce(d.value, '')` 为空时输出 identity `"__unknown__"`、label 走 i18n 的「未知」。** 排名里显式占一行。

这样「排名之和 + 未知行 == 合计」成立，可作为 AC15 的硬断言。前端拿到 `identity == "__unknown__"` 时按未知渲染，不参与下钻（下钻到「未知」没有意义，且 `filters` 无法表达）。

`__unknown__` 这个哨兵值要写进 DTO 文档，并在 `dimension_dict` 上加约束或测试保证真实值不会等于它。

## 5. 容量影响

五维物化把 `traffic_hourly_dimension` 行数放大约五倍（每条连接每小时从 1 行变 5 行，实际倍数取决于维度值的基数与共现）。

必须实测而非估算：用 `monitor-bench` 的 workload 生成 30 天库，分别在一维与五维物化下记录

- `traffic_hourly_dimension` / `traffic_daily_dimension` 行数
- 数据库文件字节数与 WAL 字节数
- 一次 `materialize_hourly` 的耗时

数字填回本节。`retention_preview` 的 `hourly_rows` / `daily_dim_rows` 是设置页可见读数，行数口径变化要在 `docs/data-directory.md` 说明。

**实测结果（本轮未测 30 天库）**：`monitor-bench` 的 30 天 `average_active=250` 生成与物化未在本轮环境跑完。没有填估算数字。一维 → 五维行数比、库体积、`materialize_hourly` 耗时、`last_chain_hop` 30 天 raw 窗排名耗时均标记为 **not measured in this run**。

单测用短 fixture 验证五维 `insert or replace` 与 `traffic_daily_core` 按 `dimension_kind = 'host'` 去重，不能代替 30 天容量证据。

## 6. 分钟粒度

`Granularity` 加 `Minute1 | Minute2 | Minute5 | Minute10`，kebab-case 序列化为 `minute1` 等。用枚举而非自由整数 bucket：可穷举、可测试，也防止前端传任意值绕过保留窗口检查。

`c3/service.rs:222-226` 的映射扩展为 1 / 2 / 5 / 10 / 60 / 1440 / 43200。

分钟档只在 raw tier 有意义。`plan_capability` 里若 grouping 请求分钟档而 tier 不是 `Raw`，返回 `CapabilityUnsupported("分钟粒度只在 raw 保留期内可用")`。

`src/dto.ts:274` 同步扩联合类型，并在 `decodeReportResult` 的 `queryEcho` 校验里接受新值。

## 7. 兼容与回滚

- `Granularity` 与 `dimension_dict` 新 kind 都是纯增量，旧值与旧行不动。
- `query_fingerprint`（`c3/query.rs:565-568`）只哈希 `ReportQuery` JSON，SQL 重构与参数重排不影响已归档报告的指纹。
- 新增 `retention_watermark` 行走既有 `insert or ignore` 模式，不改表结构。
- `dimension_dict` 新 kind `'chain'` / `'rule_group'` 要加进 C3 的表清单相关检查（`c0_contract.rs:52` 已列 `target_set`，确认 `dimension_dict` 在删除本地数据与备份的表清单里）。
- 回滚：移除新枚举值、新 kind 的物化分支、新水位行，`SERIES_RAW` 等 SQL 还原为常量。已写入的新 kind 行留在表里不影响旧读路径（读路径按 kind 过滤）。

## 8. 开放项

- 五维物化的实测数字（第 5 节）：本轮未测，见第 5 节。
- `last_chain_hop` 在 30 天 raw 窗排名的耗时：本轮未测，未改判为写入期新列。
- `dimension_dict` 已在 C3 表清单中：`c3/schema.rs` 的 `C3_TABLES` / `c3_table_allowlist()` 第一项就是 `dimension_dict`。删除本地数据按数据目录整库文件删除（`c5/purge.rs` 的 `monitor.sqlite3` + WAL/SHM），不是按表 DROP。备份恢复是整库 Online Backup（`c3/backup.rs`）。`c0_contract.rs` 的 core allowlist 不含该表，也不需要含；C3 allowlist 已覆盖。本子任务不改 `c0_contract.rs` / `c3/space.rs` / `c3/backup.rs`。
