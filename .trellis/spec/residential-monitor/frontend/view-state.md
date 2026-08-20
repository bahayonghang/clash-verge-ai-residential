# 视图状态

- store 只保存导航、筛选、分页和后端 DTO 缓存。实时筛选（只看家宽、字段条件）和表头排序只留当前会话。
- 实时筛选分 draft / applied：按键只改 draft；Enter、失焦或显式应用才写入 `liveQuery` 并查询。Escape / 取消恢复为已应用条件。单调 request token 丢弃过期响应。条件文本必须 escape。
- 实时表列宽与显隐写入本机设置键 `live_table_layout`，不进控制器 JSON。非法或缺失回落默认模板。Recovery 无库时只改内存。
- 实时表以 `<colgroup>` 像素宽度为唯一尺寸源；wrapper 提供横向滚动。拖动或键盘只改目标列。pointercancel / 失焦 / 捕获丢失回滚；松手成功才持久化一次。
- `uiLocale` 为 `zh` 或 `en`，默认 `zh`。设置页切换后立即重绘 WebView。删除确认短语固定为 `删除全部本地数据`。
- `uiTheme` 为 `latte`、`frappe`、`macchiato` 或 `mocha`，默认 `mocha`。设置页切换后立即换肤。值写入本机设置键 `ui_theme`，不进控制器 JSON。非法或缺失回落 Mocha。Recovery 无库时只改内存。
- `uiFont` 为 `system`、旧别名 `yahei` / `serif` / `mono`，或一条通过校验的本机族名，默认 `system`。`uiFontSize` 为 `sm`、`md` 或 `lg`，默认 `md`。`uiDensity` 为 `comfortable` 或 `compact`，默认 `comfortable`。设置页切换后立即应用到 `document.documentElement` 的 `--ui-font`。分别写入本机设置键 `ui_font`、`ui_font_size`、`ui_density`，不进控制器 JSON。非法或缺失回落默认。Recovery 无库时只改内存。本机字体列表来自 `list_ui_fonts`，只留当前会话缓存，筛选键不得 `paint`。
- 不在前端实现分类、守恒、Top N 或导出统计。实时方向热点只渲染 `query_live_connections` 返回的 `summary`，不从当前页 rows 重算。
- 缺口、未知和未归因差额必须单独展示，不能画成零。
- 热点卡片 follow `liveHotspotStatus`：`collectorRunning === null`、未知 coverage、断连、pause/shutdown、`needResync` / frozen 隐藏方向数值和旧的 matched/sample，不得显示 0 或过期 current。`noMatch` 可显示 matched `0` 与采样时间，方向值仍为未知。
- 图表必须有对应数据表。
- 固定 route：`overview`、`live`、`reports`、`alerts`、`settings-data`。`reports` 与 `alerts` 均可用。
- 关闭连接：`204` 只标 `accepted`；后续 `remove` 才标 `closed`；超时标 `unconfirmed`。没有关闭全部入口。
- 实时连接空态必须区分：未配置、未连接、采集暂停、已连接无行、订阅缺口。禁止用验收句「关闭全部连接入口不存在」或单一「无数据」兜底。暂停看 `tray_summary.collector_running` 或 coverage `closed`/`pause_or_shutdown`，不要只判断 `health.session === "paused"`。
- C4 只替换 alerts 页面内容，不得改写桌面生命周期或实时核算。`alertChanged` 只携带小型摘要；历史走 `list_alert_center`。
- 报告图表与数据表只读同一个 `ReportResult`。前端不聚合、不传 SQL，不计算滚动速率或周期用量。
- 分析报告主区顺序：工具条、状态、总量、趋势图+表、Top N 扇形图+表、页尾自动档案。总量与图在同一结果区内，不单独占一张空卡。
- Top N 扇形图分母为 `totals.download`。其余 = 总量 − 排名下行，仅正差额同时出现在图和表，不写回 `ReportResult`，不进导出。
- `route === "reports"` 时 `connectionDelta` / `healthChanged` / `summaryChanged` / `alertChanged` 不得整页 `paint`，除非 `errorZh` 相对上次绘制已变。必要重绘写回 Top N / 趋势 / 档案表滚动、`details.report-notes` 展开和钉住探查。实时表仍用 `.live-table-wrap` 恢复滚动。
- 报告图探查只读当前 `ReportResult`：悬停或键盘焦点显示名称、流量、份额；点击钉住；Escape 取消。禁止按探查改 grouping 或自动 `run_report`。
- 自动档案列表可见约 8 行；类型筛选走 `list_report_archives.kind`。
- 进入 `reports` 时 `list_report_archives`，优先展示最新成功日档案，否则最新成功小时档案。手动「运行报告」只更新当前会话 token，不写 `report_archive`。
- C5 设置页可展示关于信息、删除预览 / 二次确认和用户主动 VACUUM。设置页与 Recovery 壳显示 `logDir` 并用 `open_log_dir` 打开目录；路径写入文本节点，不拼 `file://`。Recovery 不加删除入口。动态重绘后按元素 `id` 恢复焦点。缺口、未知和能力过期仍显示「未知」，不画成零。
- 设置页 TCP secret 默认保存到本机凭据并回填密码框（圆点）。显示/隐藏走独立按钮。密钥只写 `input.value`，不插进 `innerHTML`。
- `AboutDto.signed === true` 时解码失败。发布地址只展示固定 GitHub Releases URL。

## Settings workspace state

- `settingsSection` is a view-only session value: `appearance`, `connection`, `data`, `about`, or `danger`; the default is `connection`. It must not become a new top-level route or a persisted controller setting.
- `settingsDraft.address` and `settingsDraft.targets` hold unsaved form text across section switches and dynamic paints. On save, read the draft through the existing `save_settings` / `save_targets` commands; Rust remains the validation authority.
- The secret is never part of the settings render string. `secretFieldMarkup` creates the password control and `applySecretField` writes only `input.value`; show/hide remains a separate button.
- The connection section renders `state.snapshot ?? boot.overview` for health and uses `tray_summary.collector_running` for collector status. A `test_controller` result is explicitly a single-frame probe; `reconnect_now` is the continuous-monitoring recovery action.
- Locale, theme, font, font size, and density controls persist immediately through `save_ui_locale` / `save_ui_theme` / `save_ui_font` / `save_ui_font_size` / `save_ui_density`, then localize routes and repaint. Font choices come from a searchable local-family list plus the `system` sentinel. The family list is a session cache from `list_ui_fonts`; filter keystrokes must not `paint`. Settings section buttons expose `aria-current="page"`; narrow layouts use a horizontally scrollable secondary nav without changing top-level routes.
- Settings workspace fill: `#app:has(.settings-page)` uses `height: 100vh`. `.settings-layout` must set `grid-template-rows: minmax(0, 1fr)` (narrow: `auto minmax(0, 1fr)`). Flex on `.settings-page` alone leaves the implicit grid row as `auto`, so the card shrink-wraps and `--main` shows below. The last `.settings-card` uses `min-height: 100%`.
