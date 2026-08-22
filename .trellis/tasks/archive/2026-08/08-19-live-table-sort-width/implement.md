# 实施计划：实时表排序、数值条件与列布局

## 启动前门禁

- [ ] 用户已批准本父规划摘要。
- [ ] 不在父任务上 `task.py start`。按下表启动子任务。
- [ ] 已读 frontend / backend spec checklist。
- [ ] 不改核算、不传 mihomo 原始 JSON、不加关闭全部、不跑 `tinstall`。

## 子任务顺序

1. `08-19-live-table-columns` — 列宽、拖动、显隐、持久化、滚动。表头带 `data-col`。
2. `08-19-live-table-header-sort` — 依赖 1 的 `data-col` 表头。补 `sort_key` 与点击循环。
3. `08-19-live-table-numeric-filter` — 工具条与 `clauses` 比较。可与 2 分先后，二者都改 `query.rs` 时不要并行改同一函数而不合并。

建议串行：1 → 2 → 3。2 与 3 若分会话，后做的一方先拉齐 `query.rs`。

## 父任务在子任务完成后

- 对照 AC7 / AC8 做一次联验：排序 + 数值条件 + 隐藏列 + 拖宽后刷新，四者同时成立。
- 更新 `.trellis/spec/residential-monitor/frontend/view-state.md`：列布局为本机设置。
- 更新 `dto-and-decoding.md`：delta 后查当前 `liveQuery` 第一页。
- 更新 `backend/modules-and-errors.md`：记载 `live_table_layout` 与 `ui_theme` 同类。

## 验证

每个子任务结束：

```text
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
```

涉及 Rust 时另跑对应 `cargo test`（`c2::query`、layout sanitize、`save_live_table_layout`）。不跑 workspace clippy 全集，除非改动触发既有门。不跑 `tinstall`。

## 风险文件

- `residential-monitor/src/main.ts` — `renderLive` / `paint` / 事件
- `residential-monitor/src/styles.css` — `.live-table`
- `residential-monitor/src-tauri/src/c2/query.rs` — 过滤与排序
- `residential-monitor/src-tauri/src/c2/facade.rs` — bootstrap / 设置
- `residential-monitor/src/ipc/live-session.ts` — 查询默认值

## 回滚

按子任务分别回滚。设置键 `live_table_layout` 非法即回落默认，删键即可恢复模板。
