# 修复设置页字体选择器重绘打断

## Goal

操作员在「外观与语言」的字体下拉里，能用滚轮在列表内部滚到任意字体并点选，滚动位置不被自动拉回；在搜索框输入时插入符停在输入位置，不跳回开头。

## Background

上一轮 `08-20-settings-system-fonts-layout` 把字体控件改成应用内可搜索下拉。用户在 Windows 11 实机上报三个现象（截图 Image #1、Image #2）：

- 滚轮向下滑动字体列表后，画面被自动拉回顶部，无法滚到靠后的字体。
- 字体列表撑出下拉面板，一直延伸到窗口底部之外（Image #1）。
- 在搜索框输入后，插入符自动回到最前面（Image #2 中 `lxgw` 之后继续输入会插到前面）。

已定位到三个独立缺陷：

1. `styles.css:1808` `.font-picker-filter` 与 `styles.css:1813` `.font-picker-list`（`max-height: 14rem; overflow: auto`）是死规则。`main.ts:929-930` 生成的两个元素只有 `id`，没有对应 `class`，CSS 里也没有 `#font-picker-list` 一类 id 选择器兜底。同一段 markup 里 `font-picker` / `font-picker-trigger` / `font-picker-panel` 都带了 class，只有这两个漏了。列表因此不限高、不滚动，撑出 `.font-picker-panel`（面板有 `max-height: 18rem` 但没有 `overflow`，内容直接溢出可见）。滚轮实际滚动的是外层 `.workspace`。
2. `main.ts:1690-1714` `handleMonitorRaw` 收到监控推送就 `paint()`；`shouldSkipReportPaint`（`format/report-inspect.ts:60`）只在 `route === "reports"` 时跳过，`settings-data` 不在保护范围。`paint()` → `renderApp` → `main.ts:1330` `root.innerHTML = ...` 重建整棵 DOM。`renderApp` 只在 `route === "reports"` 时恢复 workspace 滚动（`1360-1361`），设置页重绘后 `.workspace.scrollTop` 与列表 `scrollTop` 归零。步骤 1 单独不能修采集运行时的列表滚动：列表一旦成为滚动容器，下一次 `connectionDelta` 仍会整段替换节点。
3. `main.ts:1348-1350` 重绘后只做 `document.getElementById(focusedId)?.focus()`，不保存也不恢复 `selectionStart` / `selectionEnd`。新 input 的 `value` 来自属性，程序化 `focus()` 后插入符停在位置 0；`focus()` 默认的 scroll-into-view 行为也参与把画面往上拉。`innerHTML` 重建会拆掉正在进行的输入法合成；保住合成靠跳过高频 `paint()`，不靠恢复选区。

## Confirmed facts

- 高频推送是 `connectionDelta`。`healthChanged` / `alertChanged` 只在状态变化时到达。`summaryChanged` 在 DTO 与 `MonitorHub` 枚举中存在，当前 Rust 没有构造点，不会到达前端。
- 连接状态徽标（`overview.health.session`）只在连接分区输出。外观分区不读 `overview`。人在外观分区时，`healthChanged` 触发的整页 `paint()` 不更新任何设置正文；保留该 kind 重绘是为了连接分区徽标。
- 连接分区采集器文案取 `collectorRunning`，只在 `refreshLivePage()` → `fetchTraySummary()` 更新。该拉取在 skip 判断之后，且条件是 `bootstrap` / `connectionDelta` / `route === "live"`。跳过 `connectionDelta` 后，停留设置页期间采集器文案停在上次拉取值。进入连接分区时补一次 `refreshLivePage()`，进入当下的文案为当前值。
- `handleMonitorRaw` 的 skip 判断（1704）在 `refreshLivePage()`（1707-1709）之前，跳过重绘会一并跳过该次实时页拉取。切到 `live` 路由时 `main.ts:2359-2361` 会重新 `refreshLivePage()`，与 reports 路由现有行为一致，实时页数据不会停在陈旧值。
- `readReportScroll` / `writeReportScroll`（`format/report-inspect.ts:119-148`）已经采集并写回 `.workspace.scrollTop` 与 `[data-report-scroll]` 节点，只是写回被 `route === "reports"` 条件挡住。给字体列表加 `data-report-scroll` 后，放宽写回条件即可恢复列表滚动。
- `.workspace` 在设置页可滚动（`styles.css:391` `overflow: auto`；`styles.css:402` 的 `:has(.settings-page)` 只设 `min-height: 0`，不像 live / reports 那样改 `overflow: hidden`）。
- `type="search"` 的 input 支持 `setSelectionRange`。
- `html` 的 `font-size` 是 `--ui-font-size`（`sm` 14 / `md` 16 / `lg` 18）。面板与列表限高用 `rem`，搜索框最小高度用 `40px`。`sm` 时面板上限只比内容大约 `2px`。
- vitest 默认 node 环境，`vite.config.ts` 未配 jsdom。既有 `format/*.test.ts` 只测纯函数，DOM 行为无自动化测试位。
- 选中字体后面板关闭（`main.ts:2283`），选中后不必保留列表滚动。浏览过程中若发生必要重绘，仍要写回列表 `scrollTop`。
- 抑制范围已定（2026-08-20 用户确认）：整个 `settings-data` 路由跳过与设置无关的推送重绘，不是只在字体面板打开时抑制。`healthChanged` 仍重绘。
- 实现留在当前分支 `feat/settings-system-fonts-layout`，合并目标 `dev`。`main` 上没有字体下拉。

