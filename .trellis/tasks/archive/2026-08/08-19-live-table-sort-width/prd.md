# 实时表头排序、数值筛选与列布局

## Goal

用户在「实时连接」按列比较当前页：表头排序、对字节/速率/时长做数值条件，并自己调列宽和显隐。列宽不随 1 Hz 刷新和单元格字长晃动。重启后列宽和显隐保持。

## Task Map

父任务只拥有需求源和跨子任务验收。实施从子任务开始，不在父任务上 `task.py start`。

| 子任务 | 交付 | 顺序 |
|---|---|---|
| `08-19-live-table-columns` | 固定列宽模板、单行省略、拖数据列宽、列显隐、本机持久化、重绘保持滚动 | 先做 |
| `08-19-live-table-header-sort` | 十二数据列表头排序、补齐 `sort_key`、未知排在有值之后 | 其次，改表头时依赖列 `data-col` |
| `08-19-live-table-numeric-filter` | 「添加条件」增加下载/上传/速度/时间；数字 + 单位下拉；比较原始字节或毫秒 | 可与排序并行于查询层，UI 只改工具条 |

## Background

归档任务 `08-19-live-table-filter` 交付了 Clash 十二列 + 「只看家宽」+ 字段条件，并把表头排序、列宽拖动写在范围外。用户截图 Image #1 红框十一列要求排序筛选；Image #2 表头和链路换行挤扁。用户选择 B+C，数值输入选择数字框 + 单位下拉。证据：`research/table-layout-and-sort.md`。

## Confirmed Facts

- `query_live_connections` 是表格权威。默认 `sortField=identity`，`limit=200`。`connectionDelta` 后 `paint()` 整页重写 `innerHTML`（`main.ts`）。
- `c2/query.rs` `sort_key` 已有 `host` / `process` / `rule` / `network` / `upload` / `download`。缺速率、链路、时长、来源、目标、`Tun(tcp)`。
- 「添加条件」字段：`host` / `chain` / `rule` / `process` / `source` / `destination` / `type`。`mode` 为 `exact` / `contains`。最多 8 条 AND。只留当前会话。
- `.live-table { display: block }`（`styles.css`）。无 `table-layout: fixed`，无列宽，单元格可换行。
- `paint()` 按元素 `id` 回焦点，不保存 `.live-table-wrap` 滚动位置。
- 本机偏好走 `machine_setting`（`ui_locale`、`ui_theme`）。`SETTING_VALUE_MAX` 为 4096。C2 不得 `use rusqlite`。Recovery 无库时主题只改内存。
- Clash Verge Rev 连接表用像素列宽、横向滚动、表头点击排序、拖宽/显隐持久化。本仓库禁止 UI 框架；列表顺序必须走查询层。
- 未知不得画成零。

## Key Decisions

- 范围 B+C。数值输入为数字框 + 单位下拉；前端换成整数再交给查询。
- 列重排、表头 Excel 菜单、虚拟化、翻页 UI 不做。
- 排序与筛选只留当前会话。列宽和显隐写入 `machine_setting` 键 `live_table_layout`，重启保持。
- 主机可排序、可拖宽、可显隐。操作列不可排序、不可隐藏、不拖宽。
- 至少保留一列数据列可见。
- 数值条件走现有 `clauses`，与文本条件共用 8 条上限和 AND。未知速率/时长不命中数值比较。
- 点击循环：该列降序 → 升序 → `identity` 升序。

## Requirements

### R1 列宽与显隐 — `08-19-live-table-columns`

- 默认像素模板见父 `design.md`。`table-layout: fixed`。去掉 `.live-table { display: block }`。滚动只在 `.live-table-wrap`。
- 表头和单元格单行，超出省略。数字列右对齐、等宽数字。
- 用户可拖数据列右缘改宽，受最小宽约束。指针抬起后写入本机设置。非法或缺失回落默认。
- 工具条「列」入口：十二个数据列开关 + 恢复默认列宽和显隐。操作列不在开关里。
- 1 Hz 刷新与不同字长不改变当前列宽。重绘保持容器纵向和横向滚动位置。拖动期间不跑整页 `paint()`。
- Recovery 无库时列布局只留内存。

