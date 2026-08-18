# 报告口径

- 应用内图表、数据表和 CSV / JSON / HTML 使用同一个 `ReportResult`。
- `report_snapshot_token` 返回前关闭 SQLite 读事务。
- 空区间总量可以为 0。缺口、未知和能力不支持不得写成 0。
- 30 天 raw 支持组合过滤和下钻。13 个月精确层只支持单维。更老的 core daily 只保留总量、历史主分类和 coverage。
- 自动 DELETE 保持关闭，直到守恒门通过。
