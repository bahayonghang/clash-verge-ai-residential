# 设计：家宽独立页与专用家宽读数

## 1. 判定收敛：一个模块，两个具名函数

新模块 `src-tauri/src/residential.rs`：

```rust
/// 核算口径：链路节点精确等于某个已配置 target。
/// 这是写入 connection_session_attr.primary_category_id 的判据，
/// 决定 LiveOverview.categoryUpload/Download 的键与 DimensionKind::Category 的分类值。
pub fn residential_tags(targets: &[String], chains: &[String]) -> Vec<String>;
pub fn is_residential_target(targets: &[String], chains: &[String]) -> bool;

/// 实时筛选口径：精确 target 匹配，或节点名包含「家宽」。
/// 比核算口径宽。保留启发式是为了不改变已发布的「只看家宽」行为，
/// 也为了不把中文子串写进持久化的分类键。
pub fn is_residential_filter(targets: &[String], chains: &[String]) -> bool;
```

`accounting.rs:283-291` 的 `classify` 改为调用 `residential_tags`；`c2/query.rs:124-128` 的 `is_residential` 改为调用 `is_residential_filter`。

**为什么不合并**：合并到精确匹配会让实时页少选中一些以前靠子串命中的连接（违反父任务 R5 零回退）；合并到含启发式会把「家宽」中文子串引入 `primary_category_id` 的分类键并改变历史分类归属。两种口径都有存在理由，问题只在于以前它们散落两处、各自漂移。收敛到一个模块解决漂移，不改行为。

模块文档注释必须写明这条差异及其理由，`docs/known-limits.md` 与家宽页界面同步。

## 2. 家宽份额查询

新增 named SQL `share_residential_raw` 进 `c3/sql.rs` 的 corpus：

```sql
select
  coalesce(sum(case when a.primary_category_id is not null then m.upload end), 0),
  coalesce(sum(case when a.primary_category_id is not null then m.download end), 0),
  coalesce(sum(m.upload), 0),
  coalesce(sum(m.download), 0)
from connection_minute m
left join connection_session_attr a on a.session_pk = m.session_pk
where m.utc_minute >= ?1 and m.utc_minute < ?2
```

一次扫描同时得到分子与分母，避免两次查询造成分子分母区间不一致。

新命令 `residential_share(range_start_utc, range_end_utc, display_timezone)` 返回：

```rust
pub struct ResidentialShare {
    pub schema_version: u32,
    pub residential_upload: Option<u64>,
    pub residential_download: Option<u64>,
    pub attributed_upload: Option<u64>,
    pub attributed_download: Option<u64>,
    pub coverage_status: String,
    pub named_sql: Vec<&'static str>,
    pub generated_utc: i64,
}
```

**None 的产生规则**（这是 R2 的核心）：先查 `COVERAGE_RAW`。若该区间的 `covered_sec == 0`，四个字段全部为 `None`，`coverage_status` 说明无覆盖。只有在 `covered_sec > 0` 时才填实测值——此时 0 是真实的「区间内无家宽流量」。

这样 `coalesce(...,0)` 不再产生歧义：0 只在有覆盖时出现。

界面侧：四个字段任一为 `None` → 占比显示「未知」；全部有值且分母为 0 → 显示「未知」并注明分母为零；分母 > 0 → 显示百分比。

## 3. 页面结构

```
components/features/residential/
  index.tsx              三段装配 + 未配置 targets 的引导态
  caliber-note.tsx       两种口径差异的说明块（实时段与聚合段各引一次）
  monitor-section.tsx    实时：命中数、速率、按 target 的实时占用、热点卡
  aggregate-section.tsx  聚合：target 排名条形图 + 表 + 占比 + 趋势
  share-readout.tsx      家宽占比读数（含分母口径说明与未知态）
  report-section.tsx     区间选择 + 生成 + 预览 + 导出
  target-empty.tsx       未配置 targets 的中文下一步
```

复用 `08-21-neko-overview-aggregation` 建立的 `components/charts/{trend-area,rank-bar}`、`components/common/{stat-card,overview-card,top-list-item}` 与 `hooks/use-report.ts`，不重造。

新增 hook `hooks/use-residential-share.ts` 调 `residential_share`，与 `use-report` 同样的竞态处理（请求序号、过期丢弃、失败保留上次）。按父任务 `design.md` 第 4 节，`components/**` 不直接 `invoke`。

实时段复用 `query_live_connections` 且 `filter.residential_only = true`，热点取 `ConnectionPage.summary`，不新增实时命令。

聚合段用 `use-report`，grouping = `Category`，filters 可选按单个 target 收敛。

报告段复用 `run_report` + `preview_export` + `export_report`。导出前把 targets 的 `policy_version` 写入报告元数据。

## 4. 口径差异在界面上的表达

家宽页顶部一次性说明，实时段与聚合段各带一个内联标记：

- 实时段标「筛选口径：命中已配置 target，或节点名含『家宽』」。
- 聚合段标「核算口径：仅命中已配置 target」。

`caliber-note.tsx` 承载这两段文案，双语。这不是可选装饰：两段读数在同一页并列且数值可能不同，不写明会被读成同一口径下的矛盾。

## 5. targetPolicy

默认 `historical`：历史区间的分类归属按当时的 target 集合读，与「观测下界、不重写历史」一致。提供切到 `current` 的开关；`current` 只在 raw 期内可用（`drilldownCapability.currentPolicy`），超出时显示能力说明并禁用开关。

## 6. 兼容与回滚

- 无新增表，无 schema 变更，无迁移，不触碰保留 / 备份 / 删除本地数据的表清单。
- 新增一条 named SQL 与一个命令，都是纯增量。
- 判定收敛不改行为，不需要数据回填。
- 回滚：移除 `residential.rs` 并把两个判定还原到原位；移除新命令与 named SQL；把 residential 路由改回 `<PagePending />`。

## 7. 与 C3 子任务的边界

本子任务**不改** `TOTALS_RAW` / `SERIES_RAW` / `RANK_RAW` / `fill_raw_attr_rank` / `plan_capability` / `retention.rs`。`DimensionKind::Category` 的排名路由修正与维度层物化由 `08-21-c3-dimension-capability` 交付。

如果本子任务先于 C3 子任务落地：聚合段的 target 排名会返回主机排名（30 天内）或空排名（超出 30 天）。此时聚合段的排名区渲染 `capability-note`，说明「按节点排名待后端就绪」，不展示可能错误的排名。占比读数与实时段不受影响，可正常交付。
