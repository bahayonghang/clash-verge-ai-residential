# C3 维度查询与物化能力

## Goal

补齐 C3 在维度查询与物化上的缺口，让概览页、四个聚合页与家宽页拿到的排名、序列、合计与能力标记都是诚实的。本子任务**只改 Rust**，不写任何界面代码。

交付五项：分钟级粒度、规则/链路的派生聚合键、`ReportFilters` 注入到全部排名与序列路径、精确维度层从只物化 host 扩展到五个维度加 category 排名、以及能力报告在无物化维度时不再谎报 `exact_top_n: true`。

## 父任务

`.trellis/tasks/08-21-neko-ui-refactor`。跨维降级表见父任务 `design.md` 第 6c 节。

## 依赖与被依赖

- 无前置依赖，可与前端基座并行。
- **被依赖**：`08-21-neko-overview-aggregation`（概览趋势图与四个聚合页）与 `08-21-residential-page`（家宽聚合与占比）都要等本子任务的 Rust 能力落地后才能通过各自的验收。两个前端子任务不再自行改 C3。

## Confirmed facts

### 缺口一：分钟粒度未暴露

- `c3/query.rs:108-113` 的 `Granularity` 只有 `Hour | Day | Month`；`c3/service.rs:222-226` 硬映射成 60 / 1440 / 43200 分钟。
- 但 `SERIES_RAW`（`c3/sql.rs:18-30`）已把 bucket 作为 `?3` 参数用于 `(m.utc_minute / ?3) * ?3`，底层 `connection_minute` 表在 `residential-monitor/src-tauri/src/storage.rs:168-174`，索引 `idx_connection_minute_utc` 在 `c3/schema.rs:105-106`。所以扩枚举与映射即可。
- 前端契约也要改：`residential-monitor/src/dto.ts:274` 的 `granularity` 仍是 `"hour" | "day" | "month"`。

### 缺口二：规则与链路的聚合键

- `residential-monitor/src-tauri/src/storage.rs:637` 把 `rule_id` intern 成 `row.rule`，即 mihomo 上报的原始规则类型（RuleSet / IPCIDR / Match…）。`:640-644` 把 `chain_key` 存成 `row.chains.join(">")` 的完整链路串。
- neko 的 `ref/neko-master/apps/collector/src/shared/utils/rule-name.ts:7-29` 定义 `buildRuleName`：链路多于一跳时归并到最后一跳（顶层策略组）；`ref/neko-master/AGENTS.md:70` 记录这条曾在 v1.3.9 造成回归。
- `c3/service.rs:279-288` 的 `case ?3` 覆盖 `process` / `rule` / `network` / `category`，`chain` 落到 `else a.host_id`，即 `DimensionKind::Chain` 取排名返回主机排名。
- `c3/service.rs:240` 把 `DimensionKind::Host` 与 `DimensionKind::Category` 一起路由到 `RANK_RAW`，而 `RANK_RAW`（`c3/sql.rs:32-43`）无条件 `group by s.host`。所以 Category 排名也返回主机排名。

### 缺口三：filters 不进排名与序列

- `RANK_RAW`（`c3/sql.rs:32-43`）只有时间窗与 `limit`，无任何 `ReportFilters`。
- `fill_raw_attr_rank`（`c3/service.rs:266-309`）只有时间窗与 `top_n`，无任何 `ReportFilters`。
- `SERIES_RAW`（`c3/sql.rs:18-30`）的 where 只有 `(?4 = 0 or s.host = ?5)`；process / rule / network / chain / category 一概不生效。
- `TOTALS_RAW`（`c3/sql.rs:3-16`）支持 host / process / rule / network / chain，**不支持 category**。
- `TOTALS_RAW` 的 chain 条件是 `a.chain_key = ?12`，匹配完整 `a>b>c` 串。排名键改成最后一跳后，用行标签回填 `filters.chain` 匹配不上。
- `ReportFilters::dimension_filter_count()`（`c3/query.rs:162-173`）只数 host / process / rule / chain / network，**不数 category**。因此 `needs_raw`（`:570-574`，`dimension_filter_count() > 1`）不会因 category 过滤而触发，category 在整条链路上是二等过滤器。

### 缺口四：精确维度层只物化 host

- 全仓对 `traffic_hourly_dimension` 的生产写入只有一处：`c3/retention.rs:103-119`，`dimension_kind` 写死字符串 `'host'`，`dimension_id` 取 `coalesce(a.host_id, 0)`。
- 日层从小时层复制：`c3/retention.rs:136-155` 的 `materialize_daily_from_hourly`，按 `dimension_kind` 原样带过去。
- 读取端按 `dimension_kind = ?3` 过滤：`TOTALS_HOURLY`（`c3/sql.rs:64-70`）、`SERIES_HOURLY`（`:72-80`）、`RANK_HOURLY`（`:82-92`）、`TOTALS_DAILY_DIM`（`:94-100`）、`SERIES_DAILY_DIM`（`:102-109`）、`RANK_DAILY_DIM`（`:111-121`）。
- 后果：process / rule / chain / network 在精确维度层查到空排名与零合计。
- `needs_exact_dimension`（`c3/query.rs:576-584`）不含 `Category`，所以家宽按 target 的长区间查询会掉到 DailyCore；`fill_core` 在 `c3/service.rs:456` 直接 `result.rankings.clear()`。
- 但 `plan_capability` 在维度层仍返回 `exact_top_n: true`（`c3/query.rs:629-661`），在 DailyCore 才返回 false（`:668-681`）。所以「没有物化」被报告成「精确可用」。

