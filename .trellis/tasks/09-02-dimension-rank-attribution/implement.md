# 维度排名表归属列与可调列宽 — 实施

## Order

1. **RankingRow + 解码**
   - `c3/query.rs` 加 `primary_exit: Option<String>`、`exit_mixed: bool`（默认 false）。
   - 所有 `RankingRow { ... }` 字面量补字段。
   - `dto.ts` `decodeReportResult`：字段缺省 `null`/`false`；类型非法拒绝。
   - 单测：缺字段旧 JSON 能解；非法类型拒绝。

2. **Raw 出口填充**
   - 新命名 SQL：按 ranking identity 表达式 + 非空 `chain_key` 聚合 download。
   - `fill_raw_rank` 之后仅对 Host/Rule/Process 调用 `fill_rank_exits`。
   - Chain 与非 Raw 层不调用，保持 null/false。
   - 单测（`c3/service.rs` 或邻接）：纯 `DIRECT`；两 `chain_key` 混合 + 平局升序；空 `chain_key` → 未知；Hourly 层仍为 null。

3. **本机列宽**
   - 新 `dimension_rank_table_layout.rs`（或同文件模块）：sanitize / parse / encode / 默认宽。
   - `AppFacade` boot + `save_dimension_rank_table_layout`；`BootstrapDto` 字段。
   - `lib.rs` 注册命令。
   - 单测：clamp、非法回落、重启读回、不污染 `live_table_layout`。

4. **前端布局 + 拖宽**
   - `dimension-rank-table-layout.ts` 与 Rust 列 id 对齐。
   - hook 读 bootstrap、save 走 invoke；失败保留窗口宽并设 `errorZh`。
   - 抽出或复用 `ColResizer`；RankTable 表头数据列加把手。
   - 下钻 / 序号无把手。

5. **RankTable UI**
   - DataTable class；`colgroup`；列序：排名、名称、上行、下行、连接、份额、归属、下钻。
   - `kind === "chain"` 或 `!crossDimension` 时归属仍按 R1：链路永不渲染归属；无下钻能力时无下钻列。
   - i18n：`dimension.col.attribution`、`dimension.exit_mixed`（zh 归属 / 混合，en Attribution / mixed）。
   - `rank-table.test.tsx`：AC1–AC5 对应 DOM（`data-exit`、链路无列、Hourly 未知）。更新 colSpan。
   - 拖宽单测：只改目标列、松手才 commit、cancel 回滚。

6. **规格**
   - `view-state.md` 增加 `dimension_rank_table_layout` 一条（实时表段落后）。
   - `dto-and-decoding.md` C3 rankings 合同补 `primaryExit` / `exitMixed`。
   - 不改 ADR。

## Validation

- `just monitor-check`（含 rustfmt/clippy -D、cargo test、monitor typecheck/lint/test/build）。
- 不跑根目录 `npm run ci`，除非误改扩展脚本（本任务不应改）。

## Risky files

- `c3/service.rs` `load_rankings` / `fill_raw_rank`：列数与 identity 表达式必须与现 SQL 一致。
- `dto.ts` rankings 解码：过严会打碎 archive 回看。
- `ColumnResizer` 抽取：实时表回归；抽完后跑 live column-resizer 测试。
- `BootstrapDto` 新字段：前端 `own-property` 缺字段须有默认，避免旧后端/Recovery 解失败。`dimensionRankTableLayout` 缺失 → 内存默认，不拒整份 bootstrap。

## Rollback points

- 步骤 1–2 可单独回退（UI 仍无列）。
- 步骤 3–5 依赖 bootstrap 字段；回退需前后端一起。
