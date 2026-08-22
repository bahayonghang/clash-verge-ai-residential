# 实施：进程页能力

## 顺序

1. `c3/sql.rs`：process `__unknown__` 与 category `__residential__` 谓词；补 `filter_clause` 单测（哨兵不进 params）。
2. `c3/service.rs` 既有过滤守恒测试：增加 process 未知与核算口径。
3. 前端 `filtersForDrilldown`、`RankTable`、`dimension-page`：进程未知可下钻；核算开关。
4. `LiveOverview` + decoder：`metadataCoverage`。进程页空态读 overview 覆盖计数。
5. `RankBarCard`：process + unavailable 不画 100% 条。
6. i18n 中/英。`reporting.md`。
7. `just monitor-check`。

## 风险文件

- `c3/sql.rs`：把 `__unknown__` 绑成 `dimension_dict.value` 会得到空集。
- `LiveOverview` 形状：decoder 与 Rust 必须同一提交落地。
- `RankBarCard` 误伤 host/rule/chain 的 unavailable 展示。

## 回滚点

步骤 1–2 只动查询，可单独回退。步骤 4 之后 Channel 载荷不兼容旧前端。
