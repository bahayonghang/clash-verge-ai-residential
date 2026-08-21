# 报告口径

- 应用内图表、数据表和 CSV / JSON / HTML 使用同一个 `ReportResult`。图表是 Recharts 封装；占比环图与趋势图旁保留同口径数据表。悬停或钉住高亮只读当前结果，不改 grouping、不自动重查。
- `report_snapshot_token` 返回前关闭 SQLite 读事务。
- 空区间总量可以为 0。缺口、未知和能力不支持不得写成 0。
- 30 天 raw 支持组合过滤和下钻。13 个月精确层只支持单维。更老的 core daily 只保留总量、历史主分类和 coverage。
- `granularity` 合法值为 `minute1` / `minute2` / `minute5` / `minute10` / `hour` / `day` / `month`。分钟档只在 raw 保留期内可用，不升粒度。
- 主机 identity 优先级为 `metadata.host` → `sniffHost` → 目的 IP，写入 `connection_session.host`。三者都空时排名 `identity` 为 `__unknown__`，`label` 为「未知」。
- `filters.host` 为 `__unknown__` 时匹配空 host，不把哨兵当域名绑定。主机页可对未知行下钻到规则 / 链路 / 进程。其它维度的未知行不参与下钻。
- 自动 DELETE 保持关闭，直到守恒门通过。

## 自动小时 / 日档案

- 采集节拍在 durable commit 之后最多生成 1 份默认报告。窗口是已闭合的本地小时或已闭合的本地自然日。
- 默认查询：`displayTimezone=local`，`grouping=host`，`targetPolicy=historical`，`topN=20`，`comparison.previousEqualWindow=true`。小时 `granularity=hour`，日 `granularity=day`。
- 成功结果写入 SQLite 表 `report_archive`，进程退出后仍可 `list_report_archives` / `get_report_archive`。首次成功即冻结；已有 `ok` 不覆盖。`failed` 可在后续节拍重试。
- 小时档案保留 30 天，日档案保留 13 个月（`DIMENSION_RETAIN_DAYS`，396 天）。过期删除只针对档案表，与 raw 自动 DELETE 无关。
- 近 30 天默认走 raw，不在每个整点跑全量 `RetentionService`。更早的日档案走日维；日维未就绪则记失败，不写假总量。
- 进入分析报告页加载最新成功日档案，否则最新成功小时档案。手动「运行报告」只写入当前会话的 10 分钟 spool token，不覆盖自动档案。
- 从档案导出时，`get_report_archive` 把冻结 JSON 水合进现有 snapshot token，再走 `export_report`。不为导出再查更新后的库。
- Recovery Shell 不调度自动档案。