### 缺口五：排名之和与合计不闭合

- `RANK_HOURLY` / `RANK_DAILY_DIM` 用 `join dimension_dict d on d.dimension_kind = h.dimension_kind and d.dimension_id = h.dimension_id`（INNER JOIN）。物化时缺失维度值写成 `coalesce(a.X_id, 0)`，而 `dimension_dict` 没有 `dimension_id = 0` 的行（`intern_dim` 在 `storage.rs:671-679` 对 None 直接返回 None）。
- 后果：维度值缺失的流量计入 `TOTALS_HOURLY`，但被 INNER JOIN 从排名里丢掉。排名之和 < 合计，差额无处可见。这与产品原则「未知保持未知，不填零、不静默丢弃」冲突。

### 已排除的顾虑

`query_fingerprint`（`c3/query.rs:565-568`）只对 `ReportQuery` 的 JSON 做 SHA-256，与 SQL 文本无关。所以本子任务重排 `SERIES_RAW` 的参数**不会**让 `report_snapshot_meta` 的已归档报告失效。

## Requirements

### R1. 分钟粒度

- `Granularity` 新增 `Minute1 | Minute2 | Minute5 | Minute10`，kebab-case 序列化。既有 `Hour | Day | Month` 的序列化值不得改动。
- `c3/service.rs:222-226` 的 bucket 映射同步扩展。
- 分钟档只在 raw tier 有效。落到 `HourlyDimension` / `DailyDimension` / `DailyCore` 时返回 `CapabilityUnsupported` 带中文原因，不静默升粒度。
- `residential-monitor/src/dto.ts:274` 的 `granularity` 联合类型与解码同步扩展。

### R2. 派生聚合键

- 新增纯函数 `build_rule_name(rule, rule_payload, chain_key)` 与 `last_chain_hop(chain_key)`，语义与 `buildRuleName` 等价：链路多于一跳取最后一跳；单跳或无策略组跳用 `rule(rulePayload)`；两者皆空用 `DIRECT`。
- `last_chain_hop` 注册为 SQLite 标量函数，规则维度与链路维度共用，保证判定只有一处实现。
- 链路聚合键取顶层策略组（最后一跳），不用完整 `a>b>c` 串。
- `filters.rule` 与 `filters.chain` 的匹配语义与派生键一致（`c3/sql.rs:13,15` 同步改），否则下钻查不到行。三处必须同一次提交。

### R3. filters 注入全部路径

- `RANK_RAW`、`fill_raw_attr_rank` 的链路专用 SQL、`SERIES_RAW`、`RANK_HOURLY`、`RANK_DAILY_DIM` 全部接上与 `TOTALS_RAW` 同一组过滤（host / process / rule / network / chain / category）。
- `TOTALS_RAW` 增加 category 过滤。
- `ReportFilters::dimension_filter_count()` 把 category 计入，使 category 与其他维度组合时正确触发 `needs_raw`。
- `c3/service.rs:240` 的路由修正：`DimensionKind::Category` 走 `fill_raw_attr_rank`，不走 `RANK_RAW`。
- `namedSql` 回显（`c3/sql.rs:145-160` 的 corpus）与实际执行的 SQL 名保持一致。

### R4. 五维物化与 category 排名

- `c3/retention.rs:103-119` 的物化从只写 `'host'` 扩展到 host / process / rule / chain / network 五个 `dimension_kind`。
- chain 的物化值用 `last_chain_hop(a.chain_key)`，rule 的物化值用派生键，两者与 raw 层同一套函数。
- 日层复制无需改逻辑（按 `dimension_kind` 原样带过去），但要确认 `verify_layer` 与保留删除覆盖新增行。
- `needs_exact_dimension`（`c3/query.rs:576-584`）加入 `Category`，并为 category 提供维度层排名路径（按 `category_id` 分组，不走 `dimension_kind` 过滤）。
- `retention_preview` 的 `hourly_rows` / `daily_dim_rows` 会随维度数上升；`residential-monitor/docs/data-directory.md` 与保留文案需同步说明行数口径变化。
- 历史区间的处理必须明确：回填 / 仅新数据可用 / 首次运行时重建，三者择一并在 `design.md` 写明理由。

### R5. 能力报告诚实

