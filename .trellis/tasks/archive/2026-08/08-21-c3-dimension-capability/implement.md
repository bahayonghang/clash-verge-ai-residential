# 实施：C3 维度查询与物化能力

## 阶段 A：派生键与函数注册

1. [x] 建 `src-tauri/src/c3/rule_name.rs`：`last_chain_hop` + `build_rule_name`。单测覆盖多跳、单跳、无策略组跳、`rule` 与 `chain_key` 皆空、`chain_key` 含前后空白五组样本。
2. [x] 把 `last_chain_hop` 注册为 SQLite 标量函数（`Deterministic | Innocuous`），注册点在打开连接、`apply_required_pragmas` 之后。
3. [x] **必测**：从 `StorageCoordinator` 新建连接后直接执行含 `last_chain_hop(...)` 的 SQL，断言不报 no such function。这是最容易漏的一处。
4. [x] 在 `rule_name.rs` 的文档注释里写明 SQL 侧规则键与 `build_rule_name` 在「单跳且有 payload」上的差异（见 `design.md` 第 2 节），并说明聚合路径不调用 `build_rule_name`。

## 阶段 B：filters 注入

5. [x] 写 `filter_clause(&ReportFilters) -> (String, Vec<String>)`：六段固定文本 + 绑定参数，无字符串插值。过滤值全是字符串，用 `Vec<String>` 代替 `Box<dyn ToSql>`。
6. [x] 把 `TOTALS_RAW` / `SERIES_RAW` / `RANK_RAW` / `fill_raw_attr_rank` 的 SQL 改成带 `{filters}` 占位的模板 + 一次 replace；`namedSql` 回显仍是常量名。
7. [x] `filters.chain` 改为 `last_chain_hop(a.chain_key) = ?`；`filters.rule` 改为 `design.md` 第 2 节的 SQL 定义。
8. [x] `TOTALS_RAW` 加 category 过滤；`dimension_filter_count()` 计入 category。
9. [x] `c3/service.rs:240` 去掉 `DimensionKind::Category` 分支，让它走 `fill_raw_attr_rank`。
10. [x] `fill_raw_attr_rank` 的 `case ?3` 增 chain 分支（独立 SQL，group by `last_chain_hop(a.chain_key)`，不 join `dimension_dict`）。
11. [x] 单测：六种过滤各一条，断言「series 各 bucket 之和 == totals」且「加过滤后总量 < 全局总量」；以排名行标签回填 `filters.chain` / `filters.rule` 能取到子集（含多跳样本）；Chain / Category 与 Host 排名结果不同；`namedSql` 回显一致；category 与另一维度组合触发 `needs_raw`。

## 阶段 C：分钟粒度

12. [x] `Granularity` 加 `Minute1 | Minute2 | Minute5 | Minute10`；`c3/service.rs:222-226` 映射扩展；补断言 `Hour | Day | Month` 的 kebab-case 值不变。
13. [x] `plan_capability`：分钟档落到非 raw tier 返回 `CapabilityUnsupported` 带中文原因。
14. [x] `residential-monitor/src/dto.ts:274` 扩联合类型；`decodeReportResult` 的 `queryEcho` 接受新值；跑 `npm --prefix residential-monitor run typecheck`。

## 阶段 D：五维物化

15. [x] 确认 `dimension_dict` 是否已在删除本地数据与备份恢复的表清单中（`c0_contract.rs`、`c3/space.rs`、`c3/backup.rs`）；不在则补。把结论写回 `design.md` 第 8 节。
16. [x] 物化前先 intern：把 `last_chain_hop(chain_key)` 的 distinct 值写进 `dimension_dict` 的新 kind `'chain'`；把 SQL 规则键的 distinct 值写进新 kind `'rule_group'`。不覆写既有 `'rule'` kind。
17. [x] `c3/retention.rs:103-119` 的物化从只写 `'host'` 扩到 host / process / rule_group / chain / network 五种。
18. [x] 加维度层版本的 `dimension_kind_sql`（`DimensionKind::Rule` → `'rule_group'`），与 raw 层版本分开命名。
19. [x] 写 category 的维度层排名 SQL（按 `category_id` 分组 + `and h.dimension_kind = 'host'` 去重），SQL 注释写明去重理由。
20. [x] `needs_exact_dimension` 加入 `Category`。
21. [x] 新增 `retention_watermark` 的 `layer = 'hourly_dim_v2'` 水位；查询落在水位前且 grouping 非 host 时返回中文能力说明。
22. [x] 确认 `verify_layer` 与保留删除（`c3/retention.rs:273`）覆盖新增行。
23. [x] 单测：五种 `dimension_kind` 都存在；chain / rule_group 的维度层键集合与 raw 层派生键集合一致；Process / Rule / Chain / Network / Category 在超出 raw 期的区间返回非空排名；水位前的区间返回中文能力说明；category 排名之和不超过同区间 totals。

## 阶段 E：能力诚实与未知行

