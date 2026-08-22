# 家宽独立页与专用家宽读数

## Goal

把「只看家宽」从实时页的一个复选框升级为独立页面，分实时监控、聚合统计、生成报告三段。同时收敛家宽判定的实现位置，并新增家宽视角特有的派生读数（家宽占可归因观测的比例）与家宽报告导出。

## 父任务

`.trellis/tasks/08-21-neko-ui-refactor`。目录约定、图表封装、跨维降级表见父任务 `design.md` 第 1 / 5 / 6c 节。

## 依赖

- `08-21-neko-shell-foundation`：应用壳与基元。
- `08-21-neko-overview-aggregation`：`components/charts/{trend-area,rank-bar}`、`components/common/{stat-card,overview-card,top-list-item}`、`hooks/use-report.ts`。本子任务复用，不重造。
- `08-21-c3-dimension-capability`：`DimensionKind::Category` 的排名路由修正与维度层物化，以及分钟粒度。**没有它，按 target 分组的排名会返回主机排名（30 天内）或空排名（超出 30 天）。** 本子任务 AC5 依赖它。

## Confirmed facts

### 家宽判定当前有两套，语义不一致

- **核算侧** `residential-monitor/src-tauri/src/accounting.rs:283-291` 的 `classify`：`tags` = 链路节点**精确等于**某个已配置 target 的那些 target；`primary = tags.first()`。这个 `primary` 写入 `connection_session_attr.primary_category_id`，也是 `LiveOverview.categoryUpload/Download` 的键。
- **实时筛选侧** `residential-monitor/src-tauri/src/c2/query.rs:124-128` 的 `is_residential`：链路节点等于某个 target **或者**节点名包含字符串「家宽」。
- 两者不等价。同一条连接可能在实时页被「只看家宽」选中，但在核算分类里不属于任何 target。

### 「可归因观测」的定义

`accounting.rs:229-231` 对每条连接无条件 `attributed_up += delta_up`；`MinuteFact` 的 `primary` 可为 None 仍写入 `connection_minute`。所以：

- 「可归因观测」= 每条连接 delta 之和，**包含未分类连接**。它对照 controller meter 的全局计数，不是「已分类流量」。
- 无过滤的 `TOTALS_RAW`（`c3/sql.rs:3-16`）正是这个口径，可以直接作为家宽占比的分母。
- 家宽子集的判据是 `a.primary_category_id IS NOT NULL`（等价于「命中任一已配置 target」），不需要把 `ReportFilters.category` 扩成数组——该字段是单值 `string | null`（`src/dto.ts:261-268`），本来无法表达「∈ targets」。

### 占比的 None 语义不能靠 totals 判定

`TOTALS_RAW` 对 sum 用 `coalesce(..., 0)`，空窗返回 0 而不是 NULL。所以「窗内确实零字节」与「窗内无采集覆盖」在 totals 上不可区分。占比的未知判定必须由 `coverage` 决定，不由 totals 是否为 0 决定。

### targets 的存储与配额

- `target_set(set_id, policy_version)` + `target_item(set_id, position, name)`（`residential-monitor/src-tauri/src/storage.rs:138-148`）。
- `save_targets`（`storage.rs:478-495`）每次保存把 `policy_version` +1，整表删旧 `target_item` 再按 position 重写。`load_targets` 返回 `(policy_version, names)`。
- 条数上限 `TARGET_COUNT_MAX`，校验在 `c2/settings.rs:101-114` 的 `validate_targets`。命令 `save_targets` 在 `lib.rs:479-481`，返回新 `policy_version`。

### 按 target 的历史聚合有数据基础

`DimensionKind::Category` 的分类值就是 target 名。`traffic_hourly_dimension` / `traffic_daily_dimension` 按 `category_id` + 维度双键存储（`c3/schema.rs:44-65`），`traffic_daily_core` 按 `category_id` 存总量（`:66-74`）。排名路由与物化的修复归 `08-21-c3-dimension-capability`。

### 导出与归档

- 导出走 `ExportService`（`c3/export.rs:65-84`）：`preview` + `export_to_path`，`ExportSpec` 含 `ExportFormat` 与 `RedactMode`。命令 `preview_export` / `export_report`。
- `report_archive`（`c3/schema.rs:122-140`）按 `kind`（hour / day）+ `range_start_utc` + `query_fingerprint` 唯一。
- `ReportQuery.targetPolicy` 有 `current` / `historical` 两值；`current` 只在 raw 期内可用（`drilldownCapability.currentPolicy`）。

### 现有文案

`live.filter.residential` = 「只看家宽」（`src/i18n/zh.ts:127`）。`export.html_title` = 「家宽流量报告」（`src-tauri/src/i18n.rs:105`）。

## Requirements

### R1. 判定收敛到一个模块，保留两种语义

- 新建一个模块承载家宽判定，导出两个**具名**函数：
  - `is_residential_target(targets, chains)` — 精确 target 匹配，核算侧用（`classify` 调用）。
  - `is_residential_filter(targets, chains)` — 精确 target 匹配 **或** 节点名含「家宽」，实时筛选侧用（`is_residential` 调用）。
- 两者的差异必须写在模块文档注释、`residential-monitor/docs/known-limits.md` 与家宽页界面上。**不合并成一个**：合并会改变实时页行为（违反父任务 R5 零回退）或改变已入库的分类归属。
- 前端不复制任何家宽字符串匹配。
- 家宽页必须在界面上写明：实时段用筛选口径，聚合段用核算口径，两者容纳集可能不等。

