# 视图状态

- store 只保存导航、筛选、分页和后端 DTO 缓存。
- 不在前端实现分类、守恒、Top N 或导出统计。
- 缺口、未知和未归因差额必须单独展示，不能画成零。
- 图表必须有对应数据表。
- 固定 route：`overview`、`live`、`reports`、`alerts`、`settings-data`。`reports` 已由 C3 交付；`alerts` 仍返回 `unavailableUntil=C4`，不造伪数据。
- 关闭连接：`204` 只标 `accepted`；后续 `remove` 才标 `closed`；超时标 `unconfirmed`。没有关闭全部入口。
- C4 只替换 alerts 页面内容，不得改写桌面生命周期或实时 reducer。
- 报告图表与数据表只读同一个 `ReportResult`。前端不聚合、不传 SQL。