24. [x] `plan_capability` 的 `exact_top_n` 按物化水位与 grouping 判定，无物化则 false + `note_zh` 中文原因。单测覆盖。
25. [x] `RANK_HOURLY` / `RANK_DAILY_DIM` 改 LEFT JOIN，缺失值输出 identity `"__unknown__"`、label 走 i18n「未知」。
26. [x] 单测：「排名之和 + 未知行 == 合计」；`dimension_dict` 中不存在等于 `__unknown__` 的真实值。
27. [x] 把 `__unknown__` 哨兵写进 `src/dto.ts` 的注释与 `.trellis/spec/residential-monitor/frontend/dto-and-decoding.md` 的相关约定（若该文件已覆盖排名 DTO）。

## 阶段 F：证据与收口

28. [ ] `monitor-bench` 实测：一维 vs 五维物化下的 `traffic_hourly_dimension` / `traffic_daily_dimension` 行数、库与 WAL 字节数、`materialize_hourly` 耗时。数字填回 `design.md` 第 5 节。**本轮未测 30 天库，未填估算数字。**
29. [ ] 实测 `last_chain_hop` 在 30 天 raw 窗排名的耗时并与报告 deadline 比对；超出则改判为写入期新列并记录理由。**本轮未测，未改判为写入期新列。**
30. [x] `residential-monitor/docs/data-directory.md` 说明 `retention_preview` 行数口径变化；`CHANGELOG.md` 加 English 条目。
31. [x] `cargo fmt --check`、`cargo test --workspace`、`npm --prefix residential-monitor run typecheck` 通过。本子任务代码 `clippy -D warnings` 通过。全仓 `clippy -D warnings` 还被未改动的 `credential.rs` `manual_slice_fill` 挡住（Rust 1.98 新 lint），与本子任务无关。

## 回滚点

- 阶段 A–C 是纯增量（新模块、新枚举值、SQL 模板化），回滚各自独立。
- 阶段 B 的第 7 步（`filters.chain` / `filters.rule` 改派生键）必须与第 10 步同一次提交，否则下钻查不到行。
- 阶段 D 的新 kind 行留在表里不影响旧读路径（读路径按 kind 过滤），回滚只需移除物化分支与水位行。
- 阶段 E 的 LEFT JOIN 改动会让排名多出一行，前端需要同步识别哨兵——**这一步落地时要通知 `08-21-neko-overview-aggregation` 与 `08-21-residential-page`**。

## 交接

前端两个子任务（概览聚合、家宽）依赖本子任务的：分钟档枚举值、`filters` 生效范围、`exact_top_n` 的诚实语义、`__unknown__` 哨兵。

### Granularity 序列化值

| Rust 变体 | JSON |
|---|---|
| `Minute1` | `"minute1"` |
| `Minute2` | `"minute2"` |
| `Minute5` | `"minute5"` |
| `Minute10` | `"minute10"` |
| `Hour` | `"hour"` |
| `Day` | `"day"` |
| `Month` | `"month"` |

分钟档只在 raw tier 有效。落到 HourlyDimension / DailyDimension / DailyCore 时返回 `capability_unsupported`，原因「分钟粒度只在 raw 保留期内可用」，不升粒度。

### `__unknown__` 哨兵

- 排名 `identity` 字面量：`"__unknown__"`
- 后端 `label`：`"未知"`
- 前端按未知渲染，不参与下钻；`filters` 无法表达该哨兵
- `dimension_dict.value` 不得等于该字面量

### SQL 规则键 vs `build_rule_name`

聚合键（排名、`filters.rule`、`rule_group` 物化）用 SQL：

```sql
coalesce(last_chain_hop(a.chain_key),
         (select value from dimension_dict where dimension_kind='rule' and dimension_id=a.rule_id),
         'DIRECT')
```

多跳取顶层策略组，单跳取原始 `rule`，皆空取 `DIRECT`。单跳且有 payload 时，`build_rule_name` 会给出 `rule(payload)`，SQL 只给 `rule`。前端下钻 `filters.rule` 必须用排名行的 `identity`，不要自己拼 `rule(payload)`。

`filters.chain` 匹配 `last_chain_hop(a.chain_key)`，不是完整 `a>b>c` 串。下钻用排名行 `identity`。

### `exact_top_n`

- Host：维度层始终 `true`（host 一直有物化）
- 其他 grouping：无 `hourly_dim_v2` 水位时 `false`，`note_zh` 为「该维度尚未五维物化，精确 Top N 不可用。」
- 查询起点早于 `hourly_dim_v2` 水位：`capability_unsupported`，「该维度的精确层从五维物化水位起可用」
- Category 在 13 个月精确层走 `rank_hourly_category` / `rank_daily_category`；更久仍走 DailyCore

### namedSql 新名字

raw：`rank_raw` / `rank_raw_attr` / `rank_raw_rule` / `rank_raw_chain`
维度层：`rank_hourly_dimension` / `rank_hourly_category` / `rank_daily_dimension` / `rank_daily_category`
