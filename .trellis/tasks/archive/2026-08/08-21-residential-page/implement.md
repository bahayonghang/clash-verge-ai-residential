# 实施：家宽独立页与专用家宽读数

## Rust

1. [x] 建 `src-tauri/src/residential.rs`：`residential_tags` / `is_residential_target` / `is_residential_filter`。模块文档注释写明两种口径的差异及理由。
2. [x] `accounting.rs:283-291` 的 `classify` 改调 `residential_tags`；`c2/query.rs:124-128` 的 `is_residential` 改调 `is_residential_filter`。
3. [x] 单测：两个函数 × 四种样本（精确命中、含「家宽」子串但非 target、无命中、targets 为空）。另加一条对比测试，断言同一批连接在改造前后「只看家宽」的选中集合完全一致。
4. [x] 加 named SQL `share_residential_raw` 进 `c3/sql.rs` 的 corpus。
5. [x] 实现 `residential_share` 命令：先查 `COVERAGE_RAW`；`covered_sec == 0` 时四个字段全为 `None`；`> 0` 时填实测值。返回结构含 `named_sql` 回显。
6. [x] 单测两条路径：无覆盖 → 四个 `None`；有覆盖且家宽为 0 → 四个 `Some`，家宽为 0。再加一条断言 `named_sql` 回显与实际执行一致。
7. [x] `i18n.rs` 加家宽页与口径说明的双语文案。
8. [x] `cargo fmt --check`、`cargo clippy -D warnings`、`cargo test --workspace`。全仓 clippy 仍被既有 `credential.rs` `manual_slice_fill` 挡住。

## 前端

9. [x] 确认 `08-21-neko-overview-aggregation` 已建 `components/charts/{trend-area,rank-bar}` 与 `components/common/{stat-card,overview-card,top-list-item}`；未建则等待或按其 `design.md` 第 2 节的 API 建立并回填登记。
10. [x] 写 `hooks/use-residential-share.ts`：请求序号、过期丢弃、失败保留上次。
11. [x] 写 `caliber-note.tsx`：两段口径说明，双语。
12. [x] 写 `target-empty.tsx` 与 `index.tsx` 的未配置引导态。
13. [x] 写 `monitor-section.tsx`：`query_live_connections` + `residential_only = true`，热点取 `summary`，带筛选口径标记。
14. [x] 写 `share-readout.tsx`：`None` → 「未知」；分母为 0 → 「未知」+ 注明分母为零；分母 > 0 → 百分比。分母标为「可归因观测」。
15. [x] 写 `aggregate-section.tsx`：`use-report` grouping = `Category`，条形图 + 表 + 趋势，带核算口径标记。若 C3 子任务未落地，排名区渲染 `capability-note`（见 `design.md` 第 7 节）。
16. [x] 写 `report-section.tsx`：区间 + `run_report` + `preview_export` + `export_report`；元数据带 targets 的 `policy_version` 与 `targetPolicy`；`current` 超出 raw 期时禁用并显示能力说明。
17. [x] 补 `zh.ts` / `en.ts` 新键。**不新增键集合一致性测试**——`src/i18n/index.test.ts:7-8` 已有。
18. [x] 替换 residential 路由的 `<PagePending />`。
19. [x] `npm --prefix residential-monitor run typecheck && lint && test && build`。

## 文档与实拍

20. [x] `residential-monitor/docs/known-limits.md` 写明两种家宽口径的差异；`CHANGELOG.md` 加 English 条目。
21. [ ] 四种状态逐一实测：coverage 缺口 / 采集暂停 / 控制器未连接 / 未配置 targets。
22. [ ] 占比的三态实测：未知（无覆盖）、未知（分母为零）、正常百分比。
23. [ ] 导出物检查：含 coverage、`drilldownCapability`、`policyMetadata`、`policy_version`；`RedactMode` 生效；无 secret。
24. [ ] 四款主题 × 中英文 × 1200×800 / 窄窗口。
25. [ ] C3 子任务落地后，实测按 target 排名返回 target 名而不是主机名（AC5）。

## 回滚点

- 第 8 步之前只有 Rust 增量（新模块 + 新 named SQL + 新命令），可整体还原。
- 第 18 步之前界面仍是占位。
- 无 schema 变更，无迁移，回滚不留孤儿表。
