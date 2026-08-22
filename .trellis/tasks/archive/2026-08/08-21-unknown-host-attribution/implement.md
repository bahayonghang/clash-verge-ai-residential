# 实施：未知主机归因与可检查

## 顺序

1. `session_host.rs`：`resolve_host_identity` / `prefer_host_identity` / `looks_like_ip` 与单测（空、host、sniff、IP、升级、v6）。
2. `controller.rs`：`ConnectionMeta.sniff_host`，读 `sniffHost`。解析 fixture 覆盖空 host + sniff / dest。
3. `accounting::project_live`：`host` 写解析结果。
4. `storage::ensure_session_on`：存在则按 `prefer_host_identity` 更新 `s.host`。测试：先空后 host；先 IP 后域名。
5. `c3/sql.rs`：`filter_clause` 与 `append_dim_identity` 的 `__unknown__` 分支。扩展 `filter_clause_binds_user_values`：哨兵不进 params、片段含空 host 谓词。
6. 前端：`filtersForDrilldown`、`RankTable`、`dimension-page`、`displayLiveRow`、IP 标记。改 `rank-table.test.tsx` 与 `drilldown-panel.test.tsx` 中「未知不可下钻」只约束非 host 维。
7. `RankBar` 轴宽与省略。给 `format/rank.ts` 或 chart 纯函数写宽度/省略测试。
8. `residential-monitor/docs/reporting.md` 补一句：空 host 回退 sniff/目的 IP；`__unknown__` 过滤语义。
9. `just monitor-check`。

## 风险文件

- `storage.rs` `ensure_session_on`：更新条件写错会把域名打回 IP。
- `c3/sql.rs` 过滤：把 `__unknown__` 当 `s.host = ?` 会得到空集。
- `LiveConnectionView.host` 语义变为解析值：现有 fixture 若 host 为空且 dest 有值，断言要改。

## 回滚点

步骤 4 之前只动纯函数与 DTO，可单独回退。步骤 4 之后已写入的 IP 会留在库里。

## 开工前

- 本文件与 `prd.md` / `design.md` 已对齐选项 1。
- 不升 schema。
- 英文侧栏不在本子任务改。
