# 维度排名表归属列与可调列宽 — 设计

## Boundaries

- **C3 `run_report`**：权威计算 `RankingRow.primary_exit` / `exit_mixed`。前端只渲染，不按 session 重算出口。
- **C2 `AppFacade`**：新本机设置键与 `save_dimension_rank_table_layout`。不进控制器 JSON。
- **React `RankTable`**：列、DataTable 规格、拖宽。`invoke` 只在 hooks。
- **不改**：hourly/daily 物化 schema、实时表、家宽聚合表、导出列、核算口径。

## Data flow — 归属

```
connection_minute × session_attr.chain_key
  → fill_raw_rank（现有 Top N，行不拆）
  → fill_rank_exits（仅 Top N identity × 非空 chain_key）
  → RankingRow { primaryExit, exitMixed }
  → decodeReportResult
  → RankTable 文案
```

HourlyDimension / DailyDimension / DailyCore：`traffic_*_dimension` 无 `chain_key`。`load_rankings` 后保持 `primary_exit = None`、`exit_mixed = false`。UI 显示「未知」。不为此物化新维。

链路 grouping：不跑 `fill_rank_exits`；UI 不渲染列。

## Contracts

### RankingRow

```
primaryExit: string | null   // 非空 chain_key 原文；无则 null
exitMixed: boolean           // 该 identity 在窗内 ≥2 个不同非空 chain_key
```

Rust 新报告始终序列化两字段。`decodeReportResult`：缺字段视为 `null` / `false`（旧档案）。字段存在但类型非法 → 拒绝整份结果。不升 `schemaVersion`。

主出口规则（Raw，host / rule / process）：

1. 只统计 `chain_key` 去空白后非空的 minute 行。
2. 按与排名相同的 identity 表达式分组（`RANK_RAW` / `RANK_RAW_ATTR` / `RANK_RAW_RULE` 的第 1 列）。
3. 每 identity：`sum(download)` 最大的 `chain_key` 胜出；平局 `chain_key` 升序。
4. `exit_mixed = (distinct non-empty chain_key count > 1)`。
5. 无非空 `chain_key` → `primary_exit = null`，`exit_mixed = false`。

空 `chain_key` 不是出口，不把缺口写成 `DIRECT`。

实现：`load_rankings` 仍读 5 列。随后一次参数化查询，只绑 Top N identity 列表，在 Rust 里 argmax。禁止前端传 SQL。`named_sql` 增加常量名（如 `rank_raw_exits`）。

### 展示

- `primaryExit == null` → `common.unknown`
- 否则原文；`exitMixed` 时追加 ` · ` + `dimension.exit_mixed`（en: `mixed`）
- 长串 `truncate` + `title` 全文；`data-exit` / `data-exit-mixed`

归属列不可排序（与份额、排名序号相同）。

### 列宽

新模块对齐 `live_table_layout.rs`，键 `dimension_rank_table_layout`。

数据列 id：`name` | `upload` | `download` | `connections` | `share` | `attribution`。

不入库：`rank`（序号）、`drill`（操作）。链路页忽略 `attribution` 宽度。

`BootstrapDto.dimensionRankTableLayout`；命令 `save_dimension_rank_table_layout`。sanitize：未知列丢弃、宽 clamp（建议 min 48 / max 640）、缺列补默认。encode 超 `SETTING_VALUE_MAX` 失败。Recovery 无库：内存默认，save 失败只留当前窗口（文案对齐 `live.layout_save_fail`）。

拖动手感复用实时表：`persistOnRelease`、pointercancel/失焦回滚、键盘 Arrow/Home/End。优先把 `ColumnResizer` 收成与布局类型无关的 `ColResizer`（id、width、min、max、table/col ref、onDraft、onCommit）；实时表做薄封装。禁止两套互斥失败的 pointer 逻辑。

`RankTable`：`colgroup` 像素宽为唯一尺寸源；wrapper `overflow-x-auto`；`data-table` 的 `w-auto`，禁止 `w-full`。

默认宽（可在实现时按主区微调，须写入 sanitize 默认）：name 280，upload/download 88，connections 72，share 64，attribution 160。

## Compatibility

- 旧 archive / snapshot 无新字段 → 归属未知，回看不崩。
- JSON 导出会多两字段；CSV/HTML 排名表不增列（范围外）。
- `monitor-db rank` 可不展示新字段；若 JSON 透传 `RankingRow` 则顺带带上。

## Tradeoffs

- 二次查询 vs 把出口塞进排名 SELECT：二次查询不改现有 5 列 `load_rankings`，Top N 后数据量小。
- Raw-only 出口 vs 改物化：改物化超出本任务，且 24h 主机页走 `minute10`/Raw。
- 共用一套列宽 vs 每页一键：用户已选共用。

## Rollback

删 `RankingRow` 新字段会使新前端拒解码。回滚需前后端同提交。列宽键可留在 `machine_setting`，sanitize 会忽略未知。
