# 家宽页创建并弹窗查看 HTML 报告

## Goal

家宽页按顶栏时间窗创建一份可查看的 HTML 报告：主按钮为「创建报告」，卡片显示与右上角同步的统计窗口以及创建时间，右侧「查看报告」用大弹窗打开该网页。

用户价值：创建一次就能回看冻结的家宽流量网页，不必只靠 token 哈希和页内覆盖句确认报告是否生成。

## Background

2026-08-31 用户在家宽页标出「运行报告」，要求改为创建、时间与顶栏同步、显示创建时间、右侧查看、创建时生成网页、查看时大弹窗打开。弹窗内容选定为方案 1：升级现有 HTML 导出，Dialog + iframe srcdoc，不启 HTTP 端口。

进行中的排版任务已按用户要求归档到 `archive/2026-08/08-31-residential-page-layout-refactor`。

## Confirmed facts

- `ReportSection`（`report-section.tsx:32-114`）点击时已用壳层 `timeRange` 构造查询；卡片不显示 preset、起止、`generatedUtc`。成功态为 `report.done` token 前 8 位。
- 主按钮 `report.run` = 「运行报告」。无「查看报告」，无 Dialog / iframe。离开页面回到 `report.idle`。`persistManual=true` 已写 7 天 `kind=manual` 档案，本区块不回看。`loadArchives(true)` 会水合自动日/小时档案，不能用于家宽回看。
- 顶栏 `TimeRangePicker` 文案来自 `time.recent_prefix` + `time.preset.*`。`timeRange` 为毫秒；`queryEcho` / `generatedUtc` 为 Unix 秒，`formatUtc` 乘 1000。
- `c3/export.rs` `write_html` 已能写出完整 HTML，只经 `export_report` 落盘；`preview_export` 不返回正文。现行 HTML 把 `metadata_line` 当正文，趋势表头写死「趋势」。
- 分析报告页有独立 preset，不读壳层时钟。本任务不改该页按钮文案。
- 缺口/未知不得写成 0。导出不得含 secret。禁止远程 URL / CDN。C2 不得直接写 `report_archive`。快照最多 8 个 token，回看不得为扫描档案而反复 `insert`。

## Requirements

- R1 家宽页主按钮为「创建报告」（新 i18n key，不改分析报告页的「运行报告」）。空态「尚未创建报告。」查看前若无报告，提示「请先创建报告。」
- R2 创建使用当前顶栏 `timeRange`（分钟对齐，与现 `buildResidentialManualQuery` 相同）。卡片始终显示与顶栏同一套 preset 文案及对齐后的起止。创建成功后另显示冻结窗口（`queryEcho`）和创建时间（`generatedUtc`）。顶栏之后滚动或换 preset 不改已创建报告。
- R3 主按钮右侧「查看报告」。无成功报告或 HTML 未就绪时禁用。点击打开大弹窗。Esc / 关闭可关。弹窗不改 grouping、不自动再查。
- R4 创建成功后沿用 snapshot + 7 天 manual 档案，并生成同源 HTML 网页（`write_html` 升级为可读：窗口、创建时间、策略、总量、排名表、趋势表）。失败不写网页、不写档案。
- R5 HTML 与弹窗不含 secret、不含远程 URL、不含脚本。`metadata_line` 可保留在次要块以便与 CSV 元数据对照，不得作为主阅读段落。
- R6 离开家宽再进入，加载 7 天内最新成功的家宽手动报告（`grouping=host` 且 `filters.category=__residential__`），可直接查看。扫描档案只读 SQLite JSON，匹配成功后再水合一个 snapshot token。
- R7 current / historical 只影响下一次创建。已打开报告以 `policyMetadata` 为准并显示。
- R8 页内 Coverage / Capability / Export 保留。不重做 CSV/JSON 另存。不启本机 HTTP 端口，不加 Tauri 子窗口。

## Acceptance Criteria

- [ ] AC1 家宽页主按钮为「创建报告」，右侧为「查看报告」。无成功报告时查看禁用。分析报告页仍为「运行报告」。
- [ ] AC2 顶栏「近 24 小时」时，报告卡创建前显示同一 preset 文案及该窗口起止。创建成功后显示冻结窗口与创建时间。改顶栏后「将使用」跟随顶栏，已创建报告的冻结时间不变。
- [ ] AC3 创建成功后点「查看报告」打开大弹窗，iframe 展示该次 `ReportResult` 的 HTML（家宽 filter + host 排名），可见窗口与创建时间。Esc 关闭。
- [ ] AC4 创建失败不出现可查看网页；查看仍禁用。
- [ ] AC5 创建后离开家宽再进入，仍可查看最近一次成功的家宽手动报告（7 天内）。不得把自动小时/日档案或非家宽手动报告当作本页报告。
- [ ] AC6 HTML 无 `http://`、无 secret 子串、无 `<script`。缺口/未知保持未知。`report-section` / archive / export 测试更新后通过；`just monitor-check` 通过。

## Out of Scope

- 分析报告页工具条、自动档案列表、图表探查。
- 本机 HTTP 服务、新 Tauri 子窗口、Radix Dialog 新依赖（弹窗用现有 overlay 模式）。
- 改 `report_archive` schema、自动 hour/day 调度、raw 保留、打开 C3 自动 DELETE。
- 邮件、云同步、共享链接、PDF。
- 删除 Coverage / Capability / Export。
- 家宽页其余区块排版。

## Key decisions

- 弹窗内容 = 升级后的静态 HTML + iframe srcdoc。
- 回看 = `load_latest_residential_manual` 读档案 JSON，命中后再 `snapshots.insert` 一次。
- 家宽页专用文案 key，不改 `report.run`。
