# Implement：家宽地址流量拆解与趋势明细优化

## 1. 后端排序契约

1. [x] 在 `src-tauri/src/c3/sql.rs` 把所有排名模板的固定下载排序改为内部 `{order_by}` 槽位；新增枚举驱动的安全渲染助手，覆盖 raw / hourly / daily 与 category 特例。
2. [x] 为 upload / download / name / identity × asc / desc 增加 SQL 渲染测试：无未消解槽位、无用户字符串拼接、数值排序具有 identity 升序 tie-break。
3. [x] 在 `c3/service.rs::fill_raw_rank` 与 `fill_dimension_layer` 传入 `query.sort`，保证 ORDER BY 在 `LIMIT top_n` 前生效。
4. [x] 增加 `topN=1` 对抗 fixture：上传第一与下载第一不同；分别查询后首行正确。覆盖 raw 和 dimension tier，并保留默认 download desc 回归。
5. [x] 增加 `grouping=host + filters.category=__residential__` 回归：非家宽高流量地址被排除，域名 / IP fallback 独立成行，totals / series / rankings 守恒。

## 2. 家宽地址排名与报告

6. [x] 在家宽聚合区加入 session-only 上行 / 下行方向状态；查询改为 `grouping=host`、residential category filter、当前方向 desc。
7. [x] 用 `queryEcho` 校验返回结果匹配当前方向 / Top N / grouping / filter；切换请求期间不把旧下载结果标成上传结果。
8. [x] 条形图值与份额分母跟随方向；表格保留上下行列，用 `formatRankLabel` 标记 IP，并给当前方向表头正确 `aria-sort`。
9. [x] 将家宽手动报告改为 host grouping + residential filter；保留现有 historical/current 开关和导出流程。
10. [x] 更新中文 / 英文标题、方向与份额口径文案；沿用现有键集合一致性测试。

## 3. 趋势明细

11. [x] 把家宽趋势表提取为可独立测试的局部组件或纯投影；复制 series 后按 `bucketUtc` 降序，不修改原数组。
12. [x] 优化表格容器和单元格：rounded border、sticky header、明确 padding、时间不换行、数值右对齐、tabular nums、行分隔 / hover、窄屏横向滚动。
13. [x] 回归测试同时证明：图表输入仍按旧→新；表格首行是最大 bucket，后续严格递减；空 / 单桶 / 多桶与中英文表头正确。

## 4. 规范与验证

14. [x] 更新后端规范：`ReportQuery.sort` 必须在 Top N 截断前生效，字段 / 方向仅由枚举白名单生成，稳定 identity tie-break。
15. [x] 更新前端 view-state：家宽页是 residential filter + host grouping；方向切换必须查询权威 Top N；趋势图与表可使用同一结果的不同顺序投影。
16. [x] 运行聚焦验证：
    - `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml c3::sql`
    - `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml c3::service`
    - `npm --prefix residential-monitor test -- aggregate-section index use-report`
17. [x] 运行完整自动门（结果含仓库基线阻断，见 4.1）：
    - `npm --prefix residential-monitor run check`
    - `cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check`
    - `cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
    - `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`
    - `just ci`
    - `python ./.trellis/scripts/task.py validate 08-25-residential-address-breakdown-trend-table`
    - `git diff --check`
18. [ ] 原生 WebView 视觉检查：宽 / 窄窗口、中文 / 英文、当前深浅主题至少各一组；确认 sticky header、横向滚动、方向切换、最新桶置顶。未执行的组合明确记为 `UNVERIFIED`，不以静态 HTML / Vitest 代替。

### 4.1 验证记录（2026-08-25）

- PASS：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml c3::sql`（随后完整 workspace 再覆盖新增 SQL case）。
- PASS：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml c3::service`。
- PASS：`npm --prefix residential-monitor test -- aggregate-section aggregate-model trend-table report-section index use-report`（9 files / 31 tests）。
- PASS：`npm --prefix residential-monitor run check`（61 files / 220 tests，含 typecheck、lint、build、icons）。
- PASS：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`（343 passed / 1 ignored）。
- PASS：`python ./.trellis/scripts/task.py validate 08-25-residential-address-breakdown-trend-table`（implement/check 各 6 条）。
- PASS：`git diff --check`。
- BASELINE FAIL：`cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check` 仅报告未修改的 `src-tauri/src/storage.rs` 既有 rustfmt 漂移；未把该无关机械差异纳入任务。
- BASELINE FAIL：`cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings` 仅报告未修改的 `src/data_dir.rs:77` 既有 `clippy::result_unit_err`。
- PASS（隔离基线）：同一 clippy 命令仅增加 `-A clippy::result-unit-err` 后无其它 issue，证明任务改动未引入额外 clippy 警告。
- BLOCKED BY BASELINE：`just ci` 在通过 npm ci 与完整前端 check 后，于同一 `storage.rs` fmt 漂移停止，未进入后续 clippy/test/secrets 步骤。
- UNVERIFIED：原生 WebView 宽 / 窄、中文 / 英文与深 / 浅主题视觉组合未执行。
- REVIEW FIX：方向 / Top N 切换失败时，旧 queryEcho 不匹配结果继续被拒绝，但不再把已失败请求永久显示为 loading；新增状态模型回归并补齐 grouping / filter / sort direction / topN 的陈旧结果拒绝断言。

## 5. 风险文件与回滚点

- `c3/sql.rs` / `c3/service.rs`：跨页面共享查询；先完成并验证排序 fixture，再接 UI。默认下载降序必须保持字节级结果顺序兼容。
- `aggregate-section.tsx`：方向切换与旧请求竞态；以 query echo gate 为回滚点。若拆文件，只拆家宽局部组件。
- `report-section.tsx`：只改 query 形态，不改 archive / export 生命周期。
- 无 schema / migration；任一步可通过还原对应代码回滚，不触碰用户数据库。

## 6. Start 前门

- [x] 用户审阅并明确批准本版 Goal / In Scope / Out of Scope / AC / Key Decisions。
- [x] `prd.md`、`design.md`、`implement.md` 无阻塞开放问题。
- [x] `implement.jsonl` / `check.jsonl` 均只含真实 spec / research 条目，无示例占位行。
- [x] `task.py validate` 通过；批准后已由主会话运行 `task.py start`，任务状态为 `in_progress`。