## Requirements

- R1：字体列表在下拉面板内部滚动。列表限高且自身可滚，面板自身限高，不再把 `.workspace` 撑高、不再延伸到窗口底部之外。列表未到边界时，滚轮滚动列表。列表滚到边界后，滚轮可以继续传给 `.workspace`。
- R2：`settings-data` 路由跳过 `connectionDelta`、`summaryChanged`、`alertChanged` 触发的整页重绘。`healthChanged` 仍重绘，连接分区的连接状态徽标继续更新。`bootstrap` 与错误文案变化仍重绘。跳过期间输入法合成不被整页重建拆掉。进入连接分区时补一次 `refreshLivePage()`，采集器文案按进入时的 tray 刷新；停留设置页期间不再随 `connectionDelta` 更新 `collectorRunning`。
- R3：设置页必要重绘后 `.workspace.scrollTop` 与字体列表 `scrollTop` 保持重绘前的值，复用现有 `readReportScroll` / `writeReportScroll`。
- R4：必要重绘后恢复焦点元素的 `selectionStart` / `selectionEnd` / `selectionDirection`（仅对支持选区的 input 与 textarea），焦点恢复改用 `focus({ preventScroll: true })`。
- R5：跳过重绘不得让实时页、报告页、告警页的数据停在陈旧值——切页时既有的重新拉取路径保持不变。
- R6：不新增依赖、不引入 UI 框架、不使用远程 URL 或 CDN。不改字体枚举、保存路径、`ui_font` 取值校验和布局骨架。不改 `.font-picker-filter` / `.font-picker-list` 的既有限高规则，除非 `sm` 字号下面板仍溢出。

## Acceptance Criteria

- [ ] AC1（R1）：字体下拉打开后，列表在面板内部滚动；面板自身限高，不再把 `.workspace` 撑高或把列表延伸到窗口底部之外。滚轮滚到列表末尾的字体可直接点选。字号 `sm` 下同样成立。
- [ ] AC2（R2, R3）：在设置页停留 ≥ 30 秒（采集器运行、有连接流量），字体列表滚动位置和 `.workspace` 滚动位置不被自动重置；连接状态徽标在健康状态变化时仍更新。`healthChanged` 或字体列表加载完成触发的必要重绘后，列表滚动写回。
- [ ] AC3（R2, R4）：高频推送期间，中文输入法组合不被整页重建打断。必要重绘后：在搜索框中间位置插入字符，插入符停在插入位置。外观分区打开后等到字体列表加载完成再输入，插入符仍停在插入位置。
- [ ] AC4（R2, R5）：从设置页切到实时页，连接列表显示当前数据不是进入设置页之前的旧值；报告页、告警页行为不变。从其他设置分区切到连接分区，采集器文案为进入时的当前 tray 值。
- [ ] AC5（R6）：`npm --prefix residential-monitor run typecheck`、`lint`、`test`、`build` 通过；`just ci` 通过。
- [ ] AC6：新增的重绘跳过判断有 vitest 单测，覆盖 settings-data 与 reports 两条路由、被跳过与放行的 kind、以及错误文案变化时不跳过。

## Out of scope

- 把 `renderApp` 的整页 `innerHTML` 重建改成增量 diff 或引入渲染库。
- 实时页、报告页、告警页的重绘频率与滚动行为调整。
- 字体枚举来源、`ui_font` 校验、旧别名映射、字体栈回退。
- 设置页布局骨架、主题四格、字号三档、密度两档。
- 其余设置输入框（控制器地址、secret、目标列表）的草稿语义。
- 运行中安装新字体后的热刷新。
- 列表滚到边界后禁止滚轮传递给 `.workspace`（不加 `overscroll-behavior: contain`）。
- 外观分区打开时跳过 `healthChanged` 重绘。
- 停留设置页期间持续刷新采集器文案。
- 从 `main` 另开分支。

## Key decisions

- 抑制范围：整个 `settings-data` 路由跳过无关推送（2026-08-20 用户确认）。理由：设置页所有输入框一并不再被每秒重绘打断，含中文输入法组合；外观分区没有由 `connectionDelta` 驱动的可见内容。
- 采集器文案：跳过 `connectionDelta` 后停留期间停在上次 tray 值；进入连接分区时补拉一次。不在 skip 路径上做增量 DOM 更新。
- 跳过判断保留为纯函数并留在 `format/report-inspect.ts`：`shouldSkipReportPaint` 泛化为按路由取跳过集合，并改名 `shouldSkipRoutinePaint`。不新建文件，改动面控制在 3 个文件。
- workspace 与字体列表滚动恢复复用现有 `readReportScroll` / `writeReportScroll`：列表加 `data-report-scroll="font-picker-list"`，写回条件放宽到 `settings-data`。函数名不改。
- 选区恢复与 `preventScroll` 一并做，即使已抑制高频重绘：设置页仍有 `healthChanged` 与字体列表加载完成触发的 `paint()`，那些重绘同样会跳光标。输入法合成由 R2 保护，不由 R4 保护。
- 分支：继续 `feat/settings-system-fonts-layout`，合并目标 `dev`。

## Planning status

- artifacts: `prd.md` `design.md` `implement.md`
- 阻塞项：无
- 实现留在当前分支；用户已批准按审阅结论改规划后实施
