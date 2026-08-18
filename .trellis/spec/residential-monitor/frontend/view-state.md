# 视图状态

- store 只保存导航、筛选、分页和后端 DTO 缓存。
- 不在前端实现分类、守恒、Top N 或导出统计。
- 缺口、未知和未归因差额必须单独展示，不能画成零。
- 图表必须有对应数据表。
- 固定 route：`overview`、`live`、`reports`、`alerts`、`settings-data`。`reports` 与 `alerts` 均可用。
- 关闭连接：`204` 只标 `accepted`；后续 `remove` 才标 `closed`；超时标 `unconfirmed`。没有关闭全部入口。
- C4 只替换 alerts 页面内容，不得改写桌面生命周期或实时核算。`alertChanged` 只携带小型摘要；历史走 `list_alert_center`。
- 报告图表与数据表只读同一个 `ReportResult`。前端不聚合、不传 SQL，不计算滚动速率或周期用量。
- C5 设置页可展示关于信息、删除预览 / 二次确认和用户主动 VACUUM。动态重绘后按元素 `id` 恢复焦点。缺口、未知和能力过期仍显示「未知」，不画成零。
- `AboutDto.signed === true` 时解码失败。发布地址只展示固定 GitHub Releases URL。

