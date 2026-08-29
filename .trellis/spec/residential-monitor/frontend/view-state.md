# 视图状态

- store 只保存导航、筛选、分页和后端 DTO 缓存。实时筛选（只看家宽、字段条件）和表头排序只留当前会话。
- 实时筛选分 draft / applied：按键只改 draft；Enter、失焦或显式应用才写入 `liveQuery` 并查询。Escape / 取消恢复为已应用条件。单调 request token 丢弃过期响应。条件文本必须 escape。
- 实时表列宽与显隐写入本机设置键 `live_table_layout`，不进控制器 JSON。非法或缺失回落默认模板。Recovery 无库时只改内存。
- 实时表以 `<colgroup>` 像素宽度为唯一尺寸源；wrapper 提供横向滚动。拖动或键盘只改目标列。pointercancel / 失焦 / 捕获丢失回滚；松手成功才持久化一次。
- `uiLocale` 为 `zh` 或 `en`，默认 `zh`。设置页切换后立即重绘 WebView。删除确认短语固定为 `删除全部本地数据`。
- `uiTheme` 为 `latte`、`frappe`、`macchiato` 或 `mocha`，默认 `mocha`。设置页切换后立即换肤。值写入本机设置键 `ui_theme`，不进控制器 JSON。非法或缺失回落 Mocha。Recovery 无库时只改内存。
- `uiFont` 为 `system`、旧别名 `yahei` / `serif` / `mono`，或一条通过校验的本机族名，默认 `system`。`uiFontSize` 为 `sm`、`md` 或 `lg`，默认 `md`。`uiDensity` 为 `comfortable` 或 `compact`，默认 `comfortable`。设置页切换后立即应用到 `document.documentElement` 的 `--ui-font`。分别写入本机设置键 `ui_font`、`ui_font_size`、`ui_density`，不进控制器 JSON。非法或缺失回落默认。Recovery 无库时只改内存。本机字体列表来自 `list_ui_fonts`，只留当前会话缓存，筛选键不得 `paint`。
- `uiSidebarWidth` 为 160–352 的整数 CSS 像素，默认 220。写入本机键 `ui_sidebar_width`，不进控制器 JSON。拖动应用壳 `.shell` 右缘或键盘调整；拖动期间禁止整页 `paint`，并与实时表列宽拖动互斥。pointercancel / 失焦 / 捕获丢失回滚到开始宽度；松手成功才持久化一次。非法或缺失回落 220。Recovery 无库时只改内存。设置二级导航不提供独立宽度。
- 不在前端实现分类、守恒、Top N 或导出统计。实时方向热点只渲染 `query_live_connections` 返回的 `summary`，不从当前页 rows 重算。
- 家宽聚合与手动报告固定使用 `filters.category = "__residential__"` 选择核算家宽子集，再以 `grouping = "host"` 拆解目的域名 / IP。家宽排名方向为 session-only；切换 upload / download 必须发起后端权威 Top N 查询，并用 `queryEcho` 校验 grouping、filter、sort 与 topN，禁止把旧下载候选重排后冒充上传 Top N。
- 缺口、未知和未归因差额必须单独展示，不能画成零。
- Overview 顶部是「实时 · 当前控制器」，趋势/Top 是「历史 · 已存储数据 · 时间窗」。当前 connecting 与历史 ready 可同时成立；历史区域继续显示 report coverage 与 generated time，不能用 live health 解释历史 Unknown。
- 实时数值只在 `observationPhase === "current"` 时把 0 当真实零。connecting 显示等待连接，baselinePending 显示建立差分基线；paused/disconnected/resyncRequired/decodeFailed 隐藏 current。若保留 last-known 数值或 rows，必须标为 stale / 上次值。
- 报告 `coverage` 与 `attributionQuality` 是两个独立轴。Top 与维度页显示 exact attribution quality；`__unknown__` 按 grouping 显示未归因主机、未报告链路、控制器未报告进程或未保存/未报告规则，missing bytes 不隐藏。
- 主机 identity 优先级为 `host` → `sniffHost` → 目的 IP。`filters.host == "__unknown__"` 表示空 host。主机页在 `crossDimension` 下可对未知行下钻；其它维未知行仍不可下钻。IP identity 在排名标签上加 `IP` 标记。
- 热点卡片 follow `liveHotspotStatus`：`collectorRunning === null`、未知 coverage、断连、pause/shutdown、`needResync` / frozen 隐藏方向数值和旧的 matched/sample，不得显示 0 或过期 current。`noMatch` 可显示 matched `0` 与采样时间，方向值仍为未知。
- 图表必须有对应数据表。
- 家宽趋势图继续消费后端按 bucket 升序的 `ReportResult.series`；趋势表从同一结果创建不修改源数组的 bucket 降序投影，使最新桶置顶。两者顺序不同不代表两份统计源。
- 固定 route：`overview`、`live`、`residential`、`host`、`rule`、`chain`、`process`、`reports`、`alerts`、`settings-data`。`reports` 与 `alerts` 均可用。
- 关闭连接：`204` 只标 `accepted`；后续 `remove` 才标 `closed`；超时标 `unconfirmed`。没有关闭全部入口。
- 实时连接空态必须区分：未配置、连接中、未连接、采集暂停、已连接无行、订阅缺口。断连/暂停/重连时若保留 rows，显示 stale 而不是让 `rowCount > 0` 优先冒充 current。禁止用验收句「关闭全部连接入口不存在」或单一「无数据」兜底。暂停看 `tray_summary.collector_running` 或 coverage `closed`/`pause_or_shutdown`，不要只判断 `health.session === "paused"`。
- C4 只替换 alerts 页面内容，不得改写桌面生命周期或实时核算。`alertChanged` 只携带小型摘要；历史走 `list_alert_center`。
- 报告图表与数据表只读同一个 `ReportResult`。前端不聚合、不传 SQL，不计算滚动速率或周期用量。
- 分析报告主区顺序：工具条、状态、总量、趋势图+表、Top N 扇形图+表、页尾自动档案。总量与图在同一结果区内，不单独占一张空卡。
- Top N 扇形图分母为 `totals.download`。其余 = 总量 − 排名下行，仅正差额同时出现在图和表，不写回 `ReportResult`，不进导出。
- React 壳不整页替换 `innerHTML`。`route === "reports"` 时 `connectionDelta` / `healthChanged` / `summaryChanged` / `alertChanged` 不得触发 `run_report`，除非 `errorZh` 相对上次已变。探查钉住与 `report-notes` 展开留在 React state。实时表用 `hidden` keep-alive 保住 `.live-table-wrap` 滚动与列宽。
- `route === "settings-data"` 时 `connectionDelta` / `summaryChanged` / `alertChanged` 不得刷新设置表单，除非 `errorZh` 相对上次已变。`healthChanged` 与 `bootstrap` 仍更新连接健康。进入连接分区时补一次 live 查询；停留设置页期间 `collectorRunning` 停在上次 tray 值。
- 登录自启动状态是独立 session-only 请求状态，不属于 `ControllerSettings`。进入连接分区读取 OS；loading/saving 禁止操作，saving 重进跳过读取，set 的写后回读拥有提交权。开启必须经内联 `role="alertdialog"` 二次确认并支持 Escape/焦点返回；取消不写，关闭直达。失败保留最近确认值并提供重试，不做 optimistic toggle。
- 报告图探查只读当前 `ReportResult`：悬停或键盘焦点显示名称、流量、份额；点击钉住；Escape 取消。禁止按探查改 grouping 或自动 `run_report`。
- 自动档案列表可见约 8 行；类型筛选走 `list_report_archives.kind`。
- 进入 `reports` 时 `list_report_archives`，优先展示最新成功日档案，否则最新成功小时档案，不自动选手动行。分析报告「运行报告」、告警跳转与家宽「生成报告」在成功后写入 `report_archive`（`kind=manual`），按 `generated_utc` 保留 7 天；同一窗口同一 query 再跑则覆盖。概览 / 聚合页 `useReport` 现查不写档案。
- C5 设置页可展示关于信息、删除预览 / 二次确认和用户主动 VACUUM。进入关于分区时自动 `get_about` 并缓存于当前会话；刷新强制重拉。加载中、失败、成功三态，默认不再停在未加载空文案。发布地址只展示固定 GitHub Releases URL，显示在关于卡内等宽文本，不得写入 `errorZh`。许可证、平台和「数据只留本机 / 无遥测」用 i18n 静态行，不新增 AboutDto 字段。设置页与 Recovery 壳显示 `logDir` 并用 `open_log_dir` 打开目录；路径写入文本节点，不拼 `file://`。Recovery 不加删除入口。焦点与选区由 React 受控输入保持，禁止为保焦点而整页 `innerHTML` 替换。缺口、未知和能力过期仍显示「未知」，不画成零。
- 设置页 TCP secret 默认保存到本机凭据并回填密码框（圆点）。显示/隐藏走独立按钮。密钥只写受控 `input.value`，不写 `data-*` / `title`，不进 `console`，不进 i18n 插值。`secretFieldMarkup` 仍是测试用 HTML 辅助，React 设置页不得靠它拼密钥。
- `AboutDto.signed === true` 时解码失败。发布地址只展示固定 GitHub Releases URL。