### R2 表头排序 — `08-19-live-table-header-sort`

- 十二个数据列可点。操作列不排序。
- 排序走 `query_live_connections`。改排序清 cursor，查第一页。delta 刷新保持当前 `liveQuery` 排序。
- 未知速率/时长排在有值之后。字符串缺值同样排在有值之后。平局用 `identity`。
- 表头显示当前排序方向，`aria-sort` 与可见标记。文案跟 `uiLocale`。
- 拖列宽手柄不触发表头排序。

### R3 数值条件 — `08-19-live-table-numeric-filter`

- 「添加条件」增加字段：下载、上传、下载速度、上传速度、时间。
- 文本字段：精确 / 包含 + 文本。数值字段：> ≥ < ≤ = + 数字 + 单位。
- 下载/上传单位：B、KiB、MiB、GiB。速度：B/s、KiB/s、MiB/s。时间：秒、分钟、小时。默认单位：KiB、KiB/s、分钟。
- 前端将数字 × 单位换成整数（字节或毫秒，四舍五入）。空值、负数、非数字忽略该行。溢出忽略该行。
- `clauses.value` 为十进制整数字符串。单位只留在前端条件行，供重绘。
- 比较对象为原始 `download` / `upload` / `rate_*` / `duration_ms`。无前一帧速率、缺失时长不命中。
- 文本字段配数值 `mode`、数值字段配 `exact`/`contains` 时忽略该行。

### R4 既有合同

- 单条关闭、空态五类、secret 不进 Channel / 日志 / 导出。
- 前端不按当前 200 行本地重排。
- 不改核算、Channel `schemaVersion`。

## Out of Scope

- 列重排、虚拟化、keyset 翻页 UI。
- 表头 Excel 式筛选菜单。
- 关闭全部、CLOSED 页、连接详情抽屉。
- 把未知速率写成 `0 B/s`。
- 改报告、告警、备份、核算。
- 引入 UI 框架或远程资源。

## Acceptance Criteria

- [ ] **AC1 列宽稳定**（columns）：同一窗口宽度下，刷新前后与不同字长行之间，当前列宽不变。表头不竖排。长文本省略。超出主区横向滚动。
- [ ] **AC2 拖宽与显隐**（columns）：拖数据列后宽度保持到重启。隐藏某数据列后该列不出现；至少一列数据列可见。恢复默认后回到模板宽和全显示。操作列始终可见且不可拖。
- [ ] **AC3 滚动**（columns）：重绘后表格容器滚动位置保持。拖动列宽时表格不因 1 Hz `paint()` 中断。
- [ ] **AC4 排序**（sort）：点击数据列表头可切换降序/升序/默认 identity。操作列无排序。查询参数为当前 `sortField`/`descending`。delta 后顺序仍按该排序。拖宽不触发排序。
- [ ] **AC5 未知排序**（sort）：无前一帧的速率、缺失时长与空字符串不按 0 比较，排在有值之后。
- [ ] **AC6 数值条件**（numeric）：下载/上传/速度/时间按换算后的整数比较。单位下拉改变换算。精确/包含不用于这些字段。未知速率/时长不命中。与「只看家宽」和文本条件 AND。
- [ ] **AC7 合同**（跨子任务）：表格仍以 `query_live_connections` 第一页为准。只看家宽、文本条件、单条关闭、空态五类保持。
- [ ] **AC8 回归**（跨子任务）：`npm --prefix residential-monitor` 的 typecheck / lint / test / build 通过；相关 Rust 测试通过。不跑 `tinstall`。

## Notes

- 实施前须改 frontend spec：delta 后查询当前 `liveQuery` 第一页；列布局为本机设置，不是会话筛选。
- 质量条对齐 Clash Verge Rev 的列宽稳定、表头排序、拖宽与显隐。不复制列重排。
