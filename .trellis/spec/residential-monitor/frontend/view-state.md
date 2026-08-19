# 视图状态

- store 只保存导航、筛选、分页和后端 DTO 缓存。实时筛选（只看家宽、字段条件）和表头排序只留当前会话。
- 实时表列宽与显隐写入本机设置键 `live_table_layout`，不进控制器 JSON。非法或缺失回落默认模板。Recovery 无库时只改内存。
- `uiLocale` 为 `zh` 或 `en`，默认 `zh`。设置页切换后立即重绘 WebView。删除确认短语固定为 `删除全部本地数据`。
- `uiTheme` 为 `latte`、`frappe`、`macchiato` 或 `mocha`，默认 `mocha`。设置页切换后立即换肤。值写入本机设置键 `ui_theme`，不进控制器 JSON。非法或缺失回落 Mocha。Recovery 无库时只改内存。
- 不在前端实现分类、守恒、Top N 或导出统计。
- 缺口、未知和未归因差额必须单独展示，不能画成零。
- 图表必须有对应数据表。
- 固定 route：`overview`、`live`、`reports`、`alerts`、`settings-data`。`reports` 与 `alerts` 均可用。
- 关闭连接：`204` 只标 `accepted`；后续 `remove` 才标 `closed`；超时标 `unconfirmed`。没有关闭全部入口。
- 实时连接空态必须区分：未配置、未连接、采集暂停、已连接无行、订阅缺口。禁止用验收句「关闭全部连接入口不存在」或单一「无数据」兜底。暂停看 `tray_summary.collector_running` 或 coverage `closed`/`pause_or_shutdown`，不要只判断 `health.session === "paused"`。
- C4 只替换 alerts 页面内容，不得改写桌面生命周期或实时核算。`alertChanged` 只携带小型摘要；历史走 `list_alert_center`。
- 报告图表与数据表只读同一个 `ReportResult`。前端不聚合、不传 SQL，不计算滚动速率或周期用量。
- 分析报告主区顺序：工具条、状态、总量、趋势图+表、Top N 扇形图+表、页尾自动档案。
- Top N 扇形图分母为 `totals.download`。其余 = 总量 − 排名下行，仅正差额同时出现在图和表，不写回 `ReportResult`，不进导出。
- 自动档案列表可见约 8 行；类型筛选走 `list_report_archives.kind`。
- 进入 `reports` 时 `list_report_archives`，优先展示最新成功日档案，否则最新成功小时档案。手动「运行报告」只更新当前会话 token，不写 `report_archive`。
- C5 设置页可展示关于信息、删除预览 / 二次确认和用户主动 VACUUM。动态重绘后按元素 `id` 恢复焦点。缺口、未知和能力过期仍显示「未知」，不画成零。
- 设置页 TCP secret 默认保存到本机凭据并回填密码框（圆点）。显示/隐藏走独立按钮。密钥只写 `input.value`，不插进 `innerHTML`。
- `AboutDto.signed === true` 时解码失败。发布地址只展示固定 GitHub Releases URL。