### R2. 家宽占比读数（Rust）

- 新增一条家宽份额查询，返回四个 `Option<u64>` 与覆盖状态：家宽上/下行、可归因上/下行、`coverageStatus`。
  - 家宽子集判据 `a.primary_category_id IS NOT NULL`。
  - 分母为同区间同其余过滤条件下的无分类过滤总量。
- **占比的未知判定由 coverage 决定**：该区间无采集覆盖时四个值为 `None`，界面显示「未知」；有覆盖且实测为 0 时显示 0 并注明「区间内无家宽流量」。不得把无覆盖写成 0%。
- 新增的 SQL 必须进 `c3/sql.rs` 的 `namedSql` corpus，并在返回结果里回显。

### R3. 家宽页三段

- **实时监控**：命中家宽的连接数、上/下行速率、按 target 节点分组的实时占用、方向热点。数据取 `query_live_connections` 且 `filter.residential_only = true`，热点取 `ConnectionPage.summary`，不新增实时命令。
- **聚合统计**：按 target 节点的排名（条形图 + 表，grouping = `Category`）、家宽占可归因观测的比例、分钟/小时级趋势。占比的分子分母口径写在界面上，分母明确标为「可归因观测」。
- **生成报告**：按时间区间生成家宽报告并导出。复用 `run_report` + `preview_export` + `export_report`，不新增报告命令。导出元数据含 coverage、`drilldownCapability`、`policyMetadata` 与 targets 的 `policy_version`。

### R4. 口径与未知

- coverage 缺口、采集暂停、控制器未连接、未配置 targets 四种状态各自成态。
- 未配置 targets 时给中文下一步，不显示 0。
- 每个图表旁有同口径数据表。
- 家宽报告默认 `targetPolicy = historical`；提供切到 `current` 的开关并显示说明，`current` 只在 raw 期内可用。

### R5. 双语与检查

- 新增字符串同时进 `zh.ts` 与 `en.ts`；Rust 侧文案进 `i18n.rs` 双语表。
- 沿用 `src/i18n/index.test.ts:7-8` 的既有键集合断言，不新增重复测试。
- 保留 `<table>` 语义、`aria-sort`、键盘可达、`prefers-reduced-motion`。

## Out of scope

- **不新建与 `traffic_*_dimension` 平行的家宽统计表**，不新增第二条写入路径。
- 不改 C3 的通用查询能力（分钟粒度、派生聚合键、`ReportFilters` 注入、五维物化、能力报告修正）——全部归 `08-21-c3-dimension-capability`。
- 不合并两个家宽判定函数。
- 不做节点测速、可用性探测、节点切换或任何对家宽节点的主动请求。
- 不做 Regions / GeoIP。
- 不改 C2 采集生命周期、Monitor Channel、托盘、凭据边界。
- 不改 targets 的配额上限与校验规则；settings 页的 targets 编辑入口由 `08-21-reports-alerts-settings-port` 移植。
- 不动概览页、聚合页、实时页、报告页、告警页、设置页。

## Acceptance Criteria

- [ ] AC1 (R1)：家宽判定集中在一个模块的两个具名函数里；`classify` 调 `is_residential_target`，`is_residential` 调 `is_residential_filter`。单测覆盖精确命中、含「家宽」子串但非 target、无命中、targets 为空四种样本 × 两个函数。
- [ ] AC2 (R1)：实时页「只看家宽」的选中集合与改造前完全一致（有测试比对同一批连接的筛选结果）。
- [ ] AC3 (R1)：两种口径的差异写在模块文档注释、`docs/known-limits.md` 与家宽页界面上（有 diff 确认）。
- [ ] AC4 (R2)：家宽份额查询返回四个 `Option`；区间无采集覆盖时为 `None` 且界面显示「未知」；有覆盖且实测 0 时显示 0 并注明区间内无家宽流量。两条路径各有单测。
- [ ] AC5 (R3)：聚合段按 target 节点排名（grouping = `Category`）返回 target 名而不是主机名（依赖 `08-21-c3-dimension-capability`，实测确认）。
- [ ] AC6 (R2)：新增 SQL 进 `namedSql` corpus 并在结果里回显（有测试）。
- [ ] AC7 (R3/R4)：三段可用；未配置 targets 时显示中文下一步而非 0；占比的分母在界面上标为「可归因观测」。
- [ ] AC8 (R3)：家宽报告可导出，元数据含 coverage、`drilldownCapability`、`policyMetadata` 与 targets 的 `policy_version`；`RedactMode` 生效；导出物中无 secret。
- [ ] AC9 (R4)：报告标注 `targetPolicy` 为 `current` 或 `historical`，不混称；切到 `current` 且超出 raw 期时显示能力说明。
- [ ] AC10 (R5)：`zh.ts` / `en.ts` 键集合一致（沿用既有断言）；`i18n.rs` 新增键双语齐备。
- [ ] AC11：`npm --prefix residential-monitor run typecheck && lint && test && build` 通过；`cargo fmt --check`、`cargo clippy -D warnings`、`cargo test --workspace` 通过。
- [ ] AC12 (R4)：四款主题 × 中英文 × 1200×800 / 窄窗口实拍无溢出；四种状态（缺口 / 暂停 / 未连接 / 未配置 targets）逐一实测互不混淆。