## Scenario: 家宽目的地址排名与趋势双投影

### 1. Scope / Trigger
- Trigger: 家宽聚合、家宽手动报告、方向切换、Top N、`queryEcho` 校验或趋势表顺序发生变化。

### 2. Signatures
- `residentialReportFilters() -> ReportFilters`
- `matchesResidentialRankQuery(result, direction, topN) -> result is ReportResult`
- `shouldShowResidentialRankLoading(requestLoading, errorZh, hasRetainedResult, hasMatchingResult) -> boolean`
- `newestFirstSeries(series) -> ReportResult["series"]`

### 3. Contracts
- 聚合与手动报告请求固定为 `grouping="host"`、`filters.category="__residential__"`；方向请求为 upload/download desc，方向状态只留当前会话。
- 排名只渲染完整匹配当前 grouping、category filter、sort field、descending 与 topN 的 `queryEcho`。旧结果可被 hook 保留，但不得冒充当前方向结果。
- 新请求仍运行且回显不匹配时显示 loading；请求失败后显示错误空态，不得因旧结果仍在而永久 loading。
- RankBar 数值与份额分母跟随当前方向；行仍同时展示 upload/download，IP identity 使用共享 `formatRankLabel`。
- `TrendArea` 消费后端原始升序 series；趋势表复制后按 `bucketUtc` 降序。不得原地排序共享数组。