- `plan_capability` 在维度层返回 `exact_top_n` 时，必须反映该 grouping 是否真有物化数据。任一维度无物化则 `exact_top_n: false` 且 `note_zh` 给出中文原因。
- 维度值缺失（`dimension_id = 0`，无 `dimension_dict` 行）的流量不得被 INNER JOIN 静默丢弃。改为 LEFT JOIN 并把缺失值标为「未知」一行，或在 `note_zh` 中显式声明差额。二者择一并在 `design.md` 写明。

### R6. 容量与性能证据

- 五维物化会把 `traffic_hourly_dimension` / `traffic_daily_dimension` 的行数放大约五倍。必须用 `monitor-bench` 实测 30 天库的体积与物化耗时，不得沿用估算。
- `last_chain_hop` 在 30 天 raw 窗的排名耗时必须实测，超出报告 deadline 则改判为写入期新列并记录改判理由。

## Out of scope

- 不写任何界面代码。
- 不改 `MonitorStreamMessage`、C2 采集生命周期、Monitor Channel、托盘、凭据边界。
- 不改家宽判定（`accounting.rs` 的 `classify` 与 `c2/query.rs` 的 `is_residential`）——归 `08-21-residential-page`。
- 不新增 neko 那套按维度独立建表的 schema；继续用 `dimension_dict` + `traffic_*_dimension`。
- 不做 GeoIP / MMDB / country 维度。
- 不改保留天数上限、备份格式、删除确认短语。

## Acceptance Criteria

- [ ] AC1 (R1)：`Granularity` 新增四个分钟档有单测；既有 `Hour | Day | Month` 的 kebab-case 序列化值不变（有断言）；`src/dto.ts:274` 的联合类型与 Rust 枚举一致（typecheck 通过）。
- [ ] AC2 (R1)：分钟档落到非 raw tier 时返回 `CapabilityUnsupported` 带中文原因，不静默升粒度（有单测）。
- [ ] AC3 (R2)：`build_rule_name` 单测覆盖多跳、单跳、无策略组跳、`rule` 与 `chain_key` 皆空、`chain_key` 含前后空白五组样本；`last_chain_hop` 作为 SQLite 标量函数可用。
- [ ] AC4 (R2)：`DimensionKind::Rule` 的排名在多跳链路下按顶层策略组归并，排行不出现 `RuleSet` / `Match` 这类原始规则类型作为多跳链路的行。
- [ ] AC5 (R2)：`DimensionKind::Chain` 的排名返回链路而不是主机；有单测断言 chain 与 host 在同一 fixture 上排名结果不同。
- [ ] AC6 (R2)：以某行排名标签回填 `filters.chain` / `filters.rule` 后能取到该行的子集（有单测，覆盖多跳链路）。
- [ ] AC7 (R3)：`DimensionKind::Category` 的排名返回 target 名而不是主机；有单测断言 Category 与 Host 在同一 fixture 上排名结果不同。
- [ ] AC8 (R3)：六种过滤（host / process / rule / network / chain / category）在 totals、series、rankings 三处均生效。每种过滤各一条单测断言「series 各 bucket 之和 == totals」且「加过滤后总量 < 全局总量」。
- [ ] AC9 (R3)：`dimension_filter_count()` 计入 category；category 与另一维度组合时 `needs_raw` 为 true（有单测）。
- [ ] AC10 (R3)：`namedSql` 回显与实际执行 SQL 名一致（有测试）。
- [ ] AC11 (R4)：`traffic_hourly_dimension` 在物化后含 host / process / rule / chain / network 五种 `dimension_kind`；chain 与 rule 的物化值与 raw 层派生键一致（有单测在同一 fixture 上比对 raw 排名与维度层排名的键集合）。
- [ ] AC12 (R4)：`DimensionKind::Process` / `Rule` / `Chain` / `Network` / `Category` 在超出 raw 期的区间返回非空排名（有单测）。
- [ ] AC13 (R4)：历史区间的处理按 `design.md` 定稿实施；若选「仅新数据可用」，则超出物化水位的区间返回中文能力说明而不是空排名（有单测）。
- [ ] AC14 (R5)：无物化维度时 `exact_top_n` 为 false 且 `note_zh` 有中文原因（有单测）。
- [ ] AC15 (R5)：维度值缺失的流量在排名里以「未知」一行出现，或 `note_zh` 显式声明差额；断言「排名之和 + 未知 == 合计」或「note_zh 含差额说明」。
- [ ] AC16 (R6)：`monitor-bench` 实测 30 天库在五维物化下的体积与物化耗时，数字写入 `design.md`；`last_chain_hop` 的排名耗时实测并与报告 deadline 比对。
- [ ] AC17：`cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace` 通过；`npm --prefix residential-monitor run typecheck` 通过（`dto.ts` 改动）。
- [ ] AC18 (R4)：`retention_preview` 的行数变化已在 `residential-monitor/docs/data-directory.md` 说明；`CHANGELOG.md` 有 English 条目。