### 4. Validation & Error Matrix
- grouping / filter / sort field / descending / topN 任一不匹配 → 排名结果视为 stale，不渲染。
- 不匹配且 request loading → loading；不匹配且 `errorZh != null` → error，不得 loading。
- 空 series → 运行中或空态行；单桶保持原值；多桶表格严格新→旧。
- 未知 host → grouping 对应未知文案；合法 IP → 带 `IP` 标签；域名不带 IP 标签。

### 5. Good/Base/Bad Cases
- Good: 切到 upload 后发起权威 upload Top N 请求，响应匹配前不展示旧 download 榜；失败则展示错误态。
- Base: 默认 download desc；图表旧→新，表格新→旧，两者使用同一 `ReportResult`。
- Bad: 在前端对后端 download Top N 候选重排为 upload；对 `result.series.sort(...)` 原地倒序；失败后永久显示 spinner。

### 6. Tests Required
- model：queryEcho 对 grouping、住宅 filter、sort field、descending、topN 的任一漂移均返回 false。
- model/section：请求中、匹配响应、无保留响应、失败且保留旧响应四种 loading/error 组合。
- aggregate：方向切换请求、方向流量/份额、`aria-sort` 与 IP 标签。
- report：手动运行与导出都使用 host + residential filter。
- trend：断言传给图表的数组仍升序且引用内容未被修改，表格首行 bucket 最大；覆盖空/单桶/多桶和中英文表头。

### 7. Wrong vs Correct
#### Wrong
```ts
const rows = result.rankings.sort((a, b) => b.upload - a.upload);
const tableSeries = result.series.reverse();
```

#### Correct
```ts
const rankResult = matchesResidentialRankQuery(result, direction, topN) ? result : null;
const tableSeries = [...result.series].sort((a, b) => b.bucketUtc - a.bucketUtc);
```

## Settings workspace state

- `settingsSection` is a view-only session value: `appearance`, `connection`, `data`, `about`, or `danger`; the default is `connection`. It must not become a new top-level route or a persisted controller setting.
- `settingsDraft.address` and `settingsDraft.targets` hold unsaved form text across section switches and dynamic paints. On save, read the draft through the existing `save_settings` / `save_targets` commands; Rust remains the validation authority.
- The secret is never part of the settings render string. The React settings page uses a controlled password input plus `applySecretField` to write only `input.value`; show/hide remains a separate button. `secretFieldMarkup` is a test helper, not the settings render path.
- The connection section renders `state.snapshot ?? boot.overview` for health and uses `tray_summary.collector_running` for collector status. Entering the connection section refreshes tray once; staying on settings does not refresh collector status on `connectionDelta`. A `test_controller` result is explicitly a single-frame probe; `reconnect_now` is the continuous-monitoring recovery action.
- Locale, theme, font, font size, and density controls persist immediately through `save_ui_locale` / `save_ui_theme` / `save_ui_font` / `save_ui_font_size` / `save_ui_density`, then localize routes and repaint. Font choices come from a searchable local-family list plus the `system` sentinel. The family list is a session cache from `list_ui_fonts`; filter keystrokes must not `paint`. Settings section buttons expose `aria-current="page"`; narrow layouts use a horizontally scrollable secondary nav without changing top-level routes.
- Settings workspace fill: `#app:has(.settings-page)` uses `height: 100vh`. `.settings-layout` must set `grid-template-rows: minmax(0, 1fr)` (narrow: `auto minmax(0, 1fr)`). Flex on `.settings-page` alone leaves the implicit grid row as `auto`, so the card shrink-wraps and `--main` shows below. The last `.settings-card` uses `min-height: 100%`.
